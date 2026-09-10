//! The edge-channel contract: how ANY adjacent pair in a relay chain
//! exchanges frames — independent of the surrounding framework.
//!
//! One trait ([`EdgeTransport`]) with two operations mirrors the two
//! fixed bridge functions of the profile: a request/response call and
//! an event/notification lane. Tauri (invoke + event), egui channels,
//! stdio or an in-process direct connection are all just adapters;
//! [`MemoryTransport`] is the canonical in-process adapter and the test
//! vehicle.

use std::sync::Arc;

use futures::future::BoxFuture;
use serde_json::Value;

use super::host::BridgeError;

/// Frame exchanged over an edge channel: canonical JSON-RPC 2.0.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeFrame {
    /// JSON-RPC method (`"enrollment.mint"`, `"relay.net.set_proxy"`).
    pub method: String,
    /// JSON-RPC params, verbatim.
    pub params: Option<Value>,
    /// The relay extension member (`relay: {hops, via}`), absent on
    /// frames an edge originates.
    pub relay: Option<super::hops::RelayContext>,
}

/// The dispatcher an edge channel's far end runs (the relay host).
pub trait EdgeFarEnd: Send + Sync {
    fn call(&self, frame: EdgeFrame) -> BoxFuture<'_, Result<Value, BridgeError>>;
}

/// Notification handler handed to [`EdgeTransport::listen`].
pub type EdgeListener = Arc<dyn Fn(&str, Option<&Value>) + Send + Sync>;

/// A transport connecting one edge (the caller side) to its far end.
/// Cloning yields another handle to the SAME channel.
pub trait EdgeTransport: Send + Sync {
    fn call(&self, frame: EdgeFrame) -> BoxFuture<'_, Result<Value, BridgeError>>;
    /// Subscribe to notifications; returns an unsubscribe.
    fn listen(&self, handler: EdgeListener) -> BoxFuture<'_, Box<dyn FnOnce() + Send>>;
}

/// In-process edge channel: the far end is dispatched inline. The test
/// vehicle and the reference adapter shape.
#[derive(Clone)]
pub struct MemoryTransport {
    far: Arc<dyn EdgeFarEnd>,
}

impl MemoryTransport {
    pub fn new(far: Arc<dyn EdgeFarEnd>) -> Self {
        Self { far }
    }
}

