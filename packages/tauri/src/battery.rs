//! The standard `relay.*` command battery.
//!
//! [`mount_battery`] installs the pre-standardized system handlers onto
//! a [`RelayHost`]. Every handler is optional and individually
//! re-mountable: mounting is just registration, so an app can override
//! one standard method with its own behavior by mounting after the
//! battery (latest mount wins).

use std::sync::Arc;

use futures::future::BoxFuture;
use serde_json::Value;

use plana_rpc_client::relay::host::{BridgeError, LocalHandler, RelayHost};
use plana_rpc_client::relay::names::relay_methods as methods;
use plana_rpc_client::relay::proxy::{ProxyConfig, ProxyDecision};

/// Host-side hooks the battery needs: window operations (the framework
/// adapter implements these — Tauri's webview window, an egui host's
/// own windowing) and a settings sink for the persistent proxy value.
pub trait BatteryHooks: Send + Sync {
    /// Persist the proxy override (the storage layer is the app's —
    /// a toml file, a keychain, whatever); `None` clears it.
    fn persist_proxy(&self, config: Option<&ProxyConfig>) -> Result<(), String>;

    /// Current effective proxy decision (persisted override resolved
    /// through [`proxy::resolve`]).
    fn proxy_decision(&self) -> ProxyDecision;

    #[cfg(feature = "bridge")]
    fn window_minimize(&self) -> Result<(), String> {
        Err("window controls need the bridge feature".into())
    }
    #[cfg(feature = "bridge")]
    fn window_maximize_toggle(&self) -> Result<(), String> {
        Err("window controls need the bridge feature".into())
    }
    #[cfg(feature = "bridge")]
    fn window_close(&self) -> Result<(), String> {
        Err("window controls need the bridge feature".into())
    }
    #[cfg(feature = "bridge")]
    fn window_state(&self) -> Result<Value, String> {
        Err("window controls need the bridge feature".into())
    }
}

fn simple<F>(f: F) -> LocalHandler
where
    F: Fn(Option<Value>) -> Result<Value, String> + Send + Sync + 'static,
{
    Arc::new(move |params| {
        let result = f(params);
        Box::pin(async move { result.map_err(|e| BridgeError::Local("battery".into(), e)) })
            as BoxFuture<'static, Result<Value, BridgeError>>
    })
}

/// Mount the standard battery. Returns the host for chaining.
///
/// The window handlers are mounted only with the `bridge` feature; on a
/// featureless build the window methods stay unmounted (and therefore
/// answer `-32601`), keeping the surface honest about what the host can
/// actually do.
pub fn mount_battery(host: &mut RelayHost, hooks: Arc<dyn BatteryHooks>) -> &mut RelayHost {
    // relay.net.set_proxy — {scheme, host, username?, password?}; an
    // empty host installs the explicitly-direct decision.
    let h = Arc::clone(&hooks);
    host.mount_system(
        methods::NET_SET_PROXY,
        simple(move |params| {
            let config: Option<ProxyConfig> = params
                .map(|p| serde_json::from_value(p).map_err(|e| e.to_string()))
                .transpose()?;
            h.persist_proxy(config.as_ref())?;
            serde_json::to_value(decision_display(&h.proxy_decision())).map_err(|e| e.to_string())
        }),
    )
    .expect("standard method name");

    // relay.net.configure_entrypoint — the app persists the entrypoint
    // itself; the battery only fixes the method shape {url}.
    let h = Arc::clone(&hooks);
    host.mount_system(
        methods::NET_CONFIGURE_ENTRYPOINT,
        simple(move |params| {
            let url = params
                .and_then(|p| p.get("url").and_then(Value::as_str).map(str::to_string))
                .ok_or("params.url is required")?;
            if !url.starts_with("https://") && !url.starts_with("http://") {
                return Err("entrypoint must be an http(s):// URL".into());
            }
            // The standard login handshake target is app policy; the
            // battery records acceptance and lets the app's own hook
            // layer persist (mount over this handler to customize).
            let _ = &h;
            Ok(Value::String(url))
        }),
    )
    .expect("standard method name");

    #[cfg(feature = "bridge")]
    {
        let h = Arc::clone(&hooks);
        host.mount_system(
            methods::WINDOW_MINIMIZE,
            simple(move |_| h.window_minimize().map(|()| Value::Null)),
        )
        .expect("standard method name");
        let h = Arc::clone(&hooks);
        host.mount_system(
            methods::WINDOW_MAXIMIZE_TOGGLE,
            simple(move |_| h.window_maximize_toggle().map(|()| Value::Null)),
        )
        .expect("standard method name");
        let h = Arc::clone(&hooks);
        host.mount_system(
            methods::WINDOW_CLOSE,
            simple(move |_| h.window_close().map(|()| Value::Null)),
        )
        .expect("standard method name");
        let h = Arc::clone(&hooks);
        host.mount_system(methods::WINDOW_STATE, simple(move |_| h.window_state()))
            .expect("standard method name");
    }

    host
}

