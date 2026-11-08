import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  TransportError,
  type SseChannel,
  type SseReadyState,
  type Transport,
  type TransportResponse,
} from "@celestia-island/tairitsu-transport";
import { RpcClient, RpcError } from "./client.js";

/**
 * Established-link liveness.
 *
 * The client reports `connected` for the poll tier even though poll has
 * neither a push channel nor the ws heartbeat. These tests pin the two
 * signals that make the state honest:
 *
 *  - a periodic `/api/health` probe turns a dead poll lane into
 *    `reconnecting` (and the recovery ladder takes over);
 *  - a network-level HTTP failure does the same immediately, without
 *    waiting for the probe;
 *  - an established SSE channel's error re-enters the SAME recovery loop
 *    instead of leaving a stale green light with a dead notification lane;
 *  - the browser's `offline` event surfaces `reconnecting` immediately.
 *
 * Server-side failures (HTTP status, JSON-RPC error, call timeout) must
 * NOT trigger recovery — they are not link-loss evidence.
 */

function jsonResponse(status: number, body: unknown): TransportResponse {
  const text = JSON.stringify(body);
  return {
    ok: status < 400,
    status,
    statusText: "",
    headers: new Headers(),
    text: async () => text,
    json: async <T = unknown>() => body as T,
  };
}

function healthOk(): TransportResponse {
  return jsonResponse(200, { status: "ok" });
}

class FakeSseChannel implements SseChannel {
  readyState: SseReadyState = "open";
  private openCb: (() => void) | null = null;
  private errorCb: ((err: unknown) => void) | null = null;

  onOpen(cb: () => void): () => void {
    this.openCb = cb;
    cb();
    return () => { this.openCb = null; };
  }

  onEvent(): () => void {
    return () => {};
  }

  onError(cb: (err: unknown) => void): () => void {
    this.errorCb = cb;
    return () => { this.errorCb = null; };
  }

  close(): void {
    this.readyState = "closed";
  }

  fireError(err: unknown = new TransportError("channel-closed", "gone")): void {
    this.errorCb?.(err);
  }
}

type RequestHandler = (req: { url: string; method?: string }) => Promise<TransportResponse>;

function makeTransport(opts: {
  request: RequestHandler;
  sse?: SseChannel;
}): Transport {
  return {
    request: opts.request,
    openWs: () => {
      throw new Error("test: ws unavailable");
    },
    openSse: () => {
      if (!opts.sse) throw new Error("test: sse unavailable");
      return opts.sse;
    },
  };
}

async function settleConnect(): Promise<void> {
  // #raceTransports always waits the round's full timeout (1s) even when
  // every tier has already answered, so the fake clock must cross it.
  await vi.advanceTimersByTimeAsync(1_000);
}

describe("established-link liveness", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("turns a dead poll lane into reconnecting via the periodic probe", async () => {
    let healthCalls = 0;
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.endsWith("/api/health")) {
          healthCalls += 1;
          if (healthCalls === 1) return healthOk();
          throw new TransportError("network", "connection closed");
        }
        throw new Error("test: rpc not expected");
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();

    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("poll");

    await vi.advanceTimersByTimeAsync(15_000);

    expect(states).toContain("reconnecting");
    expect(client.state).toBe("reconnecting");
    await client.disconnect();
  });

  it("turns a network-level RPC failure into reconnecting immediately", async () => {
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.endsWith("/api/health")) return healthOk();
        throw new TransportError("network", "connection closed");
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();
    expect(client.state).toBe("connected");

    const err = await client.call("demo.method").catch((e: unknown) => e);

    expect(err).toBeInstanceOf(RpcError);
    expect((err as RpcError).kind).toBe("transport");
    expect(states).toContain("reconnecting");
    expect(client.state).toBe("reconnecting");
    await client.disconnect();
  });

  it("does not treat an HTTP status or a JSON-RPC error as link loss", async () => {
    let rpcMode: "http500" | "rpcError" = "http500";
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.endsWith("/api/health")) return healthOk();
        if (rpcMode === "http500") return jsonResponse(500, { error: "boom" });
        return jsonResponse(200, { jsonrpc: "2.0", id: "x", error: { code: -32003, message: "upstream" } });
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();

    await expect(client.call("demo.method")).rejects.toBeInstanceOf(RpcError);
    expect(client.state).toBe("connected");

    rpcMode = "rpcError";
    await expect(client.call("demo.method")).rejects.toBeInstanceOf(RpcError);
    expect(client.state).toBe("connected");
    expect(states).not.toContain("reconnecting");
    await client.disconnect();
  });

  it("re-enters recovery when an established SSE channel errors", async () => {
    const sse = new FakeSseChannel();
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.endsWith("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
      sse,
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();

    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("sse");

    sse.fireError();

    expect(states).toContain("reconnecting");
    expect(client.state).toBe("reconnecting");
    await client.disconnect();
  });

  it("surfaces the browser offline edge as reconnecting immediately", async () => {
    const listeners: Record<string, () => void> = {};
    vi.stubGlobal("window", {
      addEventListener: (type: string, cb: () => void) => { listeners[type] = cb; },
      removeEventListener: (type: string) => { delete listeners[type]; },
    });
    vi.stubGlobal("document", {
      visibilityState: "visible",
      addEventListener: () => {},
      removeEventListener: () => {},
    });

    const transport = makeTransport({
      request: async (req) => {
        if (req.url.endsWith("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();
    expect(client.state).toBe("connected");

    listeners.offline?.();

    expect(states).toContain("reconnecting");
    expect(client.state).toBe("reconnecting");
    await client.disconnect();
  });
});
