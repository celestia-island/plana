/**
 * Hashcash-style proof-of-work helpers (upstreamed from shittim-chest,
 * P5 auth-kit consolidation).
 *
 * Given a `{ seed, bits }` challenge, `solvePow` finds a `counter` such
 * that `SHA-256(UTF-8(seed) || UTF-8(decimal counter))` has at least
 * `bits` leading zero bits. The byte layout is part of the wire contract
 * and must match the verifying backend exactly.
 */

const BATCH = 64;

export interface PowChallenge {
  seed: string;
  bits: number;
}

export interface PowSolution {
  seed: string;
  bits: number;
  counter: number;
}

/**
 * Solve a PoW challenge via `crypto.subtle.digest` in batches (correct
 * everywhere, async per batch — the pure-logic path without a worker).
 */
export async function solvePow(
  seed: string,
  bits: number,
  onProgress?: (hashed: number) => void,
): Promise<number> {
  const encoder = new TextEncoder();
  const seedBytes = encoder.encode(seed);
  const digest = crypto.subtle ? crypto.subtle.digest.bind(crypto.subtle) : null;
  if (!digest) {
    throw new Error("crypto.subtle is unavailable in this context");
  }

  let base = 0;
  for (;;) {
    const batch = Math.min(BATCH, 1 << 20 - base.toString().length);
    const tasks: Array<Promise<{ counter: number; hash: Uint8Array }>> = [];
    for (let i = 0; i < batch; i++) {
      const counter = base + i;
      tasks.push(
        digest("SHA-256", concat(seedBytes, encoder.encode(String(counter)))).then(
          (buf) => ({ counter, hash: new Uint8Array(buf) }),
        ),
      );
    }
    const results = await Promise.all(tasks);
    for (const { counter, hash } of results) {
      if (leadingZeroBits(hash) >= bits) {
        return counter;
      }
    }
    base += batch;
    onProgress?.(base);
  }
}

function concat(a: Uint8Array, b: Uint8Array): Uint8Array {
  const out = new Uint8Array(a.length + b.length);
  out.set(a, 0);
  out.set(b, a.length);
  return out;
}

/** Number of leading zero bits of a 32-byte SHA-256 digest. */
export function leadingZeroBits(hash: Uint8Array): number {
  let count = 0;
  for (let i = 0; i < hash.length; i++) {
    const byte = hash[i];
    if (byte === 0) {
      count += 8;
    } else {
      count += Math.clz32(byte) - 24;
      break;
    }
  }
  return count;
}

/** Verify a solution against the challenge (async; mirrors the backend check). */
export async function verifyPow(
  challenge: PowChallenge,
  counter: number,
): Promise<boolean> {
  const encoder = new TextEncoder();
  const buf = await crypto.subtle.digest(
    "SHA-256",
    concat(encoder.encode(challenge.seed), encoder.encode(String(counter))),
  );
  return leadingZeroBits(new Uint8Array(buf)) >= challenge.bits;
}