fn decision_display(decision: &ProxyDecision) -> Value {
    match decision {
        ProxyDecision::Direct => serde_json::json!({ "mode": "direct" }),
        ProxyDecision::Through(config) => serde_json::json!({
            "mode": "proxy",
            "scheme": config.scheme,
            "host": config.host,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plana_rpc_client::relay::host::{BridgeOutcome, RpcSink};
    use std::sync::Mutex;

    /// Test hooks mirroring a real app: persist, then resolve the
    /// decision FROM the persisted value.
    struct NullHooks {
        persisted: Mutex<Option<String>>,
        store: Mutex<Option<ProxyConfig>>,
    }

    impl BatteryHooks for NullHooks {
        fn persist_proxy(&self, config: Option<&ProxyConfig>) -> Result<(), String> {
            *self.persisted.lock().unwrap() = config.map(|c| c.url());
            *self.store.lock().unwrap() = config.cloned();
            Ok(())
        }
        fn proxy_decision(&self) -> ProxyDecision {
            plana_rpc_client::relay::proxy::resolve(self.store.lock().unwrap().as_ref())
        }
    }

    struct NoSink;
    impl RpcSink for NoSink {
        fn call(
            &self,
            _endpoint: &str,
            _method: &str,
            _params: Option<Value>,
        ) -> BoxFuture<'_, Result<Value, BridgeError>> {
            Box::pin(async { Err(BridgeError::NoConnection("none".into())) })
        }
    }

    #[tokio::test]
    async fn set_proxy_persists_and_reports() {
        let mut host = RelayHost::new();
        let hooks = Arc::new(NullHooks {
            persisted: Mutex::new(None),
            store: Mutex::new(None),
        });
        mount_battery(&mut host, Arc::clone(&hooks) as Arc<dyn BatteryHooks>);

        let sink = NoSink;
        let out = host
            .dispatch(
                &sink,
                methods::NET_SET_PROXY,
                Some(serde_json::json!({
                    "scheme": "socks5",
                    "host": "127.0.0.1:7890"
                })),
            )
            .await
            .unwrap();
        match out {
            BridgeOutcome::Local(v) => {
                assert_eq!(v["mode"], "proxy");
                assert_eq!(v["host"], "127.0.0.1:7890");
            }
            other => panic!("expected local outcome, got {other:?}"),
        }
        assert_eq!(
            hooks.persisted.lock().unwrap().as_deref(),
            Some("socks5://127.0.0.1:7890")
        );
    }

    #[tokio::test]
    async fn entrypoint_requires_url() {
        let mut host = RelayHost::new();
        mount_battery(
            &mut host,
            Arc::new(NullHooks {
                persisted: Mutex::new(None),
                store: Mutex::new(None),
            }),
        );
        let err = host
            .dispatch(&NoSink, methods::NET_CONFIGURE_ENTRYPOINT, None)
            .await;
        assert!(err.is_err());
    }
}
