import { computed, defineComponent, onMounted, onUnmounted, ref, type PropType } from "vue";

import { Maximize2, ZoomIn, ZoomOut } from "lucide-vue-next";

import "./PlanaMinimap.scss";

export interface PlanaMinimapBox {
  id: string;
  bounds: { x: number; y: number; w: number; h: number };
  color: string;
}

/**
 * Minimal overview ("minimap") for pan/zoom canvases: renders the content
 * bounds as a scaled-down SVG with optional node boxes, a hub marker and an
 * optional background image (image navigator mode). Dragging pans the
 * canvas; the zoom bar offers in/out/reset with percentage readout.
 *
 * Upstreamed from shittim-chest (P5#A A1); used by topology viewers, image
 * viewers and the media pipeline canvas.
 */
export default defineComponent({
  name: "PlanaMinimap",
  props: {
    boxes: { type: Array as PropType<PlanaMinimapBox[]>, default: () => [] },
    hubPos: { type: Object as PropType<{ x: number; y: number } | null>, default: null },
    /** Optional image rendered as the minimap background (image navigator).
     *  When set, the minimap renders even with no boxes. */
    imageSrc: { type: String, default: undefined },
    imageBounds: {
      type: Object as PropType<{ x: number; y: number; w: number; h: number }>,
      default: undefined,
    },
    zoom: { type: Number, default: 1 },
    panX: { type: Number, default: 0 },
    panY: { type: Number, default: 0 },
    viewportWidth: { type: Number, default: 800 },
    viewportHeight: { type: Number, default: 600 },
    contentBounds: {
      type: Object as PropType<{ x: number; y: number; w: number; h: number }>,
      default: () => ({ x: 0, y: 0, w: 1200, h: 800 }),
    },
    zoomPercent: { type: Number, default: 100 },
    canZoomIn: { type: Boolean, default: true },
    canZoomOut: { type: Boolean, default: true },
    onZoomIn: { type: Function as PropType<() => void>, default: undefined },
    onZoomOut: { type: Function as PropType<() => void>, default: undefined },
    onReset: { type: Function as PropType<() => void>, default: undefined },
    onPanDelta: { type: Function as PropType<(dx: number, dy: number) => void>, default: undefined },
    /** Override the zoom bar button tooltips (i18n). */
    zoomOutTitle: { type: String, default: "Zoom out" },
    zoomInTitle: { type: String, default: "Zoom in" },
    resetTitle: { type: String, default: "Fit" },
  },
  setup(props) {
    const svgW = 160;
    const svgH = 110;
    const rootRef = ref<HTMLElement | null>(null);
    const dragging = ref(false);
    const dragStart = ref({ x: 0, y: 0 });

    const cb = computed(() => props.contentBounds);
    const overpanW = computed(() => Math.max(props.contentBounds.w * 0.5, props.viewportWidth * 0.3));
    const overpanH = computed(() => Math.max(props.contentBounds.h * 0.5, props.viewportHeight * 0.3));

    const mapRect = computed(() => ({
      x: cb.value.x - overpanW.value,
      y: cb.value.y - overpanH.value,
      w: cb.value.w + overpanW.value * 2,
      h: cb.value.h + overpanH.value * 2,
    }));

    const scale = computed(() => {
      const sx = svgW / mapRect.value.w;
      const sy = svgH / mapRect.value.h;
      return Math.min(sx, sy);
    });

    const contentOffset = computed(() => {
      const sw = mapRect.value.w * scale.value;
      const sh = mapRect.value.h * scale.value;
      return { ox: (svgW - sw) / 2, oy: (svgH - sh) / 2 };
    });

    function toMap(wx: number, wy: number): [number, number] {
      const s = scale.value;
      const { ox, oy } = contentOffset.value;
      return [(wx - mapRect.value.x) * s + ox, (wy - mapRect.value.y) * s + oy];
    }

    const viewportRect = computed(() => {
      const z = props.zoom;
      const tl = toMap(-props.panX / z, -props.panY / z);
      const br = toMap(-props.panX / z + props.viewportWidth / z, -props.panY / z + props.viewportHeight / z);
      return { x: tl[0], y: tl[1], w: Math.max(1, br[0] - tl[0]), h: Math.max(1, br[1] - tl[1]) };
    });

    const onDown = (e: PointerEvent) => {
      if ((e.target as HTMLElement).closest(".mm-zoom-bar")) return;
      e.stopPropagation();
      e.preventDefault();
      dragging.value = true;
      dragStart.value = { x: e.clientX, y: e.clientY };
      rootRef.value?.setPointerCapture(e.pointerId);
    };
    const onMove = (e: PointerEvent) => {
      if (!dragging.value) return;
      e.stopPropagation();
      const dx = e.clientX - dragStart.value.x;
      const dy = e.clientY - dragStart.value.y;
      dragStart.value = { x: e.clientX, y: e.clientY };
      if (scale.value > 0 && props.onPanDelta) {
        props.onPanDelta((-dx / scale.value) * props.zoom, (-dy / scale.value) * props.zoom);
      }
    };
    const onUp = (e: PointerEvent) => {
      if (!dragging.value) return;
      e.stopPropagation();
      dragging.value = false;
      rootRef.value?.releasePointerCapture(e.pointerId);
    };

    onMounted(() => {
      const el = rootRef.value;
      if (!el) return;
      el.addEventListener("pointerdown", onDown);
      el.addEventListener("pointermove", onMove);
      el.addEventListener("pointerup", onUp);
      el.addEventListener("pointercancel", onUp);
    });
    onUnmounted(() => {
      const el = rootRef.value;
      if (!el) return;
      el.removeEventListener("pointerdown", onDown);
      el.removeEventListener("pointermove", onMove);
      el.removeEventListener("pointerup", onUp);
      el.removeEventListener("pointercancel", onUp);
    });

    return () => {
      if (props.boxes.length === 0 && !props.imageSrc) return null;

      const s = scale.value;
      const vr = viewportRect.value;

      const ib = props.imageBounds ?? props.contentBounds;
      const imgPos = props.imageSrc ? toMap(ib.x, ib.y) : null;

      const boxRects = props.boxes.map((b) => {
        const p = toMap(b.bounds.x, b.bounds.y);
        return (
          <rect
            key={`mm-${b.id}`}
            x={p[0]}
            y={p[1]}
            width={b.bounds.w * s}
            height={b.bounds.h * s}
            rx={2}
            fill={b.color}
            opacity="0.28"
            stroke={b.color}
            stroke-width="0.6"
          />
        );
      });

      const hubP = props.hubPos ? toMap(props.hubPos.x, props.hubPos.y) : null;

      return (
        <div
          ref={rootRef}
          class="radial-minimap"
          data-dragging={dragging.value ? "" : undefined}
        >
          <svg viewBox={`0 0 ${svgW} ${svgH}`} width={svgW} height={svgH} class="radial-minimap-svg">
            {imgPos && (
              <image
                href={props.imageSrc}
                x={imgPos[0]}
                y={imgPos[1]}
                width={ib.w * s}
                height={ib.h * s}
                preserveAspectRatio="none"
                class="mm-image"
              />
            )}
            {boxRects}
            {hubP && (
              <circle
                cx={hubP[0]}
                cy={hubP[1]}
                r={3}
                fill="rgb(var(--color-primary))"
                filter="drop-shadow(0 0 2px rgb(var(--color-primary) / 0.6))"
              />
            )}
            <rect
              x={vr.x}
              y={vr.y}
              width={Math.max(1, vr.w)}
              height={Math.max(1, vr.h)}
              fill="none"
              stroke="rgb(var(--color-primary))"
              stroke-width="1"
              stroke-dasharray="3 2"
              rx="2"
              opacity="0.85"
            />
          </svg>
          <div class="mm-zoom-bar">
            <button class="mm-zoom-btn" onClick={() => props.onZoomOut?.()} disabled={!props.canZoomOut} title={props.zoomOutTitle}>
              <ZoomOut size={12} />
            </button>
            <span class="mm-zoom-label">{props.zoomPercent}%</span>
            <button class="mm-zoom-btn" onClick={() => props.onZoomIn?.()} disabled={!props.canZoomIn} title={props.zoomInTitle}>
              <ZoomIn size={12} />
            </button>
            {props.onReset && (
              <button class="mm-zoom-btn mm-zoom-reset-btn" onClick={() => props.onReset?.()} title={props.resetTitle}>
                <Maximize2 size={11} />
              </button>
            )}
          </div>
        </div>
      );
    };
  },
});
