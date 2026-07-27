import { defineComponent, onBeforeUnmount, onMounted, ref } from "vue";
import { Circle, Square, Triangle, X } from "lucide-vue-next";

const CYCLE_MS = 1000;
const TICK_MS = 30_000;

const BUTTONS = [
  { component: Triangle, colorVar: "--color-success", name: "triangle" },
  { component: Circle, colorVar: "--color-error", name: "circle" },
  { component: X, colorVar: "--color-warning", name: "x" },
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
      <div class="s-status-bar-right">
        <div class="s-status-bar-gamepad">
          {BUTTONS.map(({ component: Icon, colorVar, name }, i) => (
            <span
              key={name}
              class="s-status-bar-btn"
              data-shape={name}
              data-active={(i === activeIndex.value) || undefined}
              style={i === activeIndex.value ? { color: `rgb(var(${colorVar}))` } : undefined}
            >
              <Icon />
            </span>
          ))}
        </div>
        <span class="s-status-bar-time">{now.value}</span>
      </div>
    );
  },
});
