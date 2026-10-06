#!/usr/bin/env python3
"""Check the flat-surface export lists against the generated bindings.

`packages/celestia-types/index.ts` publishes the package's flat surface.
Star-exported files contribute every type; a handful of files are
re-exported through EXPLICIT NAMED LISTS instead (their names collide with
earlier star exports), and the file's own maintenance warning says a type
added to such a generated file stays deep-import-only until somebody
appends it to the list. Nothing checked that — which is exactly how
`VersionReport` shipped absent from the surface once, and how a future
type would silently repeat it (2026-10-06 data-contract audit).

This tool parses the generated files and the index, and fails when:

  1. a type is declared in a named-list file but is neither listed nor
     documented as a deliberate shadow in the collision ledger;
  2. a listed name no longer exists in its file (a stale list entry —
     tsc catches this too, but the tool reports it with the file context).

Known boundary (accepted, SG-R2): ledger rows are trusted as written —
the tool does not verify that a documented shadow name actually exists,
so a hand-written `wins vs` row could mask a genuinely missing type. The
exposure is deliberate-editorial only: an accidental drift cannot produce
a ledger row, and editing one sits in the same review surface as editing
this script.

The collision ledger is the comment block at the top of index.ts; its
entries look like:

    //   RbacGroup                httpTypes           wins vs protocol-core-httpTypes

`wins vs <file>` marks the SHADOWED source — a type missing from that
file's named list is on purpose (the flat surface resolves the name from
the winning file instead).

Usage:
    python3 scripts/check_flat_surface.py           # exit 1 on any drift
    python3 scripts/check_flat_surface.py --list    # also print the inventory
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX = ROOT / "packages" / "celestia-types" / "index.ts"

# `export type { A, B as C, ... } from "./bindings/file";`  (multi-line lists included)
NAMED_EXPORT_RE = re.compile(
    r'export\s+type\s*\{([^}]*)\}\s*from\s*"(\./bindings/[^"]+)"\s*;',
)
# A type declaration in a generated file: `export type Name = ...`
DECL_RE = re.compile(r"^export type (\w+)", re.MULTILINE)
# One ledger row: `//   <Name> ... wins vs <file>`
LEDGER_RE = re.compile(r"^//\s+(\w+)\s+.*?\bwins vs\s+(\S+)", re.MULTILINE)


def parse_named_lists(index_text: str) -> dict[str, set[str]]:
    """file path (as written in the import) → base names in its list."""
    out: dict[str, set[str]] = {}
    for match in NAMED_EXPORT_RE.finditer(index_text):
        names_blob, file_ref = match.group(1), match.group(2)
        names = set()
        for item in names_blob.split(","):
            item = item.strip()
            if not item:
                continue
            # `HealthResponse as ServiceHealthResponse` — the listed BASE name
            # is what must exist in the file.
            names.add(item.split(" as ")[0].strip())
        out.setdefault(file_ref, set()).update(names)
    return out


def parse_ledger_shadows(index_text: str) -> dict[str, set[str]]:
    """shadowed file basename → names deliberately kept off the surface."""
    out: dict[str, set[str]] = {}
    for match in LEDGER_RE.finditer(index_text):
        name, shadowed = match.group(1), match.group(2)
        out.setdefault(shadowed, set()).add(name)
    return out


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--list", action="store_true",
                        help="print the per-file inventory as it is checked")
    args = parser.parse_args(argv)

    if not INDEX.exists():
        print(f"error: missing {INDEX.relative_to(ROOT)}", file=sys.stderr)
        return 2

    index_text = INDEX.read_text(encoding="utf-8")
    named_lists = parse_named_lists(index_text)
    shadows = parse_ledger_shadows(index_text)

    if not named_lists:
        # Zero-hit discipline: a parse regression must never read as "clean".
        print("error: no named export lists found in index.ts — parser broken?",
              file=sys.stderr)
        return 2

    problems: list[str] = []
    for file_ref, listed in sorted(named_lists.items()):
        ref = file_ref.removeprefix("./")
        path = INDEX.parent / (ref if ref.endswith(".ts") else ref + ".ts")
        if not path.exists():
            problems.append(f"{file_ref}: listed file does not exist")
            continue
        declared = set(DECL_RE.findall(path.read_text(encoding="utf-8")))
        if args.list:
            print(f"{file_ref}: {len(declared)} declared, {len(listed)} listed")

        # Positive control per file: the list must reference real types.
        for name in sorted(listed - declared):
            problems.append(f"{file_ref}: `{name}` is listed but not declared "
                            f"in the generated file (stale list entry)")

        # The actual gate: a declared type missing from the list is
        # deep-import-only unless the ledger documents the shadow.
        # Ledger rows name the shadowed file as a bindings-relative path
        # without extension (`tools/kalos`, `httpTypes`, ...) — match on
        # that, falling back to the bare basename for same-dir rows.
        rel = ref.removeprefix("bindings/").removesuffix(".ts")
        documented = shadows.get(rel, set()) | shadows.get(
            rel.removeprefix("tools/"), set()
        ) if rel.startswith("tools/") else shadows.get(rel, set())
        for name in sorted(declared - listed):
            if name in documented:
                continue
            problems.append(
                f"{file_ref}: `{name}` is generated but NOT on the flat "
                f"surface — append it to the named export list in index.ts "
                f"(or document the deliberate shadow in the collision ledger)"
            )

    if problems:
        print("error: flat-surface export lists are out of sync with the "
              "generated bindings:", file=sys.stderr)
        for p in problems:
            print(f"  {p}", file=sys.stderr)
        return 1

    total_files = len(named_lists)
    total_names = sum(len(v) for v in named_lists.values())
    print(f"ok: {total_files} named export lists cover {total_names} types, "
          f"no undocumented off-surface types")
    return 0


if __name__ == "__main__":
    sys.exit(main())
