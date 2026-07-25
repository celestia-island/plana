import { computed, defineComponent, onBeforeUnmount, onMounted, ref, type PropType } from "vue";

export interface GaugeItem {
  label: string;
  value: number;
  unit: string;
  pct: number;
}

const GAUGE_CAROUSEL_STEP = 50; // % of container — 2 items visible

export default defineComponent({
  name: "PlanaGaugeCarousel",
  props: {
    items: { type: Array as PropType<GaugeItem[]>, required: true },
    intervalMs: { type: Number, default: 3000 },
    /** Called when slide transition starts — consumer plugs into animation bus. */
    onTransitionStart: { type: Function as PropType<(() => void) | undefined>, default: undefined },
    /** Called when slide transition ends — consumer cancels animation bus handle. */
    onTransitionEnd: { type: Function as PropType<(() => void) | undefined>, default: undefined },
  },
  setup(props, { slots }) {
    const page = ref(0);
    const transitioning = ref(false);
    let timer: ReturnType<typeof setInterval> | null = null;

    const total = computed(() => props.items.length);
    const maxPage = computed(() => Math.max(0, total.value - 1));

    const isActive = computed(() => total.value > 2);
    const offsetPct = computed(() => -(page.value * GAUGE_CAROUSEL_STEP));

    // For seamless wrap: render items twice so the track is always full.
    const doubled = computed(() => [...props.items, ...props.items]);

    function advance() {
      if (!isActive.value) return;
      props.onTransitionStart?.();
      transitioning.value = true;
      // Allow page to advance through the doubled track for seamless wrap
      page.value = page.value + 1;
    }

    function handleTransitionEnd() {
      transitioning.value = false;
      props.onTransitionEnd?.();
      // When we've scrolled past the original set, jump back to the
      // equivalent position in the first copy (no visual change, no transition)
      if (page.value >= total.value) {
        page.value = page.value % total.value;
      }
    }

    function resetTimer() {
      if (timer) clearInterval(timer);
      if (isActive.value) {
        timer = setInterval(advance, props.intervalMs);
      }
    }

    onMounted(() => { resetTimer(); });
    onBeforeUnmount(() => { if (timer) clearInterval(timer); });

    return () => (
      <div class="plana-gauge-carousel">
        {!isActive.value ? (
          <div class="plana-gauge-carousel__track" style={{ gap: "var(--plana-gc-gap, 8px)" }}>
            {props.items.map((it) => (
              <div key={it.label} class="plana-gauge-carousel__item">
                {slots.default?.({ item: it })}
              </div>
            ))}
          </div>
        ) : (
          <div class="plana-gauge-carousel__viewport">
            <div
              class="plana-gauge-carousel__track plana-gauge-carousel__track--animated"
              style={{
                transform: `translateX(${offsetPct.value}%)`,
                transition: transitioning.value ? "transform 0.3s var(--ease-out-expo)" : "none",
                gap: "var(--plana-gc-gap, 8px)",
              }}
              onTransitionend={handleTransitionEnd}
            >
              {doubled.value.map((it, i) => (
                <div key={`${it.label}-${i < total.value ? "a" : "b"}`} class="plana-gauge-carousel__item">
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
