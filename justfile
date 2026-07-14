# arona — single-crate repo (protocol types + TS bindings + build scripts).
# scripts/ hosts the shared Python build scripts; consumer repos now reach them
# via the shared celestia-devtools justfile import.
#
# Verb-first dispatch: actions are first-level commands (build, test, gen, …).

set shell := ["bash", "-c"]
set windows-shell := ["bash.exe", "-c"]
set unstable
set lists

# Shared celestia-devtools recipes — NOT in git. Stage with: just fetch.
# `import?` silently skips when absent, so this justfile parses pre-fetch.
import? "./.just/git-bash-interop.just"
import? "./.just/celestia-devtools.just"

# Stage shared celestia-devtools recipes into .just/ (gitignored).
# Source order: explicit URL arg → local pip bundle (offline) → GitHub raw.
# curl honors HTTP_PROXY/HTTPS_PROXY/ALL_PROXY env vars automatically.
[script('bash')]
fetch URL='':
    #!/usr/bin/env bash
    set -euo pipefail
    out=.just/celestia-devtools.just
    mkdir -p .just
    if [ -n "{{URL}}" ]; then
      echo "[fetch] {{URL}} -> $out"
      curl -fsSL "{{URL}}" -o "$out"
    elif command -v celestia-devtools >/dev/null 2>&1; then
      src=$(celestia-devtools include-path)
      echo "[fetch] local bundle ($src) -> $out"
      cp "$src" "$out"
    else
      echo "[fetch] github raw -> $out"
      curl -fsSL "https://raw.githubusercontent.com/celestia-island/celestia-devtools/dev/src/celestia_devtools/common.just" -o "$out"
    fi
    echo "[fetch] wrote $out"

default:
    @just --list

# ── Lifecycle ────────────────────────────────────────────────────────

# Pre-stage all dependencies (cargo fetch + node install) so subsequent
# builds can run fully offline. Run once after cloning (needs network).
install:
    just cache-guard
    just prefetch

# ── Data ─────────────────────────────────────────────────────────────

# Fetch provider-registry data into target/provider-registry/ (consumed by build.rs).
#   just fetch-provider-registry            # clone from upstream
#   just fetch-provider-registry /local/path  # sync from a local checkout
fetch-provider-registry LOCAL="":
    {{python_cmd}} scripts/fetch_provider_registry.py {{LOCAL}}

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
[script('sh')]
gen target='bindings':
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
