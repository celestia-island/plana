// Published types-only entry for @celestia-island/plana-evernight-protocol.
//
// Re-exports the .d.ts mirror of the ts-rs generated bindings. Only the
// mirror ships to npm (see `files` in package.json): npm packages must not
// publish raw TypeScript, so `bindings/evernightProtocol.d.ts` is kept a
// byte-identical copy of `bindings/evernightProtocol.ts` — a pin enforced
// by tests/bindings_dts.rs.
export * from "./bindings/evernightProtocol";
