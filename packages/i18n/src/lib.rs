//! plana-i18n
//!
//! Language enum, BCP 47 detection, and compile-time embedded i18n locale
//! resources for the PLANA protocol family.
//!
//! - [`lang`] — the [`Language`] enum, the supported-code constants, and
//!   system-locale detection
//! - [`i18n`] — the locale directories embedded from `res/i18n/locales`
#![allow(clippy::type_complexity)]

pub mod lang;
pub mod i18n;

// Re-export commonly used types and constants at the crate root
pub use lang::{Language, SUPPORTED_LANG_CODES, SUPPORTED_LANGUAGES};
