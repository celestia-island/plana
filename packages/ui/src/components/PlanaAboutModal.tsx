import { defineComponent, type PropType } from "vue";
import { HBadge, HModal, mergeMessages, useI18n } from "@celestia-island/hikari";

import "./PlanaAboutModal.scss";

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

export interface PAboutLink {
  label: string;
  href: string;
}

/**
 * PAboutModal — version / about dialog.
 *
 * Shows the app identity and version metadata plus optional external
 * links. Version/build hashes render as short hashes when longer than 12
 * chars (full value in `title` tooltip).
 */
export const PAboutModal = defineComponent({
  name: "PlanaAboutModal",
  props: {
    modelValue: { type: Boolean, default: false },
    /** Application display name. */
    appName: { type: String, required: true },
    /** Application version (e.g. "0.1.4"). */
    version: { type: String, required: true },
    /** Optional app build hash / commit. */
    buildHash: { type: String, default: undefined },
    /** Optional engine version (backend), e.g. "0.2.1". */
    engineVersion: { type: String, default: undefined },
    /** Optional engine build hash / commit. */
    engineBuildHash: { type: String, default: undefined },
    /** Optional external links (e.g. GitHub, docs). */
    links: { type: Array as PropType<PAboutLink[]>, default: () => [] },
    title: { type: String, default: undefined },
  },
  emits: {
    "update:modelValue": (_v: boolean) => true,
  },
  setup(props, { emit }) {
    const { t } = useI18n();

    function shortHash(hash: string): string {
      return hash.length > 12 ? `${hash.slice(0, 12)}…` : hash;
    }

    return () => (
      <HModal
        modelValue={props.modelValue}
        onUpdate:modelValue={(v: boolean) => emit("update:modelValue", v)}
        title={props.title ?? t("plana::about.title")}
        width="30rem"
      >
        <div class="s-about-modal">
          <header class="s-about-modal-header">
            <div class="s-about-modal-logo">{props.appName.slice(0, 1).toUpperCase()}</div>
            <div>
              <h2 class="s-about-modal-name">{props.appName}</h2>
              <p class="s-about-modal-version">
                {t("plana::about.version")} {props.version}
              </p>
            </div>
          </header>

          <div class="s-about-modal-rows">
            {props.buildHash && (
              <div class="s-about-modal-row">
                <span class="s-about-modal-row-label">{t("plana::about.buildHash")}</span>
                <span class="s-about-modal-row-value" title={props.buildHash}>
                  {shortHash(props.buildHash)}
                </span>
              </div>
            )}
            {props.engineVersion && (
              <div class="s-about-modal-row">
                <span class="s-about-modal-row-label">{t("plana::about.engineVersion")}</span>
                <span class="s-about-modal-row-value">{props.engineVersion}</span>
              </div>
            )}
            {props.engineBuildHash && (
              <div class="s-about-modal-row">
                <span class="s-about-modal-row-label">{t("plana::about.engineBuildHash")}</span>
                <span class="s-about-modal-row-value" title={props.engineBuildHash}>
                  {shortHash(props.engineBuildHash)}
                </span>
              </div>
            )}
          </div>

          {props.links.length > 0 && (
            <div class="s-about-modal-links">
              <span class="s-about-modal-row-label">{t("plana::about.links")}</span>
              <div class="s-about-modal-links-list">
                {props.links.map((link) => (
                  <a key={link.href} href={link.href} target="_blank" rel="noopener noreferrer" class="s-about-modal-link">
                    {link.label}
                  </a>
                ))}
              </div>
            </div>
          )}

          <footer class="s-about-modal-footer">
            <HBadge variant="muted">
              © {new Date().getFullYear()} {props.appName}
            </HBadge>
          </footer>
        </div>
      </HModal>
    );
  },
});
