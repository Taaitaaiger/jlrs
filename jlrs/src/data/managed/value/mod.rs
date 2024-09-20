//! Managed type for arbitrary Julia data.
//!
//! Julia data returned by the C API is often returned as a pointer to `jl_value_t`, which is
//! an opaque type. This pointer is wrapped in jlrs by [`Value`]. The layout of the data that is
//! pointed to depends on its underlying type. Julia guarantees that the data is preceded in
//! memory by a header which contains a pointer to the data's type information, its [`DataType`].
//!
//! For example, if the `DataType` is `UInt8`, the pointer points to a `u8`. If the
//! `DataType` is some Julia array type like `Array{Int, 2}`, the pointer points to
//! Julia's internal array type, `jl_array_t`. In the first case tha value can be unboxed as a
//! `u8`, in the second case it can be cast to [`Array`] or [`TypedArray<isize>`].
//!
//! The `Value` type is very commonly used in jlrs. A `Value` can be called as a Julia function,
//! the arguments such a function takes are all `Value`s, and it will return either a `Value` or
//! an exception which is also a `Value`. This type also provides methods to create new `Value`s,
//! access their fields, cast them to the appropriate managed type, and unbox their contents.
//!
//! One special kind of value is the `NamedTuple`. You will need to create values of this type in
//! order to call functions with keyword arguments. The macro [`named_tuple`] is defined in this
//! module which provides an easy way to create values of this type.
//!
//! [`Array`]: crate::data::managed::array::Array
//! [`TypedArray<isize>`]: crate::data::managed::array::TypedArray
//! [`named_tuple`]: crate::named_tuple!

/*
    TODO

    Atomic operations:

        jl_atomic_cmpswap_bits
        jl_atomic_bool_cmpswap_bits
        jl_atomic_new_bits
        jl_atomic_store_bits
        jl_atomic_swap_bits
*/

pub mod field_accessor;
pub mod tracked;
pub mod typed;

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    ($name:expr => $value:expr) => {
        2
    };
    ($name:expr => $value:expr, $($rest:tt)+) => {
        count!(2, $($rest)+)
    };
    ($n:expr, $name:expr => $value:expr) => {
        $n + 1
    };
    ($n:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        count!($n + 1, $($rest)+)
    };
}

/// Create a new named tuple. You will need a named tuple to call functions with keyword
/// arguments.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// # fn main() {
/// # let mut julia = Builder::new().start_local().unwrap();
/// // Three slots; two for the inputs and one for the output.
/// julia.local_scope::<_, 3>(|mut frame| {
///     // Create the two arguments, each value requires one slot
///     let i = Value::new(&mut frame, 2u64);
///     let j = Value::new(&mut frame, 1u32);
///
///     let _nt = named_tuple!(&mut frame, "i" => i, "j" => j);
/// });
/// # }
/// ```
#[macro_export]
macro_rules! named_tuple {
    ($frame:expr, $name:expr => $value:expr) => {
        {
            let name = $crate::convert::to_symbol::ToSymbol::to_symbol(&$name, &$frame);
            $crate::data::managed::value::Value::new_named_tuple($frame, &[(name, $value)])
        }
    };
    ($frame:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        {
            const N: usize = $crate::count!($($rest)+);
            let mut pairs: [::std::mem::MaybeUninit::<($crate::data::managed::symbol::Symbol, $crate::data::managed::value::Value)>; N] = [::std::mem::MaybeUninit::uninit(); N];
            let name = $crate::convert::to_symbol::ToSymbol::to_symbol(&$name, &$frame);

            pairs[0].write((name, $value));
            $crate::named_tuple!($frame, 1, &mut pairs, $($rest)+)
        }
    };
    ($frame:expr, $i:expr, $pairs:expr, $name:expr => $value:expr, $($rest:tt)+) => {
        {
            let name = $crate::convert::to_symbol::ToSymbol::to_symbol(&$name, &$frame);
            $pairs[$i].write((name, $value));
            named_tuple!($frame, $i + 1, $pairs, $($rest)+)
        }
    };
    ($frame:expr, $i:expr, $pairs:expr, $name:expr => $value:expr) => {
        {
            let name = $crate::convert::to_symbol::ToSymbol::to_symbol(&$name, &$frame);
            $pairs[$i].write((name, $value));

            let pairs: &[($crate::data::managed::symbol::Symbol, $crate::data::managed::value::Value); N] = unsafe {
                ::std::mem::transmute::<
                    &[::std::mem::MaybeUninit::<($crate::data::managed::symbol::Symbol, $crate::data::managed::value::Value)>; N],
                    &[($crate::data::managed::symbol::Symbol, $crate::data::managed::value::Value); N]
                >($pairs)
            };

            $crate::data::managed::value::Value::new_named_tuple($frame, pairs)
        }
    };
}

use std::{
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
    mem::MaybeUninit,
    path::Path,
    ptr::NonNull,
    usize,
};

#[julia_version(since = "1.7")]
use jl_sys::jl_pair_type;
use jl_sys::{
    jl_an_empty_string, jl_an_empty_vec_any, jl_any_type, jl_apply_type, jl_array_any_type,
    jl_array_int32_type, jl_array_symbol_type, jl_array_uint8_type, jl_bottom_type, jl_call,
    jl_call0, jl_call1, jl_call2, jl_call3, jl_diverror_exception, jl_emptytuple, jl_eval_string,
    jl_exception_occurred, jl_false, jl_field_index, jl_gc_add_finalizer, jl_gc_add_ptr_finalizer,
    jl_get_nth_field, jl_get_nth_field_noalloc, jl_has_typevar, jl_interrupt_exception, jl_isa,
    jl_memory_exception, jl_new_struct_uninit, jl_nothing, jl_object_id,
    jl_readonlymemory_exception, jl_set_nth_field, jl_stackovf_exception, jl_static_show,
    jl_stderr_obj, jl_stderr_stream, jl_stdout_obj, jl_stdout_stream, jl_subtype, jl_true,
    jl_typeof_str, jl_undefref_exception, jl_value_t, jlrs_call_unchecked, jlrs_egal,
    jlrs_field_isptr,
};
use jlrs_macros::julia_version;

use self::{field_accessor::FieldAccessor, typed::TypedValue};
use super::{type_var::TypeVar, Ref};
use crate::{
    args::Values,
    call::{Call, ProvideKeywords, WithKeywords},
    catch::{catch_exceptions, unwrap_exc},
    convert::{into_julia::IntoJulia, to_symbol::ToSymbol, unbox::Unbox},
    data::{
        layout::{
            is_bits::IsBits,
            typed_layout::HasLayout,
            valid_layout::{ValidField, ValidLayout},
        },
        managed::{
            datatype::DataType,
            module::Module,
            private::ManagedPriv,
            string::JuliaString,
            symbol::Symbol,
            union::Union,
            union_all::UnionAll,
            value::tracked::{Tracked, TrackedMut},
            Managed,
        },
        types::{
            construct_type::ConstructType,
            typecheck::{NamedTuple, Typecheck},
        },
    },
    error::{AccessError, IOError, JlrsError, JlrsResult, TypeError, CANNOT_DISPLAY_TYPE},
    memory::{
        context::ledger::Ledger,
        get_tls,
        target::{unrooted::Unrooted, Target, TargetException, TargetResult},
    },
    prelude::NTuple,
    private::Private,
};

