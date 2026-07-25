import { computed, defineComponent, onBeforeUnmount, onMounted, ref, type PropType } from "vue";
import "./PlanaGaugeCarousel.scss";

export interface GaugeItem {
  label: string;
  value: number;
  unit: string;
  pct: number;
}

export default defineComponent({
  name: "PlanaGaugeCarousel",
  props: {
    items: { type: Array as PropType<GaugeItem[]>, required: true },
    intervalMs: { type: Number, default: 3000 },
    onTransitionStart: { type: Function as PropType<(() => void) | undefined>, default: undefined },
    onTransitionEnd: { type: Function as PropType<(() => void) | undefined>, default: undefined },
  },
  setup(props, { slots }) {
    const page = ref(0);
    const transitioning = ref(false);
    let timer: ReturnType<typeof setInterval> | null = null;

    const total = computed(() => props.items.length);
    const isActive = computed(() => total.value > 2);

    // Track width as %: each item = 50%, doubled = 2 * total items
    const trackPct = computed(() => total.value * 100);

    const offsetPct = computed(() => {
      // With 2 visible items (step=50%), advance by 50% per page
      return -(page.value * 50);
    });

    // Double items for seamless infinite scroll
    const doubled = computed(() => [...props.items, ...props.items]);

    function advance() {
      if (!isActive.value) return;
      props.onTransitionStart?.();
      transitioning.value = true;
      page.value = page.value + 1;
    }

    function handleTransitionEnd() {
      transitioning.value = false;
      props.onTransitionEnd?.();
      if (page.value >= total.value) {
        page.value = page.value % total.value;
      }
    }

    onMounted(() => {
      if (isActive.value) timer = setInterval(advance, props.intervalMs);
    });
    onBeforeUnmount(() => { if (timer) clearInterval(timer); });

    const itemStyle = {
      flex: "0 0 calc(50% - 4px)",
      display: "flex",
      alignItems: "center",
    };

    return () => (
      <div style={{ width: "100%", overflow: "hidden" }}>
        {!isActive.value ? (
          <div style={{ display: "flex", gap: "8px" }}>
            {props.items.map((it) => (
              <div key={it.label} style={itemStyle}>
                {slots.default?.({ item: it })}
              </div>
            ))}
          </div>
        ) : (
          <div style={{ overflow: "hidden", width: "100%" }}>
            <div
              style={{
                display: "flex",
                flexWrap: "nowrap",
                width: `${trackPct.value}%`,
                transform: `translateX(${offsetPct.value}%)`,
                transition: transitioning.value ? "transform 0.3s var(--ease-out-expo)" : "none",
                gap: "8px",
              }}
              onTransitionend={handleTransitionEnd}
            >
              {doubled.value.map((it, i) => (
                <div key={`${it.label}-${i < total.value ? "a" : "b"}`} style={itemStyle}>
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
