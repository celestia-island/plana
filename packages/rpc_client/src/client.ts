export type ConnectionState =
  | "connected"
  | "disconnected"
  | "connecting"
  | "reconnecting"
  | "failed";

export interface ConnectionStateEvent {
  state: ConnectionState;
  retryIn?: number;
  retryCount?: number;
  maxRetries?: number;
  transportTier?: string;
  attemptNumber?: number;
  countdown?: number;
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
  sseMaxRetries?: number;
  pollIntervalMs?: number;
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
const MAX_RETRIES = 3;
const POLL_INTERVAL = 30_000;
const ATTEMPT_TIMEOUTS = [1_000, 3_000, 5_000];

function isLocalhost(baseUrl: string): boolean {
  try {
    const host = new URL(baseUrl).hostname;
    return host === "localhost" || host === "127.0.0.1" || host === "[::1]";
  } catch { return false; }
}

export class RpcClient {
  readonly #baseUrl: string;
  readonly #rpcPath: string;
  readonly #getToken: () => string | null;
  readonly #onAuthLost?: () => void;
  readonly #heartbeatInterval: number;
  readonly #heartbeatTimeout: number;
  readonly #callTimeoutMs: number;
  readonly #pollIntervalMs: number;

  #ws: WebSocket | null = null;
  #wsGen = 0;
  #idCounter = 0;
  #pending = new Map<string, PendingCall>();
  #disposed = false;

  #sessionId: string;
  #eventSource: EventSource | null = null;

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
  #retryCount = 0;

  get state(): ConnectionState { return this.#state; }
  get connected(): boolean { return this.#ws?.readyState === WebSocket.OPEN; }
  get transportTier(): TransportTier { return this.#tier; }
  get retryCount(): number { return this.#retryCount; }

  constructor(opts: RpcClientOpts) {
    this.#baseUrl = opts.baseUrl.replace(/\/+$/, "");
    this.#rpcPath = opts.rpcPath ?? "/api/rpc";
    this.#getToken = opts.getToken;
    this.#onAuthLost = opts.onAuthLost;
    this.#heartbeatInterval = opts.heartbeatInterval ?? HB_INTERVAL;
    this.#heartbeatTimeout = opts.heartbeatTimeout ?? HB_TIMEOUT;
    this.#callTimeoutMs = opts.callTimeoutMs ?? CALL_TIMEOUT;
    this.#pollIntervalMs = opts.pollIntervalMs ?? POLL_INTERVAL;
    this.#sessionId = crypto.randomUUID();
    this.#local = opts.local ?? isLocalhost(this.#baseUrl);
  }

  // ── main API ────────────────────────────────────────────

  async call<T>(method: string, params?: unknown, timeoutMs?: number): Promise<T> {
    const timeout = timeoutMs ?? (this.#tier === "local" ? LOCAL_CALL_TIMEOUT : this.#callTimeoutMs);

    if (this.#tier !== "ws") {
      try {
        return await this.#sendOverHttp<T>(method, params, timeout);
      } catch (e) {
        if (this.#tier === "local" && !this.#disposed) {
          console.info("[RpcClient:local] request failed, downgrading to ws");
          this.#downgradeToWs();
        }
        throw e;
      }
    }

    if (this.connected) {
      return this.#sendOverWs<T>(method, params, timeout);
    }

    return this.#sendOverHttp<T>(method, params, timeout);
  }

  connect(): void {
    this.#disposed = false;
    this.#retryCount = 0;
    if (this.#local) {
      this.#tier = "local";
      console.info("[RpcClient:local] detected localhost, using direct HTTP");
      this.#setState("connected");
    } else {
      this.#progressiveConnect();
    }
  }

  async disconnect(): Promise<void> {
    this.#disposed = true;
    this.#teardownAll();
    this.#setState("disconnected");
  }

  forceReconnect(): void {
    if (this.#disposed) return;
    this.#retryCount = 0;
    this.#teardownAll();
    if (this.#local) {
      this.#tier = "local";
      this.#setState("connected");
    } else {
      this.#progressiveConnect();
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
  // Progressive connect
  // ═══════════════════════════════════════════════════════════

  async #progressiveConnect(): Promise<void> {
    const tiers: TransportTier[] = ["ws", "sse", "poll"];

    for (const tier of tiers) {
      if (this.#disposed) return;
      this.#tier = tier;

      for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
        if (this.#disposed) return;
        const timeoutMs = ATTEMPT_TIMEOUTS[attempt];
        const attemptNum = attempt + 1;
        this.#retryCount = attemptNum;

        this.#setState("connecting", undefined, tier, attemptNum, Math.ceil(timeoutMs / 1000));

        await sleep(0);

        let remaining = Math.ceil(timeoutMs / 1000);
        const countdownTimer = setInterval(() => {
          remaining--;
          if (remaining >= 0) {
            this.#stateHandlers.forEach((h) => h({
              state: "connecting",
              transportTier: tier,
              attemptNumber: attemptNum,
              countdown: remaining,
              retryCount: attemptNum,
              maxRetries: MAX_RETRIES,
            }));
          }
        }, 1000);

        const success = await this.#tryTransportOnce(tier, timeoutMs);
        clearInterval(countdownTimer);

        if (success) return;
      }
    }

    this.#setState("failed", undefined, "poll", undefined, undefined);
  }

  async #tryTransportOnce(tier: TransportTier, timeoutMs: number): Promise<boolean> {
    switch (tier) {
      case "ws": return this.#tryWsOnce(timeoutMs);
      case "sse": return this.#trySseOnce(timeoutMs);
      case "poll": return this.#tryPollOnce(timeoutMs);
      default: return false;
    }
  }

  // ═══════════════════════════════════════════════════════════
  // Tier 1 — WebSocket (single attempt, promise-based)
  // ═══════════════════════════════════════════════════════════

  async #tryWsOnce(timeoutMs: number): Promise<boolean> {
    return new Promise((resolve) => {
      const token = this.#getToken();
      if (!token) { resolve(false); return; }

      const wsUrl = this.#baseUrl.replace(/^http/, "ws") + this.#rpcPath;
      const gen = ++this.#wsGen;
      const ws = new WebSocket(wsUrl, ["jwt." + token]);
      ws.binaryType = "arraybuffer";
      this.#ws = ws;
      let settled = false;

      const timer = setTimeout(() => {
        if (settled) return;
        settled = true;
        this.#cleanupWs(ws, gen);
        resolve(false);
      }, timeoutMs);

      ws.onopen = () => {
        if (settled || this.#wsGen !== gen) return;
        settled = true;
        clearTimeout(timer);
        this.#setState("connected");
        this.#startHeartbeat();
        resolve(true);
      };

      ws.onerror = () => {
        if (settled || this.#wsGen !== gen) return;
        settled = true;
        clearTimeout(timer);
        this.#cleanupWs(ws, gen);
        resolve(false);
      };

      ws.onclose = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        this.#cleanupWs(ws, gen);
        if (this.#tier === "ws" && !this.#disposed) {
          this.#setState("disconnected");
        }
        resolve(false);
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
    });
  }

  #cleanupWs(ws: WebSocket, gen: number): void {
    ws.onopen = null;
    ws.onerror = null;
    ws.onclose = null;
    ws.onmessage = null;
    ws.close();
    if (this.#ws === ws) this.#ws = null;
    this.#clearHeartbeat();
    this.#rejectAllPending("connection lost");
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
  // Tier 2 — EventSource (SSE, single attempt)
  // ═══════════════════════════════════════════════════════════

  async #trySseOnce(timeoutMs: number): Promise<boolean> {
    if (typeof EventSource === "undefined") return false;

    return new Promise((resolve) => {
      const cleanPath = this.#rpcPath.split("?")[0];
      const url = this.#baseUrl + cleanPath + "/events?session=" + this.#sessionId;
      let settled = false;

      const timer = setTimeout(() => {
        if (settled) return;
        settled = true;
        es.close();
        this.#eventSource = null;
        resolve(false);
      }, timeoutMs);

      let es: EventSource;
      try {
        es = new EventSource(url);
        this.#eventSource = es;
      } catch {
        resolve(false);
        return;
      }

      es.onopen = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        this.#setState("connected");
        resolve(true);
      };

      es.onerror = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        es.close();
        this.#eventSource = null;
        resolve(false);
      };

      es.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.method && data.params !== undefined) {
            this.#notifHandlers.forEach((h) => h({ method: data.method, params: data.params }));
          }
          if (data.method === "Base.HeartbeatAck") {
            this.#heartbeatHandlers.forEach((h) => h());
          }
        } catch { /* ignore */ }
      };
    });
  }

