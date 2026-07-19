use anyhow::Result;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use _jsonrpc::{
    JsonRpcRequest, core_message_to_method_and_params, deserialize_from_jsonrpc,
    from_jsonrpc_method, serialize_to_jsonrpc,
};
use _state_sync::gateway::{BaseMessage, Message as CoreMessage, TuiMessage};

fn bench_serialize_roundtrip(c: &mut Criterion) {
    let msg = CoreMessage::Tui(TuiMessage::OpenWorkspace {
        uri: "git://https://github.com/org/repo.git".to_string(),
    });
    c.bench_function("serialize_roundtrip", |b| {
        b.iter(|| {
            if let Err(e) = (|| -> Result<()> {
                let json = serialize_to_jsonrpc(black_box(&msg), false)?;
                let _ = deserialize_from_jsonrpc(&json);
                Ok(())
            })() {
                eprintln!("bench iteration failed: {e}");
                std::process::exit(1);
            }
        })
    });
}

fn bench_deserialize_notification(c: &mut Criterion) {
    let msg = CoreMessage::Base(BaseMessage::Heartbeat { timestamp: 12345 });
    let json = match serialize_to_jsonrpc(&msg, true) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("bench setup failed: {e}");
            std::process::exit(1);
        }
    };
    c.bench_function("deserialize_notification", |b| {
        b.iter(|| deserialize_from_jsonrpc(black_box(&json)))
    });
}

fn bench_core_message_to_method(c: &mut Criterion) {
    let msg = CoreMessage::Tui(TuiMessage::OpenWorkspace {
        uri: "git://https://github.com/org/repo.git".to_string(),
    });
    c.bench_function("core_message_to_method", |b| {
        b.iter(|| core_message_to_method_and_params(black_box(&msg)))
    });
}

fn bench_from_jsonrpc_method(c: &mut Criterion) {
    let params = serde_json::json!({"uri": "git://https://github.com/org/repo.git"});
    c.bench_function("from_jsonrpc_method", |b| {
        b.iter(|| {
            from_jsonrpc_method(
                black_box("Sync.OpenWorkspace"),
                black_box(Some(params.clone())),
            )
        })
    });
}

fn bench_request_serialize(c: &mut Criterion) {
    let req = JsonRpcRequest::mcp_call("file_read", serde_json::json!({"path": "/tmp/test.txt"}));
    c.bench_function("request_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&req)))
    });
}

criterion_group!(
    benches,
    bench_serialize_roundtrip,
    bench_deserialize_notification,
    bench_core_message_to_method,
    bench_from_jsonrpc_method,
    bench_request_serialize,
);
criterion_main!(benches);
