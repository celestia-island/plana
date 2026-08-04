import { v4, v5, v7 } from "uuid";

/**
 * Random UUID (v4). The `uuid` package uses `crypto.getRandomValues`
 * internally, which is available in ALL contexts — unlike ad-hoc
 * `crypto.randomUUID()` calls that are undefined outside a secure context
 * (HTTPS or localhost) and throw on load over plain HTTP on a
 * non-localhost origin.
 */
export function uuid(): string {
  return v4();
}

/** Time-ordered UUID (v7). */
export function uuidv7(): string {
  return v7();
}

/**
 * Deterministic UUIDv5 (SHA-1). The `uuid` package bundles its own SHA-1
 * implementation (it does NOT depend on `crypto.subtle`), so this works in
 * non-secure contexts too.
 *
 * NOTE: argument order is `(namespace, name)` — the `uuid` package's `v5`
 * is `(name, namespace)`, so the arguments are swapped here. Kept `async`
 * because existing callers await it (await on a string is a no-op).
 */
export async function uuidv5(
  namespaceUuid: string,
  name: string,
): Promise<string> {
  return v5(name, namespaceUuid);
}
