import { ref, type Ref } from "vue";

export interface UseAsyncDataReturn<T> {
  data: Ref<T | null>;
  loading: Ref<boolean>;
  error: Ref<Error | null>;
  refresh: () => Promise<void>;
}

/**
 * Async data loader (upstreamed from shittim-chest, vueuse-free).
 *
 * `{ data, loading, error, refresh }` surface; non-Error rejects are
 * normalized into `Error`. `refresh` keeps the previous data visible
 * while in flight.
 */
export function useAsyncData<T>(
  fetcher: () => Promise<T>,
  options?: { immediate?: boolean },
): UseAsyncDataReturn<T> {
  const data = ref<T | null>(null) as Ref<T | null>;
  const loading = ref(false);
  const error = ref<Error | null>(null);

  async function refresh(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const result = await fetcher();
      data.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e : new Error(String(e));
    } finally {
      loading.value = false;
    }
  }

  if (options?.immediate !== false) {
    void refresh();
  }

  return { data, loading, error, refresh };
}
