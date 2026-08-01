import { defineComponent, onMounted, type PropType } from "vue";
import { HPopover, HSpinner, mergeMessages, useI18n } from "@celestia-island/hikari";
import type { PopupPlacement } from "@celestia-island/hikari";

import type { PVoicePopupMode } from "./PlanaChatTypes";
import "./PlanaVoiceInputPopup.scss";

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

/**
 * PVoiceInputPopup — shared, anchored voice-input popup.
 *
 * One component, three faces, driven by a `PVoiceState`:
 *
 *   - `notConfigured` — Whisper isn't set up; offers a deep link to the
 *     admin voice page so the user can install it.
 *   - `listening`     — an animated CSS waveform so the user sees capture
 *     is live. Tap the mic again (handled by the caller) to stop.
 *   - `transcribing`  — a quiet spinner while the final window is recognized.
 *
 * The popup owns no state — the caller feeds `open` / `mode` and emits
 * `close` / `openSettings`. Every voice button (chat input, expanded
 * composer, keyword search) renders this popup anchored to its mic button.
 */
export const PVoiceInputPopup = defineComponent({
  name: "PlanaVoiceInputPopup",
  props: {
    open: { type: Boolean, default: false },
    mode: {
      type: String as PropType<PVoicePopupMode>,
      default: "listening",
    },
    anchorRef: { type: Object as PropType<HTMLElement | null>, default: null },
    placement: {
      type: String as PropType<PopupPlacement>,
      default: "top",
    },
    offset: { type: Number, default: 8 },
  },
  emits: {
    close: () => true,
    openSettings: () => true,
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

    function openSettings() {
      emit("openSettings");
    }

    return () => (
      <HPopover
        modelValue={props.open}
        onUpdate:modelValue={(v: boolean) => { if (!v) emit("close"); }}
        placement={props.placement}
        offset={props.offset}
        anchorRef={props.anchorRef}
        closeOnBackdrop={false}
        closeOnEscape={true}
        class="s-voice-popup"
      >
        {props.mode === "notConfigured" ? (
          <div class="s-voice-popup-body" data-phase="install">
            <p class="s-voice-popup-text">
              {t("plana::chat.voice_not_configured", "Voice input requires the Whisper service.")}
            </p>
            <button class="s-voice-popup-link" type="button" onClick={openSettings}>
              {t("plana::chat.voice_go_settings", "Open Voice Settings →")}
            </button>
          </div>
        ) : (
          <div class="s-voice-popup-body" data-phase={props.mode}>
            {props.mode === "listening" ? (
              <div class="s-voice-wave" aria-hidden="true">
                <span class="s-voice-wave-bar" />
                <span class="s-voice-wave-bar" />
                <span class="s-voice-wave-bar" />
                <span class="s-voice-wave-bar" />
                <span class="s-voice-wave-bar" />
              </div>
            ) : (
              <span class="s-voice-popup-spinner">
                <HSpinner size="xs" />
              </span>
            )}
            <span class="s-voice-popup-hint">
              {props.mode === "listening"
                ? t("plana::chat.listening", "Listening… tap to stop")
                : t("plana::chat.transcribing", "Transcribing…")}
            </span>
          </div>
        )}
      </HPopover>
    );
  },
});
