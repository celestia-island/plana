import { defineComponent, ref, onMounted, onUnmounted } from "vue";

export const PClock = defineComponent({
  name: "PlanaClock",
  setup() {
    const now = ref("");
    let timer: ReturnType<typeof setInterval> | null = null;

    function tick() {
      const d = new Date();
      now.value = String(d.getHours()).padStart(2, "0") + ":" + String(d.getMinutes()).padStart(2, "0");
    }

    onMounted(() => {
      tick();
      timer = setInterval(tick, 30000);
    });
    onUnmounted(() => {
      if (timer) clearInterval(timer);
    });

    return () => (
      <span style={{ fontSize: "0.75rem", color: "rgb(var(--color-muted))", userSelect: "none" }}>
        {now.value}
      </span>
    );
  },
});
