import { computed, defineComponent, onMounted, ref, watch, type PropType } from "vue";
import { Film, File as FileIcon, ImageIcon, Music } from "lucide-vue-next";
import {
  HBadge,
  HImageViewer,
  HMarkdownRenderer,
  HMediaPlayer,
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

/** Kind of rich preview to render; inferred from MIME unless hinted. */
export type PAttachmentPreviewType = "image" | "video" | "audio" | "other";

/** Resolve the preview kind: an explicit hint wins, MIME prefix otherwise. */
export function previewKindFor(
  att: Pick<PAttachmentDetail, "type"> | null | undefined,
  hint?: PAttachmentPreviewType,
): PAttachmentPreviewType {
  if (hint) return hint;
  const type = att?.type ?? "";
  if (type.startsWith("image/")) return "image";
  if (type.startsWith("video/")) return "video";
  if (type.startsWith("audio/")) return "audio";
  return "other";
}

function isTextFile(att: PAttachmentDetail): boolean {
  const ext = att.name.slice(att.name.lastIndexOf(".")).toLowerCase();
  return att.type.startsWith("text/") || TEXT_EXTS.has(ext) || CODE_EXTS.has(ext);
}

/**
 * PAttachmentModal — generic file-picker preview modal.
 *
 * Renders an attachment by preview kind: image (HImageViewer), video/audio
 * (HMediaPlayer), text/code (HMarkdownRenderer, with syntax fences for
 * known code extensions) or a generic file chip.
 *
 * URL handling is transport-agnostic: when the attachment carries no
 * `preview`/`url`, the consumer may pass `resolveUrl` (its own API client /
 * transport knows how to turn a backend file name into an authed URL).
 * The resolved URL is used for media previews, the text fetch and the
 * download action. Text content is read via the attachment's `file` handle
 * when provided.
 */
export const PAttachmentModal = defineComponent({
  name: "PlanaAttachmentModal",
  props: {
    modelValue: { type: Boolean, default: false },
    attachment: { type: Object as PropType<PAttachmentDetail | null>, default: null },
    /** Transport-provided URL resolver; called with the file name when the
     *  attachment has neither `preview` nor `url`. */
    resolveUrl: { type: Function as PropType<(name: string) => Promise<string>>, default: undefined },
    /** Preview kind hint; inferred from the MIME type when omitted. */
    previewType: { type: String as PropType<PAttachmentPreviewType>, default: undefined },
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

    const kind = computed(() => previewKindFor(props.attachment, props.previewType));
    const isText = computed(() => (props.attachment ? isTextFile(props.attachment) : false));

    /* ── URL resolution ──────────────────────────────────────────── */
    const resolvedSrc = ref("");
    const srcLoading = ref(false);
    const srcError = ref<string | null>(null);

    async function resolveSrc() {
      const att = props.attachment;
      srcLoading.value = false;
      srcError.value = null;
      if (!att) {
        resolvedSrc.value = "";
        return;
      }
      if (att.preview || att.url) {
        resolvedSrc.value = att.preview || att.url || "";
        return;
      }
      if (props.resolveUrl) {
        srcLoading.value = true;
        try {
          resolvedSrc.value = await props.resolveUrl(att.name);
        } catch (e) {
          resolvedSrc.value = "";
          srcError.value = e instanceof Error ? e.message : String(e);
        } finally {
          srcLoading.value = false;
        }
        return;
      }
      resolvedSrc.value = "";
    }

    watch(
      () => [props.attachment, props.resolveUrl, props.previewType] as const,
      () => { void resolveSrc(); },
      { immediate: true },
    );

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
        let raw: string;
        if (att.file) {
          raw = await att.file.text();
        } else if (resolvedSrc.value) {
          // The resolved URL comes from the consumer's transport (via
          // `resolveUrl`) or was already a usable preview/object URL.
          const res = await fetch(resolvedSrc.value);
          if (!res.ok) throw new Error(`HTTP ${res.status}`);
          raw = await res.text();
        } else {
          raw = "";
        }
        const lang = codeLanguage(att.name);
        const isMd = /\.(md|markdown)$/i.test(att.name);
        if (isMd) {
          textContent.value = raw;
          textPlain.value = false;
        } else if (lang) {
          textContent.value = "```" + lang + "\n" + raw + "\n```";
          textPlain.value = false;
        } else {
          textContent.value = raw;
          textPlain.value = true;
        }
      } catch (e) {
        textError.value = e instanceof Error ? e.message : String(e);
      } finally {
        textLoading.value = false;
      }
    }

    function startTextLoad() {
      if (!props.modelValue || !props.attachment) return;
      if (isText.value) void loadText();
    }

    watch(
      () => [props.modelValue, props.attachment] as const,
      () => startTextLoad(),
      { immediate: true },
    );

    // Re-fetch text once the transport URL lands (resolveUrl is async).
    watch(resolvedSrc, () => {
      if (!props.attachment?.file) startTextLoad();
    });

    function download() {
      if (!resolvedSrc.value) return;
      const a = document.createElement("a");
      a.href = resolvedSrc.value;
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
          { label: t("plana::attachment.download", "Download"), onClick: download, disabled: !resolvedSrc.value },
          { label: t("plana::attachment.close", "Close"), variant: "secondary", onClick: () => emit("update:modelValue", false) },
        ]}
      >
        <div class="s-attachment-modal">
          {srcLoading.value && (
            <p class="s-attachment-modal-text-empty">{t("plana::attachment.loading", "Loading…")}</p>
          )}

          {/* Image — zoomable viewer with minimap navigator */}
          {!srcLoading.value && kind.value === "image" && resolvedSrc.value && (
            <HImageViewer src={resolvedSrc.value} alt={props.attachment?.name ?? ""} />
          )}

          {/* Video — hikari media player with control bar */}
          {!srcLoading.value && kind.value === "video" && resolvedSrc.value && (
            <HMediaPlayer type="video" src={resolvedSrc.value} />
          )}

          {/* Audio — hikari media player with visualizer + control bar */}
          {!srcLoading.value && kind.value === "audio" && resolvedSrc.value && (
            <HMediaPlayer type="audio" src={resolvedSrc.value} />
          )}

          {/* Text / code — markdown + highlight.js via HMarkdownRenderer */}
          {isText.value && (
            <HScrollContainer class="s-attachment-modal-text">
              {props.attachment?.file || resolvedSrc.value ? (
                <HMarkdownRenderer
                  content={textContent.value}
                  loading={textLoading.value}
                  plain={textPlain.value}
                />
              ) : (
                <p class="s-attachment-modal-text-empty">
                  {srcError.value
                    ? srcError.value
                    : t("plana::attachment.noPreview", "No preview available.")}
                </p>
              )}
              {textError.value && (
                <p class="s-attachment-modal-text-error">{textError.value}</p>
              )}
            </HScrollContainer>
          )}

          {/* Generic file */}
          {!srcLoading.value && kind.value === "other" && !isText.value && (
            <div class="s-attachment-modal-file">
              {fileIcon()}
              <p class="s-attachment-modal-file-name">{props.attachment?.name}</p>
            </div>
          )}

          {srcError.value && kind.value !== "other" && (
            <p class="s-attachment-modal-text-error">{srcError.value}</p>
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
