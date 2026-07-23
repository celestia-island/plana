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
  reconnectMax?: number;
  callTimeoutMs?: number;
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

const HB_INTERVAL = 15_000;
const HB_TIMEOUT = 10_000;
const RECONNECT_MAX = 10;
const CALL_TIMEOUT = 30_000;

export class RpcClient {
  readonly #baseUrl: string;
  readonly #rpcPath: string;
  readonly #getToken: () => string | null;
  readonly #onAuthLost?: () => void;
  readonly #heartbeatInterval: number;
  readonly #heartbeatTimeout: number;
  readonly #reconnectMax: number;
  readonly #callTimeoutMs: number;

  #ws: WebSocket | null = null;
  #wsGen = 0;
  #idCounter = 0;
  #pending = new Map<string, PendingCall>();
  #disposed = false;

  // HTTP long-polling fallback
  #sessionId: string;
  #eventSource: EventSource | null = null;
  #useHttpOnly = false;

  #hbTimer: ReturnType<typeof setInterval> | null = null;
  #hbAckTimer: ReturnType<typeof setTimeout> | null = null;
  #reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  #reconnectCountdown: ReturnType<typeof setInterval> | null = null;
  #reconnectDelay = 1000;
  #consecutiveFailures = 0;

  #notifHandlers = new Set<NotificationHandler>();
  #binaryHandlers = new Set<BinaryHandler>();
  #stateHandlers = new Set<StateHandler>();
  #heartbeatHandlers = new Set<HeartbeatHandler>();
  #authLostHandlers = new Set<AuthLostHandler>();

  #state: ConnectionState = "disconnected";
  #wsConnectGate: Promise<void> | null = null;