/// Arbitrary Julia data.
///
/// A `Value` is essentially a non-null pointer to some data owned by the Julia garbage
/// collector with two lifetimes: `'scope` and `'data`. The first of these ensures that a
/// `Value` can only be used while it's rooted, the second accounts for data borrowed from Rust.
/// The only way to borrow data from Rust is to create an Julia array that borrows its contents
///  by calling [`ConstructTypedArray::from_slice`]; if a Julia function is called with such an
/// array as an argument the result will inherit the second lifetime of the borrowed data to
/// ensure that such a `Value` can only be used while the borrow is active.
///
/// See the [module-level documentation] for more information.
///
/// [`ConstructTypedArray::from_slice`]: crate::data::managed::array::ConstructTypedArray::from_slice
#[repr(transparent)]
#[derive(Copy, Clone, Eq)]
pub struct Value<'scope, 'data>(
    NonNull<jl_value_t>,
    PhantomData<&'scope ()>,
    PhantomData<&'data mut ()>,
);

impl<'scope, 'data, T: Managed<'scope, 'data>> PartialEq<T> for Value<'_, '_> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.egal(other.as_value())
    }
}

// Safety: it's always safe to treat managed data as a `Value`.
unsafe impl Typecheck for Value<'_, '_> {
    #[inline]
    fn typecheck(_: DataType) -> bool {
        true
    }
}

/// # Create new `Value`s
///
/// Several methods are available to create new values. The simplest of these is [`Value::new`],
/// which can be used to convert relatively simple data from Rust to Julia. Data that can be
/// converted this way must implement [`IntoJulia`], which is the case for types like the
/// primitive number types. This trait is also automatically derived by JlrsReflect.jl for types
/// that are trivially guaranteed to be bits-types: the type must have no type parameters, no
/// unions, and all fields must be immutable bits-types themselves.
impl Value<'_, '_> {
    /// Create a new Julia value, any type that implements [`IntoJulia`] can be converted using
    /// this function.
    #[inline]
    pub fn new<'target, V, Tgt>(target: Tgt, value: V) -> ValueData<'target, 'static, Tgt>
    where
        V: IntoJulia,
        Tgt: Target<'target>,
    {
        value.into_julia(target)
    }

    /// Create a new Julia value, any type that implements [`IsBits`] can be converted using
    /// this function.
    pub fn new_bits<'target, T, Tgt>(target: Tgt, layout: T) -> ValueData<'target, 'static, Tgt>
    where
        T: ConstructType + ValidLayout + IsBits,
        Tgt: Target<'target>,
    {
        unsafe {
            let ty = T::construct_type(&target)
                .as_value()
                .cast_unchecked::<DataType>();
            let val = NonNull::new_unchecked(jl_new_struct_uninit(ty.unwrap(Private)));
            val.cast::<MaybeUninit<T>>().as_mut().write(layout);
            target.data_from_ptr(val, Private)
        }
    }

    /// Create a new Julia value using `T` to construct the type. The layout must implement
    /// `IsBits`.
    pub fn new_bits_from_layout<'target, T, Tgt>(
        target: Tgt,
        layout: T::Layout,
    ) -> JlrsResult<ValueData<'target, 'static, Tgt>>
    where
        T: HasLayout<'target, 'static>,
        T::Layout: IsBits,
        Tgt: Target<'target>,
    {
        unsafe {
            let ty = T::construct_type(&target).as_value().cast::<DataType>()?;
            let val = NonNull::new_unchecked(jl_new_struct_uninit(ty.unwrap(Private)));
            val.cast::<MaybeUninit<T::Layout>>().as_mut().write(layout);
            Ok(target.data_from_ptr(val, Private))
        }
    }

    /// Create a new Julia value using `T` to construct the type and an arbitrary layout `L`.
    ///
    /// If the layout is not valid for `T` `TypeError::InvalidLayout` is returned.
    pub fn new_bits_with_type<'target, T, L, Tgt>(
        target: Tgt,
        layout: L,
    ) -> JlrsResult<ValueData<'target, 'static, Tgt>>
    where
        T: ConstructType,
        L: IsBits + ValidLayout,
        Tgt: Target<'target>,
    {
        unsafe {
            let ty = T::construct_type(&target).as_value();
            if !L::valid_layout(ty) {
                let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE);
                Err(TypeError::InvalidLayout { value_type })?;
            }

            let ty = ty.cast_unchecked::<DataType>();
            let val = NonNull::new_unchecked(jl_new_struct_uninit(ty.unwrap(Private)));
            val.cast::<MaybeUninit<L>>().as_mut().write(layout);
            Ok(target.data_from_ptr(val, Private))
        }
    }

    /// Create a new `Value` from the provided layout and type constructor.
    ///
    /// This is a more powerful version of [`Value::new`]. While that method is limited to types
    /// that implement `IntoJulia`, this method can create instances of any constructible type by
    /// providing a layout which is compatible with that type.
    ///
    /// This method returns an error if `L` is not a valid layout for `V`.
    ///
    /// Safety:
    ///
    /// If the layout contains references to Julia data, those fields must either be `None` or
    /// point to valid data.
    pub unsafe fn try_new_with<'target, Ty, L, Tgt>(
        target: Tgt,
        layout: L,
    ) -> JlrsResult<ValueData<'target, 'static, Tgt>>
    where
        Ty: ConstructType,
        L: ValidLayout,
        Tgt: Target<'target>,
    {
        let _: () = L::ASSERT_NOT_REF;

        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let ty = Ty::construct_type(&mut frame);
            let ty_dt = ty.cast::<DataType>()?;

            if !ty_dt.is_concrete_type() {
                let value = ty.display_string_or(CANNOT_DISPLAY_TYPE);
                Err(TypeError::NotConcrete { value })?;
            }

            if !L::valid_layout(ty) {
                let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE);
                Err(TypeError::InvalidLayout { value_type })?;
            }

            if let Some(n_fields) = ty_dt.n_fields() {
                for i in 0..n_fields as usize {
                    let ft = ty_dt.field_type_unchecked(i);

                    if ty_dt.is_pointer_field_unchecked(i) {
                        let offset = ty_dt.field_offset_unchecked(i) as usize;
                        check_field_isa(ft, &layout, offset)?;
                    } else if let Ok(u) = ft.cast::<Union>() {
                        check_union_equivalent::<L, _>(&frame, i, u)?;
                    }
                }
            }

            let ptr = jl_new_struct_uninit(ty_dt.unwrap(Private));
            std::ptr::write(ptr.cast::<L>(), layout);
            Ok(target.data_from_ptr(NonNull::new_unchecked(ptr), Private))
        })
    }

    /// Create a new named tuple, you should use the `named_tuple` macro rather than this method.
    pub fn new_named_tuple<'target, 'value, 'data, Tgt, const N: usize>(
        target: Tgt,
        pairs: &[(Symbol<'value>, Value<'value, 'data>); N],
    ) -> ValueData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target
                .with_local_scope::<_, _, 1>(|target, mut frame| -> JlrsResult<_> {
                    // Safety: this method can only be called from a thread known to Julia. The
                    // unchecked methods are used because it can be guaranteed they won't throw
                    // an exception for the given arguments.
                    let field_names = pairs.map(|(sym, _)| sym.as_value());

                    let names = NTuple::<Symbol, N>::construct_type(&frame)
                        .as_value()
                        .cast::<DataType>()?
                        .instantiate_unchecked(&mut frame, &field_names);

                    let values = pairs.map(|(_, val)| val);
                    let field_types = values.map(|val| val.datatype().as_value());

                    let field_types = DataType::anytuple_type(&frame)
                        .as_value()
                        .apply_type_unchecked(&frame, &field_types)
                        .as_value();

                    let ty = UnionAll::namedtuple_type(&frame)
                        .as_value()
                        .apply_type_unchecked(&frame, &[names, field_types])
                        .as_value()
                        .cast_unchecked::<DataType>();

                    Ok(ty.instantiate_unchecked(target, values))
                })
                .unwrap()
        }
    }

    /// Apply the given types to `self`.
    ///
    /// If `self` is the [`DataType`] `anytuple_type`, calling this method will return a new
    /// tuple type with the given types as its field types. If it is the [`DataType`]
    /// `uniontype_type`, calling this method is equivalent to calling [`Union::new`]. If
    /// the value is a `UnionAll`, the given types will be applied and the resulting type is
    /// returned.
    ///
    /// If the types can't be applied to `self` this methods catches and returns the exception.
    ///
    /// [`Union::new`]: crate::data::managed::union::Union::new
    pub fn apply_type<'target, 'value, 'data, V, Tgt>(
        self,
        target: Tgt,
        types: V,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
        V: AsRef<[Value<'value, 'data>]>,
    {
        // Safety: if an exception is thrown it's caught, the result is immediately rooted
        unsafe {
            let types = types.as_ref();

            let callback =
                || jl_apply_type(self.unwrap(Private), types.as_ptr() as *mut _, types.len());

            let res = match catch_exceptions(callback, unwrap_exc) {
                Ok(ptr) => Ok(NonNull::new_unchecked(ptr)),
                Err(e) => Err(e),
            };

            target.result_from_ptr(res, Private)
        }
    }

    /// Apply the given types to `self`.
    ///
    /// If `self` is the [`DataType`] `anytuple_type`, calling this method will return a new
    /// tuple type with the given types as its field types. If it is the [`DataType`]
    /// `uniontype_type`, calling this method is equivalent to calling [`Union::new`]. If
    /// the value is a `UnionAll`, the given types will be applied and the resulting type is
    /// returned.
    ///
    /// If an exception is thrown it isn't caught
    ///
    /// Safety: an exception must not be thrown if this method is called from a `ccall`ed
    /// function.
    ///
    /// [`Union::new`]: crate::data::managed::union::Union::new
    #[inline]
    pub unsafe fn apply_type_unchecked<'target, 'value, 'data, Tgt, V>(
        self,
        target: Tgt,
        types: V,
    ) -> ValueData<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
        V: AsRef<[Value<'value, 'data>]>,
    {
        let types = types.as_ref();
        let applied = jl_apply_type(self.unwrap(Private), types.as_ptr() as *mut _, types.len());
        target.data_from_ptr(NonNull::new_unchecked(applied), Private)
    }
}

