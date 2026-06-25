#!/usr/bin/env python3
"""Short per-repo bootstrap wrapper — locate arona's shared devtool scripts
and delegate argv to a shared module there.

Each consumer repo ships ONE copy of this short wrapper. The heavy shared
modules (logger, cargo_cache_guard, ...) live only in arona/scripts/utils/
and are kept out of arona's published crate (arona/Cargo.toml
`exclude = ["scripts"]`).

arona is located, in priority order:
  1. $ARONA_ROOT                         (explicit override)
  2. the cargo [patch] path for `arona`  (~/.cargo/config.toml, or any
                                         [patch.*] arona = { path = ".." } in
                                         this repo's Cargo.toml)
  3. a sibling ../arona checkout         (the org dev layout)
  4. `git clone` into <repo>/targets/arona-shared  (last resort)

Usage:
    python scripts/_arona_devtools.py <module> [args...]
    e.g.
    python scripts/_arona_devtools.py cargo_cache_guard . --dry-run
    python scripts/_arona_devtools.py cargo_cache_guard . --clean-incremental
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
ARONA_GIT = os.environ.get("ARONA_GIT", "https://github.com/celestia-island/arona.git")
_SHARED_MARKER = Path("scripts") / "utils" / "cargo_cache_guard.py"


def _has_shared(arona: Path) -> bool:
    return (arona / _SHARED_MARKER).is_file()


def _candidate_roots() -> list[Path]:
    roots: list[Path] = []
    if env := os.environ.get("ARONA_ROOT"):
        roots.append(Path(env).expanduser())

    # [patch...] arona = { path = "..." } — from ~/.cargo/config.toml and this
    # repo's own Cargo.toml(s). arona is patched in as a path dep in dev.
    pat = re.compile(r'arona\s*=\s*\{\s*[^}]*\bpath\s*=\s*"([^"]+)"', re.S)
    cfgs = [Path.home() / ".cargo" / "config.toml", REPO_ROOT / "Cargo.toml"]
    cfgs += list(REPO_ROOT.glob("**/Cargo.toml"))
    for cfg in cfgs:
        try:
            text = cfg.read_text(errors="ignore")
        except OSError:
            continue
        for m in pat.finditer(text):
            p = Path(m.group(1))
            if not p.is_absolute():
                p = cfg.parent / p
            roots.append(p)

    roots.append(REPO_ROOT.parent / "arona")  # sibling layout
    return roots


def find_arona() -> Path | None:
    for cand in _candidate_roots():
        if _has_shared(cand):
            return cand.resolve()

    # Last resort: shallow clone into the repo's targets/ scratch dir.
    clone = REPO_ROOT / "targets" / "arona-shared"
    if not _has_shared(clone):
        try:
            clone.parent.mkdir(parents=True, exist_ok=True)
            _log(("warn", f"arona shared scripts not found locally — cloning into {clone}"))
            subprocess.run(
                ["git", "clone", "--depth", "1", ARONA_GIT, str(clone)],
                check=False,
            )
        except OSError as exc:
            _log(("error", f"git clone of arona failed: {exc}"))
    if _has_shared(clone):
        return clone.resolve()
    return None


def _log(level_msg: tuple[str, str]) -> None:
    level, msg = level_msg
    print(f"[arona-devtools] {level}: {msg}", file=sys.stderr)


def main() -> int:
    if len(sys.argv) < 2:
        _log(("error", "usage: _arona_devtools.py <module> [args...]"))
        return 2
    module = sys.argv[1]
    arona = find_arona()
    if arona is None:
        _log(("error", "could not locate arona shared scripts; set ARONA_ROOT"))
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
        _log(("error", f"shared module '{module}' not found in arona: {exc}"))
        return 127

    sys.argv = [module, *sys.argv[2:]]
    entry = getattr(mod, "main", None)
    if not callable(entry):
        _log(("error", f"shared module '{module}' has no main()"))
        return 2
    return int(entry() or 0)


if __name__ == "__main__":
    raise SystemExit(main())
