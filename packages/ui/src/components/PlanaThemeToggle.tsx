import { computed, defineComponent, onMounted, ref, type PropType } from "vue";
import Check from "lucide-vue-next/dist/esm/icons/check";
import ChevronDown from "lucide-vue-next/dist/esm/icons/chevron-down";
import Monitor from "lucide-vue-next/dist/esm/icons/monitor";
import Moon from "lucide-vue-next/dist/esm/icons/moon";
import Palette from "lucide-vue-next/dist/esm/icons/palette";
import Sun from "lucide-vue-next/dist/esm/icons/sun";
import Trash2 from "lucide-vue-next/dist/esm/icons/trash";
import { HDivider, HPopover, mergeMessages, useI18n, useTheme, type PopupPlacement, type ThemeId } from "@celestia-island/hikari";

import { PColorSchemeDialog, type PCustomTheme } from "./PlanaColorSchemeDialog";
import "./PlanaThemeToggle.scss";

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
 * PThemeToggle — light/dark/auto theme control over hikari's theme engine.
 *
 * Composes hikari's `useTheme()` (theme presets + custom themes + mode
 * persistence); it does NOT reimplement the theme engine. A main button
 * cycles light/dark (auto keeps a Monitor glyph); the popover offers
 * explicit Light/Dark/Auto modes, preset/custom theme selection (custom
 * themes are removable), and opens PColorSchemeDialog to create a new
 * custom scheme.
 */
export const PThemeToggle = defineComponent({
  name: "PlanaThemeToggle",
  props: {
    /** Popover placement. */
    popoverPlacement: { type: String as PropType<PopupPlacement>, default: "bottom-end" },
  },
  emits: {
    "update:scheme": (_theme: PCustomTheme) => true,
  },
  setup(props, { emit, slots }) {
    const { t } = useI18n();
    const { currentTheme, currentMode, effectiveMode, setTheme, setMode, toggleMode, allThemeList, addCustomTheme, removeCustomTheme } = useTheme();

    const menuOpen = ref(false);
    const triggerRef = ref<HTMLElement | null>(null);
    const schemeDialogOpen = ref(false);

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

    const modeLabel = computed(() => {
      const map: Record<string, string> = {
        system: t("plana::theme.modeAuto"),
        light: t("plana::theme.modeLight"),
        dark: t("plana::theme.modeDark"),
      };
      return map[currentMode.value] ?? currentMode.value;
    });

    const modeOptions = computed(() => [
      { key: "light", label: t("plana::theme.modeLight"), icon: () => <Sun size={12} /> },
      { key: "dark", label: t("plana::theme.modeDark"), icon: () => <Moon size={12} /> },
      { key: "system", label: t("plana::theme.modeAuto"), icon: () => <Monitor size={12} /> },
    ]);

    function onSelectTheme(id: ThemeId) {
      setTheme(id);
      menuOpen.value = false;
    }

    function onConfirmScheme(theme: PCustomTheme) {
      addCustomTheme(theme);
      setTheme(theme.id);
      emit("update:scheme", theme);
    }

    return () => (
      <div class="s-theme-toggle" ref={triggerRef}>
        <button
          type="button"
          class="s-theme-toggle-btn"
          data-variant="main"
          onClick={toggleMode}
          title={modeLabel.value}
          aria-label={t("plana::theme.mode")}
        >
          {currentMode.value === "system" ? (
            <Monitor size={14} />
          ) : effectiveMode.value === "dark" ? (
            <Moon size={14} />
          ) : (
            <Sun size={14} />
          )}
        </button>
        <button
          type="button"
          class="s-theme-toggle-btn"
          data-variant="arrow"
          onClick={() => { menuOpen.value = !menuOpen.value; }}
          aria-label={t("plana::theme.themes")}
        >
          <ChevronDown size={12} />
        </button>

        <HPopover
          modelValue={menuOpen.value}
          onUpdate:modelValue={(v: boolean) => { menuOpen.value = v; }}
          placement={props.popoverPlacement}
          anchorRef={triggerRef.value ?? null}
        >
          <div class="s-theme-menu">
            <div class="s-theme-menu-label">{t("plana::theme.mode")}</div>
            <div class="s-theme-mode-row">
              {modeOptions.value.map((opt) => (
                <button
                  key={opt.key}
                  type="button"
                  class="s-theme-mode-btn"
                  data-active={currentMode.value === opt.key || undefined}
                  onClick={() => setMode(opt.key as "light" | "dark" | "system")}
                >
                  {opt.icon()}
                  {opt.label}
                </button>
              ))}
            </div>

            <HDivider spacing="md" />

            <div class="s-theme-menu-label">{t("plana::theme.themes")}</div>
            {allThemeList.value.map((th) => (
              <div key={th.id} class="s-theme-item-row">
                <button
                  type="button"
                  class="s-theme-item-btn"
                  data-active={currentTheme.value === th.id || undefined}
                  onClick={() => onSelectTheme(th.id)}
                >
                  <span>{th.name}</span>
                  {currentTheme.value === th.id && <Check size={14} />}
                </button>
                {th.isCustom && (
                  <button
                    type="button"
                    class="s-theme-item-delete"
                    title={t("plana::theme.deleteTheme")}
                    onClick={() => removeCustomTheme(th.id)}
                  >
                    <Trash2 size={12} />
                  </button>
                )}
              </div>
            ))}

            <button
              type="button"
              class="s-theme-item-btn"
              onClick={() => { schemeDialogOpen.value = true; menuOpen.value = false; }}
            >
              <Palette size={14} />
              <span>{t("plana::theme.editScheme")}</span>
            </button>
          </div>
          {slots["menu-extra"]?.()}
        </HPopover>

        <PColorSchemeDialog
          modelValue={schemeDialogOpen.value}
          onUpdate:modelValue={(v: boolean) => { schemeDialogOpen.value = v; }}
          onConfirm={onConfirmScheme}
        />
      </div>
    );
  },
});
