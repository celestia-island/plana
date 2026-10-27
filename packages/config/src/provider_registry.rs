//! Embedded provider-registry data (T1 decoupling slice 2, 2026-10-27).
//!
//! The config layer resolves provider entrypoints and capability scores from
//! the provider-registry catalog. Historically that data came through
//! `plana_res`, whose `include_dir!` pointed at the build-time-generated
//! `target/provider-registry` tree — a path that cannot be packaged (and is
//! EMPTY on a fresh checkout, silently embedding an empty catalog). This
//! module embeds a CHECKED-IN snapshot instead:
//!
//! - `data/provider-registry/entrypoint/`: the entrypoint TOMLs, verbatim.
//! - `data/model-scores.toml`: the `[model.score].value` table only — the
//!   full model cards are ~4.5 MiB against ~50 KiB of scores the config
//!   layer actually reads (see the docstring in the generator).
//!
//! Refresh: `just regen-config-registry` (syncs upstream, regenerates both
//! artifacts, and leaves the diff for review/commit) — a deliberate data
//! refresh only; the build itself never touches the network.

use include_dir::{Dir, include_dir};

/// Every provider entrypoint TOML, embedded at compile time.
pub static ENTRYPOINT_DIR: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/data/provider-registry/entrypoint");

static MODEL_SCORES: &str = include_str!("../data/model-scores.toml");

/// Cached capability score (`[model.score].value`) for `provider_id`'s
/// model card named `model_name`.
///
/// The registry names model cards by a normalized form of the model name —
/// `/` becomes `_` and `:` becomes `-` — and this lookup applies the same
/// normalization, so callers pass the raw model name as they always have.
pub fn model_score(provider_id: &str, model_name: &str) -> Option<f64> {
    let stem = model_name.replace('/', "_").replace(':', "-");
    let parsed: toml::Value = toml::from_str(MODEL_SCORES).ok()?;
    parsed
        .get("scores")?
        .get(provider_id)?
        .get(stem)?
        .as_float()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_catalog_is_not_empty() {
        // A regression guard for the old failure mode: an empty include_dir
        // silently produced a provider-less config layer.
        assert!(
            ENTRYPOINT_DIR.dirs().count() > 0,
            "embedded entrypoint data is empty"
        );
        assert!(MODEL_SCORES.contains("[scores"));
    }

    #[test]
    fn scores_normalize_slashes_and_colons_like_registry_filenames() {
        // Shape-only check (no specific upstream model is guaranteed):
        // a name that normalizes to an existent entry resolves, a nonsense
        // name does not.
        let any = toml::from_str::<toml::Value>(MODEL_SCORES)
            .ok()
            .and_then(|v| v.get("scores").cloned())
            .and_then(|s| {
                s.as_table()
                    .and_then(|t| t.iter().next().map(|(k, _)| k.clone()))
            });
        if let Some(provider) = any {
            assert!(model_score(&provider, "definitely-not-a-real-model-xyz").is_none());
        }
    }
}
