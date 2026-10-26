//! Build script for plana_llm_provider: generates the pricing fallback
//! table from CHECKED-IN data (`pricing/bands.toml` + `pricing/models-
//! snapshot.toml`). Never touches the network — offline reproducibility is
//! a hard constraint (2026-10-26 arch review P2-4).
//!
//! Refresh flow (explicit, networked, run by a human or CI on purpose):
//! `just regen-pricing` syncs provider-registry and regenerates the
//! snapshot via `scripts/gen_pricing_snapshot.py` (atomic write; a failed
//! regeneration leaves the previous snapshot in place). The snapshot is
//! the "last generated copy" the build falls back to by construction.
//!
//! Failure semantics are fail-closed: missing or malformed input files,
//! unresolvable anchors, or an empty result panic the build. There is NO
//! path to a silently empty or silently stale table — the old hand-written
//! table's drift mode (a comment-dated manual copy of registry data) is
//! what this generator exists to eliminate.

use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=pricing/bands.toml");
    println!("cargo:rerun-if-changed=pricing/models-snapshot.toml");

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("pricing_table.rs");

    let code = generate();
    fs::write(&dest, code).expect("write pricing_table.rs");
}

#[derive(Debug)]
struct Band {
    id: String,
    all: Vec<String>,
    any: Vec<Vec<String>>,
    pricing: [f64; 3],
}

