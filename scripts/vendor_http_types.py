#!/usr/bin/env python3
"""Vendor the plana foundation's generated bindings into the published package.

`packages/celestia-types` is the crate whose bindings ship as
`@celestia-island/plana-types`. npm's `files` list cannot reach a sibling
package's directory, so files the foundation generates are copied into this
package — some under a distinct name to avoid colliding with this crate's own
ts-rs output — with a provenance header. Those copies are published, and
consumers import them from there.

WHY A SCRIPT. The copies used to be hand-maintained, and they drifted:
`protocol-core-httpTypes.ts` lost the `VersionReport` type and kept a stale
`HealthResponse` doc, so everyone consuming the published package read a
contract the source had already moved past (2026-10-06 data-contract audit).
Nothing could catch it — CI's bindings-drift step named that file as a "known
vendored hand-curated shape" and excluded it, and `ops.ts` (a second copy of
the same kind) had no gate at all. Both are generated here now, and `--check`
fails when either is out of sync; that is what CI runs.

The generator is deliberately boring: header constant + the source file, byte
for byte (read and written as bytes, so newline style is part of the
contract), so a diff of a vendored file is always a diff of the real contract.

Usage:
    python3 scripts/vendor_http_types.py           # rewrite every vendored copy
    python3 scripts/vendor_http_types.py --check   # exit 1 when any is out of sync
"""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


@dataclass(frozen=True)
class Pair:
    """One vendored copy: the generated source, its published twin, and the
    provenance header prepended to it."""

    source: Path
    target: Path
    header: str


HTTP_TYPES_HEADER = """// Vendored snapshot of the plana foundation's generated bindings
// (packages/plana/bindings/httpTypes.ts — the former
// packages/protocol-core/bindings/httpTypes.ts, which the protocol-core
// re-export shim no longer owns) so the published npm tarball is
// self-contained: npm `files` cannot reach a sibling package's directory.
// Distinct filename to avoid colliding with this crate's own ts-rs-generated
// bindings/httpTypes.ts. Regenerate with `python3 scripts/vendor_http_types.py`
// (CI runs `--check`; do not hand-edit this copy).
"""

OPS_HEADER = """// Vendored snapshot of packages/plana/bindings/ops.ts (ts-rs generated) so the
// published npm tarball carries the deferred-operation wire shapes: npm `files`
// cannot reach a sibling package's directory, exactly as with
// `protocol-core-httpTypes.ts` above. Regenerate with
// `python3 scripts/vendor_http_types.py` (CI runs `--check`; do not hand-edit
// this copy).
"""

PAIRS = (
    Pair(
        source=ROOT / "packages" / "plana" / "bindings" / "httpTypes.ts",
        target=ROOT / "packages" / "celestia-types" / "bindings" / "protocol-core-httpTypes.ts",
        header=HTTP_TYPES_HEADER,
    ),
    Pair(
        source=ROOT / "packages" / "plana" / "bindings" / "ops.ts",
        target=ROOT / "packages" / "celestia-types" / "bindings" / "ops.ts",
        header=OPS_HEADER,
    ),
)


def expected_bytes(pair: Pair) -> bytes:
    """Header + the source file, byte for byte (the source's trailing
    newlines collapse to exactly one, the only normalisation performed)."""
    body = pair.source.read_bytes().rstrip(b"\n")
    return pair.header.encode("utf-8") + body + b"\n"


def describe_divergence(have: bytes, want: bytes) -> list[str]:
    """Line-precise pointers at the first divergence.

    Covers both shapes: a changed line (the interesting case when the source
    gains or edits a type) and a pure add/remove at the tail, where a plain
    line-by-line walk would run out of lines before finding anything.
    """
    have_lines = have.decode("utf-8", "replace").splitlines()
    want_lines = want.decode("utf-8", "replace").splitlines()
    for i, (a, b) in enumerate(zip(have_lines, want_lines), start=1):
        if a != b:
            return [
                f"  first difference at line {i}:",
                f"    vendored: {a}",
                f"    expected: {b}",
            ]
    if len(have_lines) != len(want_lines):
        i = min(len(have_lines), len(want_lines)) + 1
        side = "vendored" if len(have_lines) > len(want_lines) else "expected"
        longer = have_lines if len(have_lines) > len(want_lines) else want_lines
        return [
            f"  line counts differ: vendored {len(have_lines)} vs expected {len(want_lines)}",
            f"  first unmatched line {i} exists only in {side}:",
            f"    {longer[i - 1]}",
        ]
    return ["  contents differ but every line matches (trailing bytes differ)"]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify every vendored copy is in sync instead of rewriting it",
    )
    args = parser.parse_args(argv)

    missing = [
        path
        for pair in PAIRS
        for path in (pair.source, pair.target)
        if not path.exists()
    ]
    if missing:
        for path in missing:
            print(f"error: missing {path.relative_to(ROOT)}", file=sys.stderr)
        return 2

    failures = 0
    for pair in PAIRS:
        want = expected_bytes(pair)
        have = pair.target.read_bytes()
        target = pair.target.relative_to(ROOT)
        source = pair.source.relative_to(ROOT)

        if have == want:
            print(f"{'ok' if args.check else 'unchanged'}: {target}")
            continue
        if args.check:
            failures += 1
            print(
                f"error: {target} is out of sync with {source} — "
                "run `python3 scripts/vendor_http_types.py`",
                file=sys.stderr,
            )
            for line in describe_divergence(have, want):
                print(line, file=sys.stderr)
            continue
        pair.target.write_bytes(want)
        print(f"wrote {target} from {source}")

    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
