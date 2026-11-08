import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  TransportError,
  type SseChannel,
  type SseReadyState,
  type Transport,
  type TransportResponse,
  type WsChannel,
  type WsCloseInfo,
  type WsReadyState,
} from "@celestia-island/tairitsu-transport";
import { RpcClient, RpcError } from "./client.js";

/**
 * Established-link liveness.
 *
 * The client reports `connected` for the poll tier even though poll has
 * neither a push channel nor the ws heartbeat. These tests pin the signals
 * that make the state honest:
 *
 *  - a periodic `/api/health` probe turns a dead poll lane into
 *    `reconnecting` and the SAME recovery ladder brings it back;
 *  - a network-level HTTP failure does the same immediately, without
 *    waiting for the probe;
 *  - an established SSE channel's error re-enters the SAME recovery loop;
 *  - the browser's `offline` event surfaces `reconnecting` AND arms the
 *    ladder (a bare state flip would park the light yellow forever);
 *  - a non-JSON 200 from the liveness URL is not a live backend.
 *
 * Server-side failures (HTTP status, JSON-RPC error, malformed 200 body,
 * call timeout) must NOT trigger recovery — they are not link-loss
 * evidence. Every behavior test advances the fake clock past the recovery
 * delay and asserts a real `connected` return, so a mutation that replaces
 * the ladder with a bare state flip goes red.
 */

function jsonResponse(status: number, body: unknown): TransportResponse {
  const text = JSON.stringify(body);
  return {
    ok: status < 400,
    status,
    statusText: "",
    headers: new Headers({ "content-type": "application/json" }),
    text: async () => text,
    json: async <T = unknown>() => body as T,
  };
}

function healthOk(): TransportResponse {
  return jsonResponse(200, { status: "ok" });
}

/** A 200 that is not the backend: an HTML maintenance / SPA interstitial. */
function nonJsonResponse(): TransportResponse {
  return {
    ok: true,
    status: 200,
    statusText: "",
    headers: new Headers({ "content-type": "text/html" }),
    text: async () => "<!doctype html><html><body>maintenance</body></html>",
    json: async () => {
      throw new SyntaxError("Unexpected token '<'");
    },
  };
}

class FakeWsChannel implements WsChannel {
  readyState: WsReadyState = "open";
  sendCount = 0;
  closeCount = 0;
  private closeCb: ((info: WsCloseInfo) => void) | null = null;

  constructor(
    private readonly failSend = false,
    private readonly failClose = false,
  ) {}

  send(): void {
    this.sendCount += 1;
    if (this.failSend) throw new Error("test: ws send failed");
  }

  close(): void {
    if (this.failClose) throw new Error("test: ws close failed");
    this.closeCount += 1;
    this.readyState = "closed";
    this.closeCb?.({ code: 1006, reason: "gone", wasClean: false });
  }

  onOpen(cb: () => void): () => void {
    cb();
    return () => {};
  }

  onMessage(): () => void {
    return () => {};
  }

  onClose(cb: (info: WsCloseInfo) => void): () => void {
    this.closeCb = cb;
    return () => { this.closeCb = null; };
  }

  onError(): () => void {
    return () => {};
  }
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
  sse?: SseChannel | (() => SseChannel);
  ws?: WsChannel | (() => WsChannel);
}): Transport {
  return {
    request: opts.request,
    openWs: () => {
      if (!opts.ws) throw new Error("test: ws unavailable");
      return typeof opts.ws === "function" ? opts.ws() : opts.ws;
    },
    openSse: () => {
      if (!opts.sse) throw new Error("test: sse unavailable");
      return typeof opts.sse === "function" ? opts.sse() : opts.sse;
    },
  };
}

async function settleConnect(): Promise<void> {
  // #raceTransports always waits the round's full timeout (1s) even when
  // every tier has already answered, so the fake clock must cross it.
  await vi.advanceTimersByTimeAsync(1_000);
}

/** Cross the recovery delay (1s + ≤0.5s jitter) and one 1s connect round. */
async function settleRecovery(): Promise<void> {
  await vi.advanceTimersByTimeAsync(3_000);
}

/** Install the minimal browser lifecycle surface the client binds to. */
function stubPageLifecycle(): Record<string, () => void> {
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
  return listeners;
}

