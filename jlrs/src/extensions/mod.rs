//! Optional extensions

#[cfg(feature = "f16")]
pub mod f16;
#[cfg(feature = "async")]
pub mod multitask;
#[cfg(feature = "jlrs-ndarray")]
pub mod ndarray;
#[cfg(feature = "pyplot")]
pub mod pyplot;
