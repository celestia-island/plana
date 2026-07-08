use criterion::{Criterion, black_box, criterion_group, criterion_main};

use _core::{ContainerId, LlmSessionId, truncate};

fn bench_container_id_new(c: &mut Criterion) {
    c.bench_function("container_id_new", |b| {
        b.iter(|| ContainerId::new(black_box("042")))
    });
}

fn bench_container_id_demiurge(c: &mut Criterion) {
    c.bench_function("container_id_demiurge", |b| {
        b.iter(|| ContainerId::new(black_box("demiurge")))
    });
}

fn bench_llm_session_id_parse(c: &mut Criterion) {
    c.bench_function("llm_session_id_parse", |b| {
        b.iter(|| LlmSessionId::parse(black_box("#001.42")))
    });
}

fn bench_truncate_short(c: &mut Criterion) {
    c.bench_function("truncate_short", |b| {
        b.iter(|| truncate(black_box("hello world"), black_box(50)))
    });
}

fn bench_truncate_long(c: &mut Criterion) {
    let long = "a".repeat(10000);
    c.bench_function("truncate_long", |b| {
        b.iter(|| truncate(black_box(&long), black_box(100)))
    });
}

fn bench_truncate_unicode(c: &mut Criterion) {
    let unicode = "你好世界".repeat(100);
    c.bench_function("truncate_unicode", |b| {
        b.iter(|| truncate(black_box(&unicode), black_box(50)))
    });
}

fn bench_base64_roundtrip(c: &mut Criterion) {
    let data = vec![0u8; 1024];
    c.bench_function("base64_roundtrip_1kb", |b| {
        b.iter(|| {
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                black_box(&data),
            );
            let _ = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                black_box(&encoded),
            );
        })
    });
}

criterion_group!(
    benches,
    bench_container_id_new,
    bench_container_id_demiurge,
    bench_llm_session_id_parse,
    bench_truncate_short,
    bench_truncate_long,
    bench_truncate_unicode,
    bench_base64_roundtrip,
);
criterion_main!(benches);
