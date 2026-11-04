import { describe, expect, it } from "vitest";
import type { Transport, TransportResponse, WsChannel, WsReadyState } from "@celestia-island/tairitsu-transport";
import { RpcClient, RpcError } from "./client.js";

/**
 * RpcError.code threading (L67 fix, Layer-1).
 *
 * Both request lanes (WS frame and HTTP body) already READ
 * `error.code` to classify `kind` (-32002/-32005 → forbidden) but used
 * to drop it on the floor — so web consumers could only string-match
 * message prose, which the web wire shape never embeds the code into.
 * These tests feed REAL envelope shapes (the exact JSON the server
 * sends; only the socket layer is faked, per the seam rule that mocks
 * may only play the far side) and assert the code survives onto the
 * rejected RpcError.
 */

function jsonResponse(status: number, body: unknown): TransportResponse {
  const text = JSON.stringify(body);
  return {
    ok: status < 400,
    status,
    statusText: "",
    headers: new Headers(),
    text: async () => text,
    // biome-ignore lint/suspicious/noExplicitAny: test fake matches the contract's <T> shape
    json: async <T = unknown>() => body as T,
  };
}

function httpOnlyTransport(status: number, body: unknown): Transport {
  return {
    request: async () => jsonResponse(status, body),
    openWs: () => {
      throw new Error("test: ws not expected on this lane");
    },
    openSse: () => {
      throw new Error("test: sse not expected on this lane");
    },
  };
}

/**
 * A WS channel that is already open and records outbound frames so the
 * test can answer with a same-id envelope — the shape a real server
 * emits. `onOpen` fires synchronously so the progressive-connect ladder
 * adopts this channel on its first round without waiting on real
 * timers.
 */
class FakeOpenWsChannel implements WsChannel {
  readyState: WsReadyState = "open";
  readonly sent: string[] = [];
  private messageHandler: ((data: string | ArrayBuffer) => void) | null = null;

  send(frame: string): void {
    this.sent.push(frame);
  }
  close(): void {
    this.readyState = "closed";
  }
  onOpen(cb: () => void): () => void {
    cb();
    return () => {};
  }
  onMessage(cb: (data: string | ArrayBuffer) => void): () => void {
    this.messageHandler = cb;
    return () => {
      this.messageHandler = null;
    };
  }
  onClose(): () => void {
    return () => {};
  }
  onError(): () => void {
    return () => {};
  }
  /** Deliver a server-shaped frame into the client. */
  deliver(data: unknown): void {
    this.messageHandler?.(JSON.stringify(data));
  }
}

function wsClient(): Promise<{ client: RpcClient; ws: FakeOpenWsChannel }> {
  const ws = new FakeOpenWsChannel();
  const transport: Transport = {
    request: async () => {
      throw new Error("test: http not expected on this lane");
    },
    openWs: () => ws,
    openSse: () => {
      throw new Error("test: sse not expected on this lane");
    },
  };
  const client = new RpcClient({
    baseUrl: "https://backend.example.com",
    getToken: () => "token-1",
    transport,
  });
  client.connect();
  // connect() flips the tier to "poll" for the ladder's first round and
  // only adopts ws when the race settles (the losing sentinel holds the
  // round open for its full attempt timeout). Calls made before the
  // "connected (ws)" state would silently ride HTTP — wait for adoption.
  return new Promise((resolve) => {
    const off = client.on("state", (e) => {
      if (e.state === "connected" && e.transportTier === "ws") {
        off();
        resolve({ client, ws });
      }
    });
  });
}

async function wsCallExpectError(envelope: {
  code: number;
  message: string;
}): Promise<RpcError> {
  const { client, ws } = await wsClient();
  try {
    const pending = client.call("demo.method", { a: 1 });
    // Let the send loop flush, then answer with the request's own id.
    await new Promise((r) => setTimeout(r, 0));
    const req = JSON.parse(ws.sent[0]) as { id: string };
    ws.deliver({
      jsonrpc: "2.0",
      id: req.id,
      error: { code: envelope.code, message: envelope.message },
    });
    await pending;
    throw new Error("expected the call to reject");
  } catch (e) {
    if (e instanceof RpcError) return e;
    throw e;
  } finally {
    await client.disconnect();
  }
}

describe("RpcError.code threading (HTTP body envelope)", () => {
  it("carries a not-found gate code (-32001) through to the rejected error", async () => {
    const client = new RpcClient({
      baseUrl: "https://backend.example.com",
      getToken: () => null,
      transport: httpOnlyTransport(200, {
        jsonrpc: "2.0",
        id: "http-1",
        error: { code: -32001, message: "revision not found (or not an applied content revision)" },
      }),
    });
    const err = await client.call("panel.revision.get", {}).catch((e: unknown) => e);
    expect(err).toBeInstanceOf(RpcError);
    const rpcErr = err as RpcError;
    expect(rpcErr.kind).toBe("rpc");
    expect(rpcErr.code).toBe(-32001);
    expect(rpcErr.message).toBe("revision not found (or not an applied content revision)");
  });

  it("keeps the forbidden classification (-32002) AND exposes the code", async () => {
    const client = new RpcClient({
      baseUrl: "https://backend.example.com",
      getToken: () => null,
      transport: httpOnlyTransport(200, {
        jsonrpc: "2.0",
        id: "http-2",
        error: { code: -32002, message: "workspace role gate refused" },
      }),
    });
    const err = await client.call("admin.rbac.listGroups", {}).catch((e: unknown) => e);
    expect(err).toBeInstanceOf(RpcError);
    const rpcErr = err as RpcError;
    expect(rpcErr.kind).toBe("forbidden");
    expect(rpcErr.code).toBe(-32002);
  });

  it("leaves code undefined when the envelope carries no code", async () => {
    const client = new RpcClient({
      baseUrl: "https://backend.example.com",
      getToken: () => null,
      transport: httpOnlyTransport(200, {
        jsonrpc: "2.0",
        id: "http-3",
        error: { message: "legacy server without codes" },
      }),
    });
    const err = await client.call("legacy.method", {}).catch((e: unknown) => e);
    expect(err).toBeInstanceOf(RpcError);
    const rpcErr = err as RpcError;
    expect(rpcErr.kind).toBe("rpc");
    expect(rpcErr.code).toBeUndefined();
  });
});

describe("RpcError.code threading (WS frame envelope)", () => {
  it("carries the generic-upstream code (-32003) on the WS lane too", async () => {
    const err = await wsCallExpectError({ code: -32003, message: "upstream blew up" });
    expect(err.kind).toBe("rpc");
    expect(err.code).toBe(-32003);
    expect(err.message).toBe("upstream blew up");
  });

  it("forbidden classification on the WS lane keeps its code (-32005)", async () => {
    const err = await wsCallExpectError({ code: -32005, message: "platform proxy grant missing" });
    expect(err.kind).toBe("forbidden");
    expect(err.code).toBe(-32005);
  });
});

describe("success path untouched", () => {
  it("still resolves envelope results without inventing errors", async () => {
    const client = new RpcClient({
      baseUrl: "https://backend.example.com",
      getToken: () => null,
      transport: httpOnlyTransport(200, {
        jsonrpc: "2.0",
        id: "http-ok",
        result: { value: 42 },
      }),
    });
    await expect(client.call<{ value: number }>("ok.method", {})).resolves.toEqual({ value: 42 });
  });
});
