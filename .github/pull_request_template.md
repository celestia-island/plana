## Summary

Brief description of what this PR changes and why.

## Checklist

- [ ] Code follows the project's style guidelines (rustfmt / clippy clean)
- [ ] Self-reviewed the diff
- [ ] Added or updated tests for the changes
- [ ] Updated documentation if needed
- [ ] No new warnings introduced

## Version governance (publishable members only)

Skip this section when no `publish != false` member's public API changes
(`plana`, `plana-celestia-types`, `plana-celestia-config`,
`plana-evernight-client`, `plana-rpc-client`, `plana-rpc-server`,
`plana-tauri`). Internal (`publish = false`) members are not on the release
line.

- [ ] **Breaking public-API change** (removed/renamed pub item, tightened
      signature, new required trait method, enum variant removal): the
      crate's own version line MUST bump its minor digit (`0.2.x → 0.3.0`)
      in this PR — under semver a 0.x minor IS the breaking digit. A patch
      bump (`0.2.x → 0.2.x+1`) is only for additive/fix changes. The
      `cargo-v*` release lane runs cargo-semver-checks and refuses a
      breaking diff on a non-breaking bump, so bump here, not at tag time.
- [ ] **Additive public-API change** (new pub item, new optional dep or
      feature): patch bump of the crate's own line.
- [ ] **No public-API change in any publishable member**: no bump needed;
      the workspace `0.1.x` line is frozen for internal members and does
      not move for feature work.
