import { watch, type Ref, type ComputedRef } from "vue";

export function useRouteTitle(
  siteName: string,
  currentRoute: ComputedRef<string> | Ref<string>,
  routeMap: Record<string, string>,
  t: (key: string, fallback?: string) => string,
): void {
  watch(currentRoute, (route) => {
    const key = routeMap[route];
    if (key) {
      document.title = `${t(key)} \u2014 ${siteName}`;
    } else {
      document.title = siteName;
    }
  }, { immediate: true });
}

export function usePageTitle(
  pageTitle: ComputedRef<string> | Ref<string>,
  siteName: string,
): void {
  watch(pageTitle, (title) => {
    if (title) {
      document.title = `${title} \u2014 ${siteName}`;
    } else {
      document.title = siteName;
    }
  }, { immediate: true });
}
