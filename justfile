# arona — minimal justfile. arona is the hub for the shared Python devtool
# scripts (scripts/utils/), so it calls them directly; consumer repos reach
# them via scripts/_arona_devtools.py.

set shell := ["bash", "-c"]

python_cmd := if which("python3") != "" { "python3" } else { "python" }

default:
    @just --list

# Pre-stage all dependencies (cargo fetch + node install) so subsequent
# builds can run fully offline. Run once after cloning (needs network).
install:
    just cache-guard
    {{python_cmd}} scripts/utils/prefetch.py .

# target/ cache guard (see scripts/utils/cargo_cache_guard.py).
# Hard floor: free disk < 10 GiB → cargo clean.
# Soft threshold: target/ >= 40 GiB → cargo sweep --time 7 (needs cargo-sweep).
cache-guard *ARGS='':
    {{python_cmd}} scripts/utils/cargo_cache_guard.py . {{ARGS}}

# Manually remove target/**/incremental/ (keeps compiled dep artifacts).
clean-incremental:
    {{python_cmd}} scripts/utils/cargo_cache_guard.py . --clean-incremental

clean:
    cargo clean

build:
    just cache-guard
    cargo build

test:
    cargo test
