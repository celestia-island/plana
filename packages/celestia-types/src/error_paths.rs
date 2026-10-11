//! Semantic error-path trees — the family's error identity (2026-10-11
//! user ruling, direction C).
//!
//! The JSON-RPC numeric error code is ALWAYS the -32000 placeholder for
//! application errors (the spec requires a code; the -32000..-32099 band
//! is its "implementation-defined server-errors" slot). The application's
//! error IDENTITY is a METHOD-STYLE PATH STRING (`Usage.Forbidden`,
//! `Scheduler.QueryFailed`), declared here as yuuka `derive_enum!`
//! trees — one per big namespace — and shipped to TypeScript consumers
//! through the generated `bindings/errorPaths.ts` (the export test in
//! this module regenerates it; `cargo test` is the generator, CI is the
//! drift gate).
//!
//! Wire shape: the path rides as the message prefix —
//! `"<Path.Here>: <human prose>"` — the channel every family client
//! already carries end-to-end (the plana rpc client transports the
//! message verbatim and discards the numeric code). Denials end in
//! `.Forbidden`.
//!
//! Family services host their domain trees HERE (the unified table) and
//! send errors through a typed `&impl AppErrorPath` helper so a bare
//! string path cannot compile (chest's `send_app_error` is the
//! reference). New namespaces get a new tree + a `*_PATHS` const + a
//! pin test + a writer entry — do NOT mint new numeric codes.

use yuuka::derive_enum;

/// The one code application errors carry. Placeholder by design — see
/// the module docs; the semantic identity lives in the path.
pub const APP_ERROR_CODE: i64 = -32000;

/// The typed channel for error paths: only the trees in this module
/// implement it, so a caller cannot pass a bare string (a
/// numeric-code-shaped "path" is a compile error, not a silent drift).
pub trait AppErrorPath {
    /// The method-style semantic path ("Usage.Forbidden").
    fn error_path(&self) -> &'static str;
}

// ── Usage (chest's usage.get seam) ───────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum UsageErrors {
        Forbidden,
        Internal,
        GroupByInvalid,
    }
);

impl UsageErrors {
    /// The arms draw from `USAGE_PATHS` slots BY DESIGN (R1 MAJOR-1's
    /// drift guard): a new variant forces a new match arm (compile
    /// error without one), and the per-tree pin test asserts the exact
    /// literal — going green REQUIRES the const to carry the new path.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Forbidden => USAGE_PATHS[0],
            Self::Internal => USAGE_PATHS[1],
            Self::GroupByInvalid => USAGE_PATHS[2],
        }
    }
}

impl AppErrorPath for UsageErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Usage tree, in declaration order (the TS export's
/// source of truth for this namespace).
pub const USAGE_PATHS: &[&str] = &["Usage.Forbidden", "Usage.Internal", "Usage.GroupByInvalid"];

// ── Scheduler (the arona metering plane — shared by usage.get and
//    chest's self-hosted registry) ────────────────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum SchedulerErrors {
        Unavailable,
        QueryFailed,
    }
);

impl SchedulerErrors {
    /// See UsageErrors::path — arms draw from `SCHEDULER_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Unavailable => SCHEDULER_PATHS[0],
            Self::QueryFailed => SCHEDULER_PATHS[1],
        }
    }
}

impl AppErrorPath for SchedulerErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Scheduler tree. The two paths must stay distinct:
/// the split exists so a scheduler-side HTTP 403 (stale admin token)
/// classifies as `QueryFailed`, never as a caller denial.
pub const SCHEDULER_PATHS: &[&str] = &["Scheduler.Unavailable", "Scheduler.QueryFailed"];

// ── Platform (the P86 platform-pinning gate) ─────────────────────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum PlatformErrors {
        Forbidden,
        IdInvalid,
        Internal,
    }
);

impl PlatformErrors {
    /// See UsageErrors::path — arms draw from `PLATFORM_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Forbidden => PLATFORM_PATHS[0],
            Self::IdInvalid => PLATFORM_PATHS[1],
            Self::Internal => PLATFORM_PATHS[2],
        }
    }
}

