use criterion::{Criterion, black_box, criterion_group, criterion_main};

use plana_text::LlmTextBuilder;

const CHUNK: &str = "The quick brown fox jumps over the lazy dog. ";

fn bench_push_chunk(c: &mut Criterion) {
    c.bench_function("llm_text_builder_push_100_chunks", |b| {
        b.iter(|| {
            let mut builder = LlmTextBuilder::new();
            for _ in 0..100 {
                builder.push_chunk(black_box(CHUNK));
            }
            builder
        })
    });
}

fn bench_seal(c: &mut Criterion) {
    c.bench_function("llm_text_builder_seal", |b| {
        b.iter(|| {
            let mut builder = LlmTextBuilder::new();
            for _ in 0..100 {
                builder.push_chunk(CHUNK);
            }
            builder.seal()
        })
    });
}

fn bench_slice(c: &mut Criterion) {
    let mut builder = LlmTextBuilder::new();
    for _ in 0..100 {
        builder.push_chunk(CHUNK);
    }
    c.bench_function("llm_text_builder_slice", |b| {
        b.iter(|| black_box(builder.slice(10..100)))
    });
}

criterion_group!(benches, bench_push_chunk, bench_seal, bench_slice);
criterion_main!(benches);
