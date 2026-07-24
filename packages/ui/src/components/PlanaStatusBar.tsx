import { defineComponent, type PropType } from "vue";

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
  },
  setup(props) {
    const dotColorMap: Record<string, string> = {
      connected: "rgb(var(--color-success))",
      reconnecting: "rgb(var(--color-warning))",
      disconnected: "rgb(var(--color-error))",
    };

    return () => (
      <footer
        class="s-status-bar"
        style={{
          position: "fixed",
          bottom: 0,
          left: 0,
          right: 0,
          height: "var(--s-footer-height)",
          display: "flex",
          alignItems: "center",
          padding: "0 var(--space-16, 1rem)",
          background: "rgb(var(--color-surface))",
          backdropFilter: "blur(var(--blur-md))",
          borderTop: "1px solid var(--border-faint, rgb(var(--color-border) / 10%))",
          zIndex: "var(--z-sidebar, 30)",
          flexShrink: 0,
        }}
      >
        <span
          class="s-status-bar-tag"
          style={{
            display: "inline-flex",
            alignItems: "center",
            height: "24px",
            gap: "5px",
            padding: "0 8px",
            borderRadius: "var(--radius-md)",
            fontSize: "var(--text-2xs, 0.625rem)",
            lineHeight: 1,
            background: "rgb(var(--color-surface) / var(--opacity-half))",
            color: "rgb(var(--color-muted))",
            userSelect: "none",
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
          <span style={{ fontFamily: "var(--font-mono)", color: "rgb(var(--color-text))", opacity: 0.85 }}>
            {props.version}
          </span>
        </span>
      </footer>
    );
  },
});
