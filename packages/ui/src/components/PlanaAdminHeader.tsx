import { defineComponent, type PropType } from "vue";
import { LogOut } from "lucide-vue-next";
import { PlanaLocalePicker } from "./PlanaLocalePicker";

export interface LocaleOption {
  code: string;
  label: string;
}

export const PlanaAdminHeader = defineComponent({
  name: "PlanaAdminHeader",
  props: {
    title: {
      type: String,
      default: "",
    },
    authenticated: {
      type: Boolean,
      default: false,
    },
    username: {
      type: String,
      default: "",
    },
    locales: {
      type: Array as PropType<LocaleOption[]>,
      default: () => [],
    },
    currentLocale: {
      type: String,
      default: "en",
    },
    tLocale: {
      type: Function as PropType<(key: string) => string>,
      default: undefined,
    },
  },
  emits: ["logout", "localeSelect"],
  setup(props, { emit }) {
    return () => (
      <header
        class="plana-admin-header"
        style={{
          position: "relative",
          display: "flex",
          alignItems: "center",
          gap: "0.75rem",
          padding: "0 1.5rem",
          height: "48px",
          flexShrink: 0,
        }}
      >
        <div style={{
          position: "absolute", inset: 0,
          background: "rgb(var(--color-surface))",
          backdropFilter: "blur(var(--blur-md, 12px))",
          borderBottom: "1px solid var(--border-faint, rgb(var(--color-border) / 10%))",
        }} />
        <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
          {props.authenticated && props.username ? (
            <>
              <span
                style={{
                  width: "28px",
                  height: "28px",
                  borderRadius: "50%",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  background: "var(--c-primary-light, rgb(var(--color-primary) / 15%))",
                  color: "var(--c-primary, rgb(var(--color-primary)))",
                  fontSize: "0.75rem",
                  fontWeight: 700,
                  userSelect: "none",
                }}
              >
                {props.username.charAt(0).toUpperCase()}
              </span>
              <span style={{ fontSize: "0.875rem", fontWeight: 600, color: "var(--color-text-primary, var(--color-text))", maxWidth: "8rem", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {props.username}
              </span>
            </>
          ) : null}
        </div>

        {props.title && (
          <h2 style={{ fontSize: "0.875rem", fontWeight: 600, color: "var(--color-text-primary, var(--color-text))", margin: 0 }}>
            {props.title}
          </h2>
        )}

        <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: "0.375rem" }}>
          <PlanaLocalePicker
            locales={props.locales}
            currentLocale={props.currentLocale}
            onSelect={(code: string) => emit("localeSelect", code)}
          />

          {props.authenticated && (
            <button
              class="plana-admin-header-logout"
              type="button"
              title={props.tLocale ? props.tLocale("common.logout") : "Logout"}
              style={{
                display: "inline-flex",
                alignItems: "center",
                justifyContent: "center",
                width: "32px",
                height: "32px",
                padding: 0,
                border: "1px solid transparent",
                borderRadius: "var(--radius-sm, 4px)",
                background: "transparent",
                color: "rgb(var(--color-muted))",
                cursor: "pointer",
                transition: "color 0.15s, background 0.15s, border-color 0.15s",
              }}
              onMouseenter={(e: MouseEvent) => {
                const target = e.currentTarget as HTMLElement;
                target.style.color = "rgb(var(--color-error))";
                target.style.background = "rgb(var(--color-error) / 10%)";
                target.style.borderColor = "rgb(var(--color-error) / 25%)";
              }}
              onMouseleave={(e: MouseEvent) => {
                const target = e.currentTarget as HTMLElement;
                target.style.color = "rgb(var(--color-muted))";
                target.style.background = "transparent";
                target.style.borderColor = "transparent";
              }}
              onClick={() => emit("logout")}
            >
              <LogOut size={16} />
            </button>
          )}
        </div>
      </header>
    );
  },
});
