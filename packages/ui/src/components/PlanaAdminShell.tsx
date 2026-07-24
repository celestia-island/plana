import { defineComponent } from "vue";

export const PlanaAdminShell = defineComponent({
  name: "PlanaAdminShell",
  props: {
    sidebarCollapsed: {
      type: Boolean,
      default: false,
    },
    sidebarWidth: {
      type: String,
      default: "224px",
    },
  },
  setup(props, { slots }) {
    return () => (
      <div class="plana-shell" style={{ display: "flex", flexDirection: "column", height: "100vh", width: "100vw", overflow: "hidden" }}>
        {slots.header?.() && (
          <header class="plana-shell-header" style={{ flexShrink: 0 }}>
            {slots.header()}
          </header>
        )}
        <div class="plana-shell-body" style={{ display: "flex", flex: 1, minHeight: 0 }}>
          {!props.sidebarCollapsed && slots.sidebar?.() && (
            <aside
              class="plana-shell-sidebar"
              style={{
                width: props.sidebarWidth,
                flexShrink: 0,
                borderRight: "1px solid var(--border-faint, rgb(var(--color-border) / 10%))",
                background: "rgb(var(--color-surface))",
                overflow: "hidden",
              }}
            >
              {slots.sidebar()}
            </aside>
          )}
          <main class="plana-shell-content" style={{ flex: 1, minWidth: 0, minHeight: 0, display: "flex", flexDirection: "column" }}>
            {slots.default?.()}
          </main>
        </div>
        {slots.footer?.() && (
          <footer class="plana-shell-footer" style={{ flexShrink: 0 }}>
            {slots.footer()}
          </footer>
        )}
      </div>
    );
  },
});
