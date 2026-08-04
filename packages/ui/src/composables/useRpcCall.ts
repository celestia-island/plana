import { useToast } from "@celestia-island/hikari";
import {
  getMockRpcData,
  hasMockRpcData,
  isDemoHost,
  type MockRpcRegistry,
} from "./mockRpcData";

export interface RpcTransport {
  rpcCall: <T>(method: string, params?: unknown, timeoutMs?: number) => Promise<T>;
}

export interface RpcCallOptions {
  transport: RpcTransport;
  /** Optional mock registry for demo-host fallback. */
  mockData?: MockRpcRegistry;
  /** Override the demo-host probe (tests). */
  demoHostname?: string | null;
}

export interface RpcCallContext {
  /** Whether a mock payload was served for the method. */
  usedMock: boolean;
}

/**
 * JSON-RPC call helper with demo-host mock fallback and error toasting
 * (upstreamed from shittim-chest's useJsonRpc).
 *
 * - On the demo host with registered mock data, returns the mock payload.
 * - Transport errors with registered mock data return the mock payload too
 *   (demo-explorability), but only for transport/timeout failures —
 *   application-level errors (e.g. 403) reach the caller so health pollers
 *   never get fake data.
 * - Genuine application-level RPC errors are toasted (unless silent).
 */
export function createRpcCall(opts: RpcCallOptions) {
  const toast = useToast();
  const demo = () => (opts.demoHostname === undefined ? isDemoHost() : isDemoHost(opts.demoHostname));

  async function call<T>(
    method: string,
    params?: unknown,
    _httpFallback?: () => Promise<T>,
    opts2?: { silent?: boolean },
  ): Promise<T> {
    const silent = opts2?.silent ?? false;

    if (demo() && opts.mockData && hasMockRpcData(method)) {
      const mock = getMockRpcData(method, params as Record<string, unknown> | undefined);
      return (typeof structuredClone === "function"
        ? structuredClone(mock)
        : JSON.parse(JSON.stringify(mock))) as T;
    }

    try {
      return await opts.transport.rpcCall<T>(method, params);
    } catch (e) {
      const err = e as { kind?: string };
      const kind = err?.kind ?? "rpc";
      const msg = e instanceof Error ? e.message : String(e);

      // Demo fallback for transport/timeout failures only.
      if ((kind === "transport" || kind === "timeout") && opts.mockData && hasMockRpcData(method)) {
        const mock = getMockRpcData(method, params as Record<string, unknown> | undefined);
        return (typeof structuredClone === "function"
          ? structuredClone(mock)
          : JSON.parse(JSON.stringify(mock))) as T;
      }

      if (!silent && kind === "rpc") {
        toast.error(`${method}: ${msg}`);
      }
      throw e instanceof Error ? e : new Error(`RPC "${method}" failed: ${e}`);
    }
  }

  return { call };
}

export type { RpcCallContext };
