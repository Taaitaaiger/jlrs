//! All traits used by this crate.
//!
//! Most of these traits are intended for internal use only and you should never manually
//! implement them for your own types nor call any of their trait methods directly.
//!
//! The one major exception is the [`Frame`] trait. This trait is implemented by the two frame
//! types that are provided, [`StaticFrame`] and [`DynamicFrame`] which are used to ensure the
//! garbage collector doesn't drop the data that's used from Rust. It provides the common
//! functionality these frame types offer.
//!
//! Two of the traits in this module are available as custom derive traits, [`JuliaStruct`] and
//! [`IntoJulia`], which can be used to map a struct between Julia and Rust. Deriving the first
//! will implement [`JuliaType`], [`JuliaTypecheck`], [`ValidLayout`], and [`Cast`], which will let you
//! safely access the raw contents of a value; [`IntoJulia`] can be derived for bits types and lets
//! you create new instances of that type using [`Value::new`]. While it's possible to manually
//! implement and annotate these mapping structs, you should use `JlrsReflect.jl` which can
//! generate these structs for you. If you do want to do this manually, see the documentation of
//! [`JuliaStruct`] for instructions.
//!
//! [`Frame`]: trait.Frame.html
//! [`StaticFrame`]: ../frame/struct.StaticFrame.html
//! [`DynamicFrame`]: ../frame/struct.DynamicFrame.html
//! [`Value::new`]: ../value/struct.Value.html#method.new
//! [`Value::cast`]: ../value/struct.Value.html#method.cast
//! [`JuliaStruct`]: trait.JuliaStruct.html
//! [`JuliaType`]: trait.JuliaType.html
//! [`Cast`]: trait.Cast.html
//! [`ValidLayout`]: trait.ValidLayout.html
//! [`IntoJulia`]: trait.IntoJulia.html
//! [`JuliaTypecheck`]: trait.JuliaTypecheck.html
//! [`Value::is`]: ../value/struct.Value.html#method.is
//! [`DataType::is`]: ../value/datatype/struct.DataType.html#method.is

pub mod bits_union;
pub mod cast;
pub mod frame;
pub mod gc;
pub mod into_julia;
pub mod julia_type;
pub mod julia_typecheck;
#[cfg(all(feature = "async", target_os = "linux"))]
pub mod multitask;
pub mod temporary_symbol;
pub mod valid_layout;

pub use bits_union::{Align, BitsUnion, Flag};
pub use cast::Cast;
pub use frame::Frame;
pub use gc::Gc;
pub use into_julia::IntoJulia;
pub use julia_type::JuliaType;
pub use julia_typecheck::JuliaTypecheck;
pub use temporary_symbol::TemporarySymbol;
pub use valid_layout::ValidLayout;

/// This trait can be derived in order to provide a mapping between a type in Julia and one in
/// Rust. When this trait is derived, the following traits are implemented:
///
/// - [`JuliaType`]
/// - [`JuliaTypecheck`]
/// - [`ValidLayout`]
/// - [`Cast`]
///
/// With these traits implemented you can use [`Value::cast`] with this custom type.
///
/// Rather than manually implement the appropriate structs, you should use `JlrsReflect.jl` to
/// generate them for you.  If you do choose to implement this trait manually, the following rules
/// apply.
///
/// First, the struct must be annotated with `#[repr(C)]` to ensure the compiler won't change the
/// layout. Second, the struct must be annotated with `#[jlrs(julia_type = "Path.To.Type")]` where
/// the path provides the full name of the type, eg the path for a struct named`Bar` in the module
/// `Foo` which is a submodule of `Main` is `Main.Foo.Bar`. When this type is used, it must be
/// available at that location. This path must not contain any type parameters.
///
/// Struct have fields and these fields have types. The type can belong to one of the following
/// classes:
///  - DataType
///  - UnionAll
///  - Union
///  - TypeVar
///
/// If the field type is a DataType the field will either be allocated inline or stored as a
/// `Value`. If it's allocated inline, a valid binding for that field must be used. In some cases,
/// for example a field that contains a `Module`, that type can be used as a specialized type.
/// Many of the types defined in the submodules of `value` can be used this way.
///
/// Special care must be taken if the field type is a tuple type. Unlike other types, tuples are
/// covariant in the parameters. This means that a tuple like `Tuple{Int32, Int64}` is a subtype
/// of `Tuple{Int32, Real}`. As a result, a tuple type can only be instantiated if all of its
/// fields are concrete types. If the field type is a concrete tuple type, it is stored inline and
/// can be represented by the appropriate type from the `tuple` module, otherwise it will not be
/// stored inline and a `Value` must be used instead.
///
/// `UnionAll`s are straightforward, they're never allocated inline and must always be mapped to a
/// `Value`.
///
/// Similar to tuples, unions can have two representation depending on the type parameters. If all
/// types are pointer-free, the bits union optimization will apply. Otherwise it is stored as a
/// `Value`.
///
/// The bits union optimization is not straightforward to map to Rust. In fact, three fields are
/// required. Unlike normal structs the size of a bits union field doesn't have to be an integer
/// multiple of its alignment; it will have the alignment of the variant with the largest alignment
/// and is as large as the largest possible variant. Additionally, there will be another `u8` that
/// is used as a flag to indicate the active variant.
///
/// The first field is the correct zero-sized `Align#`-type defined in the `union` module. The
/// second a `BitsUnion` from that same module, its type parameter must be an array of
/// `MaybeUninit<u8>`s with the appropriate numbber of elements. Finally, a `u8` must be used as
/// a flag. In order for the derive macro to handle these fields correctly, they must be annotated
/// with `#[jlrs(bits_union_align)]`, `#[jlrs(bits_union)]`, and `#[jlrs(bits_union_flag)]`
/// respectively.
///
/// Finally, a `TypeVar` field will be mapped to a type parameter in Rust. A parameter that
/// doesn't affect the layout must be elided. The type parameter must implement both `ValidLayout`
/// and `Copy`.
///
/// [`JuliaType`]: trait.JuliaType.html
/// [`JuliaTypecheck`]: trait.JuliaTypecheck.html
/// [`ValidLayout`]: trait.ValidLayout.html
/// [`Cast`]: trait.Cast.html
/// [`Value::cast`]: ../value/struct.Value.html#method.cast
pub unsafe trait JuliaStruct: Copy {}

pub(crate) mod private {
    // If a trait A is used in a trait bound, the trait methods from traits that A extends become
    // available without explicitly using those base traits. By taking this struct, which can only
    // be created inside this crate, as an argument, these methods can only be called from this
    // crate.
    pub struct Internal;
}
