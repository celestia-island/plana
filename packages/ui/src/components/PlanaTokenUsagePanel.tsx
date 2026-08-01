import { computed, defineComponent, onMounted, type PropType } from "vue";
import {
  HDrawer,
  HListTransition,
  mergeMessages,
  useI18n,
} from "@celestia-island/hikari";

import type { PModelCosts, PModelUsageEntry } from "./PlanaChatTypes";
import { formatTokenCount, formatPriceUsd } from "../utils/format";
import "./PlanaTokenUsagePanel.scss";

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

const modelBarColors = [
  "rgb(var(--color-primary))",
  "rgb(var(--color-success))",
  "rgb(var(--color-warning))",
  "rgb(var(--color-error))",
  "rgb(168 85 247)",
  "rgb(236 72 153)",
  "rgb(20 184 166)",
  "rgb(249 115 22)",
];

/**
 * PTokenUsagePanel — read-only token metering drawer.
 *
 * Fully data-driven: the parent passes per-model entries, the
 * prompt/completion/total breakdown and (optionally) estimated costs as
 * plain numbers. No provider coupling — costs are precomputed upstream.
 */
export const PTokenUsagePanel = defineComponent({
  name: "PlanaTokenUsagePanel",
  props: {
    modelValue: { type: Boolean, required: true },
    entries: {
      type: Array as PropType<PModelUsageEntry[]>,
      default: () => [],
    },
    promptTokens: { type: Number, default: 0 },
    completionTokens: { type: Number, default: 0 },
    /** Optional estimated costs in USD (per bucket). */
    costs: { type: Object as PropType<PModelCosts | null>, default: null },
    currency: { type: String, default: "$" },
  },
  emits: {
    "update:modelValue": (_v: boolean) => true,
  },
  setup(props, { emit }) {
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

    const totalTokens = computed(() => props.promptTokens + props.completionTokens);
    const totalCost = computed(() => {
      if (!props.costs) return null;
      return (props.costs.prompt ?? 0) + (props.costs.completion ?? 0) + (props.costs.cached ?? 0);
    });
    const maxTokens = computed(() => {
      if (!props.entries.length) return 1;
      return Math.max(...props.entries.map((m) => m.tokenCount), 1);
    });

    return () => (
      <HDrawer
        modelValue={props.modelValue}
        onUpdate:modelValue={(v: boolean) => emit("update:modelValue", v)}
        title={t("plana::tokenUsage.title", "Token Usage")}
        side="right"
        size="340px"
      >
        <div class="s-token-panel">
          {props.entries.length === 0 ? (
            <div class="s-token-panel-empty">
              {t("plana::tokenUsage.noData", "No token usage data available.")}
            </div>
          ) : (
            <>
              {/* Totals card */}
              <div class="s-token-panel-total">
                <div class="s-token-panel-total-row">
                  <span>{t("plana::tokenUsage.prompt", "Prompt")}</span>
                  <span class="s-token-panel-num">{formatTokenCount(props.promptTokens)}</span>
                </div>
                <div class="s-token-panel-total-row">
                  <span>{t("plana::tokenUsage.completion", "Completion")}</span>
                  <span class="s-token-panel-num">{formatTokenCount(props.completionTokens)}</span>
                </div>
                <div class="s-token-panel-total-row is-total">
                  <span>{t("plana::tokenUsage.total", "Total")}</span>
                  <span class="s-token-panel-num">{formatTokenCount(totalTokens.value)}</span>
                </div>
                {props.costs && (
                  <div class="s-token-panel-total-row is-cost">
                    <span>{t("plana::tokenUsage.estimatedCost", "Est. cost")}</span>
                    <span class="s-token-panel-num">{formatPriceUsd(totalCost.value ?? 0, props.currency)}</span>
                  </div>
                )}
              </div>

              {/* Per-model breakdown */}
              <div class="s-token-panel-models">
                <div class="s-token-panel-models-title">{t("plana::tokenUsage.byModel", "By model")}</div>
                <HListTransition tag="div" class="s-token-panel-model-list" variant="grow" move={false}>
                  {props.entries.map((m, i) => {
                    const pct = Math.round((m.tokenCount / maxTokens.value) * 100);
                    const color = modelBarColors[i % modelBarColors.length];
                    return (
                      <div key={m.model} class="s-token-panel-model">
                        <div class="s-token-panel-model-row">
                          <span class="s-token-panel-model-name" title={m.model}>{m.model}</span>
                          <span class="s-token-panel-model-count">{formatTokenCount(m.tokenCount)}</span>
                        </div>
                        <div class="s-token-panel-model-track">
                          <div class="s-token-panel-model-bar" style={{ width: `${pct}%`, background: color }} />
                        </div>
                      </div>
                    );
                  })}
                </HListTransition>
              </div>
            </>
          )}
        </div>
      </HDrawer>
    );
  },
});
