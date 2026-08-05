import { computed, defineComponent, onBeforeUnmount, onMounted, ref, watch, type PropType } from "vue";
import { ShieldAlert } from "lucide-vue-next";
import { mergeMessages, useI18n } from "@celestia-island/hikari";

import "./PlanaCaptchaWidget.scss";

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
 * PCaptchaWidget — provider-agnostic silent-captcha widget.
 *
 * Renders a third-party captcha widget (Cloudflare Turnstile or Google
 * reCAPTCHA protocol) and emits the verification token once the challenge
 * succeeds. The provider CDN script is loaded lazily, only once per URL,
 * and never bundled locally.
 *
 * Provider-agnostic contract:
 *  - `provider: "turnstile" | "recaptcha"` selects the render protocol.
 *  - `scriptUrl` overrides the official CDN URL (defaults per provider),
 *    so any platform compatible with one of those protocols — including a
 *    self-hosted one — works by pointing `scriptUrl` at it.
 *  - No-op placeholder mode: when `siteKey` is empty (or `disabled` is
 *    true) the widget renders a placeholder box and emits nothing. This
 *    lets deployments ship without a captcha provider; wire a real
 *    `siteKey`/`scriptUrl` per deployment to enable verification.
 *
 * Tokens are single-use; bump `attempt` (e.g. after a failed submit) to
 * force a reset so a fresh token is produced.
 */
export type PCaptchaProvider = "turnstile" | "recaptcha";

const DEFAULT_SCRIPT_URLS: Record<PCaptchaProvider, string> = {
  turnstile: "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit",
  recaptcha: "https://www.google.com/recaptcha/api.js?render=explicit",
};

const SCRIPT_LOADERS: Record<string, Promise<unknown> | undefined> = {};

function loadScript(src: string): Promise<unknown> {
  if (SCRIPT_LOADERS[src]) return SCRIPT_LOADERS[src];
  SCRIPT_LOADERS[src] = new Promise((resolve, reject) => {
    const el = document.createElement("script");
    el.src = src;
    el.async = true;
    el.defer = true;
    el.onload = () => resolve(undefined);
    el.onerror = () => {
      delete SCRIPT_LOADERS[src];
      reject(new Error(`failed to load ${src}`));
    };
    document.head.appendChild(el);
  });
  return SCRIPT_LOADERS[src];
}

type TurnstileApi = {
  render: (el: HTMLElement, opts: Record<string, unknown>) => string;
  reset: (id?: string) => void;
  remove: (id: string) => void;
};
type GrecaptchaApi = {
  ready: (cb: () => void) => void;
  render: (el: HTMLElement, opts: Record<string, unknown>) => number;
  reset: (id?: number) => void;
};

function getGlobal(name: string): unknown {
  return (window as unknown as Record<string, unknown>)[name];
}

export const PCaptchaWidget = defineComponent({
  name: "PlanaCaptchaWidget",
  props: {
    /** Site key issued by the provider (empty = placeholder/no-op mode). */
    siteKey: { type: String, required: true },
    /** Provider protocol: "turnstile" (default) or "recaptcha". */
    provider: { type: String as () => PCaptchaProvider, default: "turnstile" },
    /** CDN script URL override (defaults to the official provider URL). */
    scriptUrl: { type: String, default: undefined },
    /** Bump to force a token refresh (e.g. after a failed submit). */
    attempt: { type: Number, default: 0 },
    /** Disables rendering and enters no-op placeholder mode. */
    disabled: { type: Boolean, default: false },
  },
  emits: {
    verify: (_token: string) => true,
    error: (_message: string) => true,
  },
  setup(props, { emit }) {
    const { t } = useI18n();
    const container = ref<HTMLElement>();
    const loading = ref(false);
    let widgetId: string | number | null = null;

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

    const placeholder = computed(() => !props.siteKey || props.disabled);

    function reset() {
      if (widgetId == null) return;
      if (props.provider === "turnstile") {
        (getGlobal("turnstile") as TurnstileApi | undefined)?.reset(widgetId as string);
      } else {
        (getGlobal("grecaptcha") as GrecaptchaApi | undefined)?.reset(widgetId as number);
      }
    }

    async function render() {
      if (!container.value || placeholder.value) return;
      const onToken = (token: string) => emit("verify", token);

      loading.value = true;
      try {
        const src = props.scriptUrl || DEFAULT_SCRIPT_URLS[props.provider];
        if (props.provider === "turnstile") {
          await loadScript(src);
          const api = getGlobal("turnstile") as TurnstileApi | undefined;
          if (!api) throw new Error("turnstile not available");
          widgetId = api.render(container.value, {
            sitekey: props.siteKey,
            callback: onToken,
            "error-callback": () => emit("error", "turnstile error"),
          });
        } else {
          await loadScript(src);
          const api = getGlobal("grecaptcha") as GrecaptchaApi | undefined;
          if (!api) throw new Error("grecaptcha not available");
          await new Promise<void>((resolve) => api.ready(() => resolve()));
          widgetId = api.render(container.value, {
            sitekey: props.siteKey,
            callback: onToken,
          });
        }
      } catch (e) {
        emit("error", e instanceof Error ? e.message : String(e));
      } finally {
        loading.value = false;
      }
    }

    onMounted(render);
    watch(
      () => props.attempt,
      () => {
        if (widgetId != null) reset();
      },
    );
    onBeforeUnmount(() => {
      if (widgetId == null) return;
      if (props.provider === "turnstile") {
        (getGlobal("turnstile") as TurnstileApi | undefined)?.remove(widgetId as string);
      }
      widgetId = null;
    });

    return () => {
      if (placeholder.value) {
        return (
          <div class="s-captcha-widget s-captcha-widget-placeholder">
            <ShieldAlert size={16} />
            <span>{t("plana::captcha.placeholder")}</span>
          </div>
        );
      }
      return (
        <div class="s-captcha-widget">
          {loading.value && <p class="s-captcha-widget-loading">{t("plana::captcha.loading")}</p>}
          <div ref={container} class="s-captcha-widget-host" />
        </div>
      );
    };
  },
});
