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

# Repo definitions override the shared template's (imported above).
set allow-duplicate-recipes
set allow-duplicate-variables

# Local fallbacks for the shared template's tool resolution — byte-identical
# semantics, so a fresh clone (no gitignored .just/ staging yet) parses and
# runs the same; when the staged template is present it re-defines the same
# values and allow-duplicate-variables lets either order win.
python_cmd := if os_family() == "windows" {
    if which("python") != "" { "python" } else { "python3" }
} else {
    if which("python3") != "" { "python3" } else { "python" }
}

# Shared celestia-devtools recipes — NOT in git. Stage with: just fetch.
# `import?` silently skips when absent, so this justfile parses pre-fetch.
import? "./.just/git-bash-interop.just"
import? "./.just/celestia-devtools.just"

# Stage shared celestia-devtools recipes into .just/ (gitignored).
# Source order: explicit URL arg → local pip bundle (offline) → GitHub raw.
# curl honors HTTP_PROXY/HTTPS_PROXY/ALL_PROXY env vars automatically.
fetch URL='':
    {{ if os_family() == "windows" { "python" } else { "python3" } }} -c "import os; os.makedirs('.just', exist_ok=True)"
    {{ if URL != "" { "curl -fsSL " + URL + " -o .just/celestia-devtools.just" } else if which("celestia-devtools") != "" { "celestia-devtools fetch-just" } else { "curl -fsSL https://raw.githubusercontent.com/celestia-island/celestia-devtools/dev/src/celestia_devtools/common.just -o .just/celestia-devtools.just" } }}
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
[script('python')]
gen target='bindings':
    import subprocess, sys

    def run(cmd):
        rc = subprocess.run(cmd).returncode
        if rc != 0:
            sys.exit(rc)

    t = "{{target}}"
    if t == "bindings":
        run(["cargo", "test", "--package", "plana", "--package", "plana-celestia-types", "--package", "plana_evernight_protocol"])
    else:
        print("Usage: just gen bindings")
        sys.exit(1)

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
