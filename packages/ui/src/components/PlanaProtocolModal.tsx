import { computed, defineComponent } from "vue";
import { HMarkdownRenderer, HModal, useClipboard, useI18n, mergeMessages, type ModalAction } from "@celestia-island/hikari";

import "./PlanaProtocolModal.scss";

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
 * PProtocolModal — markdown EULA / privacy / terms modal.
 *
 * Renders arbitrary markdown `content` (via hikari HMarkdownRenderer) with
 * a Decline/Accept footer. `accept`/`decline` are the caller's commit
 * actions; closing (overlay click / ESC / X) only emits `update:modelValue`
 * so the caller can decide whether close === decline.
 */
export const PProtocolModal = defineComponent({
  name: "PlanaProtocolModal",
  props: {
    modelValue: { type: Boolean, default: false },
    /** Modal title (defaults to "Agreement"). */
    title: { type: String, default: undefined },
    /** Markdown content to render (or plain text when `plain`). */
    content: { type: String, default: "" },
    /** Render `content` as escaped plain text instead of markdown. */
    plain: { type: Boolean, default: false },
    /** Accept button label override. */
    acceptLabel: { type: String, default: undefined },
    /** Decline button label override. */
    declineLabel: { type: String, default: undefined },
    /** Allow dismissing without a decision (overlay/ESC/X). Default true. */
    closable: { type: Boolean, default: true },
    width: { type: String, default: "48rem" },
    /** Cap the scroll body height (e.g. "60vh"). */
    bodyHeight: { type: String, default: undefined },
  },
  emits: {
    "update:modelValue": (_v: boolean) => true,
    accept: () => true,
    decline: () => true,
  },
  setup(props, { emit }) {
    const { t } = useI18n();
    const clipboard = useClipboard();
    const copied = computed(() => clipboard.copied.value);

    const footerActions = computed<ModalAction[]>(() => [
      {
        label: t("plana::protocol.copy"),
        variant: "secondary" as const,
        onClick: () => void clipboard.copy(props.content),
        disabled: !props.content,
      },
      {
        label: props.declineLabel ?? t("plana::protocol.decline"),
        variant: "secondary" as const,
        onClick: () => emit("decline"),
      },
      {
        label: props.acceptLabel ?? t("plana::protocol.accept"),
        variant: "primary" as const,
        onClick: () => emit("accept"),
        disabled: !props.content,
      },
    ]);

    return () => (
      <HModal
        modelValue={props.modelValue}
        onUpdate:modelValue={(v: boolean) => emit("update:modelValue", v)}
        title={props.title ?? t("plana::protocol.title")}
        width={props.width}
        closable={props.closable}
        footerActions={footerActions.value}
      >
        <div
          class="s-protocol-modal"
          style={props.bodyHeight ? { maxHeight: props.bodyHeight, overflowY: "auto" } : undefined}
        >
          {copied.value && <p class="s-protocol-modal-copied">{t("plana::protocol.copied")}</p>}
          <HMarkdownRenderer content={props.content} plain={props.plain} />
        </div>
      </HModal>
    );
  },
});
