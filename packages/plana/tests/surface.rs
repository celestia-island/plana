//! Umbrella surface compatibility contract.
//!
//! This integration test is compiled as an *external* crate, so it proves
//! the paths consumers of the `plana` umbrella rely on keep resolving:
//! merged `http`/`enums` modules, the `jsonrpc` framing surface, the RBAC
//! and protocol modules, and the re-exported `namespace!` macro.
//!
//! Mostly compile-only (empty `resolves::<T>()` calls); the small runtime
//! assertions guard the macro-expansion semantics of a local namespace.

use plana::jsonrpc::{namespace, MessageKind};
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
fn merged_http_module_carries_generic_and_domain_types() {
    resolves::<plana::http::HealthResponse>();
    resolves::<plana::http::NetworkInfo>();
    resolves::<plana::http::BackendKind>();
    resolves::<plana::http::ServiceStatus>();
    #[cfg(feature = "celestia")]
    resolves::<plana::http::AgentItem>();
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
    resolves::<plana::protocol::jsonrpc::JsonRpcError>();
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
fn root_handshake_and_capability_surface_resolves() {
    // Generic core primitives (always available):
    resolves::<plana::HandshakeAckParams>();
    let _ = plana::HANDSHAKE_VERSION;
    // Domain capability payload (celestia feature):
    #[cfg(feature = "celestia")]
    {
        resolves::<plana::ClientCapability>();
        resolves::<plana::ConnectHandshakeParams>();
    }
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
    // table (arona matches on it in ~25 auth-guard dispatch sites); the
    // profile crate's parallel copy is touched below via SNAPSHOT_FAILED in
    // the celestia module.
    let _ = plana::jsonrpc::error_codes::AUTH_ERROR;
    assert_eq!(plana::jsonrpc::error_codes::AUTH_ERROR, -32005);
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
    // covering every namespace (Sync, Cli, Mcp, Skill, Base, Device,
    // Screen) keeps resolving, with the same wire names as the inner enums.
    let agent_chunk_count = plana::jsonrpc::Method::SyncAgentChunkCount;
    assert_eq!(agent_chunk_count.method_name(), "Sync.AgentChunkCount");
    let base_heartbeat = plana::jsonrpc::Method::BaseHeartbeat;
    assert_eq!(base_heartbeat.method_name(), "Base.Heartbeat");
    let device_terminal_open = plana::jsonrpc::Method::DeviceTerminalOpen;
    assert_eq!(device_terminal_open.method_name(), "Device.TerminalOpen");
    let cli_status = plana::jsonrpc::Method::CliStatus;
    assert_eq!(cli_status.method_name(), "Cli.Status");
    let mcp_list_tools = plana::jsonrpc::Method::McpListTools;
    assert_eq!(mcp_list_tools.method_name(), "Mcp.ListTools");
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
    // transport/geo metadata and `SessionManager` for per-client sessions.
    resolves::<plana::rpc_server::SessionManager>();
    let _ = plana::rpc_server::detect_network;
    // `sse` module: keep-alive heartbeat constants for SSE event streams.
    let _ = plana::rpc_server::sse::SSE_HEARTBEAT_INTERVAL_SECS;
    let _ = plana::rpc_server::sse::SSE_CONNECTED_COMMENT;
    let _ = plana::rpc_server::sse::SSE_HEARTBEAT_COMMENT;
}

#[cfg(feature = "celestia")]
mod celestia_surface {
    use super::*;

    #[test]
    fn celestia_domain_types_resolve() {
        resolves::<plana::celestia::Agent>();
        resolves::<plana::celestia::TaskStatus>();
        resolves::<plana::celestia::malkuth::WorkerStatus>();
        resolves::<plana::celestia::http::AgentItem>();
        resolves::<plana::celestia::protocol::handshake::ConnectHandshakeParams>();
        let _ = plana::celestia::protocol::jsonrpc::error_codes::SNAPSHOT_FAILED;
        // The full domain surface is also reachable at the crate root via
        // the `celestia` glob (feature-gated in the umbrella).
        resolves::<plana::Agent>();
    }

    #[test]
    fn celestia_http_domain_types_resolve_at_root() {
        // Live consumer paths (arona / e-world admin panels):
        resolves::<plana::http::ModelInfo>();
        resolves::<plana::http::ProviderPublic>();
        resolves::<plana::http::TierDefinition>();
        resolves::<plana::http::UserPreferences>();
    }

    #[test]
    fn celestia_enums_used_by_entelecheia_resolve() {
        // entelecheia consumes the annotation and file-operation vocabulary
        // through `plana::enums` (merged core + domain enums module):
        resolves::<plana::enums::AnnotationType>();
        resolves::<plana::enums::FileOperationType>();
        resolves::<plana::enums::ObservationType>();
        let _ = plana::enums::AnnotationType::Todo.as_str();
        let _ = plana::enums::FileOperationType::Reading.as_str();
        let _ = plana::enums::ObservationType::Watching.as_str();
    }

    #[test]
    fn celestia_engine_extended_surface_resolves() {
        // arona consumes the embeddings / invoke-start / stats vocabulary
        // at `plana::engine`; construct with the real field sets to pin the
        // wire shapes.
        let _emb = plana::engine::EngineEmbeddingsParams {
            model: "m".into(),
            input: vec!["text".into()],
        };
        let _start = plana::engine::EngineInvokeStartResult {
            ok: true,
            error: None,
            stream_id: "s-1".into(),
        };
        let _stats = plana::engine::EngineStatsResult {
            gpu_utilization: vec![42],
            uptime_secs: 7,
            model_loaded: None,
        };
    }

    #[test]
    fn celestia_engine_mcp_and_external_mcp_resolve() {
        // Engine (CEP) domain protocol:
        resolves::<plana::engine::EngineHandshakeParams>();
        // Per-tool MCP I/O vocabulary:
        resolves::<plana::tools::philia::McpToolDetail>();
        // External MCP server registry file:
        resolves::<plana::external_mcp::McpServersFile>();
    }

    #[test]
    fn celestia_engine_and_philia_surfaces_resolve() {
        // Engine (CEP) request/result vocabulary — the gateway-facing
        // handshake, chat, invoke, stream, model-list and binary-transfer
        // payloads stay reachable at `plana::engine`:
        resolves::<plana::engine::EngineChatParams>();
        resolves::<plana::engine::EngineInvokeParams>();
        resolves::<plana::engine::EngineStreamChunk>();
        resolves::<plana::engine::EngineModality>();
        resolves::<plana::engine::EngineBinaryStartParams>();
        resolves::<plana::engine::EngineCapabilities>();
        resolves::<plana::engine::EngineIdentity>();
        resolves::<plana::engine::EngineModelsResult>();
        resolves::<plana::engine::EngineHandshakeResult>();
        let _ = plana::engine::ENGINE_PROTOCOL_VERSION;
        // philia (memory tool) per-tool I/O vocabulary:
        resolves::<plana::tools::philia::MemoryQueryItem>();
        resolves::<plana::tools::philia::MemoryQueryParams>();
        resolves::<plana::tools::philia::MemoryQueryResult>();
        resolves::<plana::tools::philia::MemoryStoreParams>();
        resolves::<plana::tools::philia::MemoryStoreResult>();
        // malkuth supervision gate: restart proposal payload.
        resolves::<plana::malkuth::RestartProposal>();
    }

    #[test]
    fn celestia_domain_base_messages_resolve_to_generic_types() {
        // I4: the domain copy is a re-export — `plana::celestia` and the
        // umbrella root must expose the same generic type instance. The type
        // annotation on the right proves identity at compile time: if the two
        // paths named different types, this would not typecheck.
        resolves::<plana::celestia::protocol::base_messages::BaseHeartbeatParams>();
        resolves::<plana::protocol::base_messages::BaseHeartbeatParams>();
        let _: plana::protocol::base_messages::BaseHeartbeatParams =
            plana::celestia::protocol::base_messages::BaseHeartbeatParams { timestamp: 1 };
    }

    #[test]
    fn malkuth_health_response_is_distinct_from_root() {
        // Constructing each with its own field set proves they are two
        // unrelated structs sharing only the name.
        let _root = plana::http::HealthResponse::ok(
            "1.0.0",
            plana::http::BackendKind::Dev,
            1,
            plana::http::NetworkInfo::unknown(),
        );
        let _supervision = plana::malkuth::HealthResponse {
            worker_id: "w-1".into(),
            healthy: true,
            ready: true,
            not_ready_reason: None,
            uptime_secs: 42,
            version: "1.0.0".into(),
        };
        let _supervision_via_celestia = plana::celestia::malkuth::HealthResponse {
            worker_id: "w-1".into(),
            healthy: true,
            ready: true,
            not_ready_reason: None,
            uptime_secs: 42,
            version: "1.0.0".into(),
        };
    }
}
