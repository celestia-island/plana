+++
title = "Contributing Guide"
description = """Thank you for your interest in contributing! This guide covers everything you"""
lang = "en"
category = "meta"
+++

# Contributing to Arona

Thank you for your interest in contributing! This guide covers everything you
need to get started.

## Contribution policy (read this first)

Arona defines the shared JSON-RPC 2.0 protocol types consumed across the
Entelecheia platform, so **correctness, backward compatibility, and stability
outweigh contribution throughput**. Please read this before opening a pull
request.

- **High merge bar, not a public roadmap.** Opening a PR does not imply it will

be merged. We accept a deliberately small number of changes, and only when
they fit the architecture and pass review. This is by design, not rudeness.

- **What we welcome:** bug reports, focused fixes, additive (non-breaking)

protocol fields, improved documentation, and prior design discussions before
code.

- **What we generally will not merge:** large unsolicited rewrites, breaking

changes to the protocol type surface, architectural changes without a prior
design discussion, bulk "vibe-coded" PRs, and anything that lowers the
compatibility bar of the type contract.

- **Core vs. periphery.** The protocol type definitions and their serialization

surface are held to the strictest bar and maintained by the core team.

- **CLA required.** Every accepted contribution requires a signed Contributor

License Agreement. See [`CLA.md`](CLA.md). Commits must carry a
`Signed-off-by` line (`git commit -s`).

> **The license may open; the merge bar will not.** On **2030-01-01** this
> project converts from BUSL-1.1 to Apache-2.0 or MIT (recipient's choice) — see
> [`LICENSE`](LICENSE). That broadens *what you may do with the code*; it does
> **not** lower the review bar, remove the CLA, or mean we accept more PRs. The
> contribution policy is unchanged before and after the change date.

## Security

Do **not** open public issues for security vulnerabilities. Report them privately
via [GitHub Security Advisories](https://github.com/celestia-island/arona/security/advisories/new).
See [`SECURITY.md`](SECURITY.md).

## Code of Conduct

Be respectful, constructive, and inclusive. We follow the
[Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).

## Development

Arona is a small Rust crate. Quick start:

```bash
git clone https://github.com/celestia-island/arona.git
cd arona
cargo build
cargo test
cargo clippy -- -D warnings
```

- Rust 1.85+.
- Types derive `ts-rs` (`#[derive(TS)]`) to generate TypeScript bindings — keep

`serde` attributes and `ts-rs` annotations consistent.

- Do not introduce breaking changes to existing protocol types; prefer additive

fields with `#[serde(default)]`.

## Pull request process

1. Fork and branch from `main`.
1. Discuss large or protocol-affecting changes in an issue first.
1. Make atomic commits following [Conventional Commits](https://www.conventionalcommits.org/).
1. Ensure `cargo fmt`, `cargo clippy -D warnings`, and `cargo test` pass.
1. Sign the CLA and add `Signed-off-by` to each commit.
1. Address review feedback; keep force-pushes to rebase only.

## License & CLA

Arona is licensed under the **Business Source License 1.1 (BUSL-1.1)** with a
**Change Date of 2030-01-01**, on which it converts to the recipient's choice of
**Apache-2.0 or MIT**. For all internal, academic, government, educational, and
non-commercial use it is already equivalent to Apache-2.0 or MIT today (see the
Additional Use Grant in [`LICENSE`](LICENSE)). Restricted commercial uses
(hosting, resale, or rebranding as a service) require a separate commercial
license until the Change Date.

By contributing, you agree that your contributions are licensed under the
project's license and that you sign the CLA ([`CLA.md`](CLA.md)). The CLA grants
the project a permissive license **including the right to relicense**, so the
project can keep its BUSL→Apache/MIT path and adapt its licensing in the future.
