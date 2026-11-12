import { describe, expect, it, vi } from "vitest";

import {
  createErrorFunnel,
  embeddedErrorCode,
  isNetworkFailureMessage,
  NETWORK_FAILURE_PATTERNS,
  parseServerErrorBody,
  type TranslateFn,
} from "./error_funnel";

/** A `t` that echoes the KEY — assertions pin the lookup itself, while
 *  `calls` records which fallback rode along. The fallback-specific tests
 *  assert on `calls` (or on the English the fallback carries). */
function recordingT() {
  const calls: Array<{ key: string; fallback?: string }> = [];
  const t = ((key: string, fallback?: string) => {
    calls.push({ key, fallback });
    return key;
  }) as TranslateFn;
  return { t, calls };
}

/** A minimal catalog-aware `t`: resolves known keys, falls back otherwise. */
function catalogT(catalog: Record<string, string>): TranslateFn {
  return ((key: string, fallback?: string) =>
    catalog[key] ?? fallback ?? key) as TranslateFn;
}

describe("network failure detection", () => {
  it("matches every documented pattern, case-insensitively", () => {
    for (const pattern of NETWORK_FAILURE_PATTERNS) {
      expect(isNetworkFailureMessage({ message: pattern })).toBe(true);
      expect(isNetworkFailureMessage({ message: pattern.toUpperCase() })).toBe(true);
    }
  });

  it("does not match ordinary prose or non-string messages", () => {
    expect(isNetworkFailureMessage({ message: "upstream said no" })).toBe(false);
    expect(isNetworkFailureMessage({ message: 42 })).toBe(false);
    expect(isNetworkFailureMessage(null)).toBe(false);
  });
});

describe("parseServerErrorBody", () => {
  it("parses { error, message }; alternative spellings are NOT guessed", () => {
    expect(parseServerErrorBody('{"error":"rate_limited","message":"slow down"}')).toEqual({
      code: "rate_limited",
      message: "slow down",
    });
    // Both webuis read `message` only — a `msg` body yields an empty
    // message rather than a guessed alternative (the funnel never
    // invents fields the server did not send).
    expect(parseServerErrorBody('{"error":"x","msg":"y"}')).toEqual({ code: "x", message: "" });
  });

  it("returns null for non-JSON, missing error, or non-string error", () => {
    expect(parseServerErrorBody("not json")).toBeNull();
    expect(parseServerErrorBody('{"message":"no code"}')).toBeNull();
    expect(parseServerErrorBody('{"error":42}')).toBeNull();
    expect(parseServerErrorBody('{"error":""}')).toEqual({ code: "", message: "" });
  });
});

describe("embeddedErrorCode", () => {
  it("extracts a snake_case prefix followed by ': '", () => {
    expect(embeddedErrorCode("rate_limited: too fast")).toBe("rate_limited");
    expect(embeddedErrorCode("security_required: step up")).toBe("security_required");
  });

  it("rejects prose, capitals, and missing separators", () => {
    expect(embeddedErrorCode("Something failed")).toBeNull();
    expect(embeddedErrorCode("NotACode: x")).toBeNull();
    expect(embeddedErrorCode("rate_limited:too fast")).toBeNull();
    expect(embeddedErrorCode("")).toBeNull();
  });
});

