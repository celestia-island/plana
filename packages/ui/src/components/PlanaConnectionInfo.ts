import { computed, type Ref } from "vue";

function detectRegion(): string {
  const lang = navigator.language;
  const regionMap: Record<string, string> = {
    zh: "CN", "zh-CN": "CN", "zh-Hans": "CN",
    "zh-TW": "TW", "zh-HK": "HK",
    ja: "JP", ko: "KR", de: "DE", fr: "FR", es: "ES", pt: "PT",
    ar: "SA", ru: "RU", en: "US",
  };
  return regionMap[lang] ?? lang.split("-")[1]?.toUpperCase() ?? lang.toUpperCase();
}

function isLocalhostUrl(): boolean {
  try {
    const host = window.location.hostname;
    return host === "localhost" || host === "127.0.0.1" || host === "[::1]";
  } catch { return false; }
}

export type ConnectionStateInput =
  | "connected"
  | "disconnected"
  | "connecting"
  | "reconnecting"
  | "failed";

export interface PlanaConnectionInfo {
  state: "connected" | "reconnecting" | "disconnected";
  tier: string;
  quality: string;
  latencyMs: number | null;
  isLocalhost: boolean;
  region: string;
}

export function useConnectionInfo(
  connectionState: Ref<ConnectionStateInput>,
  transportTier?: Ref<string>,
): { connectionInfo: Ref<PlanaConnectionInfo> } {
  const info = computed<PlanaConnectionInfo>(() => {
    const s = connectionState.value;
    let state: PlanaConnectionInfo["state"] = "disconnected";
    if (s === "connected") state = "connected";
    else if (s === "connecting" || s === "reconnecting") state = "reconnecting";

    const tierValue = transportTier?.value ?? (isLocalhostUrl() ? "local" : "ws");
    let quality = "unknown";
    if (s === "connected") {
      quality = tierValue === "local" ? "excellent" : tierValue === "ws" ? "good" : "fair";
    }

    return {
      state,
      tier: tierValue,
      quality,
      latencyMs: null,
      isLocalhost: isLocalhostUrl(),
      region: detectRegion(),
    };
  });

  return { connectionInfo: info };
}
