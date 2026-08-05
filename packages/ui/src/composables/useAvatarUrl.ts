import { computed, type Ref } from "vue";

export interface AvatarUrlUser {
  avatar_url?: string;
}

/**
 * Resolve a user's avatar URL, cache-busting static avatar paths so a
 * changed upload is picked up immediately.
 */
export function useAvatarUrl(user: Ref<AvatarUrlUser | null>, bust?: Ref<number>) {
  return computed(() => {
    const u = user.value;
    if (!u) return "";

    if (u.avatar_url) {
      const base = u.avatar_url;
      if (base.startsWith("/static/avatars/")) {
        const t = bust?.value ?? Date.now();
        return `${base}?t=${t}`;
      }
      return base;
    }

    return "";
  });
}
