import { defineComponent, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "@celestia-island/hikari";

import enLocale from "../i18n/locales/en/connection.json";
import zhsLocale from "../i18n/locales/zhs/connection.json";
import zhtLocale from "../i18n/locales/zht/connection.json";

const CYCLE_MS = 1000;
const TICK_MS = 30_000;

const Triangle = () => <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M12 2L2 22h20z"/></svg>;
const Circle = () => <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><circle cx="12" cy="12" r="10"/></svg>;
const Cross = () => <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M2 2l20 20M22 2L2 20"/></svg>;
const Square = () => <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><rect x="2" y="2" width="20" height="20"/></svg>;

const BUTTONS = [
  { component: Triangle, colorVar: "--color-success", name: "triangle" },
  { component: Circle, colorVar: "--color-error", name: "circle" },
  { component: Cross, colorVar: "--color-warning", name: "cross" },
  { component: Square, colorVar: "--color-primary", name: "square" },
] as const;

export const PSystemTray = defineComponent({
  name: "PlanaSystemTray",
  setup() {
    const now = ref("");
    const activeIndex = ref(0);
    let cycleHandle: ReturnType<typeof setInterval> | null = null;
    let clockHandle: ReturnType<typeof setInterval> | null = null;

    onMounted(() => {
      const tick = () => {
        const d = new Date();
        now.value = String(d.getHours()).padStart(2, "0") + ":" + String(d.getMinutes()).padStart(2, "0");
      };
      tick();
      cycleHandle = setInterval(() => { activeIndex.value = (activeIndex.value + 1) % BUTTONS.length; }, CYCLE_MS);
      clockHandle = setInterval(tick, TICK_MS);
    });

    onBeforeUnmount(() => {
      if (cycleHandle) clearInterval(cycleHandle);
      if (clockHandle) clearInterval(clockHandle);
    });

    return () => (
      <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
        {BUTTONS.map(({ component: Icon, colorVar, name }, i) => (
          <span
            key={name}
            data-active={i === activeIndex.value || undefined}
            style={{
              display: "inline-flex", opacity: i === activeIndex.value ? 1 : 0.3,
              color: i === activeIndex.value ? `rgb(var(${colorVar}))` : undefined,
              transition: "opacity 0.15s, color 0.15s",
            }}
          >
            <Icon />
          </span>
        ))}
        <span style={{ fontFamily: "var(--font-mono, monospace)", fontSize: "0.75rem", color: "rgb(var(--color-muted))" }}>
          {now.value}
        </span>
      </div>
    );
  },
});
