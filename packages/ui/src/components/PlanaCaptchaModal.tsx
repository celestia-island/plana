import { defineComponent, type PropType } from "vue";
import { HModal, mergeMessages, useI18n } from "@celestia-island/hikari";

import { PCaptchaWidget, type PCaptchaProvider } from "./PlanaCaptchaWidget";
import "./PlanaCaptchaModal.scss";

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
 * PCaptchaModal — modal host for PCaptchaWidget.
 *
 * Opens on demand (at submit time), renders the provider widget, and emits
 * the verification token once the challenge succeeds. Closes itself when
 * the provider reports an error so the caller can surface its own message.
 */
export const PCaptchaModal = defineComponent({
  name: "PlanaCaptchaModal",
  props: {
    modelValue: { type: Boolean, default: false },
    siteKey: { type: String, required: true },
    provider: { type: String as () => PCaptchaProvider, default: "turnstile" },
    scriptUrl: { type: String, default: undefined },
    attempt: { type: Number, default: 0 },
    title: { type: String, default: undefined },
    width: { type: String, default: "30rem" },
  },
  emits: {
    "update:modelValue": (_v: boolean) => true,
    verify: (_token: string) => true,
    error: (_message: string) => true,
  },
  setup(props, { emit }) {
    const { t } = useI18n();

    return () => (
      <HModal
        modelValue={props.modelValue}
        onUpdate:modelValue={(v: boolean) => emit("update:modelValue", v)}
        title={props.title ?? t("plana::captcha.title")}
        width={props.width}
      >
        <div class="s-captcha-modal">
          <p class="s-captcha-modal-prompt">{t("plana::captcha.prompt")}</p>
          {props.modelValue && (
            <PCaptchaWidget
              siteKey={props.siteKey}
              provider={props.provider}
              scriptUrl={props.scriptUrl}
              attempt={props.attempt}
              onVerify={(token: string) => emit("verify", token)}
              onError={(message: string) => {
                emit("error", message);
                emit("update:modelValue", false);
              }}
            />
          )}
        </div>
      </HModal>
    );
  },
});
