import { defineComponent, type PropType } from "vue";
import { ArrowDown, ArrowUp } from "lucide-vue-next";
import { HRollingNumber } from "@celestia-island/hikari";

import { formatTokenCount } from "../utils/format";
import "./PlanaTokenUsageBadge.scss";

/**
 * PTokenUsageBadge — compact token metering pill.
 *
 * Shows input (↑) and output (↓) token counts, optionally mcpCount and
 * duration. Pure formatting — numbers are passed in by the parent.
 */
export const PTokenUsageBadge = defineComponent({
  name: "PlanaTokenUsageBadge",
  props: {
    input: { type: Number, default: 0 },
    output: { type: Number, default: 0 },
    mcpCount: { type: Number as PropType<number | undefined>, default: undefined },
    durationSec: { type: Number as PropType<number | undefined>, default: undefined },
    compact: { type: Boolean, default: false },
    fixed: { type: Boolean, default: false },
  },
  setup(props) {
    return () => (
      <div
        class="s-token-usage"
        data-compact={props.compact || undefined}
        data-fixed={props.fixed || undefined}
      >
        <span class="s-token-usage-stat">
          <ArrowUp size={props.compact ? 9 : 10} class="s-token-usage-arrow" data-direction="in" />
          <span class="s-token-usage-val">{formatTokenCount(props.input)}</span>
        </span>
        <span class="s-token-usage-stat">
          <ArrowDown size={props.compact ? 9 : 10} class="s-token-usage-arrow" data-direction="out" />
          <span class="s-token-usage-val"><HRollingNumber value={formatTokenCount(props.output)} /></span>
        </span>
        {props.mcpCount != null && (
          <span class="s-token-usage-stat">
            <span class="s-token-usage-icon-mcp">⇄</span>
            <span class="s-token-usage-val"><HRollingNumber value={props.mcpCount} /></span>
          </span>
        )}
        {props.durationSec != null && (
          <span class="s-token-usage-stat">
            <span class="s-token-usage-duration">{props.durationSec}s</span>
          </span>
        )}
      </div>
    );
  },
});
