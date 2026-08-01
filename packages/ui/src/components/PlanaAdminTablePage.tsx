import { defineComponent, onMounted, type PropType } from "vue";
import { HAlert, HCard, HEmptyState, HSpinner, HTable, useI18n, mergeMessages } from "@celestia-island/hikari";
import { PPageHeader } from "./PlanaPageHeader";

import enLocale from "../i18n/locales/en/connection.json";
import zhsLocale from "../i18n/locales/zh-Hans/connection.json";
import zhtLocale from "../i18n/locales/zh-Hant/connection.json";
import jaLocale from "../i18n/locales/ja/connection.json";
import koLocale from "../i18n/locales/ko/connection.json";
import ruLocale from "../i18n/locales/ru/connection.json";
import arLocale from "../i18n/locales/ar/connection.json";
import deLocale from "../i18n/locales/de/connection.json";
import esLocale from "../i18n/locales/es/connection.json";
import frLocale from "../i18n/locales/fr/connection.json";
import ptLocale from "../i18n/locales/pt/connection.json";

/** Column definition passed through to HTable. */
export interface PTableColumn {
  key: string;
  title: string;
  width?: string;
  sortable?: boolean;
  align?: "left" | "center" | "right";
}

/**
 * CRUD table-page scaffold: PPageHeader + loading spinner + error alert +
 * empty state + HTable. Slots:
 * - `actions`      — header actions (e.g. a "Create" button)
 * - `cell-<key>`   — per-column cell templates, forwarded to HTable
 * - `create-modal` — create dialog, rendered after the table
 * - `edit-modal`   — edit dialog, rendered after the table
 */
export const PAdminTablePage = defineComponent({
  name: "PlanaAdminTablePage",
  props: {
    title: { type: String, default: "" },
    loading: { type: Boolean, default: false },
    error: { type: String as PropType<string | undefined>, default: undefined },
    rows: { type: Array as PropType<Record<string, unknown>[]>, required: true },
    columns: { type: Array as PropType<PTableColumn[]>, required: true },
    rowKey: { type: String, default: "id" },
    emptyTitle: { type: String, default: "" },
    emptyDescription: { type: String as PropType<string | undefined>, default: undefined },
  },
  setup(props, { slots }) {
    onMounted(() => {
      mergeMessages(enLocale.connection, "en");
      mergeMessages(zhsLocale.connection, "zh-Hans");
      mergeMessages(zhtLocale.connection, "zh-Hant");
      mergeMessages(jaLocale.connection, "ja");
      mergeMessages(koLocale.connection, "ko");
      mergeMessages(ruLocale.connection, "ru");
      mergeMessages(arLocale.connection, "ar");
      mergeMessages(deLocale.connection, "de");
      mergeMessages(esLocale.connection, "es");
      mergeMessages(frLocale.connection, "fr");
      mergeMessages(ptLocale.connection, "pt");
    });

    return () => {
      const { t } = useI18n();
      const emptyTitle = props.emptyTitle || t("plana::tablePage.emptyTitle", "No data");
      return (
        <div>
          {props.title ? (
            <PPageHeader title={props.title}>
              {{ actions: () => slots.actions?.() }}
            </PPageHeader>
          ) : slots.actions ? (
            <div style={{ display: "flex", justifyContent: "flex-end", marginBottom: "var(--space-16, 1rem)" }}>
              {slots.actions()}
            </div>
          ) : null}
          {props.error ? (
            <HAlert message={props.error} />
          ) : props.loading && !props.rows.length ? (
            <HSpinner center />
          ) : !props.rows.length ? (
            <HEmptyState title={emptyTitle} description={props.emptyDescription} />
          ) : (
            <HCard padded={false}>
              <HTable columns={props.columns} rows={props.rows} rowKey={props.rowKey}>
                {slots}
              </HTable>
            </HCard>
          )}
          {slots["create-modal"]?.()}
          {slots["edit-modal"]?.()}
        </div>
      );
    };
  },
});
