import { computed, defineComponent, type PropType } from "vue";
import { HkTooltip } from "@celestia-island/hikari";

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
  sse: "Server-Sent Events",
  poll: "HTTP poll",
};

const qualityLabel: Record<string, string> = {
  excellent: "Excellent",
  good: "Good",
  fair: "Fair",
  poor: "Poor",
  unknown: "Unknown",
};

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
    const dotColorMap: Record<string, string> = {
      connected: "rgb(var(--color-success))",
      reconnecting: "rgb(var(--color-warning))",
      disconnected: "rgb(var(--color-error))",
    };

    const tooltip = computed(() => {
      const info = props.connectionInfo;
      if (!info) {
        const s = props.connectionStatus;
        return s === "connected" ? "\u590d\u5236\u7248\u672c\u53f7" : s === "reconnecting" ? "\u7acb\u5373\u91cd\u8bd5" : "\u70b9\u51fb\u91cd\u65b0\u8fde\u63a5";
      }
      const lines = [
        `\u534f\u8bae: ${tierLabel[info.tier] ?? info.tier}`,
        `\u8d28\u91cf: ${qualityLabel[info.quality] ?? info.quality}`,
      ];
      if (info.latencyMs !== null) lines.push(`\u5ef6\u8fdf: ${info.latencyMs} ms`);
      if (info.region) lines.push(`\u533a\u57df: ${info.region}`);
      if (info.isLocalhost) lines.push("\u7f51\u7edc: \u672c\u5730\u5185\u7f51");
      return lines.join(" \u00b7 ");
    });

    return () => (
      <footer
        class="s-status-bar"
        style={{
          position: "fixed",
          bottom: 0,
          left: 0,
          right: 0,
          height: "var(--s-footer-height, 2.5rem)",
          display: "flex",
          alignItems: "center",
          padding: "0 var(--space-16, 1rem)",
          background: "rgb(var(--color-surface))",
          backdropFilter: "blur(var(--blur-md, 12px))",
          borderTop: "1px solid var(--border-faint, rgb(var(--color-border) / 10%))",
          zIndex: "var(--z-sidebar, 30)",
          flexShrink: 0,
        }}
      >
        <HkTooltip text={tooltip.value} placement="top">
          <span
            class="s-status-bar-tag"
            role="button"
            tabindex={0}
            style={{
              display: "inline-flex",
              alignItems: "center",
              height: "24px",
              gap: "5px",
              padding: "0 8px",
              borderRadius: "var(--radius-md, 6px)",
              fontSize: "var(--text-2xs, 0.625rem)",
              lineHeight: 1,
              background: "rgb(var(--color-surface) / var(--opacity-half, 0.5))",
              color: "rgb(var(--color-muted))",
              userSelect: "none",
              cursor: "pointer",
            }}
          >
            <span
              class="s-status-bar-dot"
              style={{
                width: "7px",
                height: "7px",
                borderRadius: "50%",
                flexShrink: 0,
                background: dotColorMap[props.connectionStatus] ?? dotColorMap.disconnected,
              }}
            />
            <span style={{ opacity: 0.6 }}>面板</span>
            <span style={{ fontFamily: "var(--font-mono, monospace)", color: "rgb(var(--color-text))", opacity: 0.85 }}>
              {props.version}
            </span>
          </span>
        </HkTooltip>
      </footer>
    );
  },
});
