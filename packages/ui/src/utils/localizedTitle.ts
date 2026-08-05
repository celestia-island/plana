/** Locale-family fallback table used by `resolveLocalizedTitle`. */
export const LOCALE_FAMILY: Record<string, string[]> = {
  "zh-Hant": ["zh-Hans"],
  "zh-Hans": ["zh-Hant"],
};

/** Resolve the best available translation for the requested UI locale.
 *
 * Fallback order:
 *  1. Exact match: `translations[locale]`
 *  2. Same-family locale (e.g. zh-Hans ↔ zh-Hant)
 *  3. English (`en`)
 *  4. Any non-empty translation (first key found)
 *  5. `fallback` — the source/default title
 */
export function resolveLocalizedTitle(
  translations: Record<string, string> | undefined,
  locale: string,
  fallback: string,
): string {
  if (!translations) return fallback;

  const trimmed = (k: string) => (translations[k] ?? "").trim();
  const has = (k: string) => trimmed(k).length > 0;

  // 1. Exact match
  if (has(locale)) return trimmed(locale);

  // 2. Same-family
  const family = LOCALE_FAMILY[locale];
  if (family) {
    for (const code of family) {
      if (has(code)) return trimmed(code);
    }
  }

  // 3. English
  if (has("en")) return trimmed("en");

  // 4. Any non-empty translation
  for (const key of Object.keys(translations)) {
    if (has(key)) return trimmed(key);
  }

  // 5. Source title
  return fallback;
}
