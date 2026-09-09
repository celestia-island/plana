//! The two fixed transport functions as Tauri commands/events.
//!
//! `bridge_call` is THE request/response function; `bridge_event` is the
//! backend→webview notification lane (emitted on the app handle with
//! that name, payload `{method, params}`). The webview side ships in
//! `@celestia-island/plana-rpc-client` (TS) as a matching client over
//! Tauri v2's global bridge — apps never invent another transport
//! surface.

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};

use plana_rpc_client::relay::Route;
use plana_rpc_client::relay::hops::{HopGuard, RelayContext};
use plana_rpc_client::relay::host::{BridgeError, BridgeOutcome, RelayHost, RpcSink};
use plana_rpc_client::relay::proxy::{self, ProxyConfig, ProxyDecision};

use crate::battery::BatteryHooks;

/// Managed state every relay Tauri app installs once at setup.
pub struct RelayState {
    pub host: std::sync::RwLock<RelayHost>,
    pub sink: Arc<dyn RpcSink>,
    pub guard: HopGuard,
}

impl RelayState {
    pub fn new(host: RelayHost, sink: Arc<dyn RpcSink>) -> Self {
        Self {
            host: std::sync::RwLock::new(host),
            sink,
            guard: HopGuard::default(),
        }
    }
}

/// Tauri-backed battery hooks: window controls ride the app handle's
/// main webview window; proxy persistence delegates to closures so the
/// app keeps its own storage layer.
/// Persistence closure for the proxy override (the app's storage layer).
pub type ProxyStore = Arc<dyn Fn(Option<&ProxyConfig>) -> Result<(), String> + Send + Sync>;
/// Load closure returning the persisted proxy override, if any.
pub type ProxyLoad = Arc<dyn Fn() -> Option<ProxyConfig> + Send + Sync>;

pub struct TauriHooks {
    pub app: AppHandle,
    pub proxy_store: ProxyStore,
    pub proxy_load: ProxyLoad,
}

impl BatteryHooks for TauriHooks {
    fn persist_proxy(&self, config: Option<&ProxyConfig>) -> Result<(), String> {
        (self.proxy_store)(config)
    }

    fn proxy_decision(&self) -> ProxyDecision {
        proxy::resolve((self.proxy_load)().as_ref())
    }

    fn window_minimize(&self) -> Result<(), String> {
        self.main_window(|w| w.minimize().map_err(|e| e.to_string()))
    }

    fn window_maximize_toggle(&self) -> Result<(), String> {
        self.main_window(|w| {
            if w.is_maximized().unwrap_or(false) {
                w.unmaximize()
            } else {
                w.maximize()
            }
            .map_err(|e| e.to_string())
        })
    }

    fn window_close(&self) -> Result<(), String> {
        self.main_window(|w| w.close().map_err(|e| e.to_string()))
    }

    fn window_state(&self) -> Result<Value, String> {
        self.main_window(|w| {
            Ok(serde_json::json!({
                "label": w.label(),
                "maximized": w.is_maximized().unwrap_or(false),
            }))
        })
    }
}

impl TauriHooks {
    fn main_window<T>(
        &self,
        f: impl FnOnce(&tauri::WebviewWindow) -> Result<T, String>,
    ) -> Result<T, String> {
        let window = self
            .app
            .get_webview_window("main")
            .ok_or("main webview window not found")?;
        f(&window)
    }
}

/// The one request/response bridge function. Everything the webview can
/// ask of the backend — system control AND forwarded service calls —
/// rides this single Tauri command.
#[tauri::command]
pub async fn bridge_call(
    state: State<'_, RelayState>,
    method: String,
    params: Option<Value>,
    relay: Option<RelayContext>,
) -> Result<Value, String> {
    let context = relay.unwrap_or_default();
    // Snapshot the host (cheap: Arc handlers) so no lock is held across
    // the await — a concurrently-running relay.fwd.map write would
    // otherwise deadlock against a held read guard.
    let host = state.host.read().expect("relay host poisoned").clone();
    // Loop guard fires before any forward (local frames the edge itself
    // originated are always admissible).
    if matches!(host.forwarding_table().route(&method), Route::Forward(_))
        && context.hops >= state.guard.max_hops
    {
        return Err(format!(
            "relay hop limit reached ({} >= {}); refusing to forward — possible loop",
            context.hops, state.guard.max_hops
        ));
    }
    dispatch_dyn(&host, state.sink.as_ref(), &method, params)
        .await
        .map(|outcome| match outcome {
            BridgeOutcome::Local(v) | BridgeOutcome::Forwarded { result: v, .. } => v,
        })
        .map_err(|e| e.to_string())
}

/// Dispatch through the trait object (the generic host API wants `Sized`;
/// one monomorphization over the dyn adapter closes the gap).
async fn dispatch_dyn(
    host: &RelayHost,
    sink: &dyn RpcSink,
    method: &str,
    params: Option<Value>,
) -> Result<BridgeOutcome, BridgeError> {
    host.dispatch(&DynSink(sink), method, params).await
}

struct DynSink<'a>(&'a dyn RpcSink);

impl RpcSink for DynSink<'_> {
    fn call(
        &self,
        endpoint: &str,
        method: &str,
        params: Option<Value>,
    ) -> futures::future::BoxFuture<'_, Result<Value, BridgeError>> {
        self.0.call(endpoint, method, params)
    }
}

/// Emit one notification onto the `bridge_event` lane.
pub fn emit_event(app: &AppHandle, method: &str, params: Option<Value>) {
    let _ = app.emit(
        "bridge_event",
        serde_json::json!({ "method": method, "params": params }),
    );
}

/// Map a dispatch failure to the standard JSON-RPC error shape the
/// webview client expects (kept public for adapters that answer over a
/// different edge transport).
pub fn error_payload(error: &BridgeError) -> Value {
    serde_json::json!({
        "code": -32601,
        "message": error.to_string(),
    })
}
