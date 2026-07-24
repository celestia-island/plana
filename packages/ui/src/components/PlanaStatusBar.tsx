import { computed, defineComponent, ref, type PropType, onBeforeUnmount } from "vue";
import { Wifi, WifiOff, Globe, Cpu } from "lucide-vue-next";

export interface PlanaConnectionInfo {
  state: "connected" | "reconnecting" | "disconnected";
  tier: string;
  quality: string;
  latencyMs: number | null;
  isLocalhost: boolean;
  region: string;
}

const tierLabel: Record<string, string> = {
  local: "HTTP local",
  ws: "WebSocket",
  sse: "SSE events",
  poll: "HTTP poll",
};

const tierOrder: Record<string, number> = { local: 0, ws: 1, sse: 2, poll: 3 };

const regionFlag: Record<string, string> = {
  CN: "\ud83c\udde8\ud83c\uddf3", JP: "\ud83c\uddef\ud83c\uddf5", KR: "\ud83c\uddf0\ud83c\uddf7",
  US: "\ud83c\uddfa\ud83c\uddf8", GB: "\ud83c\uddec\ud83c\udde7", DE: "\ud83c\udde9\ud83c\uddea",
  FR: "\ud83c\uddeb\ud83c\uddf7", SA: "\ud83c\uddf8\ud83c\udde6", TW: "\ud83c\uddf9\ud83c\uddfc",
  HK: "\ud83c\udded\ud83c\uddf0", BR: "\ud83c\udde7\ud83c\uddf7", RU: "\ud83c\uddf7\ud83c\uddfa",
  CA: "\ud83c\udde8\ud83c\udde6", AU: "\ud83c\udde6\ud83c\uddfa", PT: "\ud83c\uddf5\ud83c\uddf9",
  ES: "\ud83c\uddea\ud83c\uddf8",
};

const regionLabel: Record<string, string> = {
  CN: "\u4e2d\u56fd\u5927\u9646", JP: "\u65e5\u672c", KR: "\u97e9\u56fd",
  US: "\u7f8e\u56fd", GB: "\u82f1\u56fd", DE: "\u5fb7\u56fd",
  FR: "\u6cd5\u56fd", SA: "\u6c99\u7279", TW: "\u53f0\u6e7e",
  HK: "\u9999\u6e2f", BR: "\u5df4\u897f", RU: "\u4fc4\u7f57\u65af",
  CA: "\u52a0\u62ff\u5927", AU: "\u6fb3\u5927\u5229\u4e9a",
  PT: "\u8461\u8404\u7259", ES: "\u897f\u73ed\u7259",
};

function latencyColor(ms: number | null): string {
  if (ms === null) return "var(--color-muted)";
  if (ms < 30) return "rgb(var(--color-success))";
  if (ms < 100) return "rgb(var(--color-warning))";
  return "rgb(var(--color-error))";
}

function qualityIcon(quality: string, size: number) {
  if (quality === "excellent" || quality === "good") return <Wifi size={size} />;
  return <WifiOff size={size} />;
}

