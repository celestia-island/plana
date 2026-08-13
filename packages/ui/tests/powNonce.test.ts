import { describe, expect, it, vi } from "vitest";

import { fetchChallenge, negotiateNonce } from "../src/utils/powNonce";

interface ResponseLike {
  ok: boolean;
  status: number;
  json: () => Promise<unknown>;
}

function resp(ok: boolean, body: unknown): Response {
  return {
    ok,
    status: ok ? 200 : 400,
    json: async () => body,
  } as unknown as Response;
}

function fetchSpy() {
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  const fn = vi.fn(
    async (url: string | URL | Request, init?: RequestInit): Promise<Response> => {
      calls.push({ url: String(url), init });
      if (String(url).endsWith("/health")) {
        return resp(true, {
          challenge: { type: "pow", seed: "seed-1", bits: 8 },
        });
      }
      return resp(true, { nonce: "nonce-1" });
    },
  );
  return { fn: fn as unknown as typeof fetch, calls };
}

describe("fetchChallenge", () => {
  it("returns the descriptor when the gate is active", async () => {
    const { fn } = fetchSpy();
    const challenge = await fetchChallenge("http://x", fn);
    expect(challenge).toEqual({ type: "pow", seed: "seed-1", bits: 8 });
  });

  it("returns null when the backend has no challenge configured", async () => {
    const fn = vi.fn(async () => resp(true, { status: "ok" })) as unknown as typeof fetch;
    const challenge = await fetchChallenge("http://x", fn);
    expect(challenge).toBeNull();
  });

  it("returns undefined when the fetch throws", async () => {
    const fn = vi.fn(async () => {
      throw new Error("network");
    }) as unknown as typeof fetch;
    const challenge = await fetchChallenge("http://x", fn);
    expect(challenge).toBeUndefined();
  });

  it("returns undefined on a non-ok response", async () => {
    const fn = vi.fn(async () => resp(false, {})) as unknown as typeof fetch;
    const challenge = await fetchChallenge("http://x", fn);
    expect(challenge).toBeUndefined();
  });
});

describe("negotiateNonce", () => {
  it("solves the pow challenge and posts the solution", async () => {
    const { fn, calls } = fetchSpy();
    const solve = vi.fn(async () => 42);
    const nonce = await negotiateNonce("http://x", { solve, fetchFn: fn });
    expect(nonce).toBe("nonce-1");
    expect(solve).toHaveBeenCalledWith({ seed: "seed-1", bits: 8 });
    const post = calls.find((c) => c.url.endsWith("/auth/nonce"));
    expect(post).toBeDefined();
    expect(JSON.parse(post!.init!.body as string)).toEqual({
      type: "pow",
      seed: "seed-1",
      counter: 42,
    });
  });

  it("posts an empty body when the gate is disabled", async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = [];
    const fn = vi.fn(async (url: string | URL | Request, init?: RequestInit) => {
      calls.push({ url: String(url), init });
      if (String(url).endsWith("/health")) return resp(true, {});
      return resp(true, { nonce: "nonce-2" });
    }) as unknown as typeof fetch;
    const nonce = await negotiateNonce("http://x", { fetchFn: fn });
    expect(nonce).toBe("nonce-2");
    const post = calls.find((c) => c.url.endsWith("/auth/nonce"));
    expect(post).toBeDefined();
    expect(post!.init!.body).toBeUndefined();
  });

  it("fails cleanly for a captcha challenge without a token", async () => {
    const calls: string[] = [];
    const fn = vi.fn(async (url: string | URL | Request) => {
      calls.push(String(url));
      return resp(true, {
        challenge: {
          type: "captcha",
          provider: "turnstile",
          sitekey: "k",
          script_url: "s",
        },
      });
    }) as unknown as typeof fetch;
    const nonce = await negotiateNonce("http://x", { fetchFn: fn });
    expect(nonce).toBeUndefined();
    expect(calls).not.toContainEqual(expect.stringContaining("/auth/nonce"));
  });

  it("passes a captcha token through when provided", async () => {
    const calls: Array<{ url: string; init?: RequestInit }> = [];
    const fn = vi.fn(async (url: string | URL | Request, init?: RequestInit) => {
      calls.push({ url: String(url), init });
      return resp(true, { nonce: "nonce-3" });
    }) as unknown as typeof fetch;
    const nonce = await negotiateNonce("http://x", {
      captchaToken: "tok",
      fetchFn: fn,
    });
    expect(nonce).toBe("nonce-3");
    const post = calls.find((c) => c.url.endsWith("/auth/nonce"));
    expect(JSON.parse(post!.init!.body as string)).toEqual({
      type: "captcha",
      token: "tok",
    });
  });

  it("returns undefined when the challenge fetch fails", async () => {
    const fn = vi.fn(async () => {
      throw new Error("boom");
    }) as unknown as typeof fetch;
    const nonce = await negotiateNonce("http://x", { fetchFn: fn });
    expect(nonce).toBeUndefined();
  });
});
