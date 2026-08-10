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
  /** Round-trip time of the latest heartbeat, in milliseconds (ws tier only). */
  latencyMs?: number;
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
  /**
   * Called when a request is rejected with 401: return a fresh access token
   * to retry the request once (the callback owns persisting it), or null to
   * give up and trigger authLost.
   */
  refreshToken?: () => Promise<string | null>;
  heartbeatInterval?: number;
  heartbeatTimeout?: number;
  callTimeoutMs?: number;
  sseMaxRetries?: number;
  pollIntervalMs?: number;
  local?: boolean;
  /**
   * Probe-only mode: without a token the client performs a single anonymous
   * `/api/health` handshake, reports `connected` once it succeeds, then
   * immediately tears the connection down. Used by status bars to show
   * backend health for anonymous visitors without holding a long-lived
   * connection (abuse prevention).
   */
  probeOnly?: boolean;
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
  readonly #refreshToken?: () => Promise<string | null>;
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
  #hbSentAt: number | null = null;
  #latencyMs: number | null = null;

  #notifHandlers = new Set<NotificationHandler>();
  #binaryHandlers = new Set<BinaryHandler>();
  #stateHandlers = new Set<StateHandler>();
  #heartbeatHandlers = new Set<HeartbeatHandler>();
  #authLostHandlers = new Set<AuthLostHandler>();

  #state: ConnectionState = "disconnected";
  #retryCount = 0;
  #probeOnly = false;

  get state(): ConnectionState { return this.#state; }
  get connected(): boolean { return this.#ws?.readyState === WebSocket.OPEN; }
  get transportTier(): TransportTier { return this.#tier; }
  get retryCount(): number { return this.#retryCount; }
  /** Last measured heartbeat round-trip time (ws tier only); null when unknown. */
  get latencyMs(): number | null { return this.#latencyMs; }

  constructor(opts: RpcClientOpts) {
    this.#baseUrl = opts.baseUrl.replace(/\/+$/, "");
    this.#rpcPath = opts.rpcPath ?? "/api/rpc";
    this.#getToken = opts.getToken;
    this.#onAuthLost = opts.onAuthLost;
    this.#refreshToken = opts.refreshToken;
    this.#heartbeatInterval = opts.heartbeatInterval ?? HB_INTERVAL;
    this.#heartbeatTimeout = opts.heartbeatTimeout ?? HB_TIMEOUT;
    this.#callTimeoutMs = opts.callTimeoutMs ?? CALL_TIMEOUT;
    this.#pollIntervalMs = opts.pollIntervalMs ?? POLL_INTERVAL;
    this.#sessionId = randomSessionId();
    this.#local = opts.local ?? isLocalhost(this.#baseUrl);
    this.#probeOnly = opts.probeOnly ?? false;
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
    if (this.#probeOnly) {
      this.#probeHealthOnce();
    } else if (this.#local) {
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
  // Progressive connect: 3 rounds, each tries ws+sse+poll in parallel.
  // Priority: ws > sse > poll. Timeouts: 1s, 3s, 5s.
  // ═══════════════════════════════════════════════════════════

  async #progressiveConnect(): Promise<void> {
    this.#tier = "poll";
    for (let round = 0; round < MAX_RETRIES; round++) {
      if (this.#disposed) return;
      const timeoutMs = ATTEMPT_TIMEOUTS[round];
      const roundNum = round + 1;
      this.#retryCount = roundNum;

      this.#setState("connecting", undefined, undefined, roundNum, Math.ceil(timeoutMs / 1000));

      let remaining = Math.ceil(timeoutMs / 1000);
      const countdownTimer = setInterval(() => {
        remaining--;
        if (remaining >= 0) {
          this.#stateHandlers.forEach((h) =>
            h({
              state: "connecting",
              attemptNumber: roundNum,
              countdown: remaining,
              retryCount: roundNum,
              maxRetries: MAX_RETRIES,
            })
          );
        }
      }, 1000);

      const tier = await this.#raceTransports(timeoutMs);
      clearInterval(countdownTimer);

      if (tier) {
        this.#tier = tier;
        if (tier === "ws") this.#startHeartbeat();
        else { this.#eventSource?.close(); this.#eventSource = null; if (this.#ws) { this.#cleanupWs(this.#ws, this.#wsGen); } }
        this.#setState("connected");
        return;
      }
    }

    this.#tier = "poll";
    this.#setState("failed", undefined, "poll", undefined, undefined);
  }

  /** Try ws, sse, poll in parallel. Return highest-priority tier that succeeded.
   *  Always waits the full timeout so the countdown is visible. */
  async #raceTransports(timeoutMs: number): Promise<TransportTier | null> {
    const results = await Promise.allSettled([
      this.#tryWsOnce(timeoutMs).then((ok) => ({ tier: "ws" as TransportTier, ok })),
      this.#trySseOnce(timeoutMs).then((ok) => ({ tier: "sse" as TransportTier, ok })),
      this.#tryPollOnce(timeoutMs).then((ok) => ({ tier: "poll" as TransportTier, ok })),
      // Sentinel keeps the settled-result type homogeneous ({ tier, ok }) so
      // `r.value.ok` below is well-typed; its value is never read.
      sleep(timeoutMs).then(() => ({ tier: null, ok: false })),
    ]);

    const priority: TransportTier[] = ["ws", "sse", "poll"];
    for (const tier of priority) {
      const r = results[priority.indexOf(tier)];
      if (r.status === "fulfilled" && r.value.ok) {
        return tier;
      }
    }
    return null;
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

      const wsUrl =
        this.#baseUrl.replace(/^http/, "ws") +
        this.#rpcPath +
        "?token=" +
        encodeURIComponent(token);
      const gen = ++this.#wsGen;
      const ws = new WebSocket(wsUrl);
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
          if (this.#hbSentAt !== null) {
            this.#latencyMs = Math.max(0, Math.round(performance.now() - this.#hbSentAt));
            this.#hbSentAt = null;
            this.#emitLatency();
          }
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
  // Tier 3 — HTTP POST probe (uses health endpoint)
  // ═══════════════════════════════════════════════════════════

  async #tryPollOnce(timeoutMs: number): Promise<boolean> {
    try {
      const url = this.#baseUrl + "/api/health";
      const headers: Record<string, string> = {};
      const token = this.#getToken();
      if (token) headers["Authorization"] = `Bearer ${token}`;

      const resp = await fetch(url, {
        method: "GET",
        headers,
        signal: AbortSignal.timeout(timeoutMs),
        credentials: "include",
      });

      if (!resp.ok) return false;

      return true;
    } catch {
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
  // Anonymous health probe (probeOnly)
  // ═══════════════════════════════════════════════════════════

  async #probeHealthOnce(): Promise<void> {
    // Preferred: one long-connection attempt over the real transport path.
    // The server replies with a success ack and actively closes the socket,
    // so the probe validates the WS upgrade + roundtrip without holding an
    // anonymous connection open (no connection storm on login pages).
    if (await this.#probeWsOnce(5000)) {
      this.#tier = "ws";
      this.#setState("connected");
      this.#disposed = true;
      this.#teardownAll();
      this.#setState("disconnected");
      return;
    }
    // Fallback: plain HTTP GET on the health endpoint.
    try {
      const url = this.#baseUrl + "/api/health";
      const resp = await fetch(url, {
        method: "GET",
        credentials: "include",
        signal: AbortSignal.timeout(5000),
      });
      if (!resp.ok) {
        this.#setState("failed");
        return;
      }
      this.#tier = "poll";
      this.#setState("connected");
      // Handshake succeeded — tear down immediately. The status bar only
      // needs to know the backend is reachable; keeping a connection open
      // for anonymous visitors would invite abuse.
      this.#disposed = true;
      this.#teardownAll();
      this.#setState("disconnected");
    } catch {
      this.#setState("failed");
    }
  }

  // ═══════════════════════════════════════════════════════════
  // Anonymous WS probe (single long-connection attempt)
  // ═══════════════════════════════════════════════════════════

  async #probeWsOnce(timeoutMs: number): Promise<boolean> {
    return new Promise((resolve) => {
      let ws: WebSocket | null = null;
      try {
        ws = new WebSocket(this.#baseUrl.replace(/^http/, "ws") + this.#rpcPath);
      } catch {
        resolve(false);
        return;
      }
      let settled = false;
      const finish = (ok: boolean): void => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        try { ws?.close(); } catch { /* ignore */ }
        resolve(ok);
      };
      const timer = setTimeout(() => finish(false), timeoutMs);

      ws.onopen = () => {
        try {
          ws?.send(
            JSON.stringify({
              jsonrpc: "2.0",
              id: "probe",
              method: "system.probe",
              params: {},
            }),
          );
        } catch {
          finish(false);
        }
      };
      ws.onmessage = () => finish(true);
      ws.onerror = () => finish(false);
      ws.onclose = () => {
        clearTimeout(timer);
        if (!settled) resolve(false);
      };
    });
  }

  // ═══════════════════════════════════════════════════════════
  // HTTP POST (used by all tiers)
  // ═══════════════════════════════════════════════════════════

  async #sendOverHttp<T>(method: string, params: unknown, timeoutMs: number, retried = false): Promise<T> {
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
        if (this.#refreshToken && !retried) {
          const fresh = await this.#refreshToken();
          if (fresh) {
            return this.#sendOverHttp(method, params, timeoutMs, true);
          }
        }
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
        this.#hbSentAt = performance.now();
        this.#ws.send(JSON.stringify({ jsonrpc: "2.0", method: "Base.Heartbeat" }));
        this.#hbAckTimer = setTimeout(() => {
          this.#hbAckTimer = null;
          this.#hbSentAt = null;
          if (this.#ws && this.#ws.readyState === WebSocket.OPEN) {
            this.#ws.close(4000, "heartbeat timeout");
          }
        }, this.#heartbeatTimeout);
      } catch { /* ignore */ }
    }, this.#heartbeatInterval);
  }

  /** Notify state handlers about a fresh latency measurement (partial event). */
  #emitLatency(): void {
    const latencyMs = this.#latencyMs;
    if (latencyMs === null) return;
    this.#stateHandlers.forEach((h) => h({
      state: this.#state,
      latencyMs,
      transportTier: this.#tier,
    }));
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
    this.#latencyMs = null;
    this.#hbSentAt = null;

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

/**
 * Session id for the RPC client. `crypto.randomUUID()` only exists in secure
 * contexts (https or localhost); plain-http deployments would crash at
 * startup, so fall back to a local random id there.
 */
function randomSessionId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `sess-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`;
}
