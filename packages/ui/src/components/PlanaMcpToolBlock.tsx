import { computed, defineComponent, onMounted, ref, watch, type PropType } from "vue";
import { ChevronDown, ChevronRight } from "lucide-vue-next";
import { HDivider, mergeMessages, useI18n } from "@celestia-island/hikari";

import type { PMcpToolCallStatus } from "./PlanaChatTypes";
import { formatTokenCount } from "../utils/format";
import "./PlanaMcpToolBlock.scss";

import enLocale from "../i18n/locales/en/chat.json";
import zhsLocale from "../i18n/locales/zh-Hans/chat.json";
import zhtLocale from "../i18n/locales/zh-Hant/chat.json";
import jaLocale from "../i18n/locales/ja/chat.json";
import koLocale from "../i18n/locales/ko/chat.json";
import ruLocale from "../i18n/locales/ru/chat.json";
import arLocale from "../i18n/locales/ar/chat.json";
import deLocale from "../i18n/locales/de/chat.json";
import esLocale from "../i18n/locales/es/chat.json";
import frLocale from "../i18n/locales/fr/chat.json";
import ptLocale from "../i18n/locales/pt/chat.json";

export interface PParsedMcpCall {
  toolName: string;
  argsJson: string;
  argsObj: Record<string, unknown> | null;
}

/**
 * Parse a chest-style tool call text of the form `"toolName", {...args}`
 * into its parts. Returns null when the text does not match that shape.
 */
export function parseMcpCallText(callText: string): PParsedMcpCall | null {
  const m = callText.match(/^"(\w+)"\s*,\s*(\{[\s\S]*\})\s*$/);
  if (!m) return null;
  try {
    const v = JSON.parse(m[2]);
    const argsObj = typeof v === "object" && v !== null ? v as Record<string, unknown> : null;
    return { toolName: m[1], argsJson: m[2], argsObj };
  } catch {
    return { toolName: m[1], argsJson: m[2], argsObj: null };
  }
}

/**
 * PMcpToolBlock — collapsible tool call / result block.
 *
 * Renders a tool call header (title + status badge), the call arguments
 * and the result as monospace blocks, plus an estimated token/duration
 * footer. Pure presentation — no highlighting or JSON tree (chest's
 * exec/writeToVar variants and the interactive JSON viewer are deferred).
 */
export const PMcpToolBlock = defineComponent({
  name: "PlanaMcpToolBlock",
  props: {
    toolName: { type: String, required: true },
    agentType: { type: String, default: "" },
    status: { type: String as PropType<PMcpToolCallStatus>, required: true },
    callText: { type: String, default: "" },
    resultText: { type: String, default: "" },
    durationMs: { type: Number, default: undefined },
    defaultExpanded: { type: Boolean, default: true },
    collapsible: { type: Boolean, default: true },
  },
  setup(props) {
    onMounted(() => {
      mergeMessages(enLocale.chat, "en");
      mergeMessages(zhsLocale.chat, "zh-Hans");
      mergeMessages(zhtLocale.chat, "zh-Hant");
      mergeMessages(jaLocale.chat, "ja");
      mergeMessages(koLocale.chat, "ko");
      mergeMessages(ruLocale.chat, "ru");
      mergeMessages(arLocale.chat, "ar");
      mergeMessages(deLocale.chat, "de");
      mergeMessages(esLocale.chat, "es");
      mergeMessages(frLocale.chat, "fr");
      mergeMessages(ptLocale.chat, "pt");
    });

    const { t } = useI18n();
    const expanded = ref(props.defaultExpanded);

    watch(() => props.status, (newStatus) => {
      if (newStatus === "done") expanded.value = true;
    });

    const displayTitle = computed(() =>
      props.agentType ? `${props.agentType} :: ${props.toolName}` : props.toolName,
    );

    const statusLabel = computed(() => {
      switch (props.status) {
        case "pending": return t("plana::mcp.pending", "Pending");
        case "running": return t("plana::mcp.running", "Running");
        case "done": return t("plana::mcp.done", "Done");
        case "error": return t("plana::mcp.error", "Error");
      }
    });

    const callTokens = computed(() => Math.ceil((props.callText?.length ?? 0) / 4));
    const resultTokens = computed(() => Math.ceil((props.resultText?.length ?? 0) / 4));

    const blockClass = computed(() => [
      "s-mcp-block",
      props.status === "error" ? "is-error" : "",
      props.status === "running" ? "is-running" : "",
      props.status === "done" ? "is-success" : "",
    ].filter(Boolean).join(" "));

    function toggleExpand() {
      if (!props.collapsible) return;
      expanded.value = !expanded.value;
    }

    return () => (
      <div class={blockClass.value}>
        <div class="s-mcp-header" onClick={toggleExpand}>
          <span class="s-mcp-header-title">{displayTitle.value}</span>
          <span class={`s-mcp-header-badge is-${props.status}`}>{statusLabel.value}</span>
          {props.collapsible && (
            <span class="s-mcp-header-expand">
              {expanded.value ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
            </span>
          )}
        </div>

        {expanded.value && (
          <div class="s-mcp-body">
            {props.status === "pending" && (
              <div class="s-mcp-params">
                <div class="s-mcp-param-line is-muted-italic">{t("plana::mcp.waitingArgs", "Waiting for arguments…")}</div>
              </div>
            )}

            {props.status === "running" && !props.callText && (
              <div class="s-mcp-params">
                <div class="s-mcp-param-line is-muted-italic" style={{ color: "rgb(var(--color-primary))" }}>
                  {t("plana::mcp.executing", "Executing…")}
                </div>
              </div>
            )}

            {props.callText && (
              <div class="s-mcp-call">
                <pre class="s-mcp-code" data-role="call">{props.callText}</pre>
              </div>
            )}

            {props.callText && props.resultText && (
              <HDivider variant="dashed" tone="faint" spacing="sm" />
            )}

            {props.resultText && (
              <div class={`s-mcp-result ${props.status === "error" ? "is-error" : ""}`}>
                <pre class="s-mcp-code" data-role="result">{props.resultText}</pre>
              </div>
            )}
          </div>
        )}

        {expanded.value && (props.durationMs != null || callTokens.value > 0 || resultTokens.value > 0) && (
          <div class="s-mcp-footer">
            {callTokens.value > 0 && (
              <span class="s-mcp-stat">
                <span class="s-mcp-stat-arrow is-in">↑</span>
                <span class="s-mcp-stat-value">{formatTokenCount(callTokens.value)}</span>
              </span>
            )}
            {resultTokens.value > 0 && (
              <span class="s-mcp-stat">
                <span class="s-mcp-stat-arrow is-out">↓</span>
                <span class="s-mcp-stat-value">{formatTokenCount(resultTokens.value)}</span>
              </span>
            )}
            {props.durationMs != null && (
              <span class="s-mcp-stat">
                <span class="s-mcp-stat-label">{props.durationMs}ms</span>
              </span>
            )}
          </div>
        )}
      </div>
    );
  },
});
