//! Umbrella surface compatibility contract.
//!
//! This integration test is compiled as an *external* crate, so it proves
//! the paths consumers of the `plana` umbrella rely on keep resolving:
//! merged `http`/`enums` modules, the `jsonrpc` framing surface, the RBAC
//! and protocol modules, and the re-exported `namespace!` macro.
//!
//! Mostly compile-only (empty `resolves::<T>()` calls); the small runtime
//! assertions guard the macro-expansion semantics of a local namespace.

// `namespace!` is `#[macro_export]`ed at the crate root; after the JSON-RPC
// layer was absorbed into this crate the umbrella path
// `plana::jsonrpc::namespace` no longer exists (the doc comment in
// `jsonrpc::pending` predates the absorb), so this external consumer — the
// exact case that doc comment is about — resolves it from the root.
use plana::jsonrpc::MessageKind;
use plana::namespace;
use strum::{Display, EnumIter, EnumString};

namespace!(
    "Surface",
    Surface,
    SurfaceMethod,
    Ping as SyncReq => Pong,
    Pong as OneWay,
);

fn resolves<T>() {}

#[test]
fn merged_http_module_carries_generic_descriptors() {
    // Only the generic health/status descriptors live here. The domain DTOs
    // the former umbrella re-exported under this module (`AgentItem`,
    // `ModelInfo`, `TierDefinition`, …) are domain-profile types: they live in
    // `plana-celestia-types`, which pins them in its own surface test.
    resolves::<plana::http::HealthResponse>();
    resolves::<plana::http::NetworkInfo>();
    resolves::<plana::http::BackendKind>();
    resolves::<plana::http::ServiceStatus>();
}

#[test]
fn merged_enums_module_carries_connection_topology() {
    resolves::<plana::enums::ConnectionType>();
}

#[test]
fn jsonrpc_surface_resolves() {
    resolves::<plana::jsonrpc::RpcMethodMap>();
    resolves::<plana::jsonrpc::Method>();
    resolves::<plana::jsonrpc::MessageKind>();
    resolves::<plana::jsonrpc::JsonRpcRequest>();
    resolves::<plana::jsonrpc::JsonRpcResponse>();
    resolves::<plana::jsonrpc::JsonRpcError>();
}

#[test]
fn namespace_macro_is_invocable_from_an_external_crate() {
    // The local `Surface` namespace below is declared with the macro
    // re-exported from the umbrella (`plana::jsonrpc::namespace`), which
    // re-exports `plana_jsonrpc::namespace`.
    assert_eq!(SurfaceMethod::Ping.wire(), "Surface.Ping");
    assert_eq!(SurfaceMethod::Pong.wire(), "Surface.Pong");
    assert_eq!(SurfaceMethod::Ping.kind(), MessageKind::SyncReq);
    assert_eq!(SurfaceMethod::Pong.kind(), MessageKind::OneWay);
    assert!(!SurfaceMethod::Ping.is_one_way());
    assert!(SurfaceMethod::Pong.is_one_way());
    assert_eq!(SurfaceMethod::Ping.response(), Some(SurfaceMethod::Pong));
    assert_eq!(SurfaceMethod::Pong.response(), None);
}

#[test]
fn rpc_method_map_registers_arbitrary_method_names() {
    // Third-party extension path: string-keyed dynamic dispatch, no `Method`
    // enum variant required.
    let methods = plana::jsonrpc::RpcMethodMap::empty().method(
        "my.domain.op",
        |params: serde_json::Value| async move {
            Ok(serde_json::json!({ "echo": params["text"].clone() }))
        },
    );
    let resp = futures::executor::block_on(
        methods.dispatch(Some(serde_json::json!({ "text": "hi" })), "my.domain.op"),
    );
    assert_eq!(resp.result, Some(serde_json::json!({ "echo": "hi" })));
    let missing = futures::executor::block_on(methods.dispatch(None, "nope"));
    assert_eq!(missing.error.as_ref().map(|e| e.code), Some(-32601));
}

