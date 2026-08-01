import { computed, defineComponent, onMounted, type PropType } from "vue";
import { HButton, HProgressBar, HSpinner, mergeMessages, useI18n } from "@celestia-island/hikari";

import type { PModelDownloadStatus } from "./PlanaChatTypes";
import { formatBytes } from "../utils/format";
import "./PlanaModelDownloadCard.scss";

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

type BarStatus = "loading" | "done" | "error";

const STATUS_BAR: Record<PModelDownloadStatus, BarStatus> = {
  pending: "loading",
  downloading: "loading",
  done: "done",
  error: "error",
};

const STATUS_ICON: Record<PModelDownloadStatus, boolean> = {
  pending: true,
  downloading: true,
  done: false,
  error: false,
};

/**
 * PModelDownloadCard — single-model download progress card.
 *
 * Shows the model name, formatted size, a progress bar (indeterminate
 * when `progress` is null) and a status label. Error state surfaces
 * `retry` / `details` actions. Pure presentation — progress values are
 * fed by the parent's downloader.
 */
export const PModelDownloadCard = defineComponent({
  name: "PlanaModelDownloadCard",
  props: {
    name: { type: String, required: true },
    sizeBytes: { type: Number, default: 0 },
    status: { type: String as PropType<PModelDownloadStatus>, default: "downloading" },
    /** 0-100; null renders an indeterminate bar. */
    progress: { type: Number as PropType<number | null>, default: null },
    /** Optional error detail shown under the bar. */
    error: { type: String, default: "" },
  },
  emits: {
    retry: () => true,
    details: () => true,
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

    const statusLabel = computed(() => {
      switch (props.status) {
        case "pending": return t("plana::download.pending", "Queued");
        case "downloading": return t("plana::download.downloading", "Downloading");
        case "done": return t("plana::download.done", "Downloaded");
        case "error": return t("plana::download.error", "Failed");
      }
    });

    return () => (
      <div class="s-download-card" data-status={props.status}>
        <div class="s-download-header">
          {STATUS_ICON[props.status] && (
            <span class="s-download-spinner">
              <HSpinner size={14} />
            </span>
          )}
          <span class="s-download-name" title={props.name}>{props.name}</span>
          {props.sizeBytes > 0 && (
            <span class="s-download-size">{formatBytes(props.sizeBytes)}</span>
          )}
          <span class="s-download-status" data-status={props.status}>{statusLabel.value}</span>
        </div>

        <HProgressBar
          size="sm"
          status={STATUS_BAR[props.status]}
          value={props.status === "done" ? 100 : (props.progress ?? undefined)}
          max={100}
        />

        {props.status === "error" && (
          <div class="s-download-actions">
            {props.error && <span class="s-download-error" title={props.error}>{props.error}</span>}
            <HButton size="sm" variant="ghost" onClick={() => emit("details")}>
              {t("plana::download.details", "Details")}
            </HButton>
            <HButton size="sm" variant="danger" onClick={() => emit("retry")}>
              {t("plana::download.retry", "Retry")}
            </HButton>
          </div>
        )}
      </div>
    );
  },
});
