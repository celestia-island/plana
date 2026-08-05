import { describe, expect, it } from "vitest";

import { parseServerErrorBody, resolveErrorMessage, serverErrorToI18nKey } from "../src/utils/errors";

const t = (k: string) => k;

describe("serverErrorToI18nKey", () => {
  it("maps known error codes", () => {
    expect(serverErrorToI18nKey("invalid_credentials")).toBe(
      "auth.invalidCredentials",
    );
    expect(serverErrorToI18nKey("not_found")).toBe("api.notFound");
    expect(serverErrorToI18nKey("rate_limited")).toBe("api.rateLimited");
    expect(serverErrorToI18nKey("invalid_token")).toBe("api.invalidToken");
  });

  it("returns generic.unknown for unmapped codes", () => {
    expect(serverErrorToI18nKey("nonexistent_code")).toBe("generic.unknown");
  });
});

describe("parseServerErrorBody", () => {
  it("parses valid JSON with error field", () => {
    const result = parseServerErrorBody(
      JSON.stringify({ error: "not_found", message: "Item missing" }),
    );
    expect(result).toEqual({ code: "not_found", message: "Item missing" });
  });

  it("returns null for JSON without error field", () => {
    expect(parseServerErrorBody('{"msg":"ok"}')).toBeNull();
  });

  it("returns null for invalid JSON", () => {
    expect(parseServerErrorBody("not json")).toBeNull();
  });

  it("defaults message to empty string when absent", () => {
    const result = parseServerErrorBody(
      JSON.stringify({ error: "db_error" }),
    );
    expect(result).toEqual({ code: "db_error", message: "" });
  });
});

describe("resolveErrorMessage", () => {
  it("resolves server error body with the plana key prefix", () => {
    const msg = resolveErrorMessage(t, {
      body: JSON.stringify({ error: "not_found", message: "Item missing" }),
    });
    expect(msg).toBe("plana::errors.api.notFound\n[not_found] Item missing");
  });

  it("resolves 401 status without body", () => {
    expect(resolveErrorMessage(t, { status: 401 })).toBe(
      "plana::errors.auth.invalidCredentials",
    );
  });

  it("resolves 429 status", () => {
    expect(resolveErrorMessage(t, { status: 429 })).toBe(
      "plana::errors.api.rateLimited",
    );
  });

  it("returns unknown for null error", () => {
    expect(resolveErrorMessage(t, null)).toBe("plana::errors.generic.unknown");
  });

  it("maps network-failure messages to the connection-lost key", () => {
    expect(resolveErrorMessage(t, { message: "Failed to fetch" })).toBe(
      "plana::errors.network.connectionLost",
    );
  });
});
