import { computed, defineComponent, onMounted, ref, watch, type PropType } from "vue";
import Copy from "lucide-vue-next/dist/esm/icons/copy";
import Eraser from "lucide-vue-next/dist/esm/icons/eraser";
import Pause from "lucide-vue-next/dist/esm/icons/pause";
import Play from "lucide-vue-next/dist/esm/icons/play";
import ScrollText from "lucide-vue-next/dist/esm/icons/scroll-text";
import { HModal, HScrollContainer, mergeMessages, useClipboard, useI18n } from "@celestia-island/hikari";

import "./PlanaLogWindow.scss";

import enLocale from "../i18n/locales/en/platform.json";
import zhsLocale from "../i18n/locales/zh-Hans/platform.json";
import zhtLocale from "../i18n/locales/zh-Hant/platform.json";
import jaLocale from "../i18n/locales/ja/platform.json";
import koLocale from "../i18n/locales/ko/platform.json";
import ruLocale from "../i18n/locales/ru/platform.json";
import arLocale from "../i18n/locales/ar/platform.json";
import deLocale from "../i18n/locales/de/platform.json";
import esLocale from "../i18n/locales/es/platform.json";
import frLocale from "../i18n/locales/fr/platform.json";
import ptLocale from "../i18n/locales/pt/platform.json";

export interface PLogTab {
  key: string;
  title: string;
  lines: string[];
}

function levelOf(line: string): "error" | "warn" | "debug" | "info" {
  const upper = line.toUpperCase();
  if (upper.includes("ERROR")) return "error";
  if (upper.includes("WARN")) return "warn";
  if (upper.includes("DEBUG") || upper.includes("TRACE")) return "debug";
  return "info";
}

/**
 * PLogWindow — tabbed service log viewer.
 *
 * Displays caller-provided `tabs` (each an ordered list of log lines) with
 * pause, autoscroll, copy and clear controls. Line arrays are props: the
 * caller keeps appending; `clearTab(key)` tells the caller to reset that
 * tab's buffer. Pause is optional-controlled (`paused` + `update:paused`).
 * Autoscroll delegates to hikari's HScrollContainer `autoFollow` (pins to
 * bottom while the user stays near the end, pauses when paused).
 */
export const PLogWindow = defineComponent({
  name: "PlanaLogWindow",
  props: {
    modelValue: { type: Boolean, default: false },
    tabs: { type: Array as PropType<PLogTab[]>, required: true },
    /** Controlled pause state; when undefined the toggle is internal. */
    paused: { type: Boolean, default: false },
    /** Height of the log body (CSS length, e.g. "55vh"). */
    height: { type: String, default: "55vh" },
    title: { type: String, default: undefined },
    width: { type: String, default: "60rem" },
  },
  emits: {
    "update:modelValue": (_v: boolean) => true,
    "update:paused": (_v: boolean) => true,
    clearTab: (_key: string) => true,
  },
  setup(props, { emit }) {
    const { t } = useI18n();
    const clipboard = useClipboard();

    const activeTab = ref<string>("");
    const autoscroll = ref(true);
    const paused = ref(props.paused);

    onMounted(() => {
      mergeMessages(enLocale.platform, "en");
      mergeMessages(zhsLocale.platform, "zh-Hans");
      mergeMessages(zhtLocale.platform, "zh-Hant");
      mergeMessages(jaLocale.platform, "ja");
      mergeMessages(koLocale.platform, "ko");
      mergeMessages(ruLocale.platform, "ru");
      mergeMessages(arLocale.platform, "ar");
      mergeMessages(deLocale.platform, "de");
      mergeMessages(esLocale.platform, "es");
      mergeMessages(frLocale.platform, "fr");
      mergeMessages(ptLocale.platform, "pt");
    });

    watch(
      () => props.paused,
      (v) => { paused.value = v; },
    );

    watch(
      () => props.tabs,
      (tabs) => {
        if (tabs.length === 0) {
          activeTab.value = "";
        } else if (!tabs.some((tab) => tab.key === activeTab.value)) {
          activeTab.value = tabs[0].key;
        }
      },
      { immediate: true },
    );

    const currentTab = computed(() => props.tabs.find((tab) => tab.key === activeTab.value) ?? null);
    const currentLines = computed(() => currentTab.value?.lines ?? []);

    function togglePause() {
      paused.value = !paused.value;
      emit("update:paused", paused.value);
    }

    function handleClear() {
      if (!currentTab.value) return;
      emit("clearTab", currentTab.value.key);
    }

    function handleCopy() {
      if (currentLines.value.length === 0) return;
      void clipboard.copy(currentLines.value.join("\n"));
    }

    const copied = computed(() => clipboard.copied.value);

    return () => (
      <HModal
        modelValue={props.modelValue}
        onUpdate:modelValue={(v: boolean) => emit("update:modelValue", v)}
        title={props.title ?? t("plana::log.title")}
        width={props.width}
      >
        <div class="s-log-viewer" style={{ height: props.height }}>
          <div class="s-log-toolbar">
            <div class="s-log-tabs">
              {props.tabs.map((tab) => (
                <button
                  key={tab.key}
                  type="button"
                  class="s-log-tab"
                  data-active={tab.key === activeTab.value || undefined}
                  onClick={() => { activeTab.value = tab.key; }}
                >
                  {tab.title}
                </button>
              ))}
            </div>
            <div class="s-log-controls">
              <button
                type="button"
                class="s-log-btn"
                data-active={paused.value || undefined}
                onClick={togglePause}
                title={paused.value ? t("plana::log.resume") : t("plana::log.pause")}
              >
                {paused.value ? <Play size={12} /> : <Pause size={12} />}
              </button>
              <button
                type="button"
                class="s-log-btn"
                data-active={autoscroll.value || undefined}
                onClick={() => { autoscroll.value = !autoscroll.value; }}
                title={t("plana::log.autoscroll")}
              >
                <ScrollText size={12} />
              </button>
              <button
                type="button"
                class="s-log-btn"
                disabled={currentLines.value.length === 0}
                onClick={handleCopy}
                title={copied.value ? t("plana::log.copied") : t("plana::log.copy")}
              >
                <Copy size={12} />
              </button>
              <button
                type="button"
                class="s-log-btn"
                disabled={currentLines.value.length === 0}
                onClick={handleClear}
                title={t("plana::log.clear")}
              >
                <Eraser size={12} />
              </button>
            </div>
          </div>
          <HScrollContainer class="s-log-viewer-scroll" autoFollow={autoscroll.value && !paused.value}>
            {currentLines.value.length === 0 ? (
              <div class="s-log-empty">{t("plana::log.empty")}</div>
            ) : (
              currentLines.value.map((line, i) => (
                <div key={i} class={["s-log-entry", `s-log-entry-${levelOf(line)}`]}>
                  {line}
                </div>
              ))
            )}
          </HScrollContainer>
        </div>
      </HModal>
    );
  },
});
