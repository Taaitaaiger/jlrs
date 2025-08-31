//! The range of Julia versions supported by jlrs and current stable Julia version.
//!
//! NB: jlrs docs are currently always built for the stable version.

/// Minimum supported Julia minor version
pub const MIN_MINOR_VERSION: u32 = 10;

/// Maximum supported Julia minor version
pub const MAX_MINOR_VERSION: u32 = 13;

/// Current stable Julia major version
pub const STABLE_MAJOR_VERSION: u32 = 1;

/// Current stable Julia minor version
pub const STABLE_MINOR_VERSION: u32 = 11;