/// The `stdin` and `stdout` streams
pub enum Stream {
    Stdout,
    Stderr,
}

impl Value<'_, '_> {
    /// Show this value
    pub fn show(self, stream: Stream) -> usize {
        unsafe {
            let stream = match stream {
                Stream::Stdout => jl_stdout_stream(),
                Stream::Stderr => jl_stderr_stream(),
            };
            jl_static_show(stream, self.unwrap(Private))
        }
    }
}

/// # Type information
///
/// Every value is guaranteed to have a [`DataType`]. This contains all of the value's type
/// information.
impl<'scope, 'data> Value<'scope, 'data> {
    #[inline]
    /// Returns the `DataType` of this value.
    pub fn datatype(self) -> DataType<'scope> {
        // Safety: the pointer points to valid data, every value has a type.
        unsafe {
            let self_ptr = self.unwrap(Private);
            let ty = jl_sys::jlrs_typeof(self_ptr);
            DataType::wrap_non_null(NonNull::new_unchecked(ty.cast()), Private)
        }
    }

    /// Returns the name of this value's [`DataType`], or an error
    #[inline]
    pub fn datatype_name(self) -> &'scope str {
        // Safety: the pointer points to valid data, the C API function
        // is called with a valid argument.
        unsafe {
            let type_name = jl_typeof_str(self.unwrap(Private));
            let type_name_ref = CStr::from_ptr(type_name);
            type_name_ref.to_str().unwrap()
        }
    }
}

/// # Type checking
///
/// Many properties of Julia types can be checked, including whether instances of the type are
/// mutable, if the value is an array, and so on. The method [`Value::is`] can be used to perform
/// these checks. All these checks implement the [`Typecheck`] trait. If the type that implements
/// this trait also implements [`ValidLayout`], the typecheck indicates whether or not the value
/// can be cast to or unboxed as that type.
impl Value<'_, '_> {
    /// Performs the given typecheck:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = Builder::new().start_local().unwrap();
    /// julia.local_scope::<_, 1>(|mut frame| {
    ///     let i = Value::new(&mut frame, 2u64);
    ///     assert!(i.is::<u64>());
    /// });
    /// # }
    /// ```
    ///
    /// A full list of supported checks can be found [here].
    ///
    /// [`JuliaStruct`]: crate::data::managed::traits::julia_struct::JuliaStruct
    /// [here]: ../../../layout/typecheck/trait.Typecheck.html#implementors
    #[inline]
    pub fn is<T: Typecheck>(self) -> bool {
        self.datatype().is::<T>()
    }

    /// Returns true if `self` is a subtype of `sup`.
    #[inline]
    pub fn subtype(self, sup: Value) -> bool {
        // Safety: the pointers point to valid data, the C API function
        // is called with valid arguments.
        unsafe { jl_subtype(self.unwrap(Private), sup.unwrap(Private)) != 0 }
    }

    /// Returns true if `self` is the type of a `DataType`, `UnionAll`, `Union`, or `Union{}`.
    #[inline]
    pub fn is_kind(self) -> bool {
        // Safety: this method can only be called from a thread known to Julia, its lifetime is
        // never used
        let global = unsafe { Unrooted::new() };

        let ptr = self.unwrap(Private);
        ptr == DataType::datatype_type(&global).unwrap(Private).cast()
            || ptr == DataType::unionall_type(&global).unwrap(Private).cast()
            || ptr == DataType::uniontype_type(&global).unwrap(Private).cast()
            || ptr == DataType::typeofbottom_type(&global).unwrap(Private).cast()
    }

    /// Returns true if `self` is a `DataType`, `UnionAll`, `Union`, or `Union{}`.
    #[inline]
    pub fn is_type(self) -> bool {
        Value::is_kind(self.datatype().as_value())
    }

    /// Returns true if `self` is of type `ty`.
    #[inline]
    pub fn isa(self, ty: Value) -> bool {
        // Safety: the pointers point to valid data, the C API function
        // is called with valid arguments.
        unsafe { jl_isa(self.unwrap(Private), ty.unwrap(Private)) != 0 }
    }

    /// Returns `true` if `self` depends on the type parameter `tvar`.
    pub fn has_typevar(self, tvar: TypeVar) -> bool {
        unsafe { jl_has_typevar(self.unwrap(Private), tvar.unwrap(Private)) != 0 }
    }
}