impl AppErrorPath for PlatformErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Platform tree.
pub const PLATFORM_PATHS: &[&str] = &[
    "Platform.Forbidden",
    "Platform.IdInvalid",
    "Platform.Internal",
];

// ── Models.SelfHosted (chest's self-hosted backend registry) ─────────────
derive_enum!(
    #[derive(PartialEq, Eq)]
    pub enum SelfHostedErrors {
        Forbidden,
        KindInvalid,
        Params,
        UrlInvalid,
        ModelsInvalid,
        ModelsRequired,
        IndexRequired,
    }
);

impl SelfHostedErrors {
    /// See UsageErrors::path — arms draw from `SELF_HOSTED_PATHS` slots.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Forbidden => SELF_HOSTED_PATHS[0],
            Self::KindInvalid => SELF_HOSTED_PATHS[1],
            Self::Params => SELF_HOSTED_PATHS[2],
            Self::UrlInvalid => SELF_HOSTED_PATHS[3],
            Self::ModelsInvalid => SELF_HOSTED_PATHS[4],
            Self::ModelsRequired => SELF_HOSTED_PATHS[5],
            Self::IndexRequired => SELF_HOSTED_PATHS[6],
        }
    }
}

impl AppErrorPath for SelfHostedErrors {
    fn error_path(&self) -> &'static str {
        self.path()
    }
}

/// Every path of the Models.SelfHosted tree.
pub const SELF_HOSTED_PATHS: &[&str] = &[
    "Models.SelfHosted.Forbidden",
    "Models.SelfHosted.KindInvalid",
    "Models.SelfHosted.Params",
    "Models.SelfHosted.UrlInvalid",
    "Models.SelfHosted.ModelsInvalid",
    "Models.SelfHosted.ModelsRequired",
    "Models.SelfHosted.IndexRequired",
];

