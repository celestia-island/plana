#!/usr/bin/env python3
"""Short per-repo bootstrap wrapper around arona's shared devtool scripts.

The heavy shared modules (logger, cargo_cache_guard, ...) live ONLY in
arona/scripts/utils/ (kept out of arona's published crate via
arona/Cargo.toml `exclude = ["scripts"]`). Each consumer repo ships ONE copy
of this short wrapper and delegates to those modules — no repo duplicates the
shared code.

Locating a cargo-[patch]-ed source crate is the same algorithm everywhere
(env var → cargo [patch] path → sibling checkout → last-resort git clone into
cargo's own `target/` dir), so it is exposed as the generic `find_patched_crate`.
arona locates itself with it; shittim-chest reuses the SAME call to locate
entelecheia — only the crate name / env var differ.

Usage:
    python scripts/_arona_devtools.py <module> [args...]   # delegate
    python scripts/_arona_devtools.py --locate              # just print arona root
    e.g.
    python scripts/_arona_devtools.py cargo_cache_guard . --dry-run
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
ARONA_GIT = os.environ.get("ARONA_GIT", "https://github.com/celestia-island/arona.git")
# Last-resort clone destination lives INSIDE cargo's own `target/` dir (already
# gitignored everywhere), so no separate scratch dir or ignore rule is needed.
# Override with CELESTIA_DEV_TARGET_DIR if you really must.
TARGET_DIR = os.environ.get("CELESTIA_DEV_TARGET_DIR", "target")


def _stderr(level: str, msg: str) -> None:
    print(f"[arona-devtools] {level}: {msg}", file=sys.stderr)


# ── generic locator: env → cargo [patch] path → sibling → git clone ────


def find_patched_crate(
    crate: str,
    env_var: str,
    git_url: str,
    marker: Path,
    *,
    repo_root: Path = REPO_ROOT,
    clone_subdir: str | None = None,
) -> Path | None:
    """Resolve the checkout of a cargo path/git-patched crate.

    Priority (cheap first, so the common case never walks the tree):
      1. $env_var
      2. a `crate = { path = ".." }` under any [patch.*] in ~/.cargo/config.toml
         or this repo's top-level Cargo.toml
      3. a sibling `../<crate>` checkout (the org dev layout)
      4. (fallback) the same [patch] search across nested Cargo.toml files,
         pruning target/ / node_modules / .git so a 30+ GiB target/ doesn't
         make every invocation stall for a minute
      5. shallow `git clone` into <repo_root>/target/<crate>-shared

    `marker` is a path relative to the crate root whose presence confirms a
    valid checkout. Returns the resolved root, or None.
    """
    import os

    def _ok(cand: Path) -> bool:
        return cand and (cand / marker).exists()

    # 1. explicit override
    if env := os.environ.get(env_var):
        c = Path(env).expanduser()
        if _ok(c):
            return c.resolve()

    pat = re.compile(
        rf'\b{re.escape(crate)}\s*=\s*\{{\s*[^}}]*\bpath\s*=\s*"([^"]+)"', re.S,
    )

    def _check_cfg(text: str, base: Path) -> Path | None:
        for m in pat.finditer(text):
            p = Path(m.group(1))
            if not p.is_absolute():
                p = base / p
            if _ok(p):
                return p.resolve()
        return None

    # 2. the two config files that almost always carry the org-wide patch
    for cfg in (Path.home() / ".cargo" / "config.toml", repo_root / "Cargo.toml"):
        try:
            hit = _check_cfg(cfg.read_text(errors="ignore"), cfg.parent)
        except OSError:
            hit = None
        if hit:
            return hit

    # 3. sibling layout
    sib = repo_root.parent / crate
    if _ok(sib):
        return sib.resolve()

    # 4. fallback: scan nested Cargo.toml, pruning heavy dirs
    _SKIP_DIRS = {"target", "node_modules", ".git", ".next", "dist", "build"}
    for root, dirs, files in os.walk(repo_root):
        dirs[:] = [d for d in dirs if d not in _SKIP_DIRS]
        if "Cargo.toml" not in files:
            continue
        cfg = Path(root) / "Cargo.toml"
        try:
            hit = _check_cfg(cfg.read_text(errors="ignore"), cfg.parent)
        except OSError:
            hit = None
        if hit:
            return hit

    # 5. last resort: clone into the repo's cargo target/ dir (already ignored)
    clone = repo_root / TARGET_DIR / (clone_subdir or f"{crate}-shared")
    if not _ok(clone):
        try:
            clone.parent.mkdir(parents=True, exist_ok=True)
            _stderr("warn", f"{crate} not found locally — cloning into {clone}")
            subprocess.run(
                ["git", "clone", "--depth", "1", git_url, str(clone)], check=False,
            )
        except OSError as exc:
            _stderr("error", f"git clone of {crate} failed: {exc}")
    if _ok(clone):
        return clone.resolve()
    return None


def find_arona() -> Path | None:
    return find_patched_crate(
        "arona", "ARONA_ROOT", ARONA_GIT,
        Path("scripts") / "utils" / "cargo_cache_guard.py",
    )


# ── delegation ────────────────────────────────────────────


def _run_module(module: str, extra_argv: list[str]) -> int:
    arona = find_arona()
    if arona is None:
        _stderr("error", "could not locate arona shared scripts; set ARONA_ROOT")
        return 127
    # Repo's own scripts/utils first (so its configured logger wins for the
    # shared module's `import logger`), then arona's utils for the shared mods.
    own_utils = REPO_ROOT / "scripts" / "utils"
    if own_utils.is_dir():
        sys.path.insert(0, str(own_utils))
    sys.path.insert(0, str(arona / "scripts" / "utils"))
    try:
        mod = __import__(module)
    except ModuleNotFoundError as exc:
        _stderr("error", f"shared module '{module}' not found in arona: {exc}")
        return 127
    sys.argv = [module, *extra_argv]
    entry = getattr(mod, "main", None)
    if not callable(entry):
        _stderr("error", f"shared module '{module}' has no main()")
        return 2
    return int(entry() or 0)


def main() -> int:
    args = sys.argv[1:]
    if not args:
        _stderr("error", "usage: _arona_devtools.py <module> [args...] | --locate")
        return 2
    if args[0] in ("--locate", "-L"):
        arona = find_arona()
        if arona is None:
            _stderr("error", "arona not located; set ARONA_ROOT")
            return 127
        print(arona)
        return 0
    return _run_module(args[0], args[1:])


if __name__ == "__main__":
    raise SystemExit(main())
