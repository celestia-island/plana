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
    const viewportRef = ref<HTMLDivElement | null>(null);
    const viewportW = ref(0);
    let timer: ReturnType<typeof setInterval> | null = null;

    const total = computed(() => props.items.length);
    const isActive = computed(() => total.value > 2);
    const doubled = computed(() => [...props.items, ...props.items]);

    const itemWidthPx = computed(() => {
      const w = viewportW.value || 200;
      return Math.round(w / 2);
    });

    const gapPx = 4;
    const stepPx = computed(() => itemWidthPx.value + gapPx);
    const offsetPx = computed(() => -(page.value * stepPx.value));
    const trackStyle = computed(() => ({
      width: `${(itemWidthPx.value + gapPx) * doubled.value.length}px`,
      transform: `translateX(${offsetPx.value}px)`,
      transition: transitioning.value ? "transform 0.3s var(--ease-out-expo, ease-out)" : "none",
      display: "flex",
      flexWrap: "nowrap" as const,
      gap: `${gapPx}px`,
    }));
    const itemStyle = computed(() => ({
      flex: `0 0 ${itemWidthPx.value}px`,
    }));

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
      if (viewportRef.value) {
        viewportW.value = viewportRef.value.offsetWidth;
        new ResizeObserver(([e]) => {
          if (e) viewportW.value = e.contentRect.width;
        }).observe(viewportRef.value);
      }
      if (isActive.value) timer = setInterval(advance, props.intervalMs);
    });
    onBeforeUnmount(() => { if (timer) clearInterval(timer); });

    return () => (
      <div class="p-gauge-carousel">
        {!isActive.value ? (
          <div class="p-gauge-carousel__static">
            {props.items.map((it) => (
              <div key={it.label} class="p-gauge-carousel__item">
                {slots.default?.({ item: it })}
              </div>
            ))}
          </div>
        ) : (
          <div ref={viewportRef} class="p-gauge-carousel__viewport">
            <div style={trackStyle.value} onTransitionend={handleTransitionEnd}>
              {doubled.value.map((it, i) => (
                <div key={`${it.label}-${i < total.value ? "a" : "b"}`} style={itemStyle.value}>
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
