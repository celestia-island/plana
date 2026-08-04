/**
 * Locale catalog scaffolding shared across webuis (upstreamed from
 * shittim-chest/arona).
 *
 * `createLocaleOptions` builds the language picker list for a supported
 * locale set; `loadLocaleMessages` eagerly imports a locale directory and
 * deep-merges the message files — the two patterns both webuis hand-wrote.
 */

export interface LocaleOption {
  code: string;
  labelKey: string;
}

export function createLocaleOptions(
  supported: readonly string[],
  labelKeyPrefix = "common.locale",
): LocaleOption[] {
  return supported.map((code) => ({ code, labelKey: `${labelKeyPrefix}.${code}` }));
}

type Messages = Record<string, unknown>;

function deepMerge(target: Record<string, unknown>, source: Record<string, unknown>): void {
  for (const [key, value] of Object.entries(source)) {
    if (
      value !== null &&
      typeof value === "object" &&
      !Array.isArray(value) &&
      typeof target[key] === "object" &&
      target[key] !== null &&
      !Array.isArray(target[key])
    ) {
      deepMerge(target[key] as Record<string, unknown>, value as Record<string, unknown>);
    } else {
      target[key] = value;
    }
  }
}

/** Eagerly load every message file in `glob` for the locale and merge. */
export function loadLocaleMessages(
  glob: Record<string, { default: unknown }>,
  locale: string,
  prefix = `./locales/${locale}/`,
): Messages {
  const merged: Messages = {};
  for (const [path, mod] of Object.entries(glob)) {
    if (!path.startsWith(prefix)) continue;
    const domain = mod.default;
    if (domain !== null && typeof domain === "object") {
      deepMerge(merged, domain as Record<string, unknown>);
    }
  }
  return merged;
}
