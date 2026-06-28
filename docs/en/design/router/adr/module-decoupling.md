# Module Decoupling and Type Ownership

- **Status**: Accepted
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

The original codebase placed all data types in a single `types.rs` monolith — mixing screen, hardware, remote, protocol, and tunnel types. This created circular dependency risks and made it unclear which module "owned" which types. Adding a new feature (e.g., VNC) would require modifying the shared types file.

## Decision

Move each domain's types into its own module's `types.rs` file (e.g., `screen/types.rs`, `hardware/types.rs`). The root `types.rs` and `prelude.rs` re-export all types for backward compatibility. Each module is the sole owner of its types and can evolve independently.

## Consequences

### Positive

- Clear type ownership; no cross-module type conflicts
- New modules don't touch existing files

### Negative

- Consumers must import from the correct module or use prelude
- Re-export layer adds indirection
