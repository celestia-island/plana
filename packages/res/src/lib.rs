//! arona_res
//!
//! Resource management crate — centralized management of all compile-time embedded resources
//!
//! # Features
//!
//! - Provides language enum and constants
//! - Provides access to i18n translation resources
//! - Provides access to provider entrypoint configuration
//! - Provides access to agent documentation
//! - Compile-time validation in build.rs
#![allow(clippy::type_complexity)]

pub mod about;
pub mod agent_names;
pub mod docs;
pub mod entrypoint;
pub mod i18n;
pub mod lang;

// Re-export commonly used types and constants
pub use lang::{Language, SUPPORTED_LANG_CODES, SUPPORTED_LANGUAGES};
