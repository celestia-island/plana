import { ref, onMounted, onUnmounted, type Ref } from "vue";
import { RpcClient } from "@celestia-island/plana-rpc-client";
import type { ConnectionStateEvent } from "@celestia-island/plana-rpc-client";

export interface ProbeResult {
  connected: boolean;
  state: "connected" | "disconnected" | "connecting" | "reconnecting" | "failed";
  tier: string;
  latencyMs: number | null;
  retryCount: number;
  retryTotal: number;
  transportTier: string;
  attemptNumber: number;
  countdown: number;
}

let sharedClient: RpcClient | null = null;

export function setProbeClient(client: RpcClient): void {
  sharedClient = client;
}

export function useConnectionProbe(): {
  result: Ref<ProbeResult>;
  retryNow: () => void;
} {
  const result = ref<ProbeResult>({
    connected: false,
    state: "disconnected",
    tier: "ws",
    latencyMs: null,
    retryCount: 0,
    retryTotal: 3,
    transportTier: "ws",
    attemptNumber: 0,
    countdown: 0,
  });

  let unsub: (() => void) | null = null;

  function updateState(e: ConnectionStateEvent): void {
    result.value = {
      ...result.value,
      connected: e.state === "connected",
      state: e.state as ProbeResult["state"],
      retryCount: e.retryCount ?? 0,
      retryTotal: e.maxRetries ?? 3,
      transportTier: e.transportTier ?? result.value.transportTier,
      attemptNumber: e.attemptNumber ?? 0,
      countdown: e.countdown ?? 0,
    };
  }

  function retryNow(): void {
    sharedClient?.forceReconnect();
  }

  onMounted(() => {
    if (sharedClient) {
      updateState({
        state: sharedClient.state,
        retryCount: sharedClient.retryCount,
        transportTier: sharedClient.transportTier,
      });
      unsub = sharedClient.on("state", updateState);
    }
  });

  onUnmounted(() => {
    unsub?.();
  });

  return { result, retryNow };
}
