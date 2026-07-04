# arona — single-crate repo (protocol types + TS bindings + build scripts).
# scripts/ hosts the shared Python build scripts; consumer repos now reach them
# via the shared celestia-devtools justfile import.

set shell := ["bash", "-c"]
set unstable
set lists

python_cmd := if which("python3") != "" { "python3" } else { "python" }

import "./celestia-devtools.just"

default:
    @just --list

# Pre-stage all dependencies (cargo fetch + node install) so subsequent
# builds can run fully offline. Run once after cloning (needs network).
install:
    just cache-guard
    just prefetch

clean:
    cargo clean

build:
    just cache-guard
    cargo build

test:
    cargo test

# Regenerate TypeScript bindings into bindings/ via ts-rs.
bindings:
    cargo test --package arona

# Format Markdown docs + Rust code, then run lint checks.
# Warnings (tab characters, untranslated duplicate paragraphs) are printed
# to stderr but do not cause a non-zero exit.
fmt:
    just fmt-markdown .
    cargo fmt --all

# Check formatting without writing changes.
fmt-check:
    just fmt-markdown . --check
    cargo fmt --all -- --check
