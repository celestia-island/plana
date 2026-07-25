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
      transitioning.value = true;
      const next = (page.value + 1) % total.value;
      page.value = next;
    }

    function onTransitionEnd() {
      transitioning.value = false;
      // When we reach the "clone" half, reset silently to the original position
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
              onTransitionend={onTransitionEnd}
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
