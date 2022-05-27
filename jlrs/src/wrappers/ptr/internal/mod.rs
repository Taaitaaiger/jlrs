//! Types you probably won't need.
//!
//! To use these types you must enable the `internal-types` feature.

pub mod code_instance;
pub mod expr;
pub mod method;
pub mod method_instance;
pub mod method_match;
pub mod method_table;
#[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
pub mod opaque_closure;
pub mod typemap_entry;
pub mod typemap_level;
#[cfg(any(not(feature = "lts"), feature = "all-features-override"))]
pub mod vararg;
pub mod weak_ref;
