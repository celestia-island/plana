/**
 * The shared error-message funnel for the celestia webuis.
 *
 * `resolveErrorMessage` turns a thrown value into a user-facing i18n
 * string; both shittim-chest and celestia-erp grew a copy of that pipeline
 * and by 2026-11-12 they had drifted (an uncatalogued wire code masked the
 * HTTP-status diagnosis in one, not the other; the code tables and the
 * detail rendering answered the same question differently). This module is
 * the one implementation of the pipeline; the per-app tables and the
 * transport-specific branches are injected through
 * [`createErrorFunnel`]'s options.
 *
 * The pipeline, in order (each step returns or falls through):
 *
 * 1. `null`/`undefined` — the generic unknown headline.
 * 2. a network failure (message matches [`NETWORK_FAILURE_PATTERNS`]) —
 *    "connection lost", the browser's own words for a dead transport.
 * 3. the app's own `classify` hook (RpcError kinds, raw-string invokes …);
 *    returning a branch short-circuits the core.
 * 4. a raw string rejection (desktop bridges reject the RAW string of the
 *    native `Err(String)`): localized when the `localizeText` hook knows
 *    it, else it headlines itself — its own words beat "something went
 *    wrong".
 * 5. a `{ body }` object whose body parses as `{ error, message }`: a
 *    CATALOGUED code owns the headline (with `[code] message` as the
 *    detail line when the server authored one); an UNCATALOGUED code
 *    falls through to the status mapping instead of claiming the generic
 *    headline — the HTTP status is the better diagnosis and a wrong-password
 *    401 must not read as "no permission".
 * 6. the HTTP status mapping — 401/404/409/429 and the 5xx family, with
 *    the body's own words as the detail line when they are short enough.
 * 7. a plain `Error.message`: localized when a table knows the literal,
 *    else the message headlines itself (uncatalogued transport prose is
 *    the diagnosis, not noise).
 * 8. the generic unknown fallback — with a hard-coded English fallback so
 *    a pre-i18n-bootstrap failure renders a sentence, not a raw key.
 */

/** Both `t` flavors in the family: hikari's merged-scope `t` (with an
 *  optional in-band fallback) and vue-i18n's composer `t` (variadic). */
export type TranslateFn =
  | ((key: string, fallback?: string) => string)
  | ((key: string, ...args: unknown[]) => string);

/** A `{ error, message }` wire-error body. */
export interface ServerErrorBody {
  code: string;
  message: string;
}

/** The browser's own words for a dead transport, lowercased for matching. */
export const NETWORK_FAILURE_PATTERNS: readonly string[] = [
  "failed to fetch",
  "networkerror",
  "load failed",
  "err_empty_response",
];

/** Does the thrown value's message look like a network-level failure? */
export function isNetworkFailureMessage(err: unknown): boolean {
  if (typeof err !== "object" || err === null) return false;
  const msg = (err as { message?: unknown }).message;
  if (typeof msg !== "string") return false;
  const lower = msg.toLowerCase();
  return NETWORK_FAILURE_PATTERNS.some((p) => lower.includes(p));
}

/** Parse a server error body `{ error, message }`. Null when the string
 *  is not JSON or carries no string `error`; a body with no `message`
 *  yields an empty one (both webuis read `message` only — the funnel
 *  never guesses at alternative spellings). */
export function parseServerErrorBody(body: string): ServerErrorBody | null {
  try {
    const parsed = JSON.parse(body);
    if (parsed && typeof parsed.error === "string") {
      return { code: parsed.error, message: parsed.message || "" };
    }
  } catch {
    // Not JSON — the status mapping owns the diagnosis.
  }
  return null;
}

/** The `<code>: ` prefix of a server-authored message: a lowercase
 *  snake_case identifier followed by `": "`. Prose, capitalized words or
 *  no space are not a code. */
