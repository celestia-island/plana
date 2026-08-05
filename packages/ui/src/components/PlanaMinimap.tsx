import { computed, defineComponent, type PropType } from "vue";
import { HMinimap } from "@celestia-island/hikari";

export interface PlanaMinimapBox {
  id: string;
  bounds: { x: number; y: number; w: number; h: number };
  color: string;
}

/**
 * Minimal overview ("minimap") for pan/zoom canvases.
 *
 * Thin wrapper over hikari's HMinimap (single source of truth for the
 * design-system primitive), mapping the callback-prop surface to the
 * upstream emits.
 */
export default defineComponent({
  name: "PlanaMinimap",
  props: {
    boxes: { type: Array as PropType<PlanaMinimapBox[]>, default: () => [] },
    hubPos: { type: Object as PropType<{ x: number; y: number } | null>, default: null },
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
    zoomOutTitle: { type: String, default: "Zoom out" },
    zoomInTitle: { type: String, default: "Zoom in" },
    resetTitle: { type: String, default: "Fit" },
  },
  setup(props) {
    const mappedBoxes = computed(() =>
      props.boxes.map((b) => ({ id: b.id, bounds: b.bounds, color: b.color })),
    );
    return () => (
      <HMinimap
        boxes={mappedBoxes.value}
        hubPos={props.hubPos}
        imageSrc={props.imageSrc}
        imageBounds={props.imageBounds}
        zoom={props.zoom}
        panX={props.panX}
        panY={props.panY}
        viewportWidth={props.viewportWidth}
        viewportHeight={props.viewportHeight}
        contentBounds={props.contentBounds}
        zoomPercent={props.zoomPercent}
        canZoomIn={props.canZoomIn}
        canZoomOut={props.canZoomOut}
        onZoomIn={props.onZoomIn}
        onZoomOut={props.onZoomOut}
        onReset={props.onReset}
        onPanDelta={props.onPanDelta}
        zoomOutTitle={props.zoomOutTitle}
        zoomInTitle={props.zoomInTitle}
        resetTitle={props.resetTitle}
      />
    );
  },
});
