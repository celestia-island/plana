# plana — single-crate repo (protocol types + TS bindings + build scripts).
# scripts/ hosts the shared Python build scripts; consumer repos now reach them
# via the shared celestia-devtools justfile import.
#
# Verb-first dispatch: actions are first-level commands (build, test, gen, …).

set shell := ["bash", "-c"]
# Windows: PowerShell (the 5.1 floor ships with every Windows; pwsh 7 is
# NOT assumed). Linewise recipes must stay PS-5.1-safe: no `&&` chains,
# `cd X; cmd` instead of `cd X && cmd`. Bash-only recipes use
# [script('bash')] and need Git Bash (or WSL) when actually run.
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command", "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; $PSDefaultParameterValues['*:Encoding']='utf8';"]
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

# Sync data on demand.
#   just sync provider-registry            # clone from upstream
#   just sync provider-registry /path      # sync from a local checkout
sync target='provider-registry' *ARGS='':
    {{python_cmd}} scripts/fetch_provider_registry.py {{ARGS}}

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
# TS bindings are emitted by the test builds of the crates that own the
# exported types, into THREE directories: packages/plana/bindings/ (generic
# types — the `plana-protocol-core` package is now a re-export shim with no
# types of its own, so the plana foundation is what generates this file set),
# packages/celestia-types/bindings/ (domain types) and
# packages/evernight-protocol/bindings/ (Tier 3 wire DTOs). `just gen
# bindings` regenerates all three.
[script('sh')]
gen target='bindings':
    set -euo pipefail
    case "{{target}}" in
      bindings) cargo test --package plana --package plana-celestia-types --package plana_evernight_protocol ;;
      *) echo "Usage: just gen bindings"; exit 1 ;;
    esac

# ── Format ───────────────────────────────────────────────────────────

# Format Markdown docs + Rust code, then run lint checks.
# Warnings (tab characters, untranslated duplicate paragraphs) are printed
# to stderr but do not cause a non-zero exit.
fmt:
    just fmt-toml
    just fmt-markdown .
    cargo fmt --all

# Check formatting without writing changes.
fmt-check:
    just fmt-markdown . --check
    cargo fmt --all -- --check
