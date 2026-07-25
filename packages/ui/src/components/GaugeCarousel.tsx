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
    onTransitionStart: { type: Function as PropType<(() => void) | undefined>, default: undefined },
    onTransitionEnd: { type: Function as PropType<(() => void) | undefined>, default: undefined },
  },
  setup(props, { slots }) {
    const page = ref(0);
    const transitioning = ref(false);
    let timer: ReturnType<typeof setInterval> | null = null;

    const total = computed(() => props.items.length);
    const isActive = computed(() => total.value > 2);
    const doubled = computed(() => [...props.items, ...props.items]);

    // Track width: each item ~50% → for N unique items, track = N × 50%
    const trackWidthPct = computed(() => total.value * 50);

    const offsetPct = computed(() => -(page.value * 50));

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

    return () => (
      <div class="p-gauge-carousel">
        {!isActive.value ? (
          <div class="p-gauge-carousel__track">
            {props.items.map((it) => (
              <div key={it.label} class="p-gauge-carousel__item">
                {slots.default?.({ item: it })}
              </div>
            ))}
          </div>
        ) : (
          <div class="p-gauge-carousel__viewport">
            <div
              class="p-gauge-carousel__track"
              style={{
                width: `${trackWidthPct.value}%`,
                transform: `translateX(${offsetPct.value}%)`,
                transition: transitioning.value ? "transform 0.3s var(--ease-out-expo, ease-out)" : "none",
              }}
              onTransitionend={handleTransitionEnd}
            >
              {doubled.value.map((it, i) => (
                <div key={`${it.label}-${i < total.value ? "a" : "b"}`} class="p-gauge-carousel__item">
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