export const PlanaStatusBar = defineComponent({
  name: "PlanaStatusBar",
  props: {
    version: {
      type: String,
      default: "0.1.0",
    },
    connectionStatus: {
      type: String as PropType<"connected" | "reconnecting" | "disconnected">,
      default: "disconnected",
    },
    connectionInfo: {
      type: Object as PropType<PlanaConnectionInfo | null>,
      default: null,
    },
  },
  setup(props) {
    const popupOpen = ref(false);
    let closeTimer: ReturnType<typeof setTimeout> | null = null;
    onBeforeUnmount(() => { if (closeTimer) clearTimeout(closeTimer); });

    const dotColorMap: Record<string, string> = {
      connected: "rgb(var(--color-success))",
      reconnecting: "rgb(var(--color-warning))",
      disconnected: "rgb(var(--color-error))",
    };

    const showEnter = () => {
      if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
      popupOpen.value = true;
    };
    const showLeave = () => {
      closeTimer = setTimeout(() => { popupOpen.value = false; }, 200);
    };

    return () => {
      const info = props.connectionInfo;
      return (
        <footer
          class="s-status-bar"
          style={{
            position: "fixed", bottom: 0, left: 0, right: 0,
            height: "var(--s-footer-height, 2.5rem)",
            display: "flex", alignItems: "center",
            padding: "0 var(--space-16, 1rem)",
            background: "rgb(var(--color-surface))",
            backdropFilter: "blur(var(--blur-md, 12px))",
            borderTop: "1px solid var(--border-faint, rgb(var(--color-border) / 10%))",
            zIndex: "var(--z-sidebar, 30)", flexShrink: 0,
          }}
        >
          <div style={{ position: "relative" }}>
            <span
              class="s-status-bar-tag"
              role="button"
              tabindex={0}
              onMouseenter={showEnter}
              onMouseleave={showLeave}
              style={{
                display: "inline-flex", alignItems: "center",
                height: "24px", gap: "5px", padding: "0 8px",
                borderRadius: "var(--radius-md, 6px)",
                fontSize: "var(--text-2xs, 0.625rem)", lineHeight: 1,
                background: "rgb(var(--color-surface) / var(--opacity-half, 0.5))",
                color: "rgb(var(--color-muted))", userSelect: "none", cursor: "pointer",
              }}
            >
              <span class="s-status-bar-dot" style={{
                width: "7px", height: "7px", borderRadius: "50%", flexShrink: 0,
                background: dotColorMap[props.connectionStatus] ?? dotColorMap.disconnected,
              }} />
              <span style={{ opacity: 0.6 }}>面板</span>
              <span style={{ fontFamily: "var(--font-mono, monospace)", color: "rgb(var(--color-text))", opacity: 0.85 }}>
                {props.version}
              </span>
            </span>

            {popupOpen.value && (
              <div
                class="plana-status-popup"
                onMouseenter={showEnter}
                onMouseleave={showLeave}
                style={{
                  position: "absolute", bottom: "calc(100% + 8px)", left: 0,
                  minWidth: "220px",
                  background: "rgb(var(--color-surface))",
                  border: "1px solid var(--border-faint, rgb(var(--color-border) / 10%))",
                  borderRadius: "var(--radius-md, 6px)",
                  boxShadow: "0 4px 20px rgb(0 0 0 / 20%)",
                  padding: "10px 14px",
                  zIndex: 100,
                  fontSize: "0.75rem",
                  lineHeight: 1.6,
                  color: "rgb(var(--color-text))",
                  backdropFilter: "blur(var(--blur-md, 12px))",
                }}
              >
                {info ? (
                  <>
                    <div style={{ display: "flex", alignItems: "center", gap: "6px", marginBottom: "6px", fontWeight: 600, fontSize: "0.8125rem" }}>
                      {qualityIcon(info.quality, 14)}
                      <span style={{ color: dotColorMap[info.state] ?? dotColorMap.disconnected }}>
                        {info.state === "connected" ? "\u5df2\u8fde\u63a5" : info.state === "reconnecting" ? "\u91cd\u8fde\u4e2d" : "\u65ad\u5f00"}
                      </span>
                    </div>
                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <Cpu size={12} style={{ opacity: 0.5, flexShrink: 0 }} />
                      <span style={{ opacity: 0.5, marginRight: "auto" }}>\u534f\u8bae</span>
                      <span>{tierLabel[info.tier] ?? info.tier}</span>
                    </div>
                    {info.latencyMs !== null && (
                      <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                        <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: latencyColor(info.latencyMs), flexShrink: 0 }} />
                        <span style={{ opacity: 0.5, marginRight: "auto" }}>\u5ef6\u8fdf</span>
                        <span style={{ color: latencyColor(info.latencyMs), fontFamily: "var(--font-mono, monospace)", fontWeight: 600 }}>
                          {info.latencyMs} ms
                        </span>
                      </div>
                    )}
                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <Globe size={12} style={{ opacity: 0.5, flexShrink: 0 }} />
                      <span style={{ opacity: 0.5, marginRight: "auto" }}>\u7f51\u7edc</span>
                      <span>
                        {regionFlag[info.region] ?? ""} {regionLabel[info.region] ?? info.region}
                        {info.isLocalhost ? " \u00b7 \u672c\u5730" : ""}
                      </span>
                    </div>
                  </>
                ) : (
                  <div style={{ opacity: 0.5 }}>\u83b7\u53d6\u8fde\u63a5\u4fe1\u606f\u4e2d...</div>
                )}
              </div>
            )}
          </div>
        </footer>
      );
    };
  },
});
