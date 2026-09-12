//! Celestia domain-profile surface contract.
//!
//! This integration test is compiled as an *external* crate, so it proves the
//! paths consumers of the domain profile rely on keep resolving: the root
//! re-exports (`Agent`, `TaskStatus`, …), the `http`/`enums`/`engine`/
//! `malkuth`/`tools`/`external_mcp` domain modules, the scepter-flavored
//! handshake and client-capability payloads, and the platform error codes
//! that re-export the canonical `plana-jsonrpc` table.
//!
//! Mostly compile-only (empty `resolves::<T>()` calls). The one runtime
//! assertion pins the platform error-code value, and the type ascription in
//! `domain_base_messages_are_the_generic_core_types` is a compile-time proof
//! that the domain re-export and the generic core name one type instance.
//!
//! Migration note: these assertions lived in `plana`'s `tests/surface.rs` as
//! `#[cfg(feature = "celestia")] mod celestia_surface`, back when the `plana`
//! umbrella re-exported this crate behind that feature. The facade is gone
//! (the domain profile now depends on `plana`, never the reverse), so the
//! domain paths are pinned here — in the crate that owns them, where an
//! external consumer actually resolves them.

fn resolves<T>() {}

#[test]
fn domain_types_resolve_at_the_crate_root() {
    resolves::<plana_celestia_types::Agent>();
    resolves::<plana_celestia_types::TaskStatus>();
    resolves::<plana_celestia_types::malkuth::WorkerStatus>();
    resolves::<plana_celestia_types::http::AgentItem>();
    resolves::<plana_celestia_types::protocol::handshake::ConnectHandshakeParams>();
    resolves::<plana_celestia_types::protocol::handshake::ClientCapability>();
    assert_eq!(
        plana_celestia_types::protocol::jsonrpc::error_codes::SNAPSHOT_FAILED,
        -32001
    );
}

#[test]
fn http_domain_types_resolve() {
    // Live consumer paths (arona / e-world admin panels):
    resolves::<plana_celestia_types::http::ModelInfo>();
    resolves::<plana_celestia_types::http::ProviderPublic>();
    resolves::<plana_celestia_types::http::TierDefinition>();
    resolves::<plana_celestia_types::http::UserPreferences>();
}

#[test]
fn domain_enums_used_by_entelecheia_resolve() {
    // entelecheia consumes the annotation and file-operation vocabulary
    // through the domain `enums` module:
    resolves::<plana_celestia_types::enums::AnnotationType>();
    resolves::<plana_celestia_types::enums::FileOperationType>();
    resolves::<plana_celestia_types::enums::ObservationType>();
    let _ = plana_celestia_types::enums::AnnotationType::Todo.as_str();
    let _ = plana_celestia_types::enums::FileOperationType::Reading.as_str();
    let _ = plana_celestia_types::enums::ObservationType::Watching.as_str();
}

#[test]
fn engine_extended_surface_resolves() {
    // arona consumes the embeddings / invoke-start / stats vocabulary at
    // `engine`; construct with the real field sets to pin the wire shapes.
    let _emb = plana_celestia_types::engine::EngineEmbeddingsParams {
        model: "m".into(),
        input: vec!["text".into()],
    };
    let _start = plana_celestia_types::engine::EngineInvokeStartResult {
        ok: true,
        error: None,
        stream_id: "s-1".into(),
    };
    let _stats = plana_celestia_types::engine::EngineStatsResult {
        gpu_utilization: vec![42],
        uptime_secs: 7,
        model_loaded: None,
    };
}

#[test]
fn engine_external_mcp_and_tools_surfaces_resolve() {
    // Engine (CEP) domain protocol:
    resolves::<plana_celestia_types::engine::EngineHandshakeParams>();
    // Per-tool I/O vocabulary:
    resolves::<plana_celestia_types::tools::philia::ToolDetail>();
    // External MCP server registry file:
    resolves::<plana_celestia_types::external_mcp::McpServersFile>();
}

#[test]
fn engine_and_philia_surfaces_resolve() {
    // Engine (CEP) request/result vocabulary — the gateway-facing handshake,
    // chat, invoke, stream, model-list and binary-transfer payloads:
    resolves::<plana_celestia_types::engine::EngineChatParams>();
    resolves::<plana_celestia_types::engine::EngineInvokeParams>();
    resolves::<plana_celestia_types::engine::EngineStreamChunk>();
    resolves::<plana_celestia_types::engine::EngineModality>();
    resolves::<plana_celestia_types::engine::EngineBinaryStartParams>();
    resolves::<plana_celestia_types::engine::EngineCapabilities>();
    resolves::<plana_celestia_types::engine::EngineIdentity>();
    resolves::<plana_celestia_types::engine::EngineModelsResult>();
    resolves::<plana_celestia_types::engine::EngineHandshakeResult>();
    let _ = plana_celestia_types::engine::ENGINE_PROTOCOL_VERSION;
    // philia (memory tool) per-tool I/O vocabulary:
    resolves::<plana_celestia_types::tools::philia::MemoryQueryItem>();
    resolves::<plana_celestia_types::tools::philia::MemoryQueryParams>();
    resolves::<plana_celestia_types::tools::philia::MemoryQueryResult>();
    resolves::<plana_celestia_types::tools::philia::MemoryStoreParams>();
    resolves::<plana_celestia_types::tools::philia::MemoryStoreResult>();
    // malkuth supervision gate: restart proposal payload.
    resolves::<plana_celestia_types::malkuth::RestartProposal>();
}

#[test]
fn domain_base_messages_are_the_generic_core_types() {
    // The domain copy is a re-export — the domain profile and the generic
    // core must expose the same type instance. The type annotation on the
    // left proves identity at compile time: if the two paths named different
    // types, this would not typecheck.
    resolves::<plana_celestia_types::protocol::base_messages::BaseHeartbeatParams>();
    resolves::<plana::protocol::base_messages::BaseHeartbeatParams>();
    let _: plana::protocol::base_messages::BaseHeartbeatParams =
        plana_celestia_types::protocol::base_messages::BaseHeartbeatParams { timestamp: 1 };
}

#[test]
fn supervision_health_response_is_distinct_from_the_generic_one() {
    // The domain crate root pins the generic health descriptor (the explicit
    // re-export shadows the glob-imported supervision one); the annotation
    // proves the two paths below name the same generic type.
    let _generic: plana::http::HealthResponse = plana_celestia_types::HealthResponse::ok(
        "1.0.0",
        plana::http::BackendKind::Dev,
        1,
        plana::http::NetworkInfo::unknown(),
    );
    // Constructing each with its own field set proves the supervision
    // `HealthResponse` is an unrelated struct sharing only the name.
    let _supervision = plana_celestia_types::malkuth::HealthResponse {
        worker_id: "w-1".into(),
        healthy: true,
        ready: true,
        not_ready_reason: None,
        uptime_secs: 42,
        version: "1.0.0".into(),
    };
}