/// These methods let you track a `Value`, while it's tracked it's internal pointer is
/// dereferenced and you can access its contents directly.
///
/// Tracking works with a ledger that's shared between all active instances of jlrs. This ledger
/// contains a list of all active borrows, which lets it be used to prevent mutable aliasing.
/// Unfortunately, this system isn't perfect, it's unaware of how this data is used in Julia. It's
/// your responsibility that you only try to access data which isn't being used by some task
/// running in the background. The raw ledger API is available in `JlrsCore.Ledger`, you can prevent
/// mutable access to data by tracking from Julia by calling these functions. If you do so, you
/// should use a finalizer to ensure the borrow is removed from the ledger when the data is
/// finalized.
impl<'scope, 'data> Value<'scope, 'data> {
    /// Track `self` immutably.
    ///
    /// When this method is called on some `Value`, it's checked if the layout of `T` matches
    /// that of the data and if the data is already mutably borrowed from Rust. If it's not, the
    /// data is derefenced and returned as a `Tracked` which provides direct access to the
    /// reference.
    ///
    /// If the data is immutable the borrow isn't tracked by the ledger because it can't be
    /// mutably borrowed.
    #[inline]
    pub fn track_shared<'borrow, T: ValidLayout>(
        &'borrow self,
    ) -> JlrsResult<Tracked<'borrow, 'scope, 'data, T>> {
        let ty = self.datatype();
        if !T::valid_layout(ty.as_value()) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        if !ty.mutable() {
            unsafe {
                return Ok(Tracked::new(self));
            }
        }

        unsafe {
            Ledger::try_borrow_shared(*self)?;
            Ok(Tracked::new(self))
        }
    }

    /// Track `self` exclusively.
    ///
    /// When this method is called on some `Value`, it's checked if the layout of `T` matches
    /// that of the data and if the data is already borrowed from Rust. If it's not, the data is
    /// mutably derefenced and returned as a `TrackedMut` which provides direct access to the
    /// mutable reference.
    ///
    /// Note that if `T` contains any references to Julia data, if such a reference is mutated
    /// through `TrackedMut` you must call [`write_barrier`] after mutating it. This ensures the
    /// garbage collector remains aware of old-generation objects pointing to young-generation
    /// objects.
    ///
    /// In general, it's recommended that only fields that contain no references to Julia data are
    /// updated through `TrackedMut`.
    ///
    /// Safety:
    ///
    /// This method can only track references that exist in Rust code. It also gives unrestricted
    /// mutable access to the contents of the data, which is inherently unsafe.
    ///
    /// [`write_barrier`]: crate::memory::gc::write_barrier
    #[inline]
    pub unsafe fn track_exclusive<'borrow, T: ValidLayout>(
        &'borrow mut self,
    ) -> JlrsResult<TrackedMut<'borrow, 'scope, 'data, T>> {
        let ty = self.datatype();

        if !ty.mutable() {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(TypeError::Immutable { value_type })?;
        }

        if !T::valid_layout(ty.as_value()) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        Ledger::try_borrow_exclusive(*self)?;
        Ok(TrackedMut::new(self))
    }

    /// Returns `true` if `self` is currently tracked.
    #[inline]
    pub fn is_tracked(self) -> JlrsResult<bool> {
        Ledger::is_borrowed(self)
    }

    /// Returns `true` if `self` is currently tracked.
    #[inline]
    pub fn is_tracked_shared(self) -> JlrsResult<bool> {
        Ledger::is_borrowed_shared(self)
    }

    /// Returns `true` if `self` is currently mutably tracked.
    #[inline]
    pub fn is_tracked_exclusive(self) -> JlrsResult<bool> {
        Ledger::is_borrowed_exclusive(self)
    }
}

impl ValueUnbound {
    /// Track `self` immutably.
    ///
    /// This method is equivalent to [`Value::track_shared`] except it takes `self` by value and
    /// can only be used with `ValueUnbound`. This is intended to be used from `ccall`able
    /// functions that take a [`Value`] and operate on its contents in another thread.
    ///
    /// Because `T: Send`, it's not possible to track types that contain references to Julia data.
    ///
    /// Safety:
    ///
    /// The returned instance of `Tracked` must only be used in the `ccall`ed function and the
    /// `AsyncCallback`.
    #[inline]
    pub unsafe fn track_shared_unbound<T: ValidLayout + Send>(
        self,
    ) -> JlrsResult<Tracked<'static, 'static, 'static, T>> {
        let ty = self.datatype();
        if !T::valid_layout(ty.as_value()) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        unsafe {
            Ledger::try_borrow_shared(self)?;
            Ok(Tracked::new_owned(self))
        }
    }

    /// Track `self` exclusively.
    ///
    /// This method is equivalent to [`Value::track_exclusive`] except it takes `self` by value
    /// and can only be used with `ValueUnbound`. This is intended to be used from `ccall`able
    /// functions that take a [`Value`] and operate on its contents in another thread.
    ///
    /// Because `T: Send`, it's not possible to track types that contain references to Julia data.
    ///
    /// Safety:
    ///
    /// The returned instance of `TrackedMut` must only be used in the `ccall`ed function and the
    /// `AsyncCallback`.
    #[inline]
    pub unsafe fn track_exclusive_unbound<T: ValidLayout + Send>(
        self,
    ) -> JlrsResult<TrackedMut<'static, 'static, 'static, T>> {
        let ty = self.datatype();

        if !ty.mutable() {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(TypeError::Immutable { value_type })?;
        }

        if !T::valid_layout(ty.as_value()) {
            let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE).into();
            Err(AccessError::InvalidLayout { value_type })?;
        }

        Ledger::try_borrow_exclusive(self)?;
        Ok(TrackedMut::new_owned(self))
    }
}

/// # Lifetime management
///
/// Values have two lifetimes, `'scope` and `'data`. The first ensures that a value can only be
/// used while it's rooted, the second ensures that values that (might) borrow array data from
/// Rust are also restricted by the lifetime of that borrow. This second restriction can be
/// relaxed with [`Value::assume_owned`] if it doesn't borrow any data from Rust.
impl<'scope, 'data> Value<'scope, 'data> {
    /// If you call a Julia function with one or more borrowed arrays as arguments, its result can
    /// only be used when all the borrows are active. If this result doesn't contain any borrowed
    /// data this function can be used to relax its second lifetime to `'static`.
    ///
    /// Safety: The value must not contain any data borrowed from Rust.
    #[inline]
    pub unsafe fn assume_owned(self) -> Value<'scope, 'static> {
        Value::wrap_non_null(self.unwrap_non_null(Private), Private)
    }
}

