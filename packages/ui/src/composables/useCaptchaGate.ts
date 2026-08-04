import { ref } from "vue";

/** Captcha descriptor handed to the client (Turnstile / reCAPTCHA / hosted). */
export interface PCaptchaDescriptor {
  provider: string;
  sitekey: string;
  script_url?: string;
}

/**
 * On-demand third-party captcha gate (upstreamed from shittim-chest).
 *
 * Solving happens at submit time (not on page load) so the captcha token is
 * always fresh — Turnstile/reCAPTCHA tokens expire in a few minutes. The view
 * calls `solve(descriptor)` when it needs a token; a modal hosts the widget
 * and the returned promise resolves with the token (or `""` if cancelled).
 * Pairs with `PAuthSubmitButton`'s `onCaptcha` hook.
 */
export function useCaptchaGate() {
  const open = ref(false);
  const descriptor = ref<PCaptchaDescriptor | null>(null);
  const attempt = ref(0);
  let resolver: ((token: string) => void) | null = null;

  function solve(desc: PCaptchaDescriptor): Promise<string> {
    descriptor.value = desc;
    attempt.value += 1;
    open.value = true;
    return new Promise<string>((resolve) => {
      resolver = resolve;
    });
  }

  function onToken(token: string) {
    open.value = false;
    resolver?.(token);
    resolver = null;
  }

  function onCancel() {
    open.value = false;
    resolver?.("");
    resolver = null;
  }

  return { open, descriptor, attempt, solve, onToken, onCancel };
}
