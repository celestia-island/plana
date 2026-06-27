# arona — monorepo justfile (docs + packages).
# packages/devtools/ hosts the shared Python build scripts; consumer repos
# reach them via their own scripts/_arona_devtools.py wrapper.

set shell := ["bash", "-c"]

python_cmd := if which("python3") != "" { "python3" } else { "python" }
devtools := "packages/devtools"

default:
    @just --list

# Pre-stage all dependencies (cargo fetch + node install) so subsequent
# builds can run fully offline. Run once after cloning (needs network).
install:
    just cache-guard
    {{python_cmd}} {{devtools}}/utils/prefetch.py .

# target/ cache guard (see packages/devtools/utils/cargo_cache_guard.py).
# Hard floor: free disk < 10 GiB → cargo clean.
# Soft threshold: target/ >= 40 GiB → cargo sweep --time 7 (needs cargo-sweep).
cache-guard *ARGS='':
    {{python_cmd}} {{devtools}}/utils/cargo_cache_guard.py . {{ARGS}}

# Manually remove target/**/incremental/ (keeps compiled dep artifacts).
clean-incremental:
    {{python_cmd}} {{devtools}}/utils/cargo_cache_guard.py . --clean-incremental

clean:
    cargo clean

build:
    just cache-guard
    cargo build

test:
    cargo test

# Regenerate TypeScript bindings into packages/bindings/ via ts-rs.
bindings:
    cargo test --package arona

# Format Markdown docs + Rust code, then run lint checks.
# Warnings (tab characters, untranslated duplicate paragraphs) are printed
# to stderr but do not cause a non-zero exit.
fmt:
    {{python_cmd}} {{devtools}}/utils/format_markdown.py .
    cargo fmt --all

# Check formatting without writing changes.
fmt-check:
    {{python_cmd}} {{devtools}}/utils/format_markdown.py . --check
    cargo fmt --all -- --check