  // ═══════════════════════════════════════════════════════════
  // Tier 3 — HTTP long polling (single fetch)
  // ═══════════════════════════════════════════════════════════

  async #tryPollOnce(timeoutMs: number): Promise<boolean> {
    try {
      const cleanPath = this.#rpcPath.split("?")[0];
      const url = this.#baseUrl + cleanPath + "/events?session=" + this.#sessionId;
      const headers: Record<string, string> = {};
      const token = this.#getToken();
      if (token) headers["Authorization"] = `Bearer ${token}`;
      headers["X-Session-Id"] = this.#sessionId;

      const resp = await fetch(url, {
        headers,
        signal: AbortSignal.timeout(timeoutMs),
        credentials: "include",
      });

      if (!resp.ok) return false;

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
      return true;
    } catch {
      this.#setState("disconnected");
      return false;
    }
  }

  // ═══════════════════════════════════════════════════════════
  // Tier 0 → 1 downgrade
  // ═══════════════════════════════════════════════════════════

  #downgradeToWs(): void {
    this.#tier = "ws";
    this.#teardownAll();
    this.#progressiveConnect();
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
    if (this.#pollTimer) { clearInterval(this.#pollTimer); this.#pollTimer = null; }
    this.#clearHeartbeat();

    if (this.#ws) {
      this.#ws.onclose = null;
      this.#ws.onerror = null;
      this.#ws.onmessage = null;
      this.#ws.onopen = null;
      this.#ws.close();
      this.#ws = null;
    }
    this.#wsGen++;
    this.#rejectAllPending("disconnected");
  }

  #setState(state: ConnectionState, retryIn?: number, transportTier?: string, attemptNumber?: number, countdown?: number): void {
    this.#state = state;
    this.#stateHandlers.forEach((h) => h({
      state,
      retryIn,
      retryCount: MAX_RETRIES - this.#retryCount >= 0 ? this.#retryCount : undefined,
      maxRetries: MAX_RETRIES,
      transportTier,
      attemptNumber,
      countdown,
    }));
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
