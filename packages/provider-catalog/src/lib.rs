//! plana-provider-catalog
//!
//! Compile-time embedded provider entrypoint and model catalog data for the
//! PLANA protocol family.
//!
//! The data is synced from the provider-registry repository into
//! `target/provider-registry` before the build; outside that workspace the
//! directories are empty placeholders and this crate exposes empty catalogs.
//! [`entrypoint`] is the only surface: provider entrypoint TOMLs and their
//! model cards, plus the language-completeness check over them.

pub mod entrypoint;
