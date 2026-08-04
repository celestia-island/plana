#![allow(clippy::type_complexity)]

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(not(target_os = "linux"))]
mod stub;

#[cfg(not(target_os = "linux"))]
pub use stub::*;
