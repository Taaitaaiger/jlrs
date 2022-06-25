//! Helper struct for making public functions uncallable from other crates.

/// If a trait A is used in a trait bound, the trait methods from traits that A extends become
/// available without explicitly using those base traits. By taking this struct (which can only be
/// created inside this crate) as an argument, these methods can only be called from this crate.
pub struct Private;
