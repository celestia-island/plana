import { defineComponent, onMounted, ref, watch } from "vue";
import AlertTriangle from "lucide-vue-next/dist/esm/icons/triangle-alert";
import Check from "lucide-vue-next/dist/esm/icons/check";
import Copy from "lucide-vue-next/dist/esm/icons/copy";
import { HModal, mergeMessages, useClipboard, useI18n } from "@celestia-island/hikari";

import "./PlanaSecretRevealModal.scss";

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

/**
 * PSecretRevealModal — show-once secret / API-key reveal panel.
 *
 * Displays a one-time secret (API key, token, password) with a "shown
 * only once" notice and a copy button. `copied` fires on successful copy.
 * The value is a prop: the caller decides how to obtain/discard it (e.g.
 * clear it once the modal closes).
 */
export const PSecretRevealModal = defineComponent({
  name: "PlanaSecretRevealModal",
  props: {
    modelValue: { type: Boolean, default: false },
    /** Secret label (e.g. "API Key"). */
    label: { type: String, required: true },
    /** The secret value to reveal. */
    value: { type: String, required: true },
    /** Copy button label override. */
    copyLabel: { type: String, default: undefined },
    /** "Shown once" notice override. */
    notice: { type: String, default: undefined },
    title: { type: String, default: undefined },
  },
  emits: {
    "update:modelValue": (_v: boolean) => true,
    copied: () => true,
  },
  setup(props, { emit }) {
    const { t } = useI18n();
    const clipboard = useClipboard();
    const copied = ref(false);

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
      () => props.modelValue,
      (open) => {
        if (open) copied.value = false;
      },
    );

    async function handleCopy() {
      if (!props.value) return;
      const ok = await clipboard.copy(props.value);
      if (ok) {
        copied.value = true;
        emit("copied");
      }
    }

    return () => (
      <HModal
        modelValue={props.modelValue}
        onUpdate:modelValue={(v: boolean) => emit("update:modelValue", v)}
        title={props.title ?? `${props.label} — ${t("plana::secret.title")}`}
        width="34rem"
        footerActions={[
          {
            label: t("plana::secret.close"),
            variant: "secondary",
            onClick: () => emit("update:modelValue", false),
          },
        ]}
      >
        <div class="s-secret-modal">
          <div class="s-secret-modal-notice">
            <AlertTriangle size={14} />
            <span>{props.notice ?? t("plana::secret.notice")}</span>
          </div>
          <code class="s-secret-modal-value">{props.value}</code>
          <div class="s-secret-modal-actions">
            <button
              type="button"
              class="s-secret-modal-copy"
              data-copied={copied.value || undefined}
              onClick={() => void handleCopy()}
              disabled={!props.value}
            >
              {copied.value ? <Check size={14} /> : <Copy size={14} />}
              {copied.value ? t("plana::secret.copied") : props.copyLabel ?? t("plana::secret.copy")}
            </button>
          </div>
        </div>
      </HModal>
    );
  },
});
