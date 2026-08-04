use criterion::{Criterion, black_box, criterion_group, criterion_main};

use plana_domain_skills::ParsedToolCall;

fn bench_parse_simple(c: &mut Criterion) {
    c.bench_function("parse_tool_call_simple", |b| {
        b.iter(|| ParsedToolCall::parse(black_box("file_read")))
    });
}

fn bench_parse_with_tag(c: &mut Criterion) {
    c.bench_function("parse_tool_call_tagged", |b| {
        b.iter(|| ParsedToolCall::parse(black_box("navigate[2]")))
    });
}

fn bench_parse_with_field(c: &mut Criterion) {
    c.bench_function("parse_tool_call_field", |b| {
        b.iter(|| ParsedToolCall::parse(black_box("llm_chat.content")))
    });
}

fn bench_parse_full(c: &mut Criterion) {
    c.bench_function("parse_tool_call_full", |b| {
        b.iter(|| ParsedToolCall::parse(black_box("navigate[3].content")))
    });
}

fn bench_parse_garbage(c: &mut Criterion) {
    c.bench_function("parse_tool_call_garbage", |b| {
        b.iter(|| ParsedToolCall::parse(black_box("!!!invalid%%tool@@name")))
    });
}

criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_with_tag,
    bench_parse_with_field,
    bench_parse_full,
    bench_parse_garbage,
);
criterion_main!(benches);
