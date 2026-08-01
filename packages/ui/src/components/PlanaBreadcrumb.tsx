import { computed, defineComponent, type PropType } from "vue";
import { HBreadcrumb } from "@celestia-island/hikari";

import "./PlanaBreadcrumb.scss";

export interface PBreadcrumbItem {
  label: string;
  /** Link target; omitted (or `active`) renders a static current item. */
  to?: string;
  /** Marks the current location (rendered as static text). */
  active?: boolean;
}

export interface PBreadcrumbBadge {
  id: string;
  text: string;
  variant?: "success" | "warning" | "danger" | "muted";
  onClick?: () => void;
}

export interface PBreadcrumbParamChip {
  id: string;
  label: string;
  value: string;
  onClick?: () => void;
}

const BADGE_COLORS: Record<NonNullable<PBreadcrumbBadge["variant"]>, string> = {
  success: "rgb(var(--color-success))",
  warning: "rgb(var(--color-warning))",
  danger: "rgb(var(--color-error))",
  muted: "rgb(var(--color-muted))",
};

/**
 * PBreadcrumb — breadcrumb nav composed over hikari's HBreadcrumb.
 *
 * Renders the path via hikari `HBreadcrumb` (kept primitive) and extends
 * it with chest's badge / parameter-chip slots rendered by plana itself:
 *  - `badges`: status pills (e.g. service state), `onClick`-able.
 *  - `params`: `label:value` chips for the current context (e.g. selected
 *    item params), `onClick`-able.
 *
 * Known hikari limitation (follow-up in hikari, not this PR): HBreadcrumb
 * items only support `{label, to}` href links — there is no item-level
 * `onClick`/`disabled`/`active` support, so clickable segments must pass
 * `to`. The wrapper strips `to` from `active` items so the current
 * location renders as static text.
 */
export const PBreadcrumb = defineComponent({
  name: "PlanaBreadcrumb",
  props: {
    /** Breadcrumb path segments (leaf usually marked `active`). */
    items: { type: Array as PropType<PBreadcrumbItem[]>, required: true },
    /** Status badges shown on the right. */
    badges: { type: Array as PropType<PBreadcrumbBadge[]>, default: () => [] },
    /** Parameter chips shown on the right (after badges). */
    params: { type: Array as PropType<PBreadcrumbParamChip[]>, default: () => [] },
    /** Custom separator (defaults to hikari's chevron). */
    separator: { type: String, default: undefined },
    /** hikari HBreadcrumb size. */
    size: { type: String as PropType<"sm" | "md" | "lg">, default: "md" },
  },
  emits: {
    badgeClick: (_badge: PBreadcrumbBadge) => true,
    paramClick: (_param: PBreadcrumbParamChip) => true,
  },
  setup(props, { emit }) {
    const mappedItems = computed(() =>
      props.items.map((item) => ({
        label: item.label,
        to: item.active ? undefined : item.to,
      })),
    );

    function onBadgeClick(b: PBreadcrumbBadge) {
      if (b.onClick) b.onClick();
      emit("badgeClick", b);
    }

    function onParamClick(p: PBreadcrumbParamChip) {
      if (p.onClick) p.onClick();
      emit("paramClick", p);
    }

    return () => (
      <div class="s-breadcrumb">
        <div class="s-breadcrumb-nav">
          <HBreadcrumb items={mappedItems.value} separator={props.separator} size={props.size} />
        </div>
        {(props.badges.length > 0 || props.params.length > 0) && (
          <div class="s-breadcrumb-chips">
            {props.badges.map((b) => (
              <button
                key={b.id}
                type="button"
                class="s-breadcrumb-badge"
                data-actionable={!!b.onClick || undefined}
                style={{ color: BADGE_COLORS[b.variant ?? "muted"] }}
                onClick={() => onBadgeClick(b)}
                title={b.onClick ? b.text : undefined}
              >
                {b.text}
              </button>
            ))}
            {props.params.map((p) => (
              <button
                key={p.id}
                type="button"
                class="s-breadcrumb-param"
                data-actionable={!!p.onClick || undefined}
                onClick={() => onParamClick(p)}
                title={p.onClick ? `${p.label}: ${p.value}` : undefined}
              >
                <span class="s-breadcrumb-param-label">{p.label}</span>
                <span class="s-breadcrumb-param-value">{p.value}</span>
              </button>
            ))}
          </div>
        )}
      </div>
    );
  },
});
