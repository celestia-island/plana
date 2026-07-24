export type ConnectionState =
  | "connected"
  | "disconnected"
  | "connecting"
  | "reconnecting"
  | "failed";

export interface ConnectionStateEvent {
  state: ConnectionState;
  retryIn?: number;
}

export type RpcErrorKind =
  | "transport"
  | "timeout"
  | "forbidden"
  | "rpc";

export class RpcError extends Error {
  readonly kind: RpcErrorKind;
  readonly method: string;
  constructor(kind: RpcErrorKind, method: string, message: string) {
    super(message);
    this.name = "RpcError";
    this.kind = kind;
    this.method = method;
  }
}

export interface RpcNotification {
  method: string;
  params: unknown;
}

export interface RpcClientOpts {
  baseUrl: string;
  rpcPath?: string;
  getToken: () => string | null;
  onAuthLost?: () => void;
  heartbeatInterval?: number;
  heartbeatTimeout?: number;
  callTimeoutMs?: number;
  /** Max EventSource retries before falling back to long polling (default 3). */
  sseMaxRetries?: number;
  /** Poll interval in ms for tier-3 long polling (default 30_000). */
  pollIntervalMs?: number;
  /**
   * Force local (tier 0) mode even when baseUrl is not localhost.
   * When true, the client skips WebSocket and uses HTTP directly.
   * Auto-detected when baseUrl is localhost / 127.0.0.1 / [::1].
   */
  local?: boolean;
}

type NotificationHandler = (n: RpcNotification) => void;
type BinaryHandler = (data: ArrayBuffer) => void;
type StateHandler = (e: ConnectionStateEvent) => void;
type HeartbeatHandler = () => void;
type AuthLostHandler = () => void;

interface PendingCall {
  resolve: (v: unknown) => void;
  reject: (e: Error) => void;
  timer: ReturnType<typeof setTimeout>;
  method: string;
}

type TransportTier = "local" | "ws" | "sse" | "poll";

const HB_INTERVAL = 15_000;
const HB_TIMEOUT = 10_000;
const CALL_TIMEOUT = 30_000;
const LOCAL_CALL_TIMEOUT = 5_000;
const SSE_MAX_RETRIES = 3;
const POLL_INTERVAL = 30_000;

function isLocalhost(baseUrl: string): boolean {
  try { return new URL(baseUrl).hostname === "localhost" || new URL(baseUrl).hostname === "127.0.0.1" || new URL(baseUrl).hostname === "[::1]"; }
  catch { return false; }
}

export class RpcClient {
  readonly #baseUrl: string;
  readonly #rpcPath: string;
  readonly #getToken: () => string | null;
  readonly #onAuthLost?: () => void;
  readonly #heartbeatInterval: number;
  readonly #heartbeatTimeout: number;
  readonly #callTimeoutMs: number;
  readonly #sseMaxRetries: number;
  readonly #pollIntervalMs: number;

  #ws: WebSocket | null = null;
  #wsGen = 0;
  #idCounter = 0;
  #pending = new Map<string, PendingCall>();
  #disposed = false;

  #sessionId: string;
  #eventSource: EventSource | null = null;
  #sseRetryCount = 0;
  #sseRetryTimer: ReturnType<typeof setTimeout> | null = null;

  #pollTimer: ReturnType<typeof setInterval> | null = null;
  #tier: TransportTier = "ws";
  readonly #local: boolean;

  #hbTimer: ReturnType<typeof setInterval> | null = null;
  #hbAckTimer: ReturnType<typeof setTimeout> | null = null;

  #notifHandlers = new Set<NotificationHandler>();
  #binaryHandlers = new Set<BinaryHandler>();
  #stateHandlers = new Set<StateHandler>();
  #heartbeatHandlers = new Set<HeartbeatHandler>();
  #authLostHandlers = new Set<AuthLostHandler>();

  #state: ConnectionState = "disconnected";
  #wsConnectGate: Promise<void> | null = null;

