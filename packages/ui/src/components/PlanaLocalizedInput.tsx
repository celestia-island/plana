/**
 * PLocalizedInput — multilingual text input with a chip-based translation editor.
 * Upstreamed from shittim-chest (P5#A A3).
 *
 * ## Design
 *
 * Default mode: a plain `SInput` carrying a globe toggle in its suffix.
 * The globe is dim; clicking it switches to multi-language mode.
 *
 * Multi-language mode ON: the input collapses into a flex-wrap box of
 * translation **chips**. Each filled translation renders as a split
 * badge (primary left segment = uppercased language code, e.g. "FR" /
 * "ZHS"; muted right segment = the translated text) — the same visual
 * language as the agent-node "ProviderID#xx | ModelName" badge. Chips
 * are horizontally centred and wrap onto new rows; the box grows in
 * height as chips are added (its min-height tracks a regular input).
 *
 *   - Click a chip → it becomes an inline edit field (Enter/blur saves,
 *     Esc cancels).
 *   - The chip's × button drops that language.
 *   - The "+ Add" button opens an anchored `Popup` listing the
 *     not-yet-added languages (same primitive as the homepage language
 *     switch). Picking one inserts an empty translation and immediately
 *     enters edit mode for it.
 *   - A globe button toggles back to single-language mode.
 *
 * The globe suffix carries a subscript badge with the count of
 * non-source translations.
 */

import { computed, defineComponent, nextTick, ref, type PropType } from "vue";
import { useI18n } from "@celestia-island/hikari";

import Globe from "lucide-vue-next/dist/esm/icons/globe";
import Plus from "lucide-vue-next/dist/esm/icons/plus";
import X from "lucide-vue-next/dist/esm/icons/x";
import { HSpinner } from "@celestia-island/hikari";
import { HPopover } from "@celestia-island/hikari";
import { HBadge } from "@celestia-island/hikari";
import { HInput } from "@celestia-island/hikari";
import "./PlanaLocalizedInput.scss";

export type PLocalizedTitle = Record<string, string>;

export interface PLocaleOption {
  code: string;
  /** Display label; falls back to the code itself when omitted. */
  label?: string;
}

const DEFAULT_LOCALES: PLocaleOption[] = [
  { code: "en", label: "English" },
  { code: "zh-Hans", label: "简体中文" },
  { code: "zh-Hant", label: "繁體中文" },
  { code: "ja", label: "日本語" },
  { code: "ko", label: "한국어" },
  { code: "de", label: "Deutsch" },
  { code: "fr", label: "Français" },
  { code: "es", label: "Español" },
  { code: "pt", label: "Português" },
  { code: "ar", label: "العربية" },
  { code: "ru", label: "Русский" },
];

/** Codes for which a translation currently exists (non-empty). */
function filledCodes(translations: PLocalizedTitle): string[] {
  return Object.keys(translations).filter((k) => (translations[k] ?? "").trim().length > 0);
}

