import { defineStore } from "pinia";
import { ref } from "vue";
import { useI18n } from "@celestia-island/hikari";
import { useRunWithLoading } from "./useRunWithLoading";
import { resolveErrorMessage, type TranslateFn } from "../utils/errors";

export interface AdminCrudApi<T, CreateT = Record<string, unknown>, UpdateT = CreateT> {
  list: () => Promise<T[]>;
  create: (payload: CreateT) => Promise<unknown>;
  update: (id: string, payload: UpdateT) => Promise<unknown>;
  remove: (id: string) => Promise<unknown>;
}

/**
 * Admin resource CRUD store factory (upstreamed from shittim-chest's
 * hand-rolled admin stores).
 *
 * Produces the standard skeleton — items list, loading, error,
 * fetch/create/update/remove with refetch-on-mutation and
 * error normalization — parameterized over the API functions and the
 * store id. Domain stores can spread the returned state/actions and add
 * their own fields (vendors, quotas, …).
 */
export function createAdminCrudStore<
  T extends { id: string },
  CreateT = Record<string, unknown>,
  UpdateT = CreateT,
>(id: string, api: AdminCrudApi<T, CreateT, UpdateT>) {
  return defineStore(id, () => {
    const { t: rawT } = useI18n();
    const t: TranslateFn = (key: string, ...args: unknown[]) =>
      rawT(key, typeof args[0] === "string" ? args[0] : undefined);
    const items = ref<T[]>([]);
    const loading = ref(true);
    const error = ref<string | null>(null);

    const { run, runAction } = useRunWithLoading(loading, error, t);

    async function fetchItems() {
      const result = await run(() => api.list());
      if (result) items.value = result;
    }

    async function createItem(payload: CreateT) {
      error.value = null;
      try {
        await api.create(payload);
        await fetchItems();
      } catch (err) {
        error.value = resolveErrorMessage(t, err);
        throw err;
      }
    }

    async function updateItem(idValue: string, payload: UpdateT) {
      error.value = null;
      try {
        await api.update(idValue, payload);
        await fetchItems();
      } catch (err) {
        error.value = resolveErrorMessage(t, err);
        throw err;
      }
    }

    async function removeItem(idValue: string) {
      error.value = null;
      try {
        await api.remove(idValue);
        await fetchItems();
      } catch (err) {
        error.value = resolveErrorMessage(t, err);
        throw err;
      }
    }

    return {
      items,
      loading,
      error,
      fetchItems,
      createItem,
      updateItem,
      removeItem,
    };
  });
}
