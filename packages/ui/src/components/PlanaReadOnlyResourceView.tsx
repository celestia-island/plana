import { computed, defineComponent, onMounted, ref, type PropType } from "vue";
import { HAlert, HBadge, HButton, HCard, HEmptyState, HInput, HSpinner, HTable, useI18n } from "@celestia-island/hikari";
import RefreshCw from "lucide-vue-next/dist/esm/icons/refresh-cw";
import Search from "lucide-vue-next/dist/esm/icons/search";

export interface PReadOnlyResource {
  name: string;
  agent?: string | null;
  description?: string | null;
}

/**
 * Generic read-only resource browser (upstreamed from shittim-chest):
 * search box + refresh + count, loading/error/empty states and a table of
 * name/agent/description rows. The fetcher is injected so transports and
 * mock layers stay with the caller.
 */
export const PReadOnlyResourceView = defineComponent({
  name: "PlanaReadOnlyResourceView",
  props: {
    titleKey: { type: String, required: true },
    nameKey: { type: String, required: true },
    agentKey: { type: String, required: true },
    descriptionKey: { type: String, required: true },
    emptyKey: { type: String, required: true },
    fetchItems: { type: Function as PropType<() => Promise<PReadOnlyResource[]>>, required: true },
  },
  setup(props) {
    const { t } = useI18n();
    const items = ref<PReadOnlyResource[]>([]);
    const loading = ref(true);
    const error = ref<string | null>(null);
    const query = ref("");

    const columns = [
      { key: "name", title: t(props.nameKey) },
      { key: "agent", title: t(props.agentKey) },
      { key: "description", title: t(props.descriptionKey) },
    ];

    async function fetch() {
      loading.value = true;
      error.value = null;
      try {
        items.value = await props.fetchItems();
      } catch (err: unknown) {
        error.value = err instanceof Error ? err.message : String(err);
      } finally {
        loading.value = false;
      }
    }

    onMounted(fetch);

    const filtered = computed(() => {
      const q = query.value.trim().toLowerCase();
      if (!q) return items.value;
      return items.value.filter(
        (it) =>
          it.name.toLowerCase().includes(q) ||
          (it.agent || "").toLowerCase().includes(q) ||
          (it.description || "").toLowerCase().includes(q),
      );
    });

    return () => (
      <div>
        <div class="flex items-center gap-2 mb-3">
          <div class="flex-1 max-w-xs">
            <HInput
              modelValue={query.value}
              onUpdate:modelValue={(v: string) => (query.value = v)}
              placeholder={t("plana::readonly.search", "Search…")}
            />
          </div>
          <HButton variant="ghost" size="sm" loading={loading.value} onClick={fetch}>
            <RefreshCw size={14} />
          </HButton>
          <span class="text-xs text-muted ml-auto">{filtered.value.length}</span>
        </div>
        {loading.value && !items.value.length ? (
          <HSpinner center />
        ) : error.value ? (
          <HAlert message={error.value} />
        ) : !filtered.value.length ? (
          <HEmptyState title={query.value ? t("plana::readonly.noResults", "No matches") : t(props.emptyKey)} />
        ) : (
          <HCard padded={false}>
            <HTable columns={columns} rows={filtered.value} rowKey="name">
              {{
                "cell-name": ({ row }: { row: PReadOnlyResource }) => (
                  <span class="font-medium">{row.name}</span>
                ),
                "cell-agent": ({ row }: { row: PReadOnlyResource }) => (
                  <HBadge variant="primary">{row.agent}</HBadge>
                ),
                "cell-description": ({ row }: { row: PReadOnlyResource }) => (
                  <span class="text-sm text-muted">{row.description}</span>
                ),
              }}
            </HTable>
          </HCard>
        )}
      </div>
    );
  },
});