/// # Conversions
///
/// There are two ways to convert a [`Value`] to some other type. The first is casting, which is
/// used to convert a [`Value`] to the appropriate managed type. For example, if the
/// [`Value`] is a Julia array it can be cast to [`Array`]. Because this only involves a pointer
/// cast it's always possible to convert a managed type to a [`Value`] by calling
/// [`Managed::as_value`]. The second way is unboxing, which is used to copy the data the
/// [`Value`] points to to Rust. If a [`Value`] is a `UInt8`, it can be unboxed as a `u8`. By
/// default, jlrs can unbox the default primitive types and Julia strings, but the [`Unbox`] trait
/// can be implemented for other types. It's recommended that you use JlrsReflect.jl to do so.
/// Unlike casting, unboxing dereferences the pointer. As a result it loses its header, so an
/// unboxed value can't be used as a [`Value`] again without reallocating it.
///
/// [`Array`]: crate::data::managed::array::Array
impl<'scope, 'data> Value<'scope, 'data> {
    /// Cast the value to a managed type `T`. Returns an error if the conversion is invalid.
    #[inline]
    pub fn cast<T: Managed<'scope, 'data> + Typecheck>(self) -> JlrsResult<T> {
        if self.is::<T>() {
            // Safety: self.is::<T>() returning true guarantees this is safe
            unsafe { Ok(self.cast_unchecked()) }
        } else {
            Err(AccessError::InvalidLayout {
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }
    }

    /// Cast the value to a managed type `T` without checking if this conversion is valid.
    ///
    /// Safety: You must guarantee `self.is::<T>()` would have returned `true`.
    #[inline]
    pub unsafe fn cast_unchecked<T: Managed<'scope, 'data>>(self) -> T {
        T::from_value_unchecked(self, Private)
    }

    /// Unbox the contents of the value as the output type associated with `T`. Returns an error
    /// if the layout of `T::Output` is incompatible with the layout of the type in Julia.
    #[inline]
    pub fn unbox<T: Unbox + Typecheck>(self) -> JlrsResult<T::Output> {
        if !self.is::<T>() {
            Err(AccessError::InvalidLayout {
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        // Safety: self.is::<T>() returning true guarantees this is safe
        unsafe { Ok(T::unbox(self)) }
    }

    /// Unbox the contents of the value as the output type associated with `T` without checking
    /// if the layout of `T::Output` is compatible with the layout of the type in Julia.
    ///
    /// Safety: You must guarantee `self.is::<T>()` would have returned `true`.
    #[inline]
    pub unsafe fn unbox_unchecked<T: Unbox>(self) -> T::Output {
        T::unbox(self)
    }

    /// Convert this value to a typed value if this value is an instance of the constructed type.
    pub fn as_typed<'target, T: ConstructType, Tgt: Target<'target>>(
        self,
        target: &Tgt,
    ) -> JlrsResult<TypedValue<'scope, 'data, T>> {
        target.with_local_scope::<_, _, 1>(|_, mut frame| {
            let ty = T::construct_type(&mut frame);
            if self.isa(ty) {
                unsafe { Ok(TypedValue::<T>::from_value_unchecked(self)) }
            } else {
                Err(TypeError::NotA {
                    value: self.display_string_or("<Cannot display value>"),
                    field_type: ty.display_string_or("<Cannot display type>"),
                })?
            }
        })
    }

    /// Convert this value to a typed value without checking if the conversion is valid.
    ///
    /// Safety: the converted value must be an instance of the constructed type.
    #[inline]
    pub unsafe fn as_typed_unchecked<T: ConstructType>(self) -> TypedValue<'scope, 'data, T> {
        TypedValue::<T>::from_value_unchecked(self)
    }

    /// Returns a pointer to the data, this is useful when the output type of `Unbox` is different
    /// than the implementation type and you have to write a custom unboxing function. It's your
    /// responsibility this pointer is used correctly.
    #[inline]
    pub fn data_ptr(self) -> NonNull<c_void> {
        self.unwrap_non_null(Private).cast()
    }
}

/// # Fields
///
/// Most Julia values have fields. For example, if the value is an instance of this struct:
///
/// ```julia
/// struct Example
///    fielda
///    fieldb::UInt32
/// end
/// ```
///
/// it will have two fields, `fielda` and `fieldb`. The first field is a pointer field, the second
/// is stored inline as a `u32`. It's possible to safely access the raw contents of these fields
/// with the method [`Value::field_accessor`]. The first field can be accessed as a [`ValueRef`],
/// the second as a `u32`.
impl<'scope, 'data> Value<'scope, 'data> {
    /// Returns the field names of this value as a slice of `Symbol`s.
    #[inline]
    pub fn field_names(self) -> &'scope [Symbol<'scope>] {
        // Symbol and SymbolRef have the same layout, and this data is non-null. Symbols are
        // globally rooted.
        unsafe {
            std::mem::transmute(
                self.datatype()
                    .field_names()
                    .data()
                    .as_atomic_slice()
                    .assume_immutable_non_null(),
            )
        }
    }

    /// Returns the number of fields the underlying Julia value has.
    #[inline]
    pub fn n_fields(self) -> usize {
        self.datatype().n_fields().unwrap() as _
    }

    /// Returns an accessor to access the contents of this value without allocating temporary Julia data.
    #[inline]
    pub fn field_accessor(self) -> FieldAccessor<'scope, 'data> {
        FieldAccessor::new(self)
    }

    /// Roots the field at index `idx` if it exists and returns it, or a
    /// `JlrsError::AccessError` if the index is out of bounds.
    pub fn get_nth_field<'target, Tgt>(
        self,
        target: Tgt,
        idx: usize,
    ) -> JlrsResult<ValueData<'target, 'data, Tgt>>
    where
        Tgt: Target<'target>,
    {
        if idx >= self.n_fields() {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields(),
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        if self.is::<Module>() {
            Err(AccessError::ModuleField)?
        }

        // Safety: the bounds check succeeded, the pointer points to valid data. The result is
        // rooted immediately.
        unsafe {
            let fld_ptr = jl_get_nth_field(self.unwrap(Private), idx as _);
            if fld_ptr.is_null() {
                Err(AccessError::UndefRef)?;
            }

            Ok(target.data_from_ptr(NonNull::new_unchecked(fld_ptr), Private))
        }
    }

    /// Returns the field at index `idx` if it's a pointer field.
    ///
    /// If the field doesn't exist or if the field can't be referenced because its data is stored
    /// inline, a `JlrsError::AccessError` is returned.
    pub fn get_nth_field_ref(self, idx: usize) -> JlrsResult<ValueRef<'scope, 'data>> {
        let ty = self.datatype();
        if idx >= ty.n_fields().unwrap() as _ {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields: self.n_fields(),
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        if self.is::<Module>() {
            Err(AccessError::ModuleField)?
        }

        // Safety: the bounds check succeeded, the pointer points to valid data. All C API
        // functions are called with valid arguments. The result is rooted immediately.
        unsafe {
            if jlrs_field_isptr(ty.unwrap(Private), idx as _) == 0 {
                let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE);

                let field_name = if let Some(field_name) = self.field_names().get(idx) {
                    field_name
                        .as_str()
                        .unwrap_or("<Cannot display field name>")
                        .to_string()
                } else {
                    format!("{}", idx)
                };

                Err(AccessError::NotAPointerField {
                    value_type: value_type,
                    field_name,
                })?
            }

            Ok(ValueRef::wrap(NonNull::new_unchecked(
                jl_get_nth_field_noalloc(self.unwrap(Private), idx),
            )))
        }
    }

    /// Roots the field with the name `field_name` if it exists and returns it, or a
    /// `JlrsError::AccessError` if there's no field with that name.
    pub fn get_field<'target, N, Tgt>(
        self,
        target: Tgt,
        field_name: N,
    ) -> JlrsResult<ValueData<'target, 'data, Tgt>>
    where
        N: ToSymbol,
        Tgt: Target<'target>,
    {
        if self.is::<Module>() {
            Err(AccessError::ModuleField)?
        }

        // Safety: the pointer points to valid data, the C API function is called with valid
        // arguments, the result is rooted immediately.
        unsafe {
            let symbol = field_name.to_symbol_priv(Private);
            let idx = jl_field_index(self.datatype().unwrap(Private), symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(AccessError::NoSuchField {
                    type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            let fld_ptr = jl_get_nth_field(self.unwrap(Private), idx as _);
            if fld_ptr.is_null() {
                Err(AccessError::UndefRef)?;
            }

            Ok(target.data_from_ptr(NonNull::new_unchecked(fld_ptr), Private))
        }
    }

    /// Returns the field with the name `field_name` if it's a pointer field.
    ///
    /// If the field doesn't exist or if the field can't be referenced because its data is stored
    /// inline, a `JlrsError::AccessError` is returned.
    pub fn get_field_ref<N>(self, field_name: N) -> JlrsResult<Option<ValueRef<'scope, 'data>>>
    where
        N: ToSymbol,
    {
        if self.is::<Module>() {
            Err(AccessError::ModuleField)?
        }

        // Safety: the pointer points to valid data. All C API functions are called with valid
        // arguments.
        unsafe {
            let symbol = field_name.to_symbol_priv(Private);
            let ty = self.datatype();
            let idx = jl_field_index(ty.unwrap(Private), symbol.unwrap(Private), 0);

            if idx < 0 {
                Err(AccessError::NoSuchField {
                    type_name: ty.display_string_or(CANNOT_DISPLAY_TYPE),
                    field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
                })?
            }

            if jlrs_field_isptr(ty.unwrap(Private), idx as _) == 0 {
                let idx = idx as usize;
                let value_type = ty.display_string_or(CANNOT_DISPLAY_TYPE);

                let field_name = self.field_names()[idx]
                    .as_str()
                    .unwrap_or("<Cannot display field name>")
                    .to_string();

                Err(AccessError::NotAPointerField {
                    value_type: value_type,
                    field_name,
                })?
            }

            let ptr = jl_get_nth_field_noalloc(self.unwrap(Private), idx as _);

            if ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(ValueRef::wrap(NonNull::new_unchecked(ptr))))
            }
        }
    }

    /// Set the value of the field at `idx`. If Julia throws an exception it's caught, rooted in
    /// the frame, and returned. If the index is out of bounds or the value is not a subtype of
    /// the field an error is returned,
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.

    pub unsafe fn set_nth_field<'target, Tgt>(
        self,
        target: Tgt,
        idx: usize,
        value: Value<'_, 'data>,
    ) -> JlrsResult<TargetException<'target, 'data, (), Tgt>>
    where
        Tgt: Target<'target>,
    {
        if self.is::<Module>() {
            Err(AccessError::ModuleField)?
        }

        let n_fields = self.n_fields();
        if n_fields <= idx {
            Err(AccessError::OutOfBoundsField {
                idx,
                n_fields,
                value_type: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?;
        }

        let field_type = self
            .datatype()
            .field_types()
            .data()
            .get(&target, idx)
            .unwrap()
            .as_value();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(TypeError::NotASubtype {
                field_type: field_type.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: value.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        let callback = || jl_set_nth_field(self.unwrap(Private), idx, value.unwrap(Private));

        let res = match catch_exceptions(callback, unwrap_exc) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };

        Ok(target.exception_from_ptr(res, Private))
    }

    /// Set the value of the field at `idx`. If Julia throws an exception the process aborts.
    ///
    /// Safety: this method doesn't check if the type of the value is a subtype of the field's
    /// type. Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is also not prevented.
    #[inline]
    pub unsafe fn set_nth_field_unchecked(self, idx: usize, value: Value<'_, 'data>) {
        jl_set_nth_field(self.unwrap(Private), idx, value.unwrap(Private))
    }

    /// Set the value of the field with the name `field_name`. If Julia throws an exception it's
    /// caught, rooted in the frame, and returned. If there's no field with the given name or the
    /// value is not a subtype of the field an error is returned.
    ///
    /// Safety: Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is not prevented.
    pub unsafe fn set_field<'target, N, Tgt>(
        self,
        target: Tgt,
        field_name: N,
        value: Value<'_, 'data>,
    ) -> JlrsResult<TargetException<'target, 'data, (), Tgt>>
    where
        N: ToSymbol,
        Tgt: Target<'target>,
    {
        if self.is::<Module>() {
            Err(AccessError::ModuleField)?
        }

        let symbol = field_name.to_symbol_priv(Private);
        let idx = jl_field_index(self.datatype().unwrap(Private), symbol.unwrap(Private), 0);

        if idx < 0 {
            Err(AccessError::NoSuchField {
                type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
            })?
        }

        let field_type = self
            .datatype()
            .field_types()
            .data()
            .get(&target, idx as usize)
            .unwrap()
            .as_value();
        let dt = value.datatype();

        if !Value::subtype(dt.as_value(), field_type) {
            Err(TypeError::NotASubtype {
                field_type: field_type.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: value.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }

        let callback =
            || jl_set_nth_field(self.unwrap(Private), idx as usize, value.unwrap(Private));

        let res = match catch_exceptions(callback, unwrap_exc) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };

        Ok(target.exception_from_ptr(res, Private))
    }

    /// Set the value of the field with the name `field_name`. If Julia throws an exception the
    /// process aborts. If there's no field with the given name an error is returned.
    ///
    /// Safety: this method doesn't check if the type of the value is a subtype of the field's
    /// type. Mutating things that should absolutely not be mutated, like the fields of a
    /// `DataType`, is also not prevented.
    pub unsafe fn set_field_unchecked<N>(
        self,
        field_name: N,
        value: Value<'_, 'data>,
    ) -> JlrsResult<()>
    where
        N: ToSymbol,
    {
        let symbol = field_name.to_symbol_priv(Private);
        let idx = jl_field_index(self.datatype().unwrap(Private), symbol.unwrap(Private), 0);

        if idx < 0 {
            Err(AccessError::NoSuchField {
                type_name: self.datatype().display_string_or(CANNOT_DISPLAY_TYPE),
                field_name: symbol.as_str().unwrap_or("<Non-UTF8 symbol>").into(),
            })?
        }
        Ok(jl_set_nth_field(
            self.unwrap(Private),
            idx as usize,
            value.unwrap(Private),
        ))
    }
}

/// # Evaluate Julia code
///
/// The easiest way to call Julia from Rust is by evaluating some Julia code directly. This can be
/// used to call simple functions without any arguments provided from Rust and to execute
/// using-statements.
impl Value<'_, '_> {
    /// Execute a Julia command `cmd`, for example `Value::eval_string(&mut *frame, "sqrt(2)")` or
    /// `Value::eval_string(&mut *frame, "using LinearAlgebra")`.
    ///
    /// Safety: The command can't be checked for correctness, nothing prevents you from causing a
    /// segmentation fault with a command like `unsafe_load(Ptr{Float64}(C_NULL))`.
    #[inline]
    pub unsafe fn eval_string<'target, C, Tgt>(
        target: Tgt,
        cmd: C,
    ) -> ValueResult<'target, 'static, Tgt>
    where
        C: AsRef<str>,
        Tgt: Target<'target>,
    {
        let cmd = cmd.as_ref();
        let cmd_cstring = CString::new(cmd).map_err(JlrsError::other).unwrap();
        let cmd_ptr = cmd_cstring.as_ptr();
        let res = jl_eval_string(cmd_ptr);
        let exc = jl_exception_occurred();
        let output = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };
        target.result_from_ptr(output, Private)
    }

    /// Execute a Julia command `cmd`. This is equivalent to `Value::eval_string`, but uses a
    /// null-terminated string.
    ///
    /// Safety: The command can't be checked for correctness, nothing prevents you from causing a
    /// segmentation fault with a command like `unsafe_load(Ptr{Float64}(C_NULL))`.
    #[inline]
    pub unsafe fn eval_cstring<'target, C, Tgt>(
        target: Tgt,
        cmd: C,
    ) -> ValueResult<'target, 'static, Tgt>
    where
        C: AsRef<CStr>,
        Tgt: Target<'target>,
    {
        let cmd = cmd.as_ref();
        let cmd_ptr = cmd.as_ptr();
        let res = jl_eval_string(cmd_ptr);
        let exc = jl_exception_occurred();
        let output = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };
        target.result_from_ptr(output, Private)
    }

    /// Calls `include` in the `Main` module in Julia, which evaluates the file's contents in that
    /// module. This has the same effect as calling `include` in the Julia REPL.
    ///
    /// Safety: The content of the file can't be checked for correctness, nothing prevents you
    /// from causing a segmentation fault with code like `unsafe_load(Ptr{Float64}(C_NULL))`.
    pub unsafe fn include<'target, 'current, 'borrow, P, Tgt>(
        target: Tgt,
        path: P,
    ) -> JlrsResult<ValueResult<'target, 'static, Tgt>>
    where
        P: AsRef<Path>,
        Tgt: Target<'target>,
    {
        if path.as_ref().exists() {
            return target.with_local_scope::<_, _, 1>(|target, mut frame| {
                let path_jl_str = JuliaString::new(&mut frame, path.as_ref().to_string_lossy());
                let include_func = Module::main(&frame)
                    .function(&frame, "include")?
                    .as_managed();

                Ok(include_func.call1(target, path_jl_str.as_value()))
            });
        }

        Err(IOError::NotFound {
            path: path.as_ref().to_string_lossy().into(),
        })?
    }
}

/// # Equality
impl Value<'_, '_> {
    /// Returns the object id of this value.
    #[inline]
    pub fn object_id(self) -> usize {
        // Safety: the pointer points to valid data, the C API
        // functions is called with a valid argument.
        unsafe { jl_object_id(self.unwrap(Private)) }
    }

