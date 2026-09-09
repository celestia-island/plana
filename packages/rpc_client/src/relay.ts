// Relay extension — the edge (webview/UI) side.
//
// Mirrors `plana-rpc-client`'s Rust `relay` module and `plana-tauri`'s
// bridge: one request/response transport function plus one notification
// lane, canonical JSON-RPC 2.0 frames, the reserved `relay.*`
// system-control method names, and the hop context for chains. A webview
// picks a transport (Tauri v2 global bridge or in-memory for tests) and
// speaks the same surface in every app.

/** Reserved system-control namespace (never forwarded). */
export const RELAY = "relay";

/** Pre-standardized `relay.*` method names (see docs/en/rpc/relay-profile.md). */
export const relayMethods = {
  netSetProxy: "relay.net.set_proxy",
  netConfigureEntrypoint: "relay.net.configure_entrypoint",
  connOpen: "relay.conn.open",
  fwdMap: "relay.fwd.map",
  fwdList: "relay.fwd.list",
  windowMinimize: "relay.window.minimize",
  windowMaximizeToggle: "relay.window.maximize_toggle",
  windowClose: "relay.window.close",
  windowState: "relay.window.state",
} as const;

/** Per-frame relay context: hop count for the loop guard + trace. */
export interface RelayContext {
  hops: number;
  via?: string[];
}

/** One edge frame: canonical JSON-RPC request members + relay extension. */
export interface EdgeFrame {
  method: string;
  params?: unknown;
  relay?: RelayContext;
}

/** The transport contract any framework adapter implements. */
export interface EdgeTransport {
  call(frame: EdgeFrame): Promise<unknown>;
  /** Subscribe to the notification lane; returns an unsubscribe. */
  listen(handler: (method: string, params?: unknown) => void): () => void;
}

/**
 * Tauri v2 transport: `bridge_call` invoke + the `bridge_event` lane.
 * Reads the strict v2 global shape (`__TAURI__.core.invoke`).
 */
export function createTauriTransport(): EdgeTransport {
  const raw = (window as unknown as {
    __TAURI__?: {
      core?: { invoke?: (cmd: string, args?: Record<string, unknown>) => Promise<unknown> };
      event?: {
        listen?: (
          event: string,
          handler: (e: { payload: unknown }) => void,
        ) => Promise<() => void>;
      };
    };
  }).__TAURI__;
  const invoke = raw?.core?.invoke;
  const listen = raw?.event?.listen;
  if (!invoke || !listen) {
    throw new Error("tauri v2 global API unavailable (no withGlobalTauri shell)");
  }
  const subscribers = new Set<(method: string, params?: unknown) => void>();
  let wired = false;
  return {
    async call(frame) {
      return invoke("bridge_call", {
        method: frame.method,
        params: frame.params ?? null,
        relay: frame.relay ?? null,
      });
    },
    listen(handler) {
      if (!wired) {
        wired = true;
        void listen("bridge_event", (e) => {
          const payload = e.payload as { method?: string; params?: unknown };
          if (payload?.method) {
            for (const fn of subscribers) fn(payload.method, payload.params);
          }
        });
      }
      subscribers.add(handler);
      return () => subscribers.delete(handler);
    },
  };
}

/** In-memory transport for tests and non-framework embedding. */
export function createMemoryTransport(
  far: (frame: EdgeFrame) => Promise<unknown>,
): EdgeTransport {
  return {
    call: (frame) => far(frame),
    listen: () => () => {},
  };
}

/** The edge client every UI surface shares. */
export class RelayClient {
  constructor(private transport: EdgeTransport) {}

  /** One request/response call; the transport owns correlation. */
  call(method: string, params?: unknown, relay?: RelayContext): Promise<unknown> {
    return this.transport.call({ method, params, relay });
  }

  /** Typed helpers over the standard system-control battery. */
  setProxy(config: {
    scheme: "http" | "https" | "socks5";
    host: string;
    username?: string;
    password?: string;
  }): Promise<unknown> {
    return this.call(relayMethods.netSetProxy, config);
  }

  configureEntrypoint(url: string): Promise<unknown> {
    return this.call(relayMethods.netConfigureEntrypoint, { url });
  }

  fwdMap(namespacePrefix: string, endpoint: string): Promise<unknown> {
    return this.call(relayMethods.fwdMap, { namespacePrefix, endpoint });
  }

  fwdList(): Promise<unknown> {
    return this.call(relayMethods.fwdList);
  }

  windowMinimize(): Promise<unknown> {
    return this.call(relayMethods.windowMinimize);
  }

  windowMaximizeToggle(): Promise<unknown> {
    return this.call(relayMethods.windowMaximizeToggle);
  }

  windowClose(): Promise<unknown> {
    return this.call(relayMethods.windowClose);
  }

  windowState(): Promise<unknown> {
    return this.call(relayMethods.windowState);
  }

  onNotification(handler: (method: string, params?: unknown) => void): () => void {
    return this.transport.listen(handler);
  }
}