export function embeddedErrorCode(message: string): string | null {
  const match = /^([a-z][a-z0-9_]*): /.exec(message);
  return match?.[1] ?? null;
}

/** A resolved error split into its presentation halves: the localized
 *  cause (may carry line breaks — catalogs author per-line copy) and the
 *  muted technical context below it. */
export interface ResolvedErrorMessage {
  headline: string;
  detail: string;
}

/** One app-specific branch of the pipeline (step 3). */
export interface FunnelBranch {
  headline: string;
  detail?: string;
}

/** What the app injects: its tables and its transport-specific branches. */
export interface ErrorFunnelOptions {
  /** i18n key prefix; the family default is the shared `plana::errors`
   *  mount. An app reusing its own catalog passes e.g. `"errors"`. */
  keyPrefix?: string;
  /** The wire-code table (`invalid_token → api.invalidToken`, relative to
   *  the prefix). Required — an app with no codes passes `{}`. */
  codeToI18n: Record<string, string>;
  /** Exact-match table for stable server prose arriving WITHOUT a code —
   *  keys are the server's own strings (a backend audit is the source of
   *  truth). Unmatched prose keeps its raw text by design. */
  messageToI18n?: Record<string, string>;
  /** App branches that run before the core pipeline (RpcError kinds,
   *  semantic error paths …). Return null to fall through. */
  classify?: (
    t: TranslateFn,
    err: unknown,
    key: (relKey: string, fallback?: string) => string,
  ) => FunnelBranch | null;
  /** Localize known server prose (static catalogs, embedded codes).
   *  Returns null when the text is unknown — the caller keeps raw text,
   *  so an unmapped string can never blank a message. */
  localizeText?: (t: TranslateFn, text: string, keyPrefix: string) => string | null;
}

/** The built funnel. */
export interface ErrorFunnel {
  resolveParts(t: TranslateFn, err: unknown): ResolvedErrorMessage;
  resolve(t: TranslateFn, err: unknown): string;
  /** The catalog key for a wire code (relative to the prefix), or the
   *  generic unknown — for callers branching on precise causes. */
  serverErrorToI18nKey(code: string): string;
  /** Whether a wire code has a real catalog entry (vs the fallback) —
   *  callers use it to decide code vs status as the diagnosis. */
  hasCatalogsForServerCode(code: string): boolean;
}

