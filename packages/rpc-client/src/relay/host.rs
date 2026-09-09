//! The relay-host dispatch core behind the two fixed bridge functions.
//!
//! [`Bridge`] owns the local `tauri.*` (and app-local) handler map plus
//! the forwarding table; [`RpcSink`] is the app-supplied transport that
//! turns a `Forward` route into a plana service-profile call. The webview
//! side of the two functions is a per-app adapter; this core is pure and
//! testable without a webview.

use std::collections::HashMap;
use std::sync::Arc;

use futures::future::BoxFuture;
use serde_json::Value;

use super::router::{ForwardingTable, Route};

/// A boxed local handler: params in, result out.
pub type LocalHandler =
    Arc<dyn Fn(Option<Value>) -> BoxFuture<'static, Result<Value, BridgeError>> + Send + Sync>;

/// The app-supplied side of forwarding: carry one request to the routed
/// endpoint (a connection opened via `tauri.conn.open`, dialed through
/// the resolved [`crate::ProxyDecision`]).
pub trait RpcSink: Send + Sync {
    fn call(
        &self,
        endpoint: &str,
        method: &str,
        params: Option<Value>,
    ) -> BoxFuture<'_, Result<Value, BridgeError>>;
}

/// Bridge-level errors; the `-32601` shape pairs with `Route::Deny`.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum BridgeError {
    #[error("method not found: no handler and no forwarding entry for '{0}'")]
    MethodNotFound(String),
    #[error("endpoint '{0}' has no open connection")]
    NoConnection(String),
    #[error("local handler for '{0}' failed: {1}")]
    Local(String, String),
    #[error("forwarded call to '{endpoint}' failed: {detail}")]
    Forward { endpoint: String, detail: String },
}

/// One dispatched request's outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum BridgeOutcome {
    /// A local handler produced this result.
    Local(Value),
    /// The sink carried the call; the server produced this result.
    Forwarded { endpoint: String, result: Value },
}

/// The routing + dispatch core. Cloning takes a cheap snapshot (the
/// handler map is Arc-per-entry) — the bridge clones before awaiting so
/// no lock is held across an await point.
#[derive(Clone)]
pub struct RelayHost {
    handlers: HashMap<String, LocalHandler>,
    table: ForwardingTable,
}

impl RelayHost {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            table: ForwardingTable::new(),
        }
    }

    /// Mount a local handler. Mounting anything under the reserved
    /// `tauri.` namespace requires going through [`Self::mount_system`]
    /// so system-control surface stays visually distinct in code review.
    pub fn mount(&mut self, method: &str, handler: LocalHandler) {
        self.handlers.insert(method.to_string(), handler);
    }

    /// Mount one of the reserved `relay.*` system-control handlers
    /// (`super::names::relay_methods` carries the canonical names).
    /// Refuses names outside the reserved namespace.
    pub fn mount_system(&mut self, method: &str, handler: LocalHandler) -> Result<(), String> {
        if super::names::namespace_of(method) != super::names::RELAY {
            let relay_ns = super::names::RELAY;
            return Err(format!("'{method}' is not a '{relay_ns}.*' system method"));
        }
        self.mount(method, handler);
        Ok(())
    }

    /// Read-only view of the forwarding table (backs `tauri.fwd.list`).
    pub fn forwarding_table(&self) -> &ForwardingTable {
        &self.table
    }

    /// Mutable view for the `tauri.fwd.map` handler implementation.
    pub fn forwarding_table_mut(&mut self) -> &mut ForwardingTable {
        &mut self.table
    }

    /// Dispatch one JSON-RPC request.
    pub async fn dispatch<S: RpcSink>(
        &self,
        sink: &S,
        method: &str,
        params: Option<Value>,
    ) -> Result<BridgeOutcome, BridgeError> {
        match self.table.route(method) {
            Route::Local => {
                let handler = self
                    .handlers
                    .get(method)
                    .ok_or_else(|| BridgeError::MethodNotFound(method.to_string()))?;
                handler(params)
                    .await
                    .map(BridgeOutcome::Local)
                    .map_err(|e| BridgeError::Local(method.to_string(), e.to_string()))
            }
            Route::Forward(endpoint) => sink
                .call(&endpoint, method, params)
                .await
                .map(|result| BridgeOutcome::Forwarded {
                    endpoint: endpoint.clone(),
                    result,
                })
                .map_err(|e| BridgeError::Forward {
                    endpoint,
                    detail: e.to_string(),
                }),
            Route::Deny => Err(BridgeError::MethodNotFound(method.to_string())),
        }
    }
}

impl Default for RelayHost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::names::relay_methods;
    use super::*;
    use futures::FutureExt;
    use std::sync::Mutex;

    struct RecordingSink {
        calls: Mutex<Vec<(String, String)>>,
    }

    impl RpcSink for RecordingSink {
        fn call(
            &self,
            endpoint: &str,
            method: &str,
            _params: Option<Value>,
        ) -> BoxFuture<'_, Result<Value, BridgeError>> {
            self.calls
                .lock()
                .unwrap()
                .push((endpoint.to_string(), method.to_string()));
            let echo = method.to_string();
            async move { Ok(serde_json::json!({ "echo": echo })) }.boxed()
        }
    }

    fn ok_handler(value: Value) -> LocalHandler {
        let value = Arc::new(value);
        Arc::new(move |_params| {
            let value = Arc::clone(&value);
            async move { Ok((*value).clone()) }.boxed()
        })
    }

    #[tokio::test]
    async fn system_methods_run_locally_and_cannot_be_forwarded() {
        let mut bridge = RelayHost::new();
        bridge
            .mount_system(
                relay_methods::NET_SET_PROXY,
                ok_handler(serde_json::json!("done")),
            )
            .unwrap();
        bridge.forwarding_table_mut().allow("net", "evil").unwrap();

        let sink = RecordingSink {
            calls: Mutex::new(vec![]),
        };
        let out = bridge
            .dispatch(&sink, relay_methods::NET_SET_PROXY, None)
            .await
            .unwrap();
        assert_eq!(out, BridgeOutcome::Local(serde_json::json!("done")));
        assert!(sink.calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn whitelisted_namespaces_forward_and_unknown_deny() {
        let mut bridge = RelayHost::new();
        bridge
            .forwarding_table_mut()
            .allow("enrollment", "gateway")
            .unwrap();
        let sink = RecordingSink {
            calls: Mutex::new(vec![]),
        };

        let out = bridge
            .dispatch(
                &sink,
                "enrollment.mint",
                Some(serde_json::json!({"ttl": 30})),
            )
            .await
            .unwrap();
        assert_eq!(
            out,
            BridgeOutcome::Forwarded {
                endpoint: "gateway".into(),
                result: serde_json::json!({ "echo": "enrollment.mint" })
            }
        );

        let err = bridge.dispatch(&sink, "rescue.open_session", None).await;
        assert_eq!(
            err.unwrap_err(),
            BridgeError::MethodNotFound("rescue.open_session".into())
        );
    }

    #[tokio::test]
    async fn unmounted_system_method_is_method_not_found() {
        let bridge = RelayHost::new();
        let sink = RecordingSink {
            calls: Mutex::new(vec![]),
        };
        let err = bridge.dispatch(&sink, "tauri.fwd.list", None).await;
        assert!(matches!(err, Err(BridgeError::MethodNotFound(_))));
    }
}