impl EdgeTransport for MemoryTransport {
    fn call(&self, frame: EdgeFrame) -> BoxFuture<'_, Result<Value, BridgeError>> {
        self.far.call(frame)
    }

    fn listen(
        &self,
        _handler: Arc<dyn Fn(&str, Option<&Value>) + Send + Sync>,
    ) -> BoxFuture<'_, Box<dyn FnOnce() + Send>> {
        // In-process channels emit notifications through the host's own
        // event surface; nothing to subscribe to here.
        Box::pin(async { Box::new(|| {}) as Box<dyn FnOnce() + Send> })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::hops::{HopGuard, RelayContext};
    use crate::relay::host::{BridgeError, RelayHost, RpcSink};
    use futures::FutureExt;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    /// Sized adapter so the host's generic `dispatch<S: RpcSink>` can run
    /// against a shared trait object (same pattern as the tauri bridge).
    struct SharedSink(Arc<dyn RpcSink>);

    impl RpcSink for SharedSink {
        fn call(
            &self,
            endpoint: &str,
            method: &str,
            params: Option<Value>,
        ) -> futures::future::BoxFuture<'_, Result<Value, BridgeError>> {
            self.0.call(endpoint, method, params)
        }
    }

    /// One relay exposed over an edge channel: dispatches locally or
    /// forwards through its sink with a hop step.
    struct RelayFarEnd {
        host: RelayHost,
        sink: Arc<dyn RpcSink>,
        guard: HopGuard,
        forwarded: Mutex<u8>,
    }

    impl EdgeFarEnd for RelayFarEnd {
        fn call(&self, frame: EdgeFrame) -> BoxFuture<'_, Result<Value, BridgeError>> {
            let routed = self.host.forwarding_table().route(&frame.method);
            if let crate::relay::Route::Forward(_) = routed {
                if let Err(message) = self
                    .guard
                    .admit(frame.relay.as_ref().unwrap_or(&RelayContext::default()))
                {
                    return Box::pin(async move {
                        Err(BridgeError::Forward {
                            endpoint: String::new(),
                            detail: message,
                        })
                    });
                }
                *self.forwarded.lock().unwrap() += 1;
                let stepped = HopGuard::step(
                    frame.relay.as_ref().unwrap_or(&RelayContext::default()),
                    "next",
                );
                let sink = Arc::clone(&self.sink);
                return Box::pin(async move {
                    sink.call("upstream", &frame.method, frame.params)
                        .await
                        .map(|mut v| {
                            // The hop context rides alongside; surface it for
                            // the assertion below without breaking the shape.
                            if let Value::Object(map) = &mut v {
                                map.insert("hops".into(), json!(stepped.hops));
                            }
                            v
                        })
                });
            }
            // Local/deny routes dispatch on B's own host (local handlers
            // or the -32601 MethodNotFound the router already implies).
            Box::pin(async move {
                self.host
                    .dispatch(
                        &SharedSink(Arc::clone(&self.sink)),
                        &frame.method,
                        frame.params,
                    )
                    .await
                    .map(|outcome| match outcome {
                        crate::relay::host::BridgeOutcome::Local(v)
                        | crate::relay::host::BridgeOutcome::Forwarded { result: v, .. } => v,
                    })
            })
        }
    }

    /// The C relay's upstream service surface: a local handler host whose
    /// results come back verbatim.
    struct ServiceC;

    impl RpcSink for ServiceC {
        fn call(
            &self,
            _endpoint: &str,
            method: &str,
            params: Option<Value>,
        ) -> futures::future::BoxFuture<'_, Result<Value, BridgeError>> {
            let method = method.to_string();
            Box::pin(async move {
                match method.as_str() {
                    "enrollment.mint" => Ok(json!({ "minted": true, "echo": params })),
                    _ => Err(BridgeError::MethodNotFound(method)),
                }
            })
        }
    }

    /// A→B→C chain smoke: the edge (A) calls through a memory transport
    /// into relay B; B's whitelist forwards to C's surface; the result
    /// returns to A with exactly one hop stepped, and a frame already at
    /// the hop limit is refused by B instead of forwarded.
    #[tokio::test]
    async fn chain_smoke_edge_through_relay_to_service() {
        // C: the far service (its own relay host, exposed as a sink).
        let mut host_c = RelayHost::new();
        host_c.mount(
            "enrollment.mint",
            Arc::new(|params| async move { Ok(json!({ "minted": true, "echo": params })) }.boxed()),
        );
        let sink_c: Arc<dyn RpcSink> = Arc::new(ServiceC);

        // B: forwards whitelisted namespaces at C.
        let mut host_b = RelayHost::new();
        host_b
            .forwarding_table_mut()
            .allow("enrollment", "C")
            .unwrap();
        let far_b = Arc::new(RelayFarEnd {
            host: host_b,
            sink: Arc::clone(&sink_c),
            guard: HopGuard::default(),
            forwarded: Mutex::new(0),
        });

        // A: memory transport into B.
        let transport = MemoryTransport::new(Arc::clone(&far_b) as Arc<dyn EdgeFarEnd>);

        let result = transport
            .call(EdgeFrame {
                method: "enrollment.mint".into(),
                params: Some(json!({ "ttl": 30 })),
                relay: Some(RelayContext {
                    hops: 0,
                    via: vec![],
                }),
            })
            .await
            .expect("chain call succeeds");
        assert_eq!(result["minted"], json!(true));
        assert_eq!(result["hops"], json!(1), "exactly one hop stepped");
        assert_eq!(*far_b.forwarded.lock().unwrap(), 1);

        // Hop-limit refusal: a frame already at the limit never reaches C.
        let refused = transport
            .call(EdgeFrame {
                method: "enrollment.mint".into(),
                params: None,
                relay: Some(RelayContext {
                    hops: 4,
                    via: vec![],
                }),
            })
            .await;
        assert!(refused.is_err(), "hop limit must refuse the forward");
        assert_eq!(*far_b.forwarded.lock().unwrap(), 1, "no extra forward");
    }

    /// Unrouted frames deny with -32601 semantics end to end.
    #[tokio::test]
    async fn unrouted_methods_deny() {
        let mut host_b = RelayHost::new();
        host_b
            .forwarding_table_mut()
            .allow("enrollment", "C")
            .unwrap();
        let sink_c: Arc<dyn RpcSink> = Arc::new(ServiceC);
        let far_b = Arc::new(RelayFarEnd {
            host: host_b,
            sink: sink_c,
            guard: HopGuard::default(),
            forwarded: Mutex::new(0),
        });
        let transport = MemoryTransport::new(far_b as Arc<dyn EdgeFarEnd>);
        let denied = transport
            .call(EdgeFrame {
                method: "rescue.open_session".into(),
                params: None,
                relay: None,
            })
            .await;
        match denied {
            Err(BridgeError::MethodNotFound(method)) => {
                assert_eq!(method, "rescue.open_session");
            }
            other => panic!("expected MethodNotFound denial, got {other:?}"),
        }
    }
}
