/**
 * Demo-host mock RPC data registry (upstreamed from shittim-chest).
 *
 * A webui registers mock payloads per RPC method (defineMockRpcData); when
 * served on the demo host (or an overridden hostname) and the upstream is
 * unreachable, `getMockRpcData` returns the registered payload so admin UI
 * stays explorable. Data values may be plain values or functions of params.
 */

export type MockRpcValue =
  | unknown
  | ((params?: Record<string, unknown>) => unknown);

export type MockRpcRegistry = Record<string, MockRpcValue>;

const registry: MockRpcRegistry = {};
let mockHost: string | null = null;

/** Override the demo-host check (tests, staging). */
export function setMockHost(hostname: string | null): void {
  mockHost = hostname;
}

/** The canonical demo hostname. */
export function isDemoHost(hostname = typeof location !== "undefined" ? location.hostname : ""): boolean {
  return mockHost !== null ? hostname === mockHost : hostname === "demo.dev.celestia.world";
}

/** Register mock payloads for a set of RPC methods. */
export function defineMockRpcData(data: MockRpcRegistry): void {
  Object.assign(registry, data);
}

/** Whether a method has registered mock data. */
export function hasMockRpcData(method: string): boolean {
  return method in registry;
}

/** Get the (possibly param-dependent) mock payload for a method. */
export function getMockRpcData(
  method: string,
  params?: Record<string, unknown>,
): unknown | undefined {
  const value = registry[method];
  if (value === undefined) return undefined;
  return typeof value === "function" ? (value as (p?: Record<string, unknown>) => unknown)(params) : value;
}
