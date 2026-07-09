# arona — single-crate repo (protocol types + TS bindings + build scripts).
# scripts/ hosts the shared Python build scripts; consumer repos now reach them
# via the shared celestia-devtools justfile import.
#
# Verb-first dispatch: actions are first-level commands (build, test, gen, …).

set shell := ["bash", "-c"]
set windows-shell := ["bash.exe", "-c"]
set unstable
set lists

import "./celestia-devtools.just"

default:
    @just --list

# ── Lifecycle ────────────────────────────────────────────────────────

# Pre-stage all dependencies (cargo fetch + node install) so subsequent
# builds can run fully offline. Run once after cloning (needs network).
install:
    just cache-guard
    just prefetch

# ── Build ────────────────────────────────────────────────────────────

build:
    just cache-guard
    cargo build

clean:
    cargo clean

# ── Quality ──────────────────────────────────────────────────────────

test:
    cargo test

# ── Generate (codegen) ───────────────────────────────────────────────

# Regenerate artifacts. Default: bindings.
gen target='bindings':
    #!/usr/bin/env sh
    set -euo pipefail
    case "{{target}}" in
      bindings) cargo test --package arona ;;
      *) echo "Usage: just gen bindings"; exit 1 ;;
    esac

# ── Format ───────────────────────────────────────────────────────────

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
