/**
 * Error-message resolution shared across the celestia webuis.
 *
 * `resolveErrorMessage` turns a thrown value into a user-facing i18n string:
 * fetch network failures, server error bodies (`{ error, message }`), HTTP
 * status codes (401/404/409/429/5xx) and unknown values. The `t` function is
 * injected by the caller and the i18n key prefix defaults to `plana::errors`
 * (override it to reuse your own catalog, e.g. "errors").
 */

export type TranslateFn = (key: string, ...args: unknown[]) => string;

const NETWORK_FAILURE_PATTERNS = [
  "failed to fetch",
  "networkerror",
  "load failed",
  "err_empty_response",
];

export interface ServerErrorBody {
  code: string;
  message: string;
}

const SERVER_TO_I18N: Record<string, string> = {
  invalid_token: "api.invalidToken",
  invalid_credentials: "auth.invalidCredentials",
  rate_limited: "api.rateLimited",
  not_found: "api.notFound",
};

function isNetworkFailureMessage(err: unknown): boolean {
  const msg = (err as { message?: unknown }).message;
  if (typeof msg !== "string") return false;
  const lower = msg.toLowerCase();
  return NETWORK_FAILURE_PATTERNS.some((p) => lower.includes(p));
}

/** Parse a server error body `{ error, message }` (also `{ error, msg }`). */
export function parseServerErrorBody(body: string): ServerErrorBody | null {
  try {
    const parsed = JSON.parse(body);
    if (parsed && typeof parsed.error === "string") {
      return { code: parsed.error, message: parsed.message || "" };
    }
  } catch {
    // ignore JSON parse error
  }
  return null;
}

export function serverErrorToI18nKey(code: string): string {
  return SERVER_TO_I18N[code] || "generic.unknown";
}

/** Resolve an unknown thrown value to a user-facing message. */
export function resolveErrorMessage(
  t: TranslateFn,
  err: unknown,
  keyPrefix = "plana::errors",
): string {
  const k = (key: string) => `${keyPrefix}.${key}`;
  if (!err) {
    return t(k("generic.unknown"), "Something went wrong");
  }

  if (typeof err === "object" && err !== null && isNetworkFailureMessage(err)) {
    return t(k("network.connectionLost"), "Connection lost");
  }

  if (typeof err !== "object" || err === null) {
    return t(k("generic.unknown"));
  }

  const e = err as { status?: number; body?: string; message?: string };

  if (e.body && typeof e.body === "string") {
    const parsed = parseServerErrorBody(e.body);
    if (parsed) {
      const i18nMsg = t(k(serverErrorToI18nKey(parsed.code)));
      if (parsed.message && parsed.message !== "Internal server error") {
        return `${i18nMsg}\n[${parsed.code}] ${parsed.message}`;
      }
      return i18nMsg;
    }
  }

  if (e.status === 401) return t(k("auth.invalidCredentials"), "Invalid credentials");
  if (e.status === 404) return t(k("api.notFound"), "Not found");
  if (e.status === 409) return t(k("api.conflict"), "Conflict");
  if (e.status === 429) return t(k("api.rateLimited"), "Too many requests");
  if (e.status && e.status >= 500) {
    let msg = t(k("network.serverError"), "Server error");
    if (e.body) {
      try {
        const parsed = JSON.parse(e.body);
        if (parsed.error) msg += `\n[${parsed.error}] ${parsed.message || ""}`;
      } catch {
        if (e.body.length <= 200) msg += `\n${e.body}`;
      }
    }
    return msg;
  }

  return t(k("generic.unknown"));
}