export default defineComponent({
  name: "PlanaLocalizedInput",
  props: {
    modelValue: { type: String, default: "" },
    sourceLang: { type: String, default: "en" },
    translations: { type: Object as PropType<PLocalizedTitle>, default: () => ({}) },
    translating: { type: Boolean, default: false },
    placeholder: { type: String, default: "" },
    label: { type: String, default: undefined },
    /** Languages offered in the add-language menu. Defaults to a built-in
     *  common set; pass your own to match the host app's i18n catalog. */
    localeOptions: { type: Array as PropType<PLocaleOption[]>, default: () => DEFAULT_LOCALES },
  },
  emits: {
    "update:modelValue": (_v: string) => true,
    "update:translations": (_v: PLocalizedTitle) => true,
    translate: (_text: string, _sourceLang: string, _targets: string[]) => true,
  },
  setup(props, { emit }) {
    const { t } = useI18n();
    const multiLangMode = ref(false);

    // ── Add-language popup ──
    const addOpen = ref(false);
    const addBtnRef = ref<HTMLElement | null>(null);

    // ── Inline chip editing ──
    // `editingCode` is the language currently being edited inline; the
    // matching chip is replaced by an input field bound to `editDraft`.
    const editingCode = ref<string | null>(null);
    const editDraft = ref("");
    const editInputRef = ref<HTMLInputElement | null>(null);

    const translationCount = computed(() =>
      filledCodes(props.translations).filter((c) => c !== props.sourceLang).length,
    );

    /** Codes that already have a chip shown (excludes the source lang —
     *  the source text lives in the single-language field, not a chip). */
    const shownCodes = computed(() =>
      filledCodes(props.translations).filter((c) => c !== props.sourceLang),
    );

    /** Label for a locale code: the caller-provided label or the code. */
    function langLabel(code: string): string {
      const opt = props.localeOptions.find((o) => o.code === code);
      return opt?.label ?? code;
    }

    /** Languages available to add = all offered minus the source lang
     *  minus already-filled codes. */
    const addableLocales = computed(() =>
      props.localeOptions.filter(
        (o) => o.code !== props.sourceLang && !shownCodes.value.includes(o.code),
      ),
    );

    function toggleMultiLang() {
      multiLangMode.value = !multiLangMode.value;
      if (!multiLangMode.value) cancelEdit();
    }

    function commitTranslation(code: string, value: string) {
      const next = { ...props.translations };
      const trimmed = value.trim();
      if (trimmed) {
        next[code] = trimmed;
      } else {
        delete next[code];
      }
      emit("update:translations", next);
    }

    function startEdit(code: string) {
      editingCode.value = code;
      editDraft.value = props.translations[code] ?? "";
      nextTick(() => {
        editInputRef.value?.focus();
        editInputRef.value?.select();
      });
    }

    function saveEdit() {
      const code = editingCode.value;
      if (code) commitTranslation(code, editDraft.value);
      editingCode.value = null;
    }

    function cancelEdit() {
      editingCode.value = null;
    }

    function removeCode(code: string) {
      const next = { ...props.translations };
      delete next[code];
      emit("update:translations", next);
      if (editingCode.value === code) editingCode.value = null;
    }

    function addLanguage(code: string) {
      addOpen.value = false;
      // Drop straight into inline edit for the new language. We do NOT
      // emit an empty translation: empty entries are pruned everywhere
      // (filledCodes / commitTranslation), so emitting one would just be
      // ignored and the edit input — which only renders for codes in
      // shownCodes — wouldn't appear. The translation is committed on
      // save (and dropped if left empty).
      startEdit(code);
    }

    function autoTranslateAll() {
      const targets = props.localeOptions.map((l) => l.code).filter((c) => c !== props.sourceLang);
      emit("translate", props.modelValue, props.sourceLang, targets);
    }

    return () => (
      <div class="s-localized-input">
        {props.label && (
          <label class="s-localized-input-label">{props.label}</label>
        )}

        {/* ── Single-language mode (default) ── */}
        {!multiLangMode.value && (
          <HInput
            modelValue={props.modelValue}
            onUpdate:modelValue={(v: string) => emit("update:modelValue", v)}
            placeholder={props.placeholder}
          >
            {{
              suffixIcon: () => (
                <button
                  class="s-localized-input-toggle"
                  onClick={(e: MouseEvent) => { e.stopPropagation(); toggleMultiLang(); }}
                  aria-label={t("plana::localizedInput.enableMultilang", "Enable multi-language")}
                  type="button"
                  data-empty={translationCount.value === 0 || undefined}
                >
                  <Globe size={15} />
                  {translationCount.value > 0 && (
                    <span class="s-localized-input-badge">{translationCount.value}</span>
                  )}
                </button>
              ),
            }}
          </HInput>
        )}

        {/* ── Multi-language mode: chip area ── */}
        {multiLangMode.value && (
          <div class="s-localized-input-chips">
            {/* Codes shown as chips, plus the code currently being
             * inline-edited even if it has no committed value yet (so
             * the edit field for a freshly-added language renders). */}
            {(editingCode.value && !shownCodes.value.includes(editingCode.value)
              ? [...shownCodes.value, editingCode.value]
              : shownCodes.value
            ).map((code) =>
              editingCode.value === code ? (
                // Inline edit field replacing this chip.
                <input
                  key={code}
                  ref={(el) => { editInputRef.value = (el as HTMLInputElement) ?? null; }}
                  class="s-input s-localized-input-chip-edit"
                  type="text"
                  value={editDraft.value}
                  placeholder={langLabel(code)}
                  data-lpignore="true"
                  onInput={(e) => { editDraft.value = (e.target as HTMLInputElement).value; }}
                  onKeydown={(e: KeyboardEvent) => {
                    if (e.key === "Enter") { e.preventDefault(); saveEdit(); }
                    else if (e.key === "Escape") { e.preventDefault(); cancelEdit(); }
                  }}
                  onBlur={saveEdit}
                />
              ) : (
                <span key={code} class="s-localized-input-chip-group">
                  <button
                    type="button"
                    class="s-localized-input-chip-body"
                    onClick={() => startEdit(code)}
                    aria-label={t("plana::localizedInput.edit", "Edit")}
                  >
                    <HBadge variant="primary" size="sm" uppercase pill={false} class="s-localized-input-chip-lang">
                      {code.toUpperCase()}
                    </HBadge>
                    <HBadge variant="muted" size="sm" pill={false} class="s-localized-input-chip-text-badge">
                      <span class="s-localized-input-chip-text">{props.translations[code]}</span>
                    </HBadge>
                  </button>
                  <button
                    type="button"
                    class="s-localized-input-chip-remove"
                    aria-label={t("plana::localizedInput.remove", "Remove")}
                    onClick={() => removeCode(code)}
                  >
                    <X size={11} />
                  </button>
                </span>
              ),
            )}

            {/* "+ Add language" trigger + anchored popup */}
            <button
              ref={(el) => { addBtnRef.value = (el as HTMLElement) ?? null; }}
              type="button"
              class="s-localized-input-add"
              data-disabled={addableLocales.value.length === 0 || undefined}
              aria-label={t("plana::localizedInput.addLanguage", "Add language")}
              onClick={() => { addOpen.value = true; }}
            >
              <Plus size={13} />
            </button>
            <HPopover
              modelValue={addOpen.value}
              onUpdate:modelValue={(v: boolean) => { addOpen.value = v; }}
              anchorRef={addBtnRef.value}
              placement="bottom-start"
              backdrop={false}
              closeOnBackdrop={true}
              closeOnEscape={true}
            >
              <div class="s-localized-input-add-menu">
                {addableLocales.value.length === 0 ? (
                  <div class="s-localized-input-add-empty">
                    {t("plana::localizedInput.allAdded", "All languages added")}
                  </div>
                ) : (
                  addableLocales.value.map((loc) => (
                    <button
                      key={loc.code}
                      type="button"
                      class="s-popup-menu-item"
                      onClick={() => addLanguage(loc.code)}
                    >
                      <span>{loc.label ?? loc.code}</span>
                      <span class="s-localized-input-add-code">{loc.code.toUpperCase()}</span>
                    </button>
                  ))
                )}
                {/* Auto-translate shortcut when more than one target is available. */}
                {addableLocales.value.length > 1 && (
                  <>
                    <div class="s-localized-input-add-divider" />
                    <button
                      type="button"
                      class="s-popup-menu-item"
                      onClick={() => {
                        addOpen.value = false;
                        autoTranslateAll();
                      }}
                    >
                      {props.translating ? (
                        <HSpinner size={14} />
                      ) : (
                        <Globe size={14} />
                      )}
                      <span>{t("plana::localizedInput.autoTranslate", "Auto-translate all")}</span>
                    </button>
                  </>
                )}
              </div>
            </HPopover>

            {/* Toggle back to single-language mode. */}
            <button
              class="s-localized-input-toggle s-localized-input-chips-toggle"
              onClick={(e: MouseEvent) => { e.stopPropagation(); toggleMultiLang(); }}
              aria-label={t("plana::localizedInput.disableMultilang", "Disable multi-language")}
              type="button"
            >
              {props.translating ? <HSpinner size={15} /> : <Globe size={15} />}
              {translationCount.value > 0 && (
                <span class="s-localized-input-badge">{translationCount.value}</span>
              )}
            </button>
          </div>
        )}
      </div>
    );
  },
});
