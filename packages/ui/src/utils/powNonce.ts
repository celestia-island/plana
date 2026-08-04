import { solvePow, type PowChallenge } from "./pow";

export type ChallengeDescriptor =
  | { type: "pow"; seed: string; bits: number }
  | { type: "captcha"; provider: string; sitekey: string; script_url: string }
  | null;

/**
 * Fetch the anti-bot challenge descriptor from a public `/health`-style
 * endpoint (upstreamed from shittim-chest's api/pow.ts).
 */
export async function fetchChallenge(
  baseUrl: string,
  fetchFn: typeof fetch = fetch,
): Promise<ChallengeDescriptor> {
  const resp = await fetchFn(`${baseUrl}/health`, { credentials: "same-origin" });
  if (!resp.ok) return null;
  const json = (await resp.json().catch(() => null)) as { challenge?: ChallengeDescriptor } | null;
  return json?.challenge ?? null;
}

/**
 * Negotiate a single-use X-Nonce by solving the PoW challenge (or passing
 * a captcha token). Returns the nonce string for the X-Nonce header, or
 * undefined when the backend rejects the exchange.
 */
export async function negotiateNonce(
  baseUrl: string,
  opts?: {
    captchaToken?: string;
    solve?: (challenge: PowChallenge) => Promise<number>;
    fetchFn?: typeof fetch;
  },
): Promise<string | undefined> {
  const fetchFn = opts?.fetchFn ?? fetch;
  let body: Record<string, unknown> | undefined;
  if (opts?.captchaToken) {
    body = { type: "captcha", token: opts.captchaToken };
  } else {
    const challenge = await fetchChallenge(baseUrl, fetchFn);
    if (challenge?.type === "pow") {
      const solve = opts?.solve ?? solvePow;
      const counter = await solve({ seed: challenge.seed, bits: challenge.bits });
      body = { type: "pow", seed: challenge.seed, counter };
    }
  }
  const resp = await fetchFn(`${baseUrl}/auth/nonce`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: body ? JSON.stringify(body) : "{}",
    credentials: "same-origin",
  });
  if (!resp.ok) return undefined;
  const json = (await resp.json().catch(() => null)) as { nonce?: string } | null;
  return json?.nonce;
}