/// Every path in the table, namespace by namespace — the TS export's
/// flat surface.
pub const ALL_ERROR_PATHS: &[&[&str]] = &[
    USAGE_PATHS,
    SCHEDULER_PATHS,
    PLATFORM_PATHS,
    SELF_HOSTED_PATHS,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_paths_pinned() {
        assert_eq!(UsageErrors::Forbidden.path(), "Usage.Forbidden");
        assert_eq!(UsageErrors::Internal.path(), "Usage.Internal");
        assert_eq!(UsageErrors::GroupByInvalid.path(), "Usage.GroupByInvalid");
        assert_eq!(USAGE_PATHS.len(), 3);
    }

    #[test]
    fn scheduler_paths_pinned() {
        assert_eq!(SchedulerErrors::Unavailable.path(), "Scheduler.Unavailable");
        assert_eq!(SchedulerErrors::QueryFailed.path(), "Scheduler.QueryFailed");
        assert_ne!(
            SchedulerErrors::Unavailable.path(),
            SchedulerErrors::QueryFailed.path()
        );
        assert_eq!(SCHEDULER_PATHS.len(), 2);
    }

    #[test]
    fn platform_paths_pinned() {
        assert_eq!(PlatformErrors::Forbidden.path(), "Platform.Forbidden");
        assert_eq!(PlatformErrors::IdInvalid.path(), "Platform.IdInvalid");
        assert_eq!(PlatformErrors::Internal.path(), "Platform.Internal");
        assert_eq!(PLATFORM_PATHS.len(), 3);
    }

    #[test]
    fn self_hosted_paths_pinned() {
        assert_eq!(
            SelfHostedErrors::Forbidden.path(),
            "Models.SelfHosted.Forbidden"
        );
        assert_eq!(
            SelfHostedErrors::KindInvalid.path(),
            "Models.SelfHosted.KindInvalid"
        );
        assert_eq!(SelfHostedErrors::Params.path(), "Models.SelfHosted.Params");
        assert_eq!(
            SelfHostedErrors::UrlInvalid.path(),
            "Models.SelfHosted.UrlInvalid"
        );
        assert_eq!(
            SelfHostedErrors::ModelsInvalid.path(),
            "Models.SelfHosted.ModelsInvalid"
        );
        assert_eq!(
            SelfHostedErrors::ModelsRequired.path(),
            "Models.SelfHosted.ModelsRequired"
        );
        assert_eq!(
            SelfHostedErrors::IndexRequired.path(),
            "Models.SelfHosted.IndexRequired"
        );
        assert_eq!(SELF_HOSTED_PATHS.len(), 7);
    }

    #[test]
    fn the_code_is_the_placeholder_and_paths_are_the_identity() {
        assert_eq!(
            APP_ERROR_CODE, -32000,
            "the app-error code is the placeholder by ruling; paths are the identity"
        );
        // Pairwise-unique across the whole table — an error path names
        // exactly one condition.
        let flat: Vec<&str> = ALL_ERROR_PATHS.concat();
        let mut sorted = flat.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            flat.len(),
            sorted.len(),
            "duplicate error path in the table"
        );
    }

    /// Regenerate `bindings/errorPaths.ts` from the trees' path tables.
    /// Runs under the plain `cargo test` regime like the ts-rs export
    /// tests, so CI's regeneration pass is the drift gate: a tree change
    /// without regenerated bindings fails the suite on the diff.
    #[test]
    fn export_error_paths_bindings() {
        fn tree_const(name: &str, doc: &str, paths: &[&str]) -> String {
            let entries: Vec<String> = paths
                .iter()
                .map(|p| {
                    let variant = p.rsplit('.').next().unwrap();
                    format!("  {variant}: \"{p}\",")
                })
                .collect();
            // MINOR-1 guard: two paths in ONE tree sharing a last segment
            // would collide as a duplicate TS object key (silent
            // last-wins in JS object literals).
            let mut keys: Vec<&str> = paths
                .iter()
                .map(|p| p.rsplit('.').next().unwrap())
                .collect();
            let before = keys.len();
            keys.sort_unstable();
            keys.dedup();
            assert_eq!(before, keys.len(), "duplicate TS object key in {name}");
            format!(
                "/** {doc} */\nexport const {name} = {{\n{}\n}} as const;\n",
                entries.join("\n")
            )
        }

        let flat: Vec<&str> = ALL_ERROR_PATHS.concat();
        let flat_entries: Vec<String> = flat.iter().map(|p| format!("  \"{p}\",")).collect();
        let trees = format!(
            "{}\n{}\n{}\n{}",
            tree_const(
                "USAGE_ERROR_PATHS",
                "Usage tree paths (chest's usage.get seam).",
                USAGE_PATHS
            ),
            tree_const(
                "SCHEDULER_ERROR_PATHS",
                "Scheduler tree paths (the arona metering plane).",
                SCHEDULER_PATHS
            ),
            tree_const(
                "PLATFORM_ERROR_PATHS",
                "Platform tree paths (the platform-pinning gate).",
                PLATFORM_PATHS
            ),
            tree_const(
                "SELF_HOSTED_ERROR_PATHS",
                "Models.SelfHosted tree paths (the self-hosted backend registry).",
                SELF_HOSTED_PATHS
            ),
        );
        let content = format!(
            "// This file is generated by the error-path export test (cargo test). Do not edit manually.\n\
             // Source of truth: packages/celestia-types/src/error_paths.rs (yuuka trees).\n\n\
             {trees}\n\
             /** Every path in the table, in declaration order. */\n\
             export const ERROR_PATHS = [\n{}\n] as const;\n\n\
             export type ErrorPath = (typeof ERROR_PATHS)[number];\n\n\
             /** Exact-match helper: does an error message carry this path prefix?\n \
             * Server messages are `\"<Path>: <prose>\"`; matching on the delimited\n \
             * prefix keeps `Usage.Forbidden` from matching `Usage.ForbiddenExtra`. */\n\
             export function errorPathMatches(message: string, path: string): boolean {{\n  \
             return message === path || message.startsWith(`${{path}}:`);\n}}\n",
            flat_entries.join("\n"),
        );

        let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/bindings");
        let out_path = format!("{out_dir}/errorPaths.ts");
        let existing = std::fs::read_to_string(&out_path).unwrap_or_default();
        if existing != content {
            std::fs::write(&out_path, &content).expect("write bindings/errorPaths.ts");
        }
    }
}