fn generate() -> String {
    let bands_src = fs::read_to_string("pricing/bands.toml")
        .expect("pricing/bands.toml is checked in — a missing file is a broken package");
    let snap_src = fs::read_to_string("pricing/models-snapshot.toml")
        .expect("pricing/models-snapshot.toml is checked in (regen: just regen-pricing)");

    let bands: toml::Table = toml::from_str(&bands_src).expect("bands.toml parses");
    let snap: toml::Table = toml::from_str(&snap_src).expect("models-snapshot.toml parses");
    let snap_models: &Vec<toml::Value> = snap
        .get("model")
        .and_then(|v| v.as_array())
        .expect("models-snapshot.toml has [[model]] entries");

    // provider-qualified id -> (input, output); cached is optional in the
    // registry (many models publish no cache price) and only becomes
    // REQUIRED when the model serves as a band anchor.
    let mut prices = std::collections::BTreeMap::new();
    for m in snap_models {
        let m = m.as_table().expect("snapshot [[model]] is a table");
        let provider = m
            .get("provider")
            .and_then(|v| v.as_str())
            .expect("provider");
        let id = m.get("id").and_then(|v| v.as_str()).expect("id");
        let get_price = |key: &str| -> f64 {
            let v = m
                .get(key)
                .and_then(|v| v.as_float())
                .unwrap_or_else(|| panic!("snapshot entry {provider}/{id}: {key} missing"));
            assert_price(v, &format!("snapshot {provider}/{id}: {key}"));
            v
        };
        let input = get_price("input_per_million");
        let output = get_price("output_per_million");
        let cached = m.get("cached_per_million").and_then(|v| v.as_float());
        if let Some(c) = cached {
            assert_price(c, &format!("snapshot {provider}/{id}: cached_per_million"));
        }
        prices.insert(format!("{provider}/{id}"), (input, output, cached));
    }

    let raw_bands: Vec<&toml::Value> = bands
        .get("band")
        .and_then(|v| v.as_array())
        .expect("bands.toml has [[band]] rows")
        .iter()
        .collect();
    assert!(
        !raw_bands.is_empty(),
        "bands.toml has no [[band]] rows — refusing to generate an empty table"
    );

    let mut parsed = Vec::new();
    let mut seen_ids = std::collections::BTreeSet::new();
    for row in &raw_bands {
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .expect("band id")
            .to_string();
        assert!(!seen_ids.contains(&id), "duplicate band id {id}");
        seen_ids.insert(id.clone());

        let read_patterns = |key: &str| -> Vec<String> {
            row.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|v| v.as_str().expect("pattern is a string").to_string())
                        .collect()
                })
                .unwrap_or_default()
        };
        let all: Vec<String> = read_patterns("all");
        let any: Vec<Vec<String>> = row
            .get("any")
            .and_then(|v| v.as_array())
            .map(|groups| {
                groups
                    .iter()
                    .map(|g| {
                        g.as_array()
                            .expect("any entry is a group of strings")
                            .iter()
                            .map(|v| v.as_str().expect("pattern string").to_string())
                            .collect()
                    })
                    .collect()
            })
            .unwrap_or_default();

        for p in all.iter().chain(any.iter().flatten()) {
            assert!(!p.is_empty(), "band {id}: empty pattern");
            assert!(
                p.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || !c.is_alphabetic()),
                "band {id}: pattern {p:?} must be lowercase (matched against the lowercased model id)"
            );
        }
        assert!(
            !all.is_empty() || any.iter().any(|g| !g.is_empty()),
            "band {id}: at least one pattern overall (catch-all rows are forbidden)"
        );
        assert!(
            any.iter().all(|g| !g.is_empty()),
            "band {id}: empty any-group (drop it instead)"
        );

        let pricing: [f64; 3] = if let Some(anchor) = row.get("anchor").and_then(|v| v.as_str()) {
            let (input, output, cached) = prices
                .get(anchor)
                .unwrap_or_else(|| panic!("band {id}: anchor {anchor} not in models-snapshot.toml — refresh the snapshot or fix the anchor"));
            [
                *input,
                *output,
                cached.unwrap_or_else(|| {
                    panic!("band {id}: anchor {anchor} publishes no cached price — pick an anchor with all three fields or make the band legacy")
                }),
            ]
        } else {
            let reason = row.get("legacy").and_then(|v| v.as_str());
            assert!(
                reason.map(|r| !r.trim().is_empty()).unwrap_or(false),
                "band {id}: legacy rows need a non-empty `legacy` reason"
            );
            let get = |key: &str| -> f64 {
                let v = row
                    .get(key)
                    .and_then(|v| v.as_float())
                    .unwrap_or_else(|| panic!("band {id}: legacy row missing {key}"));
                assert_price(v, &format!("band {id}: {key}"));
                v
            };
            [
                get("input_per_million"),
                get("output_per_million"),
                get("cached_per_million"),
            ]
        };
        assert!(
            pricing[2] <= pricing[0] + 1e-9,
            "band {id}: cached {} > input {} (the conservative invariant)",
            pricing[2],
            pricing[0]
        );

        parsed.push(Band {
            id,
            all,
            any,
            pricing,
        });
    }

    let mut code = String::new();
    code.push_str(
        "// AUTO-GENERATED by build.rs from pricing/bands.toml + pricing/models-snapshot.toml\n",
    );
    code.push_str("// — do not edit. Regenerate with `just regen-pricing`.\n\n");
    code.push_str("pub(crate) struct PricingBand {\n");
    code.push_str("    pub(crate) id: &'static str,\n");
    code.push_str("    pub(crate) all: &'static [&'static str],\n");
    code.push_str("    pub(crate) any: &'static [&'static [&'static str]],\n");
    code.push_str("    pub(crate) pricing: FamilyPricing,\n");
    code.push_str("}\n\n");
    code.push_str("pub(crate) static PRICING_BANDS: &[PricingBand] = &[\n");
    for b in &parsed {
        let all = b
            .all
            .iter()
            .map(|p| format!("{p:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let any = b
            .any
            .iter()
            .map(|g| {
                let inner = g
                    .iter()
                    .map(|p| format!("{p:?}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("&[{inner}]")
            })
            .collect::<Vec<_>>()
            .join(", ");
        code.push_str(&format!(
            "    PricingBand {{ id: {:?}, all: &[{}], any: &[{}], pricing: FamilyPricing {{ input_per_million: {:?}, output_per_million: {:?}, cached_per_million: {:?} }} }},\n",
            b.id, all, any, b.pricing[0], b.pricing[1], b.pricing[2]
        ));
    }
    code.push_str("];\n");
    code
}

fn assert_price(v: f64, ctx: &str) {
    assert!(
        v.is_finite() && v >= 0.0,
        "{ctx}: prices must be finite and non-negative, got {v}"
    );
}