describe("established-link liveness", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("turns a dead poll lane into reconnecting and recovers via the periodic probe", async () => {
    let healthCalls = 0;
    const healthUrls: string[] = [];
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) {
          healthCalls += 1;
          healthUrls.push(req.url);
          // 1 = connect, 2 = the liveness beat that detects the drop,
          // 3+ = the recovery ladder's health probe.
          if (healthCalls === 2) throw new TransportError("network", "connection closed");
          return healthOk();
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
    // The liveness beat must bypass the HTTP cache; the connect probe
    // keeps the clean URL.
    expect(healthUrls[0]).not.toContain("?_=");
    expect(healthUrls[1]).toContain("?_=");

    await settleRecovery();

    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("poll");
    expect(states[states.length - 1]).toBe("connected");
    expect(healthCalls).toBeGreaterThan(2);
    await client.disconnect();
  });

  it("turns a network-level RPC failure into reconnecting immediately and recovers", async () => {
    let rpcHealthy = false;
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        if (!rpcHealthy) throw new TransportError("network", "connection closed");
        return jsonResponse(200, { result: { ok: true } });
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

    rpcHealthy = true;
    await settleRecovery();

    expect(client.state).toBe("connected");
    expect(states[states.length - 1]).toBe("connected");
    await client.disconnect();
  });

  it("arms recovery even when the ws transport throws from close()", async () => {
    const listeners = stubPageLifecycle();
    const ws = new FakeWsChannel(false, true);
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
      ws,
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();
    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("ws");

    listeners.offline?.();

    expect(states).toContain("reconnecting");
    expect(client.state).toBe("reconnecting");
    await client.disconnect();
  });

  it("turns a ws send failure into reconnecting", async () => {
    const ws = new FakeWsChannel(true);
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
      ws,
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();
    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("ws");

    const err = await client.call("demo.method").catch((e: unknown) => e);

    expect(err).toBeInstanceOf(RpcError);
    expect((err as RpcError).kind).toBe("transport");
    expect(states).toContain("reconnecting");
    expect(client.state).toBe("reconnecting");
    await client.disconnect();
  });

  it("does not treat a caller-side serialization error as link loss on the ws tier", async () => {
    const ws = new FakeWsChannel();
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
      ws,
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();
    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("ws");

    const circular: Record<string, unknown> = {};
    circular.self = circular;
    const err = await client.call("demo.method", circular).catch((e: unknown) => e);

    expect(err).toBeInstanceOf(RpcError);
    expect((err as RpcError).kind).toBe("transport");
    expect(client.state).toBe("connected");
    expect(ws.sendCount).toBe(0);
    expect(ws.closeCount).toBe(0);
    expect(states).not.toContain("reconnecting");
    await client.disconnect();
  });

  it("does not treat an HTTP status, a JSON-RPC error, a malformed 200 or a timeout as link loss", async () => {
    type RpcMode = "http500" | "rpcError" | "badJson" | "nullBody" | "arrayBody" | "scalarBody" | "timeout";
    let rpcMode: RpcMode = "http500";
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        if (rpcMode === "http500") return jsonResponse(500, { error: "boom" });
        if (rpcMode === "rpcError") {
          return jsonResponse(200, { jsonrpc: "2.0", id: "x", error: { code: -32003, message: "upstream" } });
        }
        if (rpcMode === "badJson") return nonJsonResponse();
        if (rpcMode === "nullBody") return jsonResponse(200, null);
        if (rpcMode === "arrayBody") return jsonResponse(200, [1, 2, 3]);
        if (rpcMode === "scalarBody") return jsonResponse(200, "ok");
        throw new TransportError("timeout", "slow");
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();

    for (const mode of ["http500", "rpcError", "badJson", "nullBody", "arrayBody", "scalarBody", "timeout"] as const) {
      rpcMode = mode;
      await expect(client.call("demo.method"), mode).rejects.toBeInstanceOf(RpcError);
      expect(client.state, mode).toBe("connected");
    }

    // A caller-side serialization error is not a link verdict either.
    const circular: Record<string, unknown> = {};
    circular.self = circular;
    await expect(client.call("demo.method", circular)).rejects.toBeInstanceOf(RpcError);
    expect(client.state).toBe("connected");
    expect(states).not.toContain("reconnecting");
    await client.disconnect();
  });

  it("leaves the green light alone on healthy poll beats", async () => {
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();
    expect(client.state).toBe("connected");
    const connectedCount = states.filter((s) => s === "connected").length;

    await vi.advanceTimersByTimeAsync(30_000);

    expect(client.state).toBe("connected");
    expect(states).not.toContain("reconnecting");
    expect(states.filter((s) => s === "connected")).toHaveLength(connectedCount);
    await client.disconnect();
  });

  it("re-enters recovery when an established SSE channel errors", async () => {
    const sseChannels: FakeSseChannel[] = [];
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
      sse: () => {
        const channel = new FakeSseChannel();
        sseChannels.push(channel);
        return channel;
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();

    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("sse");
    expect(sseChannels).toHaveLength(1);

    sseChannels[0]!.fireError();

    expect(states).toContain("reconnecting");
    expect(client.state).toBe("reconnecting");

    await settleRecovery();

    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("sse");
    expect(sseChannels.length).toBeGreaterThan(1);
    await client.disconnect();
  });

  it("treats a non-JSON 200 health response as a dead liveness beat", async () => {
    let healthCalls = 0;
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) {
          healthCalls += 1;
          return healthCalls === 1 ? healthOk() : nonJsonResponse();
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

    await vi.advanceTimersByTimeAsync(15_000);

    expect(states).toContain("reconnecting");
    expect(client.state).toBe("reconnecting");
    await client.disconnect();
  });

  it("closes the adopted SSE channel before recovering from an offline edge", async () => {
    const listeners = stubPageLifecycle();
    const sseChannels: FakeSseChannel[] = [];
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
      sse: () => {
        const channel = new FakeSseChannel();
        sseChannels.push(channel);
        return channel;
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();
    await settleConnect();
    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("sse");
    expect(sseChannels).toHaveLength(1);

    listeners.offline?.();

    expect(client.state).toBe("reconnecting");
    // The adopted channel must be closed (a real EventSource would keep
    // reconnecting and delivering into the old handler set).
    expect(sseChannels[0]!.readyState).toBe("closed");

    await settleRecovery();

    expect(client.state).toBe("connected");
    expect(client.transportTier).toBe("sse");
    expect(sseChannels.length).toBeGreaterThan(1);
    expect(states[states.length - 1]).toBe("connected");
    await client.disconnect();
  });

  it("does not report a connected poll tier when the health probe is non-JSON at connect", async () => {
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return nonJsonResponse();
        throw new Error("test: rpc not expected");
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    const states: string[] = [];
    client.on("state", (e) => states.push(e.state));
    client.connect();

    // Three progressive rounds: 1s + 3s + 5s.
    await vi.advanceTimersByTimeAsync(10_000);

    expect(states).not.toContain("connected");
    expect(client.state).toBe("failed");
    await client.disconnect();
  });

  it("cancels the reconnect countdown when the browser comes back online inside the backoff", async () => {
    const listeners = stubPageLifecycle();
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
        throw new Error("test: rpc not expected");
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    client.connect();
    await settleConnect();
    expect(client.state).toBe("connected");

    listeners.offline?.();
    await vi.advanceTimersByTimeAsync(100);
    listeners.online?.();

    // The leaked countdown tick used to re-stamp reconnecting after this
    // ladder had already reached connected.
    await vi.advanceTimersByTimeAsync(5_000);

    expect(client.state).toBe("connected");
    await client.disconnect();
    expect(vi.getTimerCount()).toBe(0);
  });

  it("cancels the reconnect countdown when the host retries during the backoff", async () => {
    let healthCalls = 0;
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) {
          healthCalls += 1;
          if (healthCalls === 2) throw new TransportError("network", "connection closed");
          return healthOk();
        }
        throw new Error("test: rpc not expected");
      },
    });
    const client = new RpcClient({ baseUrl: "https://backend.example.com", getToken: () => "t", transport });
    client.connect();
    await settleConnect();
    await vi.advanceTimersByTimeAsync(15_000);
    expect(client.state).toBe("reconnecting");

    client.forceReconnect();
    await vi.advanceTimersByTimeAsync(2_500);

    expect(client.state).toBe("connected");
    await client.disconnect();
    expect(vi.getTimerCount()).toBe(0);
  });

  it("surfaces the browser offline edge as reconnecting and arms the recovery ladder", async () => {
    const listeners = stubPageLifecycle();
    const transport = makeTransport({
      request: async (req) => {
        if (req.url.includes("/api/health")) return healthOk();
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

    // No `online` event here: the ladder must be armed by `offline` itself.
    await settleRecovery();

    expect(client.state).toBe("connected");
    expect(states[states.length - 1]).toBe("connected");
    await client.disconnect();
  });
});