#[test]
fn rbac_module_resolves() {
    resolves::<plana::rbac::Permission>();
    resolves::<plana::rbac::PermissionScope>();
    resolves::<plana::rbac::Role>();
    let _ = plana::rbac::Permission::SystemAdmin.as_str();
}

#[cfg(feature = "tracing-helpers")]
#[test]
fn tracing_helpers_forwarded_through_umbrella() {
    // Consumer pattern (shittim-chest): `plana` with the `tracing-helpers`
    // feature, using `plana::tracing_helpers::ShortTimer`.
    resolves::<plana::tracing_helpers::ShortTimer>();
    let _ = plana::tracing_helpers::ShortTimer;
}

#[test]
fn protocol_module_resolves() {
    // The generic JSON-RPC envelope has a single canonical definition in
    // plana-jsonrpc (re-exported as `plana::jsonrpc`); `plana::protocol`
    // carries only base messages and handshake primitives.
    resolves::<plana::jsonrpc::JsonRpcError>();
    resolves::<plana::protocol::base_messages::BaseHeartbeatParams>();
    resolves::<plana::protocol::handshake::HandshakeAckParams>();
}

#[test]
fn root_health_response_is_the_generic_one() {
    resolves::<plana::HealthResponse>();
    // `plana::http::HealthResponse` and the root one are the same generic
    // type; the malkuth supervision `HealthResponse` is a different type.
    let _ = plana::HealthResponse::ok(
        "1.0.0",
        plana::http::BackendKind::Dev,
        1,
        plana::http::NetworkInfo::unknown(),
    );
}

#[test]
fn root_handshake_primitives_resolve() {
    // Generic core primitives. The scepter-flavored capability payloads the
    // former umbrella re-exported here (`ClientCapability`,
    // `ConnectHandshakeParams`) are domain-profile types: they live in
    // `plana-celestia-types`, which pins them in its own surface test.
    resolves::<plana::HandshakeAckParams>();
    let _ = plana::HANDSHAKE_VERSION;
}

#[test]
fn identity_machine_fingerprint_resolves() {
    let _: fn() -> Option<String> = plana::identity::machine_fingerprint;
}

#[test]
fn jsonrpc_framing_surface_resolves() {
    resolves::<plana::jsonrpc::Id>();
    resolves::<plana::jsonrpc::JsonRpcNotification>();
    let _ = plana::jsonrpc::JSONRPC_VERSION;
    // The generic JSON-RPC 2.0 error code constants stay reachable through
    // the framing surface (consumers match on them in dispatch handlers):
    let _ = plana::jsonrpc::error_codes::PARSE_ERROR;
    let _ = plana::jsonrpc::error_codes::INVALID_REQUEST;
    let _ = plana::jsonrpc::error_codes::METHOD_NOT_FOUND;
    let _ = plana::jsonrpc::error_codes::INVALID_PARAMS;
    let _ = plana::jsonrpc::error_codes::INTERNAL_ERROR;
    // The asserted AUTH_ERROR is the plana-jsonrpc copy of the error-code
    // table (arona matches on it in ~25 auth-guard dispatch sites). The
    // platform-specific (-32000 range) codes live in that same canonical
    // table, and `plana-celestia-types` re-exports this very entry as
    // `protocol::jsonrpc::error_codes::SNAPSHOT_FAILED` (pinned in that
    // crate's own surface test), so the value is asserted here too.
    let _ = plana::jsonrpc::error_codes::AUTH_ERROR;
    assert_eq!(plana::jsonrpc::error_codes::AUTH_ERROR, -32005);
    assert_eq!(plana::jsonrpc::error_codes::SNAPSHOT_FAILED, -32001);
}

