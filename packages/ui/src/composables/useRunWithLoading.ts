import { type Ref } from "vue";
import { resolveErrorMessage, type TranslateFn } from "../utils/errors";

/**
 * Loading + error wrapper for async actions (upstreamed from shittim-chest).
 * `run` sets `loading` around the call; both variants normalize failures
 * through `resolveErrorMessage` into the given `error` ref.
 */
export function useRunWithLoading(
  loading: Ref<boolean>,
  error: Ref<string | null>,
  t: TranslateFn,
) {
  async function run<T>(fn: () => Promise<T>): Promise<T | undefined> {
    loading.value = true;
    error.value = null;
    try {
      return await fn();
    } catch (e) {
      error.value = resolveErrorMessage(t, e);
      return undefined;
    } finally {
      loading.value = false;
    }
  }

  async function runAction<T>(fn: () => Promise<T>): Promise<T | undefined> {
    error.value = null;
    try {
      return await fn();
    } catch (e) {
      error.value = resolveErrorMessage(t, e);
      return undefined;
    }
  }

  return { run, runAction };
}