  get state(): ConnectionState { return this.#state; }
  get connected(): boolean { return this.#ws?.readyState === WebSocket.OPEN; }

  constructor(opts: RpcClientOpts) {
    this.#baseUrl = opts.baseUrl.replace(/\/+$/, "");
    this.#rpcPath = opts.rpcPath ?? "/api/rpc";
    this.#getToken = opts.getToken;
    this.#onAuthLost = opts.onAuthLost;
    this.#heartbeatInterval = opts.heartbeatInterval ?? HB_INTERVAL;
    this.#heartbeatTimeout = opts.heartbeatTimeout ?? HB_TIMEOUT;
    this.#reconnectMax = opts.reconnectMax ?? RECONNECT_MAX;
    this.#callTimeoutMs = opts.callTimeoutMs ?? CALL_TIMEOUT;
    this.#sessionId = crypto.randomUUID();
  }

  // ── main API ────────────────────────────────────────────

  async call<T>(method: string, params?: unknown, timeoutMs?: number): Promise<T> {
    const timeout = timeoutMs ?? this.#callTimeoutMs;

    if (this.connected) {
      return this.#sendOverWs<T>(method, params, timeout);
    }

    // If we're trying to connect, wait briefly
    if (this.#wsConnectGate) {
      try {
        await Promise.race([this.#wsConnectGate, sleep(3000)]);
      } catch { /* gate timeout — fall through to HTTP */ }
      if (this.connected) return this.#sendOverWs<T>(method, params, timeout);
    }

    // HTTP fallback
    try {
      return await this.#sendOverHttp<T>(method, params, timeout);
    } catch (e) {
      // If it's a transport error (no connection), try to connect WS then retry
      if (e instanceof RpcError && e.kind === "transport" && !this.#disposed) {
        this.#connectWs();
        await sleep(200);
        if (this.connected) return this.#sendOverWs<T>(method, params, timeout);
      }
      throw e;
    }
  }

  connect(): void {
    this.#disposed = false;
    this.#connectWs();
  }

  async disconnect(): Promise<void> {
    this.#disposed = true;
    this.#teardownWs();
    this.#setState("disconnected");
  }

  forceReconnect(): void {
    if (this.#disposed) return;
    if (this.#reconnectTimer) { clearTimeout(this.#reconnectTimer); this.#reconnectTimer = null; }
    if (this.#reconnectCountdown) { clearInterval(this.#reconnectCountdown); this.#reconnectCountdown = null; }
    this.#reconnectDelay = 1000;
    this.#consecutiveFailures = 0;
    this.#teardownWs();
    this.#connectWs();
  }

  // ── events ──────────────────────────────────────────────

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

  // ── private: JSON-RPC over WS ──────────────────────────

  #connectWs(): void {
    if (!this.#getToken() || this.#disposed) return;
    if (this.#ws) {
      const rs = this.#ws.readyState;
      if (rs === WebSocket.CONNECTING || rs === WebSocket.OPEN) return;
    }
    this.#setState("connecting");
    const gen = this.#wsGen = this.#wsGen + 1;

    let wsUrl = this.#baseUrl.replace(/^http/, "ws") + this.#rpcPath;
    const token = this.#getToken();
    if (!token) return;

    const ws = new WebSocket(wsUrl, ["jwt." + token]);
    ws.binaryType = "arraybuffer";
    this.#ws = ws;

    ws.onopen = () => {
      if (this.#wsGen !== gen) { ws.close(1000, "stale"); return; }
      this.#consecutiveFailures = 0;
      this.#reconnectDelay = 1000;
      this.#setState("connected");
      this.#startHeartbeat();
    };

    ws.onclose = () => {
      if (this.#wsGen !== gen) return;
      this.#setState("disconnected");
      this.#clearHeartbeat();
      this.#rejectAllPending("connection lost");
      if (!this.#disposed) this.#scheduleReconnect();
    };

    ws.onerror = () => {
      if (this.#wsGen !== gen) return;
      this.#setState("disconnected");
      // WS failed — tear down immediately, open HTTP event stream
      this.#teardownWs();
      this.#disposed = false;
      this.#useHttpOnly = true;
      this.#openEventStream();
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

  // ── private: JSON-RPC over HTTP ─────────────────────────

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

  // ── private: heartbeat ─────────────────────────────────

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

  // ── private: reconnect ─────────────────────────────────

  #scheduleReconnect(): void {
    if (this.#reconnectTimer) clearTimeout(this.#reconnectTimer);
    if (this.#consecutiveFailures >= this.#reconnectMax) {
      this.#setState("failed");
      this.#authLostHandlers.forEach((h) => h());
      this.#onAuthLost?.();
      return;
    }
    this.#consecutiveFailures++;
    const jitter = Math.random() * this.#reconnectDelay * 0.5;
    const delay = this.#reconnectDelay + jitter;
    this.#reconnectDelay = Math.min(this.#reconnectDelay * 1.5, 30_000);
    this.#runCountdown(Math.ceil(delay / 1000));
    this.#reconnectTimer = setTimeout(() => {
      if (!this.#disposed) this.#connectWs();
    }, delay);
  }

  #runCountdown(secs: number): void {
    if (this.#reconnectCountdown) clearInterval(this.#reconnectCountdown);
    let remaining = secs;
    this.#setState("reconnecting", remaining);
    this.#reconnectCountdown = setInterval(() => {
      remaining -= 1;
      if (remaining <= 0) { clearInterval(this.#reconnectCountdown!); this.#reconnectCountdown = null; return; }
      this.#setState("reconnecting", remaining);
    }, 1000);
  }

  #setState(state: ConnectionState, retryIn?: number): void {
    this.#state = state;
    this.#stateHandlers.forEach((h) => h({ state, retryIn }));
  }

  // ── private: teardown ──────────────────────────────────

  #openEventStream(): void {
    if (this.#eventSource) {
      this.#eventSource.close();
    }
    const url = this.#baseUrl + this.#rpcPath + "/events?session=" + this.#sessionId;
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
        } catch { /* ignore parse errors */ }
      };
      es.onerror = () => {
        this.#eventSource?.close();
        this.#eventSource = null;
        if (!this.#disposed) {
          setTimeout(() => this.#openEventStream(), 3000);
        }
      };
      es.onopen = () => {
        this.#setState("connected");
      };
    } catch {
      // EventSource not supported, fall back to HTTP-only
    }
  }

  #teardownWs(): void {
    this.#eventSource?.close();
    this.#eventSource = null;
    if (this.#reconnectTimer) { clearTimeout(this.#reconnectTimer); this.#reconnectTimer = null; }
    if (this.#reconnectCountdown) { clearInterval(this.#reconnectCountdown); this.#reconnectCountdown = null; }
    this.#clearHeartbeat();
    if (this.#ws) {
      this.#ws.onclose = null;
      this.#ws.onerror = null;
      this.#ws.onmessage = null;
      this.#ws.close();
      this.#ws = null;
    }
    this.#wsGen++;
    this.#consecutiveFailures = 0;
    this.#reconnectDelay = 1000;
    this.#rejectAllPending("disconnected");
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
