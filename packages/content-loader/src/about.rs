//! About/protocol document access.
//!
//! Governance documents (license, CLA, code of conduct, security) are bundled
//! at build time from the arona monorepo. The build.rs resolves the arona
//! checkout via `$ARONA_ROOT` or sibling directory detection, reads translated
//! docs from `arona/docs/licenses/<lang>/`, and generates the embedded data.
//!
//! If arona was not available at build time, all fields are empty strings.

include!(concat!(env!("OUT_DIR"), "/about_gen.rs"));

/// Check whether about content was bundled (i.e., arona was found at build time).
pub fn is_available() -> bool {
    !about_docs_for("en").license.is_empty()
}