#[test]
fn jsonrpc_session_surface_resolves() {
    resolves::<plana::jsonrpc::session::SessionManager>();
    // sse_events_handler_impl returns `Sse<impl Stream<...>>`, so assert
    // resolvability rather than naming the fn-pointer type.
    let _ = plana::jsonrpc::session::sse_events_handler_impl;
}

#[test]
fn jsonrpc_method_sync_constructs_via_inner_and_flat_alias() {
    // Direct variant construction against the paste-generated inner enum:
    let m = plana::jsonrpc::Method::Sync(plana::jsonrpc::pending::SyncMethod::Ping);
    assert_eq!(m.method_name(), "Sync.Ping");
    // Paste-generated flat alias on `Method`:
    let flat = plana::jsonrpc::Method::SyncPing;
    assert_eq!(flat.method_name(), "Sync.Ping");
    assert_eq!(m.kind(), plana::jsonrpc::MessageKind::SyncReq);
}

#[test]
fn jsonrpc_flat_method_aliases_cover_the_consumer_namespaces() {
    // The paste-generated flat aliases are the ergonomic path shittim-chest
    // uses for ~70 built-in method families; assert a representative set
    // covering every namespace (Sync, Cli, Tool, Skill, Base, Device,
    // Screen) keeps resolving, with the same wire names as the inner enums.
    let agent_chunk_count = plana::jsonrpc::Method::SyncAgentChunkCount;
    assert_eq!(agent_chunk_count.method_name(), "Sync.AgentChunkCount");
    let base_heartbeat = plana::jsonrpc::Method::BaseHeartbeat;
    assert_eq!(base_heartbeat.method_name(), "Base.Heartbeat");
    let device_terminal_open = plana::jsonrpc::Method::DeviceTerminalOpen;
    assert_eq!(device_terminal_open.method_name(), "Device.TerminalOpen");
    let cli_status = plana::jsonrpc::Method::CliStatus;
    assert_eq!(cli_status.method_name(), "Cli.Status");
    let tool_list_tools = plana::jsonrpc::Method::ToolListTools;
    assert_eq!(tool_list_tools.method_name(), "Tool.ListTools");
    let skill_call = plana::jsonrpc::Method::SkillCallSkill;
    assert_eq!(skill_call.method_name(), "Skill.CallSkill");
    let skill_chain_start = plana::jsonrpc::Method::SyncSkillChainStart;
    assert_eq!(skill_chain_start.method_name(), "Sync.SkillChainStart");
    let screen_ice_candidate = plana::jsonrpc::Method::ScreenIceCandidate;
    assert_eq!(screen_ice_candidate.method_name(), "Screen.IceCandidate");
    let yolo_start = plana::jsonrpc::Method::SyncYoloStart;
    assert_eq!(yolo_start.method_name(), "Sync.YoloStart");
    assert_eq!(yolo_start.kind(), plana::jsonrpc::MessageKind::AsyncReq);
    // The flat aliases are `Method` values, not distinct types: the inner
    // and flat forms name the same enum instance on the wire.
    assert_eq!(
        plana::jsonrpc::Method::Device(plana::jsonrpc::pending::DeviceMethod::TerminalOpen)
            .method_name(),
        device_terminal_open.method_name()
    );
}

#[cfg(feature = "rpc-server")]
#[test]
fn rpc_server_module_surface_resolves() {
    // Consumer pattern (arona gateway): `plana` with the `rpc-server`
    // feature, using `rpc_server::detect_network` for request
    // transport/geo metadata. Per-client SSE transport sessions live in
    // `plana::jsonrpc::session` (see the session semantic boundary note).
    let _ = plana::rpc_server::detect_network;
    // `sse` module: keep-alive heartbeat constants for SSE event streams.
    let _ = plana::rpc_server::sse::SSE_HEARTBEAT_INTERVAL_SECS;
    let _ = plana::rpc_server::sse::SSE_CONNECTED_COMMENT;
    let _ = plana::rpc_server::sse::SSE_HEARTBEAT_COMMENT;
}