    /// Returns true if `self` and `other` are equal.
    #[inline]
    pub fn egal(self, other: Value) -> bool {
        // Safety: the pointer points to valid data, the C API
        // functions is called with a valid argument.
        unsafe { jlrs_egal(self.unwrap(Private), other.unwrap(Private)) != 0 }
    }
}

/// # Finalization
impl Value<'_, '_> {
    /// Add a finalizer `f` to this value. The finalizer must be a Julia function, it will be
    /// called when this value is about to be freed by the garbage collector.
    ///
    /// Safety: the finalizer must be compatible with the data.
    #[inline]
    pub unsafe fn add_finalizer(self, f: Value<'_, 'static>) {
        jl_gc_add_finalizer(self.unwrap(Private), f.unwrap(Private))
    }

    /// Add a finalizer `f` to this value. The finalizer must be an `extern "C"` function that
    /// takes one argument, the value as a void pointer.
    ///
    /// Safety: the finalizer must be compatible with the data.
    #[inline]
    pub unsafe fn add_ptr_finalizer(self, f: unsafe extern "C" fn(*mut c_void) -> ()) {
        jl_gc_add_ptr_finalizer(get_tls(), self.unwrap(Private), f as *mut c_void)
    }
}

/// # Constant values.
impl<'scope> Value<'scope, 'static> {
    /// `Union{}`.
    #[inline]
    pub fn bottom_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_bottom_type), Private) }
    }

    /// `StackOverflowError`.
    #[inline]
    pub fn stackovf_exception<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_stackovf_exception), Private) }
    }

    /// `OutOfMemoryError`.
    #[inline]
    pub fn memory_exception<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_memory_exception), Private) }
    }

    /// `ReadOnlyMemoryError`.
    #[inline]
    pub fn readonlymemory_exception<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe {
            Value::wrap_non_null(NonNull::new_unchecked(jl_readonlymemory_exception), Private)
        }
    }

    /// `DivideError`.
    #[inline]
    pub fn diverror_exception<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_diverror_exception), Private) }
    }

    /// `UndefRefError`.
    #[inline]
    pub fn undefref_exception<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_undefref_exception), Private) }
    }

    /// `InterruptException`.
    #[inline]
    pub fn interrupt_exception<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_interrupt_exception), Private) }
    }

    /// An empty `Array{Any, 1}.
    #[inline]
    pub fn an_empty_vec_any<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_an_empty_vec_any), Private) }
    }

    /// An empty immutable String, "".
    #[inline]
    pub fn an_empty_string<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_an_empty_string), Private) }
    }

    /// `Array{UInt8, 1}`
    #[inline]
    pub fn array_uint8_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_uint8_type), Private) }
    }

    /// `Array{Any, 1}`
    #[inline]
    pub fn array_any_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_any_type), Private) }
    }

    /// `Array{Symbol, 1}`
    #[inline]
    pub fn array_symbol_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_symbol_type), Private) }
    }

    /// `Array{Int32, 1}`
    #[inline]
    pub fn array_int32_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_array_int32_type), Private) }
    }

    /// The empty tuple, `()`.
    #[inline]
    pub fn emptytuple<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_emptytuple), Private) }
    }

    /// The instance of `true`.
    #[inline]
    pub fn true_v<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_true), Private) }
    }

    /// The instance of `false`.
    #[inline]
    pub fn false_v<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_false), Private) }
    }

    /// The instance of `Nothing`, `nothing`.
    #[inline]
    pub fn nothing<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_nothing), Private) }
    }

    /// The handle to `stdout` as a Julia value.
    #[inline]
    pub fn stdout<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_stdout_obj()), Private) }
    }

    /// The handle to `stderr` as a Julia value.
    #[inline]
    pub fn stderr<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_stderr_obj()), Private) }
    }

    #[julia_version(since = "1.7")]
    /// The `Pair` type
    #[inline]
    pub fn pair_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe { Value::wrap_non_null(NonNull::new_unchecked(jl_pair_type), Private) }
    }

    #[julia_version(since = "1.11")]
    /// The `Pair` type
    #[inline]
    pub fn an_empty_memory_any<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe {
            Value::wrap_non_null(
                NonNull::new_unchecked(jl_sys::jl_an_empty_memory_any),
                Private,
            )
        }
    }

    #[julia_version(since = "1.8")]
    /// The `Array{UInt64,1}` type
    #[inline]
    pub fn array_uint64_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe {
            Value::wrap_non_null(
                NonNull::new_unchecked(jl_sys::jl_array_uint64_type),
                Private,
            )
        }
    }

    #[julia_version(since = "1.11")]
    /// The `Array{UInt32,1}` type
    #[inline]
    pub fn array_uint32_type<Tgt>(_: &Tgt) -> Self
    where
        Tgt: Target<'scope>,
    {
        // Safety: global constant
        unsafe {
            Value::wrap_non_null(
                NonNull::new_unchecked(jl_sys::jl_array_uint32_type),
                Private,
            )
        }
    }
}

