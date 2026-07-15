#!/usr/bin/env python3
"""
Download provider-registry data to build directory.

Fetches the latest entrypoint/ and models/ TOML files from the
provider-registry repository for use by the mock server.

Uses content-aware incremental updates: only files whose contents
actually changed are overwritten, preserving mtimes for unchanged
files so that `cargo:rerun-if-changed` in build.rs does not trigger
spurious rebuilds.

Usage:
    python scripts/fetch_provider_registry.py
    python scripts/fetch_provider_registry.py --local /path/to/provider-registry
"""

import argparse
import subprocess
import tempfile
from pathlib import Path

PROVIDER_REGISTRY_REPO = "https://github.com/celestia-island/provider-registry.git"
DEFAULT_BRANCH = "master"
BUILD_DIR = Path(__file__).resolve().parent.parent / "target" / "provider-registry"


def _sync_directory(src: Path, dst: Path) -> int:
    """Incrementally sync src/ into dst/.

    Only overwrites files whose contents differ, preserving mtimes
    for unchanged files so cargo:rerun-if-changed stays quiet.
    Removes files in dst that no longer exist in src.

    Returns the number of files actually written.
    """
    written = 0

    # Collect all relative paths in src
    src_files: dict[Path, bytes] = {}
    for f in src.rglob("*"):
        if f.is_file():
            rel = f.relative_to(src)
            src_files[rel] = f.read_bytes()

    # Remove files in dst that no longer exist in src
    if dst.exists():
        for f in dst.rglob("*"):
            if f.is_file():
                rel = f.relative_to(dst)
                if rel not in src_files:
                    f.unlink()
        # Remove now-empty directories
        for d in sorted(dst.rglob("*"), reverse=True):
            if d.is_dir() and not any(d.iterdir()):
                d.rmdir()

    # Write only changed files
    for rel, content in src_files.items():
        dst_file = dst / rel
        if dst_file.exists() and dst_file.read_bytes() == content:
            continue
        dst_file.parent.mkdir(parents=True, exist_ok=True)
        dst_file.write_bytes(content)
        written += 1

    return written


def fetch_via_git(target_dir: Path, branch: str = DEFAULT_BRANCH):
    """Clone provider-registry into a temp dir, then incrementally sync."""
    print(f"[INFO] Cloning provider-registry ({branch}) ...")
    with tempfile.TemporaryDirectory() as tmp:
        clone_dir = Path(tmp) / "repo"
        subprocess.run(
            ["git", "clone", "--branch", branch, "--depth", "1",
             PROVIDER_REGISTRY_REPO, str(clone_dir)],
            check=True,
        )

        written = _sync_directory(clone_dir, target_dir)
        total = sum(1 for _ in target_dir.rglob("*") if _.is_file())
        print(f"[OK] Synced {written}/{total} files changed (incremental)")
        if written == 0:
            print("     No changes — existing files untouched")


def copy_from_local(source_dir: Path, target_dir: Path):
    """Copy from a local provider-registry checkout (incrementally)."""
    print(f"[INFO] Syncing from local {source_dir} -> {target_dir}")
    written = _sync_directory(source_dir, target_dir)
    total = sum(1 for _ in target_dir.rglob("*") if _.is_file()) if target_dir.exists() else 0
    print(f"[OK] Synced {written}/{total} files changed (incremental)")


def main():
    parser = argparse.ArgumentParser(description="Fetch provider-registry data")
    parser.add_argument(
        "--local",
        type=str,
        default=None,
        help="Path to local provider-registry checkout (skips git clone)",
    )
    parser.add_argument(
        "--branch",
        type=str,
        default=DEFAULT_BRANCH,
        help=f"Branch to fetch (default: {DEFAULT_BRANCH})",
    )
    args = parser.parse_args()

    target_dir = BUILD_DIR

    if args.local:
        copy_from_local(Path(args.local), target_dir)
    else:
        fetch_via_git(target_dir, args.branch)

    entrypoint_count = sum(1 for _ in (target_dir / "entrypoint").rglob("*.toml")) if (target_dir / "entrypoint").exists() else 0
    model_count = sum(1 for _ in (target_dir / "models").rglob("*.toml")) if (target_dir / "models").exists() else 0
    print(f"\n  {entrypoint_count} entrypoint files, {model_count} model files")


if __name__ == "__main__":
    main()