/** Build the funnel. The tables are captured; the options are not kept. */
export function createErrorFunnel(options: ErrorFunnelOptions): ErrorFunnel {
  const keyPrefix = options.keyPrefix ?? "plana::errors";
  const k = (relKey: string) => `${keyPrefix}.${relKey}`;
  const say = (t: TranslateFn, relKey: string, fallback?: string) =>
    (t as (key: string, fb?: string) => string)(`${keyPrefix}.${relKey}`, fallback);

  const serverErrorToI18nKey = (code: string): string =>
    options.codeToI18n[code] || "generic.unknown";

  const hasCatalogsForServerCode = (code: string): boolean =>
    Object.prototype.hasOwnProperty.call(options.codeToI18n, code);

  const resolveParts = (t: TranslateFn, err: unknown): ResolvedErrorMessage => {
    // 1. Nothing thrown-shaped at all.
    if (!err) {
      return { headline: say(t, "generic.unknown", "Something went wrong"), detail: "" };
    }

    // 2. The browser's own words for a dead transport.
    if (typeof err === "object" && err !== null && isNetworkFailureMessage(err)) {
      return { headline: say(t, "network.connectionLost", "Connection lost"), detail: "" };
    }

    // 3. The app's own branches (RpcError kinds, error paths …).
    if (options.classify) {
      const branch = options.classify(t, err, k);
      if (branch) return { headline: branch.headline, detail: branch.detail ?? "" };
    }

    // 4. Desktop bridges reject the RAW string of the native Err(String).
    if (typeof err === "string" && err.trim()) {
      const localized = options.localizeText?.(t, err.trim(), keyPrefix);
      if (localized) {
        return {
          headline: say(t, "generic.unknown", "An unexpected error occurred"),
          detail: localized,
        };
      }
      return { headline: err.trim(), detail: "" };
    }

    if (typeof err !== "object" || err === null) {
      return { headline: say(t, "generic.unknown", "Something went wrong"), detail: "" };
    }

    const e = err as { status?: number; body?: string; message?: string };

    // 5. A wire-error body: a catalogued code owns the headline; an
    //    uncatalogued one yields to the status mapping below — the
    //    generic unknown would mask the better diagnosis (dogfood
    //    2026-10-22: a 404 `{"error":"not found"}` read as "an
    //    unexpected error occurred" instead of "not found").
    if (e.body && typeof e.body === "string") {
      const parsed = parseServerErrorBody(e.body);
      if (parsed) {
        // The login endpoint reports a wrong password as 401
        // `{"error":"unauthorized","message":"Invalid username or
        //  password"}` — the permission-flavored mapping would mislead;
        // the credentials cause wins when the server says so.
        const isBadCredentials =
          parsed.code === "unauthorized" &&
          /invalid username or password/i.test(parsed.message);
        if ((isBadCredentials || hasCatalogsForServerCode(parsed.code))) {
          const relKey = isBadCredentials ? "auth.invalidCredentials" : serverErrorToI18nKey(parsed.code);
          const headline = say(t, relKey);
          // The bad-credentials override IS the server's message;
          // appending "[unauthorized] Invalid username or password"
          // would repeat it and leak the wire tag as noise.
          if (isBadCredentials) return { headline, detail: "" };
          if (parsed.message && parsed.message !== "Internal server error") {
            return { headline, detail: `[${parsed.code}] ${parsed.message}` };
          }
          return { headline, detail: "" };
        }
        // Uncatalogued: fall through to the status mapping.
      }
    }

    // 6. The HTTP status mapping.
    if (e.status === 401) return { headline: say(t, "auth.invalidCredentials", "Invalid credentials"), detail: "" };
    if (e.status === 404) return { headline: say(t, "api.notFound", "Not found"), detail: "" };
    if (e.status === 409) return { headline: say(t, "api.conflict", "Conflict"), detail: "" };
    if (e.status === 429) return { headline: say(t, "api.rateLimited", "Too many requests"), detail: "" };
    if (e.status && e.status >= 500) {
      const headline = say(t, "network.serverError", "Server error");
      if (e.body) {
        try {
          const parsed = JSON.parse(e.body);
          if (parsed.error) return { headline, detail: `[${parsed.error}] ${parsed.message || ""}` };
        } catch {
          if (e.body.length <= 200) return { headline, detail: e.body };
        }
      }
      return { headline, detail: "" };
    }

    // 7. A plain Error message: a known literal localizes (and demotes to
    //    the detail line — the generic headline told nobody anything);
    //    uncatalogued prose headlines itself, because it IS the diagnosis.
    const message = (err as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) {
      const localized = options.localizeText?.(t, message.trim(), keyPrefix)
        ?? (options.messageToI18n && options.messageToI18n[message.trim()]
          ? say(t, options.messageToI18n[message.trim()] ?? "", message.trim())
          : null);
      if (localized) {
        return {
          headline: say(t, "generic.unknown", "An unexpected error occurred"),
          detail: localized,
        };
      }
      return { headline: message.trim(), detail: "" };
    }

    // 8. The fallback — its English matters before the i18n instance is
    //    registered (the bridge returns the raw key argument).
    return { headline: say(t, "generic.unknown", "An unexpected error occurred"), detail: "" };
  };

  return {
    resolveParts,
    resolve: (t, err) => {
      const { headline, detail } = resolveParts(t, err);
      return detail ? `${headline}\n${detail}` : headline;
    },
    serverErrorToI18nKey,
    hasCatalogsForServerCode,
  };
}