impl<'data> Call<'data> for Value<'_, 'data> {
    #[inline]
    unsafe fn call0<'target, Tgt>(self, target: Tgt) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let res = jl_call0(self.unwrap(Private));
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }

    #[inline]
    unsafe fn call_unchecked<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
    ) -> ValueData<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>,
    {
        let args = args.as_pointers(Private);
        let v = jlrs_call_unchecked(
            self.unwrap(Private),
            args.as_ptr() as *mut _,
            args.len() as _,
        );
        target.data_from_ptr(NonNull::new_unchecked(v), Private)
    }

    #[inline]
    unsafe fn call1<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let res = jl_call1(self.unwrap(Private), arg0.unwrap(Private));
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }

    #[inline]
    unsafe fn call2<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let res = jl_call2(
            self.unwrap(Private),
            arg0.unwrap(Private),
            arg1.unwrap(Private),
        );
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }

    #[inline]
    unsafe fn call3<'target, Tgt>(
        self,
        target: Tgt,
        arg0: Value<'_, 'data>,
        arg1: Value<'_, 'data>,
        arg2: Value<'_, 'data>,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        Tgt: Target<'target>,
    {
        let res = jl_call3(
            self.unwrap(Private),
            arg0.unwrap(Private),
            arg1.unwrap(Private),
            arg2.unwrap(Private),
        );
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }

    #[inline]
    unsafe fn call<'target, 'value, V, Tgt, const N: usize>(
        self,
        target: Tgt,
        args: V,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        V: Values<'value, 'data, N>,
        Tgt: Target<'target>,
    {
        let args = args.as_slice(Private);
        let n = args.len();
        let res = jl_call(
            self.unwrap(Private),
            args.as_ptr() as *const _ as *mut _,
            n as _,
        );
        let exc = jl_exception_occurred();

        let res = if exc.is_null() {
            Ok(NonNull::new_unchecked(res))
        } else {
            Err(NonNull::new_unchecked(exc))
        };

        target.result_from_ptr(res, Private)
    }
}

