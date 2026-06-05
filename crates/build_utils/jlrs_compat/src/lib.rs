//! The range of Julia versions supported by jlrs and current stable Julia version.
//!
//! NB: jlrs docs are currently always built for the stable version.

/// Current stable Julia major version
pub const MAJOR_VERSION: u32 = 1;

/// Minimum supported Julia minor version
pub const MIN_MINOR_VERSION: u32 = 10;

/// Maximum supported Julia minor version
pub const MAX_MINOR_VERSION: u32 = 13;

/// Nightly Julia minor version
///
/// This value is set to `Some(x)` if version 1.x has incompatible changes with respect to
/// 1.`MAX_MINOR_VERSION`
pub const NIGHTLY_MINOR_VERSION: u32 = MAX_MINOR_VERSION + 1;

/// Julia minor version when building documentation for docs.rs
pub const DOCS_MINOR_VERSION: u32 = 12;