describe("createErrorFunnel", () => {
  const funnel = createErrorFunnel({
    codeToI18n: {
      rate_limited: "api.rateLimited",
      invalid_token: "api.invalidToken",
      unauthorized: "auth.unauthorized",
    },
    messageToI18n: { "database error": "server.databaseError" },
  });

  it("answers nothing thrown with the generic unknown (and its fallback)", () => {
    const { t, calls } = recordingT();
    expect(funnel.resolveParts(t, null)).toEqual({ headline: "plana::errors.generic.unknown", detail: "" });
    expect(calls[0]).toEqual({ key: "plana::errors.generic.unknown", fallback: "Something went wrong" });
  });

  it("resolves a network failure to connection lost", () => {
    const { t, calls } = recordingT();
    expect(funnel.resolveParts(t, { message: "Failed to fetch" })).toEqual({
      headline: "plana::errors.network.connectionLost",
      detail: "",
    });
    expect(calls[0].key).toBe("plana::errors.network.connectionLost");
  });

  it("lets a catalogued wire code own the headline, with the server's words as detail", () => {
    const { t } = recordingT();
    expect(
      funnel.resolveParts(t, { body: '{"error":"rate_limited","message":"slow down"}' }),
    ).toEqual({
      headline: "plana::errors.api.rateLimited",
      detail: "[rate_limited] slow down",
    });
  });

  it("drops the detail line for the internal-server-error boilerplate", () => {
    const { t } = recordingT();
    expect(
      funnel.resolveParts(t, { body: '{"error":"rate_limited","message":"Internal server error"}' }),
    ).toEqual({ headline: "plana::errors.api.rateLimited", detail: "" });
  });

  it("lets an uncatalogued code fall through to the status diagnosis", () => {
    // The 2026-10-22 dogfood: a 404 {"error":"not found"} used to read as
    // "an unexpected error occurred" — the status is the better answer.
    const { t } = recordingT();
    expect(funnel.resolveParts(t, { status: 404, body: '{"error":"not found"}' })).toEqual({
      headline: "plana::errors.api.notFound",
      detail: "",
    });
  });

  it("prefers the credentials cause when the server says bad password", () => {
    const { t } = recordingT();
    expect(
      funnel.resolveParts(t, {
        status: 401,
        body: '{"error":"unauthorized","message":"Invalid username or password"}',
      }),
    ).toEqual({ headline: "plana::errors.auth.invalidCredentials", detail: "" });
  });

  it("maps every documented status", () => {
    const { t } = recordingT();
    expect(funnel.resolveParts(t, { status: 401 }).headline).toBe("plana::errors.auth.invalidCredentials");
    expect(funnel.resolveParts(t, { status: 404 }).headline).toBe("plana::errors.api.notFound");
    expect(funnel.resolveParts(t, { status: 409 }).headline).toBe("plana::errors.api.conflict");
    expect(funnel.resolveParts(t, { status: 429 }).headline).toBe("plana::errors.api.rateLimited");
    expect(funnel.resolveParts(t, { status: 503 }).headline).toBe("plana::errors.network.serverError");
  });

  it("carries the 5xx body's own words as the detail line", () => {
    const { t } = recordingT();
    expect(funnel.resolveParts(t, { status: 500, body: '{"error":"db_error","message":"boom"}' })).toEqual({
      headline: "plana::errors.network.serverError",
      detail: "[db_error] boom",
    });
    // Non-JSON bodies ride along only when short.
    expect(funnel.resolveParts(t, { status: 500, body: "short text" }).detail).toBe("short text");
    expect(funnel.resolveParts(t, { status: 500, body: "x".repeat(201) }).detail).toBe("");
  });

  it("localizes a known plain-message literal into the detail line", () => {
    const { t } = recordingT();
    expect(funnel.resolveParts(t, new Error("database error"))).toEqual({
      headline: "plana::errors.generic.unknown",
      detail: "plana::errors.server.databaseError",
    });
  });

  it("lets an unknown plain message headline itself", () => {
    const { t } = recordingT();
    expect(funnel.resolveParts(t, new Error("the upstream exploded"))).toEqual({
      headline: "the upstream exploded",
      detail: "",
    });
  });

  it("runs the classify hook before the core pipeline", () => {
    const withHook = createErrorFunnel({
      codeToI18n: {},
      classify: (t, err, k) =>
        (err as { name?: string }).name === "RpcError"
          ? {
              headline: (t as (key: string, fb?: string) => string)(k("rpc.timeout"), "The request timed out"),
              detail: "method: x",
            }
          : null,
    });
    const { t, calls } = recordingT();
    expect(withHook.resolveParts(t, { name: "RpcError" })).toEqual({
      headline: "plana::errors.rpc.timeout",
      detail: "method: x",
    });
    expect(calls[0]).toEqual({ key: "plana::errors.rpc.timeout", fallback: "The request timed out" });
    // Falling through returns null-branch behavior.
    expect(withHook.resolveParts(t, null).headline).toBe("plana::errors.generic.unknown");
  });

  it("headlines a raw string rejection in its own words", () => {
    const { t } = recordingT();
    expect(funnel.resolveParts(t, "gateway exploded")).toEqual({
      headline: "gateway exploded",
      detail: "",
    });
  });

  it("demotes a localizeText hit on a raw string to the detail line", () => {
    const withLocalize = createErrorFunnel({
      codeToI18n: {},
      localizeText: (t, text) => (text === "gateway exploded" ? "translated sentence" : null),
    });
    const { t } = recordingT();
    expect(withLocalize.resolveParts(t, "gateway exploded")).toEqual({
      headline: "plana::errors.generic.unknown",
      detail: "translated sentence",
    });
  });

  it("honours a custom key prefix", () => {
    const prefixed = createErrorFunnel({ keyPrefix: "errors", codeToI18n: { x: "y" } });
    const { t, calls } = recordingT();
    prefixed.resolveParts(t, null);
    expect(calls[0].key).toBe("errors.generic.unknown");
  });

  it("resolve() joins headline and detail on one newline", () => {
    const { t } = recordingT();
    expect(funnel.resolve(t, { body: '{"error":"rate_limited","message":"slow down"}' })).toBe(
      "plana::errors.api.rateLimited\n[rate_limited] slow down",
    );
  });

  it("exposes the table probes callers branch on", () => {
    expect(funnel.serverErrorToI18nKey("rate_limited")).toBe("api.rateLimited");
    expect(funnel.serverErrorToI18nKey("unknown_code")).toBe("generic.unknown");
    expect(funnel.hasCatalogsForServerCode("rate_limited")).toBe(true);
    expect(funnel.hasCatalogsForServerCode("unknown_code")).toBe(false);
  });

  it("works with a catalog-resolving t without fallbacks", () => {
    const t = catalogT({
      "plana::errors.generic.unknown": "Etwas ist schiefgelaufen",
      "plana::errors.api.rateLimited": "Zu viele Anfragen",
    });
    expect(funnel.resolve(t, { status: 429 })).toBe("Zu viele Anfragen");
    expect(funnel.resolve(t, null)).toBe("Etwas ist schiefgelaufen");
  });

  it("does not consult the locale catalog for a fallback-only t mock", () => {
    // The fallback must ride the t() call itself: before the i18n bridge
    // is registered a bare key would otherwise render as the raw key.
    const spy = vi.fn((key: string, fallback?: string) => fallback ?? key);
    funnel.resolveParts(spy as unknown as TranslateFn, undefined);
    expect(spy).toHaveBeenCalledWith("plana::errors.generic.unknown", "Something went wrong");
  });
});