  get state(): ConnectionState { return this.#state; }
  get connected(): boolean { return this.#ws?.readyState === WebSocket.OPEN; }
  get transportTier(): TransportTier { return this.#tier; }

  constructor(opts: RpcClientOpts) {
    this.#baseUrl = opts.baseUrl.replace(/\/+$/, "");
    this.#rpcPath = opts.rpcPath ?? "/api/rpc";
    this.#getToken = opts.getToken;
    this.#onAuthLost = opts.onAuthLost;
    this.#heartbeatInterval = opts.heartbeatInterval ?? HB_INTERVAL;
    this.#heartbeatTimeout = opts.heartbeatTimeout ?? HB_TIMEOUT;
    this.#callTimeoutMs = opts.callTimeoutMs ?? CALL_TIMEOUT;
    this.#sseMaxRetries = opts.sseMaxRetries ?? SSE_MAX_RETRIES;
    this.#pollIntervalMs = opts.pollIntervalMs ?? POLL_INTERVAL;
    this.#sessionId = crypto.randomUUID();
    this.#local = opts.local ?? isLocalhost(this.#baseUrl);
  }

  // ── main API ────────────────────────────────────────────

  async call<T>(method: string, params?: unknown, timeoutMs?: number): Promise<T> {
    const timeout = timeoutMs ?? (this.#tier === "local" ? LOCAL_CALL_TIMEOUT : this.#callTimeoutMs);

    // Tier 0 / Tiers 2/3: HTTP POST
    if (this.#tier !== "ws") {
      try {
        return await this.#sendOverHttp<T>(method, params, timeout);
      } catch (e) {
        // On local tier: first failure downgrades to ws (backend may have moved)
        if (this.#tier === "local" && !this.#disposed) {
          console.info("[RpcClient:local] request failed, downgrading to ws");
          this.#downgradeToWs();
        }
        throw e;
      }
    }

    // Tier 1: WS if connected
    if (this.connected) {
      return this.#sendOverWs<T>(method, params, timeout);
    }

    if (this.#wsConnectGate) {
      try { await Promise.race([this.#wsConnectGate, sleep(3000)]); } catch { /* fall through */ }
      if (this.connected) return this.#sendOverWs<T>(method, params, timeout);
    }

    return this.#sendOverHttp<T>(method, params, timeout);
  }

  connect(): void {
    this.#disposed = false;
    if (this.#local) {
      this.#tier = "local";
      console.info("[RpcClient:local] detected localhost, using direct HTTP");
      this.#setState("connected");
    } else {
      this.#tier = "ws";
      this.#connectWs();
    }
  }

  async disconnect(): Promise<void> {
    this.#disposed = true;
    this.#teardownAll();
    this.#setState("disconnected");
  }

  forceReconnect(): void {
    if (this.#disposed) return;
    this.#teardownAll();
    if (this.#local) {
      this.#tier = "local";
      this.#setState("connected");
    } else {
      this.#tier = "ws";
      this.#connectWs();
    }
  }

  on(event: "notification", handler: NotificationHandler): () => void;
  on(event: "binary", handler: BinaryHandler): () => void;
  on(event: "state", handler: StateHandler): () => void;
  on(event: "heartbeat", handler: HeartbeatHandler): () => void;
  on(event: "authLost", handler: AuthLostHandler): () => void;
  on(event: string, handler: (...args: any[]) => void): () => void {
    const sets: Record<string, Set<(...args: any[]) => void>> = {
      notification: this.#notifHandlers,
      binary: this.#binaryHandlers,
      state: this.#stateHandlers,
      heartbeat: this.#heartbeatHandlers,
      authLost: this.#authLostHandlers,
    };
    const set = sets[event];
    if (!set) throw new Error(`unknown event: ${event}`);
    set.add(handler);
    return () => set.delete(handler);
  }

  // ═══════════════════════════════════════════════════════════
  // Tier 0 → 1 downgrade
  // ═══════════════════════════════════════════════════════════

  #downgradeToWs(): void {
    this.#tier = "ws";
    this.#teardownAll();
    this.#connectWs();
  }

  // ═══════════════════════════════════════════════════════════
  // Tier 1 — WebSocket
  // ═══════════════════════════════════════════════════════════

  #connectWs(): void {
    if (this.#tier !== "ws" || this.#disposed) return;
    if (!this.#getToken()) return;
    if (this.#ws) {
      const rs = this.#ws.readyState;
      if (rs === WebSocket.CONNECTING || rs === WebSocket.OPEN) return;
    }
    this.#setState("connecting");
    const gen = this.#wsGen = this.#wsGen + 1;

    const wsUrl = this.#baseUrl.replace(/^http/, "ws") + this.#rpcPath;
    const token = this.#getToken();
    if (!token) return;

    const ws = new WebSocket(wsUrl, ["jwt." + token]);
    ws.binaryType = "arraybuffer";
    this.#ws = ws;

    ws.onopen = () => {
      if (this.#wsGen !== gen) { ws.close(1000, "stale"); return; }
      this.#setState("connected");
      this.#startHeartbeat();
    };

    ws.onclose = () => {
      if (this.#wsGen !== gen || this.#tier !== "ws") return;
      this.#clearHeartbeat();
      this.#rejectAllPending("connection lost");
      this.#setState("disconnected");
    };

    ws.onerror = () => {
      if (this.#wsGen !== gen || this.#tier !== "ws") return;
      // WS failed — tear down and downgrade to tier 2 (SSE)
      this.#teardownAll();
      this.#downgradeToSse();
    };

    ws.onmessage = (event) => {
      if (this.#wsGen !== gen) return;
      this.#resetHeartbeatTimeout();

      if (event.data instanceof ArrayBuffer) {
        this.#binaryHandlers.forEach((h) => h(event.data as ArrayBuffer));
        return;
      }
      if (event.data instanceof Blob) {
        event.data.arrayBuffer().then((buf) => { this.#binaryHandlers.forEach((h) => h(buf)); }).catch(() => {});
        return;
      }

      let data: any;
      try { data = JSON.parse(event.data); } catch { return; }

      if (data.method === "Base.HeartbeatAck") {
        this.#resetHeartbeatTimeout();
        this.#heartbeatHandlers.forEach((h) => h());
        return;
      }

      if (data.method && data.id === undefined) {
        this.#notifHandlers.forEach((h) => h({ method: data.method, params: data.params }));
        return;
      }

      if (data.id !== undefined) {
        const id = String(data.id);
        const entry = this.#pending.get(id);
        if (entry) {
          this.#pending.delete(id);
          clearTimeout(entry.timer);
          if (data.error) {
            const msg: string = data.error.message || "unknown rpc error";
            const kind: RpcErrorKind = data.error.code === -32003 ? "forbidden" : "rpc";
            entry.reject(new RpcError(kind, entry.method, msg));
          } else {
            entry.resolve(data.result);
          }
        }
      }
    };
  }

  #sendOverWs<T>(method: string, params: unknown, timeoutMs: number): Promise<T> {
    const id = `rpc-${(++this.#idCounter).toString(36)}`;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.#pending.delete(id);
        reject(new RpcError("timeout", method, `rpc call '${method}' timed out`));
      }, timeoutMs);

      this.#pending.set(id, { resolve: resolve as (v: unknown) => void, reject, timer, method });

      try {
        this.#ws!.send(JSON.stringify({ jsonrpc: "2.0", id, method, params: params ?? undefined }));
      } catch (e) {
        this.#pending.delete(id);
        clearTimeout(timer);
        reject(new RpcError("transport", method, `Failed to send: ${e}`));
      }
    }) as Promise<T>;
  }

  // ═══════════════════════════════════════════════════════════
  // Tier 2 — EventSource (SSE / HTTP stream)
  // ═══════════════════════════════════════════════════════════

  #downgradeToSse(): void {
    this.#tier = "sse";
    this.#sseRetryCount = 0;
    this.#openEventStream();
  }

  #openEventStream(): void {
    if (this.#tier !== "sse" || this.#disposed) return;
    if (this.#eventSource) this.#eventSource.close();

    const cleanPath = this.#rpcPath.split("?")[0];
    const url = this.#baseUrl + cleanPath + "/events?session=" + this.#sessionId;
    console.info("[RpcClient:sse] opening", url);

    try {
      const es = new EventSource(url);
      this.#eventSource = es;

      es.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.method && data.params !== undefined) {
            this.#notifHandlers.forEach((h) => h({ method: data.method, params: data.params }));
          }
          if (data.method === "Base.HeartbeatAck") {
            this.#heartbeatHandlers.forEach((h) => h());
          }
        } catch { /* ignore comment / keep-alive events */ }
      };

      es.onerror = () => {
        this.#eventSource?.close();
        this.#eventSource = null;
        this.#sseRetryCount++;

        if (this.#sseRetryCount > this.#sseMaxRetries) {
          console.warn("[RpcClient:sse] exhausted retries, downgrading to long poll");
          this.#downgradeToPoll();
          return;
        }

        console.warn("[RpcClient:sse] error, retry %d/%d in 3s", this.#sseRetryCount, this.#sseMaxRetries);
        this.#sseRetryTimer = setTimeout(() => this.#openEventStream(), 3000);
      };

      es.onopen = () => {
        this.#sseRetryCount = 0;
        console.info("[RpcClient:sse] connected");
        this.#setState("connected");
      };
    } catch {
      console.warn("[RpcClient:sse] not supported, downgrading to long poll");
      this.#downgradeToPoll();
    }
  }

  // ═══════════════════════════════════════════════════════════
  // Tier 3 — HTTP long polling
  // ═══════════════════════════════════════════════════════════

  #downgradeToPoll(): void {
    this.#tier = "poll";
    this.#eventSource?.close();
    this.#eventSource = null;
    this.#setState("disconnected");
    console.info("[RpcClient:poll] starting long-poll every %dms", this.#pollIntervalMs);

    this.#pollTimer = setInterval(() => {
      this.#pollEvents();
    }, this.#pollIntervalMs);
    this.#pollEvents(); // immediate first poll
  }

  async #pollEvents(): Promise<void> {
    if (this.#tier !== "poll" || this.#disposed) return;
    const cleanPath = this.#rpcPath.split("?")[0];
    const url = this.#baseUrl + cleanPath + "/events?session=" + this.#sessionId;
    try {
      const headers: Record<string, string> = {};
      const token = this.#getToken();
      if (token) headers["Authorization"] = `Bearer ${token}`;
      headers["X-Session-Id"] = this.#sessionId;

      const resp = await fetch(url, {
        headers,
        signal: AbortSignal.timeout(10_000),
        credentials: "include",
      });

      if (!resp.ok) return;

      // SSE over HTTP — read each event line
      const text = await resp.text();
      const events = text.split("\n\n");
      for (const block of events) {
        const dataLine = block.split("\n").find((l) => l.startsWith("data:"));
        if (!dataLine) continue;
        try {
          const data = JSON.parse(dataLine.slice(5).trim());
          if (data.method && data.params !== undefined) {
            this.#notifHandlers.forEach((h) => h({ method: data.method, params: data.params }));
          }
          if (data.method === "Base.HeartbeatAck") {
            this.#heartbeatHandlers.forEach((h) => h());
          }
        } catch { /* ignore */ }
      }

      this.#setState("connected");
    } catch {
      this.#setState("disconnected");
    }
  }

  // ═══════════════════════════════════════════════════════════
  // HTTP POST (used by all tiers)
  // ═══════════════════════════════════════════════════════════

  async #sendOverHttp<T>(method: string, params: unknown, timeoutMs: number): Promise<T> {
    const url = this.#baseUrl + this.#rpcPath;
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), timeoutMs);

    try {
      const headers: Record<string, string> = { "Content-Type": "application/json" };
      const token = this.#getToken();
      if (token) headers["Authorization"] = `Bearer ${token}`;
      headers["X-Session-Id"] = this.#sessionId;

      const resp = await fetch(url, {
        method: "POST",
        headers,
        body: JSON.stringify({ jsonrpc: "2.0", id: "http-" + (++this.#idCounter).toString(36), method, params }),
        signal: controller.signal,
        credentials: "include",
      });

      if (resp.status === 401) {
        this.#authLostHandlers.forEach((h) => h());
        this.#onAuthLost?.();
        throw new RpcError("forbidden", method, "unauthorized");
      }

      if (!resp.ok) {
        throw new RpcError("transport", method, `HTTP ${resp.status}`);
      }

      const body = await resp.json() as any;
      if (body.error) {
        const msg: string = body.error.message || "unknown rpc error";
        const kind: RpcErrorKind = body.error.code === -32003 ? "forbidden" : "rpc";
        throw new RpcError(kind, method, msg);
      }
      return body.result as T;
    } catch (e) {
      if (e instanceof RpcError) throw e;
      if (e instanceof DOMException && e.name === "AbortError") {
        throw new RpcError("timeout", method, `HTTP call '${method}' timed out`);
      }
      throw new RpcError("transport", method, String(e));
    } finally {
      clearTimeout(timer);
    }
  }

  // ═══════════════════════════════════════════════════════════
  // Heartbeat (WS only)
  // ═══════════════════════════════════════════════════════════

  #startHeartbeat(): void {
    this.#clearHeartbeat();
    this.#hbTimer = setInterval(() => {
      if (!this.#ws || this.#ws.readyState !== WebSocket.OPEN) return;
      if (this.#hbAckTimer) return;
      try {
        this.#ws.send(JSON.stringify({ jsonrpc: "2.0", method: "Base.Heartbeat" }));
        this.#hbAckTimer = setTimeout(() => {
          this.#hbAckTimer = null;
          if (this.#ws && this.#ws.readyState === WebSocket.OPEN) {
            this.#ws.close(4000, "heartbeat timeout");
          }
        }, this.#heartbeatTimeout);
      } catch { /* ignore */ }
    }, this.#heartbeatInterval);
  }

  #resetHeartbeatTimeout(): void {
    if (this.#hbAckTimer) { clearTimeout(this.#hbAckTimer); this.#hbAckTimer = null; }
  }

  #clearHeartbeat(): void {
    if (this.#hbTimer) { clearInterval(this.#hbTimer); this.#hbTimer = null; }
    this.#resetHeartbeatTimeout();
  }

  // ═══════════════════════════════════════════════════════════
  // Teardown
  // ═══════════════════════════════════════════════════════════

  #teardownAll(): void {
    this.#eventSource?.close();
    this.#eventSource = null;
    if (this.#sseRetryTimer) { clearTimeout(this.#sseRetryTimer); this.#sseRetryTimer = null; }
    if (this.#pollTimer) { clearInterval(this.#pollTimer); this.#pollTimer = null; }
    this.#clearHeartbeat();

    if (this.#ws) {
      this.#ws.onclose = null;
      this.#ws.onerror = null;
      this.#ws.onmessage = null;
      this.#ws.close();
      this.#ws = null;
    }
    this.#wsGen++;
    this.#rejectAllPending("disconnected");
    this.#wsConnectGate = null;
  }

  #setState(state: ConnectionState, retryIn?: number): void {
    this.#state = state;
    this.#stateHandlers.forEach((h) => h({ state, retryIn }));
  }

  #rejectAllPending(reason: string): void {
    for (const [, entry] of this.#pending) {
      clearTimeout(entry.timer);
      entry.reject(new RpcError("transport", entry.method, reason));
    }
    this.#pending.clear();
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}
