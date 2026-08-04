import { onUnmounted, ref } from "vue";

/**
 * Confirmation-dialog state machine (upstreamed from shittim-chest).
 *
 * `confirm(title, message)` returns a promise that resolves with the user's
 * choice; the view renders its own dialog from `visible/title/message` (or
 * drives an `HConfirmDialog`). The promise is settled with `false` when the
 * owning scope unmounts, so awaiting callers never hang.
 */
export function useConfirm() {
  const visible = ref(false);
  const title = ref("");
  const message = ref("");
  let resolvePromise: ((_value: boolean) => void) | null = null;

  onUnmounted(() => {
    resolvePromise?.(false);
    resolvePromise = null;
    visible.value = false;
  });

  function confirm(titleText: string, messageText: string): Promise<boolean> {
    resolvePromise?.(false);
    title.value = titleText;
    message.value = messageText;
    visible.value = true;
    return new Promise((resolve) => {
      resolvePromise = resolve;
    });
  }

  function accept() {
    visible.value = false;
    resolvePromise?.(true);
    resolvePromise = null;
  }

  function cancel() {
    visible.value = false;
    resolvePromise?.(false);
    resolvePromise = null;
  }

  return { visible, title, message, confirm, accept, cancel };
}
