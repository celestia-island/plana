import { computed, defineComponent, onBeforeUnmount, onMounted, ref, type PropType, TransitionGroup } from "vue";
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
    let timer: ReturnType<typeof setInterval> | null = null;
    const total = computed(() => props.items.length);

    const visibleSet = computed(() => {
      const n = total.value;
      if (n <= 2) return new Set(props.items.map((_, i) => i));
      const i = (page.value * 2) % n;
      const j = (i + 1) % n;
      return new Set([i, j]);
    });

    const ordered = computed(() => {
      const n = total.value;
      const set = visibleSet.value;
      const visible = Array.from(set).sort((a, b) => a - b);
      const hidden = [];
      for (let i = 0; i < n; i++) {
        if (!set.has(i)) hidden.push(i);
      }
      return [...visible, ...hidden];
    });

    onMounted(() => {
      if (total.value > 2) timer = setInterval(() => { page.value++; }, props.intervalMs);
    });
    onBeforeUnmount(() => { if (timer) clearInterval(timer); });

    return () => (
      <div class="p-gauge-carousel" style={total.value > 2 ? { overflow: "hidden" } : undefined}>
        <TransitionGroup
          tag="div"
          class="p-gauge-carousel__grid"
          name="gc-move"
          moveClass="gc-move"
        >
          {ordered.value.map((i) => {
            const it = props.items[i];
            const isVisible = visibleSet.value.has(i);
            return (
              <div
                key={it.label}
                class="p-gauge-carousel__item"
                style={isVisible ? undefined : { opacity: 0, pointerEvents: "none" }}
              >
                {slots.default?.({ item: it })}
              </div>
            );
          })}
        </TransitionGroup>
      </div>
    );
  },
});
