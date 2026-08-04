import { ref } from "vue";
import { useClipboard as useBaseClipboard } from "@celestia-island/hikari";

/**
 * Clipboard copy with a toast on success/failure (upstreamed from
 * shittim-chest). Wraps hikari's base `useClipboard` (which already falls
 * back to `execCommand` outside secure contexts) and adds the toast
 * feedback; `copied` reflects the 2s copied state.
 */
export function useClipboardWithToast(
  toast: { success: (msg: string) => void; error: (msg: string) => void },
  defaultSuccessMessage?: () => string,
  defaultErrorMessage?: () => string,
) {
  const { copy: baseCopy, copied } = useBaseClipboard();

  async function copy(text: string, successMessage?: string) {
    const ok = await baseCopy(text);
    if (ok) {
      toast.success(successMessage ?? defaultSuccessMessage?.() ?? "Copied");
    } else {
      toast.error(defaultErrorMessage?.() ?? "Copy failed");
    }
  }

  return { copy, copied };
}
