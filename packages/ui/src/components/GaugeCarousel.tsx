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
    let timer: ReturnType<typeof setInterval> | null = null;

    const total = computed(() => props.items.length);

    const visible = computed(() => {
      const n = total.value;
      if (n <= 2) return props.items;
      const i = (page.value * 2) % n;
      const j = (i + 1) % n;
      return [props.items[i], props.items[j]];
    });

    onMounted(() => {
      if (total.value > 2) timer = setInterval(() => { page.value++; }, props.intervalMs);
    });
    onBeforeUnmount(() => { if (timer) clearInterval(timer); });

    return () => (
      <div class="p-gauge-carousel" style={total.value > 2 ? { overflow: "hidden" } : undefined}>
        <div class="p-gauge-carousel__track">
          {visible.value.map((it) => (
            <div key={it.label} class="p-gauge-carousel__item">
              {slots.default?.({ item: it })}
            </div>
          ))}
        </div>
      </div>
    );
  },
});
