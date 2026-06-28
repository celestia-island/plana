+++
title = "Contributing Guide (Entelecheia)"
description = """> This file is the English version of the contribution policy. For build commands and detailed installation steps, see the English"""
lang = "en"
category = "guides"
subcategory = "core"
+++

# Contributing Guide (Entelecheia)

> This file is the English version of the contribution policy. For build commands and detailed installation steps, see
> [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) in the repository root; the commands themselves are not translated.
> In case of ambiguity, the English version is authoritative.

## Contribution Policy (please read first)

Entelecheia can drive physical and industrial systems, so **stability and safety take priority over contribution throughput**.
Please read this section before opening a Pull Request.

- **The merge bar is high; this is not a public roadmap.** Opening a PR does not mean it will be merged. We only accept a

deliberately small number of changes that fit the architecture and pass review. This is intentional, not rude.

- **Welcomed contributions:** bug reports, focused fixes, well-scoped improvements to **extensions** (Layer 3 plugins,

device profiles, LLM provider adapters, integrations, documentation), and design discussions before writing code.

- **Usually not merged:** large unsolicited rewrites, architectural changes without prior design discussion, bulk

"vibe-coded" PRs, any change that lowers the core safety or correctness bar, and uninvited changes to the
safety-critical core that would extend review.

- **Core vs. extensions.** The core (orchestration, microkernel, security) maintains the strictest standards and is

primarily maintained by the core team. Extensions are where external contributions are most useful and most likely to
be accepted.

- **A CLA must be signed.** Every accepted contribution requires signing the Contributor License Agreement, see

[`CLA.md`](../../../CLA.md). Commits must carry `Signed-off-by` (`git commit -s`).

> **The license will open up; the merge bar will not.** On **2030-01-01**, this project transitions from BUSL-1.1 to
> SySL-1.0 (recipient's choice), see [`LICENSE`](../../../LICENSE). This relaxes *what you may do with the code*, but
> **not** the review bar, the CLA, nor does it mean we will accept more PRs. The contribution policy does not change
> around the transition date.

## Security

**Do not** report security vulnerabilities via public issues. Please report them privately via
[GitHub Security Advisories](https://github.com/celestia-island/entelecheia/security/advisories/new).
For the threat model and response SLA, see [`SECURITY.md`](../../../SECURITY.md).

## Code of Conduct

Please be respectful, constructive, and inclusive. We follow the
[Contributor Covenant Code of Conduct](../../../CODE_OF_CONDUCT.md).

## Pull Request Process

1. Fork and branch from `main`.
1. Discuss large changes in an issue first.
1. Make atomic commits that follow Conventional Commits.
1. Ensure `just ci` (or the repository's CI command) passes.
1. Sign the CLA and add `Signed-off-by`.
1. Respond to review feedback; use force-push only for rebasing.

## License & CLA

Licensed under **BUSL-1.1**, **Change Date 2030-01-01**, when it transitions to **SySL-1.0** at the recipient's choice.
Today, under internal operations, academic, government, educational, and non-commercial use, it is already equivalent to
SySL-1.0 (see the Additional Use Grant in [`LICENSE`](../../../LICENSE)). Restricted commercial use (hosting, resale,
selling as a service wrapper) requires separate commercial authorization before the Change Date.

Submitting a contribution means you agree that the contribution is licensed under this project's license and that you
sign the CLA ([`CLA.md`](../../../CLA.md)). The CLA grants the project **a permissive license including relicensing
rights**, allowing the project to maintain the BUSL→SySL path and to adjust licensing in the future.
