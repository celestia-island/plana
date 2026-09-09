export { RpcClient } from "./client.js";
export { RpcError } from "./client.js";
export type {
  RpcClientOpts,
  HeartbeatProtocol,
  ConnectionState,
  ConnectionStateEvent,
  RpcErrorKind,
  RpcNotification,
} from "./client.js";

// Relay extension (edge side) — see docs/en/rpc/relay-profile.md.
export { RELAY, relayMethods, RelayClient, createTauriTransport, createMemoryTransport } from "./relay.js";
export type { RelayContext, EdgeFrame, EdgeTransport } from "./relay.js";
