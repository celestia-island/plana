import { defineComponent, ref, type PropType } from "vue";
import { HkPopover } from "@celestia-island/hikari";
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

const regionFlag: Record<string, string> = {
  CN: "🇨🇳", JP: "🇯🇵", KR: "🇰🇷",
  US: "🇺🇸", GB: "🇬🇧", DE: "🇩🇪",
  FR: "🇫🇷", SA: "🇸🇦", TW: "🇹🇼",
  HK: "🇭🇰", BR: "🇧🇷", RU: "🇷🇺",
  CA: "🇨🇦", AU: "🇦🇺", PT: "🇵🇹",
  ES: "🇪🇸",
};

const regionLabel: Record<string, string> = {
  CN: "中国大陆", JP: "日本", KR: "韩国",
  US: "美国", GB: "英国", DE: "德国",
  FR: "法国", SA: "沙特", TW: "台湾",
  HK: "香港", BR: "巴西", RU: "俄罗斯",
  CA: "加拿大", AU: "澳大利亚",
  PT: "葡萄牙", ES: "西班牙",
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
    version: { type: String, default: "0.1.0" },
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
    const anchorRef = ref<HTMLElement | null>(null);
    let closeTimer: ReturnType<typeof setTimeout> | null = null;

    const dotColorMap: Record<string, string> = {
      connected: "rgb(var(--color-success))",
      reconnecting: "rgb(var(--color-warning))",
      disconnected: "rgb(var(--color-error))",
    };

    function onTagEnter() {
      if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
      popupOpen.value = true;
    }
    function onTagLeave() {
      closeTimer = setTimeout(() => { popupOpen.value = false; }, 250);
    }
    function onPopupEnter() {
      if (closeTimer) { clearTimeout(closeTimer); closeTimer = null; }
    }
    function onPopupLeave() {
      popupOpen.value = false;
    }

    return () => {
      const info = props.connectionInfo;
      return (
        <footer
          class="s-status-bar"
          style={{
            position: "fixed", bottom: 0, left: 0, right: 0,
            height: "var(--s-footer-height, 2.5rem)",
            display: "flex", alignItems: "center",
            zIndex: "var(--z-header, 30)", flexShrink: 0,
            padding: "0 var(--space-16, 1rem)",
          }}
        >
          <div style={{
            position: "absolute", inset: 0,
            background: "rgb(var(--color-surface))",
            backdropFilter: "blur(var(--blur-md, 12px))",
            borderTop: "1px solid var(--border-faint, rgb(var(--color-border) / 10%))",
          }} />
          <span
            ref={anchorRef}
            class="s-status-bar-tag"
            role="button"
            tabindex={0}
            onMouseenter={onTagEnter}
            onMouseleave={onTagLeave}
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

          <HkPopover
            modelValue={popupOpen.value}
            onUpdate:modelValue={(v: boolean) => { popupOpen.value = v; }}
            placement="top-start"
            backdrop={false}
            closeOnBackdrop={false}
            anchorRef={anchorRef.value}
          >
            <div
              onMouseenter={onPopupEnter}
              onMouseleave={onPopupLeave}
              style={{
                minWidth: "220px",
                padding: "10px 14px",
                fontSize: "0.75rem",
                lineHeight: 1.6,
                color: "rgb(var(--color-text))",
              }}
            >
              {info ? (
                <>
                  <div style={{ display: "flex", alignItems: "center", gap: "6px", marginBottom: "6px", fontWeight: 600, fontSize: "0.8125rem" }}>
                    {qualityIcon(info.quality, 14)}
                    <span style={{ color: dotColorMap[info.state] ?? dotColorMap.disconnected }}>
                      {info.state === "connected" ? "已连接" : info.state === "reconnecting" ? "重连中" : "断开"}
                    </span>
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                    <Cpu size={12} style={{ opacity: 0.5, flexShrink: 0 }} />
                    <span style={{ opacity: 0.5, marginRight: "auto" }}>协议</span>
                    <span>{tierLabel[info.tier] ?? info.tier}</span>
                  </div>
                  {info.latencyMs !== null && (
                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <span style={{ width: "6px", height: "6px", borderRadius: "50%", background: latencyColor(info.latencyMs), flexShrink: 0 }} />
                      <span style={{ opacity: 0.5, marginRight: "auto" }}>延迟</span>
                      <span style={{ color: latencyColor(info.latencyMs), fontFamily: "var(--font-mono, monospace)", fontWeight: 600 }}>
                        {info.latencyMs} ms
                      </span>
                    </div>
                  )}
                  <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                    <Globe size={12} style={{ opacity: 0.5, flexShrink: 0 }} />
                    <span style={{ opacity: 0.5, marginRight: "auto" }}>网络</span>
                    <span>
                      {regionFlag[info.region] ?? ""} {regionLabel[info.region] ?? info.region}
                      {info.isLocalhost ? " · 本地" : ""}
                    </span>
                  </div>
                </>
              ) : (
                <div style={{ opacity: 0.5 }}>获取连接信息中...</div>
              )}
            </div>
          </HkPopover>
        </footer>
      );
    };
  },
});
