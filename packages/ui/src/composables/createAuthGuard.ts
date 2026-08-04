export interface AuthGuardOptions {
  loginRoute: string;
  homeRoute: string;
  setupRoute: string;
  registerRoute?: string;
  useAuthStore: () => {
    isAuthenticated: boolean;
    user: unknown;
    tryRestoreSession: () => Promise<void>;
    checkSetup: () => Promise<{ needs_setup: boolean; registration_enabled?: boolean }>;
    // The guard only awaits the call — the store's `User` result is unused.
    fetchUser: () => Promise<unknown>;
  };
  onLazyLoadError?: (error: Error, target: string) => void;
  /** Permissions loader; optional — skipped when omitted. */
  permissions?: {
    loaded: boolean;
    fetch: () => Promise<void>;
  };
}

interface GuardRoute {
  name?: string | symbol | null;
  fullPath: string;
  meta: Record<string, unknown>;
}

interface GuardRouter {
  beforeEach: (fn: (to: GuardRoute) => unknown) => void;
  onError?: (fn: (error: unknown, to: GuardRoute) => void) => void;
}

export function createAuthGuard(router: GuardRouter, opts: AuthGuardOptions) {
  let sessionRestorePromise: Promise<void> | null = null;
  let setupChecked = false;

  router.beforeEach(async (to: GuardRoute) => {
    const auth = opts.useAuthStore();
    const permStore = opts.permissions;

    if (!auth.isAuthenticated) {
      if (!sessionRestorePromise) {
        sessionRestorePromise = auth.tryRestoreSession().catch(() => {});
      }
      await sessionRestorePromise;
    }

    if (!setupChecked && !auth.isAuthenticated) {
      setupChecked = true;
      try {
        const result = await auth.checkSetup();
        if (result.needs_setup && to.name !== opts.setupRoute) {
          return { name: opts.setupRoute };
        }
        if (!result.needs_setup && to.name === opts.setupRoute) {
          return { name: opts.loginRoute };
        }
        if (
          !result.registration_enabled &&
          opts.registerRoute &&
          to.name === opts.registerRoute
        ) {
          return { name: opts.loginRoute };
        }
      } catch {
        // ignore
      }
    }

    if (to.meta.requiresAuth !== false && !auth.isAuthenticated) {
      const redirect = to.fullPath !== "/" ? to.fullPath : undefined;
      return { name: opts.loginRoute, query: redirect ? { redirect } : undefined };
    }

    if (auth.isAuthenticated && permStore && !permStore.loaded) {
      await permStore.fetch();
    }

    const publicRoutes = [opts.loginRoute, opts.setupRoute];
    if (opts.registerRoute) publicRoutes.push(opts.registerRoute);
    if (publicRoutes.includes(to.name as string) && auth.isAuthenticated) {
      return "/";
    }

    if (auth.isAuthenticated && !auth.user) {
      await auth.fetchUser();
    }
  });

  if (opts.onLazyLoadError) {
    router.onError((error: unknown, to: GuardRoute) => {
      opts.onLazyLoadError!(error as Error, to.fullPath);
    });
  }
}