impl<'value, 'data> ProvideKeywords<'value, 'data> for Value<'value, 'data> {
    #[inline]
    fn provide_keywords(
        self,
        kws: Value<'value, 'data>,
    ) -> JlrsResult<WithKeywords<'value, 'data>> {
        if !kws.is::<NamedTuple>() {
            let ty = kws.datatype().display_string_or(CANNOT_DISPLAY_TYPE);
            Err(TypeError::NotANamedTuple { ty })?
        }
        Ok(WithKeywords::new(self, kws))
    }
}

impl_debug!(Value<'_, '_>);

impl<'scope, 'data> ManagedPriv<'scope, 'data> for Value<'scope, 'data> {
    type Wraps = jl_value_t;
    type WithLifetimes<'target, 'da> = Value<'target, 'da>;
    const NAME: &'static str = "Value";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

/// A reference to a [`Value`] that has not been explicitly rooted.
pub type ValueRef<'scope, 'data> = Ref<'scope, 'data, Value<'scope, 'data>>;

/// A [`ValueRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Value`].
pub type ValueRet = Ref<'static, 'static, Value<'static, 'static>>;

/// A [`Value`] with static lifetimes.
///
/// This is a useful shorthand for signatures of `ccall`able functions that take a [`Value`] and
/// operate on its contents in another thread.
pub type ValueUnbound = Value<'static, 'static>;

unsafe impl ValidLayout for ValueRef<'_, '_> {
    #[inline]
    fn valid_layout(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            !dt.is_inline_alloc()
        } else if v.is::<UnionAll>() {
            true
        } else if v.is::<Union>() {
            let u = unsafe { v.cast_unchecked::<Union>() };
            !u.is_bits_union()
        } else {
            false
        }
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        DataType::any_type(target).as_value()
    }

    const IS_REF: bool = true;
}

unsafe impl ValidLayout for Value<'static, 'static> {
    #[inline]
    fn valid_layout(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            !dt.is_inline_alloc()
        } else if v.is::<UnionAll>() {
            true
        } else if v.is::<Union>() {
            let u = unsafe { v.cast_unchecked::<Union>() };
            !u.is_bits_union()
        } else {
            false
        }
    }

    #[inline]
    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        DataType::any_type(target).as_value()
    }

    const IS_REF: bool = true;
}

unsafe impl ValidField for Option<ValueRef<'_, '_>> {
    #[inline]
    fn valid_field(v: Value) -> bool {
        if v.is::<DataType>() {
            let dt = unsafe { v.cast_unchecked::<DataType>() };
            !dt.is_inline_alloc()
        } else if v.is::<UnionAll>() {
            true
        } else if v.is::<Union>() {
            let u = unsafe { v.cast_unchecked::<Union>() };
            !u.is_bits_union()
        } else {
            false
        }
    }
}

use crate::memory::target::TargetType;

/// `Value` or `ValueRef`, depending on the target type `Tgt`.
pub type ValueData<'target, 'data, Tgt> =
    <Tgt as TargetType<'target>>::Data<'data, Value<'target, 'data>>;

/// `JuliaResult<Value>` or `JuliaResultRef<ValueRef>`, depending on the target type `Tgt`.
pub type ValueResult<'target, 'data, Tgt> =
    TargetResult<'target, 'data, Value<'target, 'data>, Tgt>;

impl_ccall_arg_managed!(Value, 2);

impl_construct_type_managed!(Value, 2, jl_any_type);

unsafe fn check_union_equivalent<'target, L: ValidLayout, Tgt: Target<'target>>(
    target: &Tgt,
    idx: usize,
    u: Union,
) -> JlrsResult<()> {
    // TODO: Union{}?

    // Field is a bits union. Check if the union in the layout and the constructed type contain
    // the same types.
    let type_obj = L::type_object(target);
    if let Ok(type_obj) = type_obj.cast::<DataType>() {
        let ft_in_layout = type_obj.field_type_unchecked(idx);
        if ft_in_layout != u {
            Err(TypeError::IncompatibleType {
                element_type: u.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: ft_in_layout.display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }
    } else if let Ok(type_obj) = type_obj.cast::<UnionAll>() {
        let base_type_obj = type_obj.base_type();
        let ft_in_layout = base_type_obj.field_type_unchecked(idx);
        if ft_in_layout != u {
            Err(TypeError::IncompatibleType {
                element_type: u.display_string_or(CANNOT_DISPLAY_TYPE),
                value_type: ft_in_layout.display_string_or(CANNOT_DISPLAY_TYPE),
            })?
        }
    } else {
        Err(TypeError::NotA {
            value: type_obj.display_string_or(CANNOT_DISPLAY_TYPE),
            field_type: "DataType or UnionAll".into(),
        })?
    }

    Ok(())
}

unsafe fn check_field_isa<L: ValidLayout>(
    ft: Value,
    l_ptr: *const L,
    offset: usize,
) -> JlrsResult<()> {
    // Field is a pointer field, check if the provided value in that position is a valid instance
    // of the field type.
    if let Some(field) = l_ptr
        .cast::<MaybeUninit<u8>>()
        .add(offset)
        .cast::<Value>()
        .as_ref()
    {
        if !field.isa(ft) {
            Err(TypeError::NotA {
                value: field.display_string_or("<Cannot display value>"),
                field_type: ft.display_string_or("<Cannot display type>"),
            })?
        }
    }

    Ok(())
}
