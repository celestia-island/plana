import { computed, defineComponent, onMounted, ref, watch, type PropType } from "vue";
import { Film, File as FileIcon, ImageIcon, Music } from "lucide-vue-next";
import {
  HBadge,
  HMarkdownRenderer,
  HModal,
  HScrollContainer,
  mergeMessages,
  useI18n,
} from "@celestia-island/hikari";

import type { PAttachmentDetail } from "./PlanaChatTypes";
import { formatBytes } from "../utils/format";
import "./PlanaAttachmentModal.scss";

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

const CODE_LANGS: Record<string, string> = {
  ".ts": "typescript", ".tsx": "typescript", ".js": "javascript",
  ".jsx": "javascript", ".json": "json", ".rs": "rust", ".go": "go",
  ".py": "python", ".java": "java", ".c": "c", ".cpp": "cpp", ".h": "c",
  ".cs": "csharp", ".sh": "bash", ".bash": "bash", ".yaml": "yaml",
  ".yml": "yaml", ".toml": "toml", ".html": "html", ".css": "css",
  ".scss": "scss", ".sql": "sql", ".xml": "xml", ".ini": "ini",
};

function codeLanguage(name: string): string | undefined {
  const idx = name.lastIndexOf(".");
  if (idx < 0) return undefined;
  return CODE_LANGS[name.slice(idx).toLowerCase()];
}

const TEXT_EXTS = new Set([".md", ".markdown", ".txt", ".log", ".csv"]);
const CODE_EXTS = new Set(Object.keys(CODE_LANGS));

/**
 * PAttachmentModal — generic file-picker preview modal.
 *
 * Renders an attachment by MIME type: image (img), video/audio (native
 * controls), text/code (HMarkdownRenderer, with syntax fences for known
 * code extensions) or a generic file chip. Text content is read via the
 * attachment's `file` handle when provided; URL-only attachments skip the
 * text preview (deferred — the chest version routes URL fetches through
 * its authed transport, which is app-specific).
 */
export const PAttachmentModal = defineComponent({
  name: "PlanaAttachmentModal",
  props: {
    modelValue: { type: Boolean, default: false },
    attachment: { type: Object as PropType<PAttachmentDetail | null>, default: null },
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

    const isImage = computed(() => props.attachment?.type.startsWith("image/") ?? false);
    const isVideo = computed(() => props.attachment?.type.startsWith("video/") ?? false);
    const isAudio = computed(() => props.attachment?.type.startsWith("audio/") ?? false);
    const isText = computed(() => {
      const att = props.attachment;
      if (!att) return false;
      const ext = att.name.slice(att.name.lastIndexOf(".")).toLowerCase();
      return att.type.startsWith("text/") || TEXT_EXTS.has(ext) || CODE_EXTS.has(ext);
    });

    const src = computed(() => props.attachment?.preview || props.attachment?.url || "");

    /* ── Text / code preview ─────────────────────────────────────── */
    const textContent = ref("");
    const textPlain = ref(false);
    const textLoading = ref(false);
    const textError = ref<string | null>(null);

    async function loadText() {
      const att = props.attachment;
      if (!att) return;
      textLoading.value = true;
      textError.value = null;
      try {
        if (att.file) {
          textContent.value = await att.file.text();
        } else {
          // URL-only: fetching through the app's authed transport is
          // deferred to the consumer. Render nothing rather than a raw
          // <a> that would leak the bearer-less URL.
          textContent.value = "";
        }
        const lang = codeLanguage(att.name);
        const isMd = /\.(md|markdown)$/i.test(att.name);
        if (isMd) {
          textPlain.value = false;
        } else if (lang) {
          textContent.value = "```" + lang + "\n" + textContent.value + "\n```";
          textPlain.value = false;
        } else {
          textPlain.value = true;
        }
      } catch (e) {
        textError.value = e instanceof Error ? e.message : String(e);
      } finally {
        textLoading.value = false;
      }
    }

    watch(
      () => [props.modelValue, props.attachment] as const,
      ([open]) => {
        if (!open || !props.attachment) return;
        if (isText.value) void loadText();
      },
      { immediate: true },
    );

    function download() {
      if (!src.value) return;
      const a = document.createElement("a");
      a.href = src.value;
      a.download = props.attachment?.name || "download";
      a.click();
    }

    function fileIcon() {
      const att = props.attachment;
      if (!att) return <FileIcon size={40} />;
      if (att.type.startsWith("image/")) return <ImageIcon size={40} />;
      if (att.type.startsWith("video/")) return <Film size={40} />;
      if (att.type.startsWith("audio/")) return <Music size={40} />;
      return <FileIcon size={40} />;
    }

    return () => (
      <HModal
        modelValue={props.modelValue}
        onUpdate:modelValue={(v: boolean) => emit("update:modelValue", v)}
        title={props.attachment?.name}
        width="44rem"
        footerActions={[
          { label: t("plana::attachment.download", "Download"), onClick: download, disabled: !src.value },
          { label: t("plana::attachment.close", "Close"), variant: "secondary", onClick: () => emit("update:modelValue", false) },
        ]}
      >
        <div class="s-attachment-modal">
          {isImage.value && src.value && (
            <div class="s-attachment-modal-preview">
              <img src={src.value} alt={props.attachment?.name ?? ""} />
            </div>
          )}

          {isVideo.value && src.value && (
            <video class="s-attachment-modal-preview" src={src.value} controls preload="metadata" />
          )}

          {isAudio.value && src.value && (
            <audio class="s-attachment-modal-preview" src={src.value} controls preload="metadata" />
          )}

          {isText.value && (
            <HScrollContainer class="s-attachment-modal-text">
              {props.attachment?.file ? (
                <HMarkdownRenderer
                  content={textContent.value}
                  loading={textLoading.value}
                  plain={textPlain.value}
                />
              ) : (
                <p class="s-attachment-modal-text-empty">
                  {textError.value ?? ""}
                </p>
              )}
              {textError.value && (
                <p class="s-attachment-modal-text-error">{textError.value}</p>
              )}
            </HScrollContainer>
          )}

          {!isImage.value && !isVideo.value && !isAudio.value && !isText.value && (
            <div class="s-attachment-modal-file">
              {fileIcon()}
              <p class="s-attachment-modal-file-name">{props.attachment?.name}</p>
            </div>
          )}

          <div class="s-attachment-modal-meta">
            <HBadge variant="muted">{props.attachment?.type || "unknown"}</HBadge>
            <HBadge variant="muted">{formatBytes(props.attachment?.size ?? 0)}</HBadge>
          </div>
        </div>
      </HModal>
    );
  },
});
