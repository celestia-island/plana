import { computed, defineComponent, onMounted, type PropType } from "vue";
import { Bot, Copy, User } from "lucide-vue-next";
import { HMarkdownRenderer, mergeMessages, useI18n } from "@celestia-island/hikari";

import type { PChatRole, PMcpToolCall } from "./PlanaChatTypes";
import { PMcpToolBlock } from "./PlanaMcpToolBlock";
import "./PlanaChatMessage.scss";

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

function copyText(text: string): Promise<void> {
  if (navigator.clipboard?.writeText) return navigator.clipboard.writeText(text);
  return Promise.reject(new Error("clipboard unavailable"));
}

/**
 * PChatMessage — chat message bubble kit.
 *
 * Renders a single chat message: role-based alignment (user right /
 * assistant left), markdown content via HMarkdownRenderer, an optional
 * streaming cursor, collapsible tool-call blocks, and a copy action.
 *
 * ## Slots
 *
 * - `avatar`  — replaces the default role icon.
 * - `actions` — extra actions rendered next to the copy button.
 * - `footer`  — content below the bubble (token badges, timestamps, …).
 *
 * The component never mutates parent state; `copy` is emitted so the
 * parent can show its own toast.
 */
export const PChatMessage = defineComponent({
  name: "PlanaChatMessage",
  props: {
    role: { type: String as PropType<PChatRole>, required: true },
    content: { type: String, required: true },
    /** Show a blinking cursor after the content (streaming in flight). */
    streaming: { type: Boolean, default: false },
    /** Tint the bubble as an error message. */
    error: { type: Boolean, default: false },
    /** Render content as plain text instead of markdown. */
    plain: { type: Boolean, default: false },
    name: { type: String, default: undefined },
    timestamp: { type: [String, Number] as PropType<string | number | undefined>, default: undefined },
    /** Tool call blocks rendered above the content. */
    tools: {
      type: Array as PropType<PMcpToolCall[]>,
      default: () => [],
    },
    /** Hide the copy action. */
    hideCopy: { type: Boolean, default: false },
  },
  emits: {
    copy: (_text: string) => true,
  },
  setup(props, { emit, slots }) {
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

    const displayName = computed(() =>
      props.name ?? (props.role === "assistant"
        ? t("plana::chat.assistant", "Assistant")
        : t("plana::chat.you", "You")),
    );

    function onCopy() {
      void copyText(props.content).then(() => emit("copy", props.content)).catch(() => {});
    }

    return () => (
      <div
        class="s-chat-message"
        data-role={props.role}
        data-error={props.error || undefined}
      >
        <div class="s-chat-message-avatar">
          {slots.avatar ? (
            slots.avatar({ role: props.role })
          ) : props.role === "assistant" ? (
            <Bot size={14} />
          ) : (
            <User size={14} />
          )}
        </div>

        <div class="s-chat-message-main">
          <div class="s-chat-message-meta">
            <span class="s-chat-message-name">{displayName.value}</span>
            {props.timestamp != null && (
              <time class="s-chat-message-time">{String(props.timestamp)}</time>
            )}
            {(!props.hideCopy && props.content) || slots.actions ? (
              <span class="s-chat-message-actions">
                {!props.hideCopy && props.content && (
                  <button
                    type="button"
                    class="s-chat-message-copy"
                    aria-label={t("plana::chat.copy", "Copy")}
                    title={t("plana::chat.copy", "Copy")}
                    onClick={onCopy}
                  >
                    <Copy size={12} />
                  </button>
                )}
                {slots.actions?.()}
              </span>
            ) : null}
          </div>

          {props.tools.length > 0 && (
            <div class="s-chat-message-tools">
              {props.tools.map((tool, i) => (
                <PMcpToolBlock
                  key={tool.id ?? `${tool.toolName}-${i}`}
                  toolName={tool.toolName}
                  agentType={tool.agentType}
                  status={tool.status}
                  callText={tool.callText}
                  resultText={tool.resultText}
                  durationMs={tool.durationMs}
                  defaultExpanded={tool.defaultExpanded}
                />
              ))}
            </div>
          )}

          {props.content && (
            <div class="s-chat-message-bubble" data-streaming={props.streaming || undefined}>
              <HMarkdownRenderer content={props.content} plain={props.plain} />
              {props.streaming && <span class="s-chat-message-cursor" aria-hidden="true" />}
            </div>
          )}

          {slots.footer?.()}
        </div>
      </div>
    );
  },
});
