import { computed, defineComponent, onBeforeUnmount, onMounted, ref, type PropType } from "vue";
import "./GaugeCarousel.scss";

export interface GaugeItem {
  label: string;
  value: number;
  unit: string;
  pct: number;
}

export default defineComponent({
  name: "GaugeCarousel",
  props: {
    items: { type: Array as PropType<GaugeItem[]>, required: true },
    intervalMs: { type: Number, default: 3000 },
  },
  setup(props, { slots }) {
    const page = ref(0);
    const transitioning = ref(false);
    const jumping = ref(false);
    let timer: ReturnType<typeof setInterval> | null = null;
    const total = computed(() => props.items.length);
    const isActive = computed(() => total.value > 2);

    // Smooth transition except when snapping back to start
    function advance() {
      transitioning.value = true;
      page.value++;
    }

    onMounted(() => {
      if (isActive.value) timer = setInterval(advance, props.intervalMs);
    });
    onBeforeUnmount(() => { if (timer) clearInterval(timer); });

    const stageStyle = computed(() => {
      const n = total.value;
      // Each item = 50% of viewport; stage holds all items
      const stagePct = n * 50;
      // The visible window starts at page%n items from left
      const shift = -((page.value % n) * 50);
      return {
        width: `${stagePct}%`,
        transform: `translateX(${shift}%)`,
        transition: transitioning.value
          ? "transform 0.35s var(--ease-out-expo, ease-out)"
          : "none",
      };
    });

    function onTransitionEnd() {
      transitioning.value = false;
      if (page.value >= total.value) {
        // Snap back: jump to equivalent position in first cycle
        jumping.value = true;
        page.value = page.value % total.value;
        // Force a re-render with transition:none, then re-enable
        requestAnimationFrame(() => {
          jumping.value = false;
        });
      }
    }

    const itemStyle = computed(() => ({
      flex: `0 0 calc(100% / ${total.value} - ${(total.value - 1) / total.value} * var(--p-gc-gap, 8px))`,
      display: "flex" as const,
      alignItems: "center" as const,
    }));

    const stepPct = computed(() => {
      const n = total.value;
      return 100 / n;
    });

    const stageStyle = computed(() => {
      const n = total.value;
      const shift = -(page.value % n) * stepPct.value;
      return {
        width: `${n * 50}%`,
        display: "flex",
        flexWrap: "nowrap" as const,
        gap: "var(--p-gc-gap, 8px)",
        transform: `translateX(${shift}%)`,
        transition: transitioning.value ? "transform 0.35s var(--ease-out-expo, ease-out)" : "none",
      };
    });

    return () => (
      <div class="p-gauge-carousel">
        {!isActive.value ? (
          <div class="p-gauge-carousel__track">
            {props.items.map((it) => (
              <div key={it.label} style={itemStyle}>
                {slots.default?.({ item: it })}
              </div>
            ))}
          </div>
        ) : (
          <div class="p-gauge-carousel__viewport">
            <div
              class="p-gauge-carousel__stage"
              style={stageStyle.value}
              onTransitionend={onTransitionEnd}
            >
              {props.items.map((it) => (
                <div key={it.label} style={itemStyle}>
                  {slots.default?.({ item: it })}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  },
});
