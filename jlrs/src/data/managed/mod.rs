//! Types that directly reference data managed by Julia.
//!
//! In this module you'll find types that represent Julia's managed types. These are mutable types
//! like [`Module`], [`DataType`], and [`Array`] which are defined by the C API and provide access
//! to some specific functionality from that API. For example, [`Module`] provides access to the
//! contents of Julia modules, and [`Array`] access to the contents of Julia arrays.
//!
//! The most common of these types is [`Value`], which represents some arbitrary managed data.
//! Whenever you call a Julia function its arguments must be of this type, and a new one is
//! returned. All managed data is a valid [`Value`] and can be converted to that type by calling
//! [`Managed::as_value`].
//!
//! One useful guarantee provided by managed types is that they point to existing data which won't
//! be freed until its lifetime has expired. If data is returned that isn't rooted, jlrs returns a
//! [`Ref`] instead of the managed type. Because the data isn't rooted it's not guaranteed to
//! remain valid while it can be used. For more information about rooting see the documentation of
//! the [`memory`] module.
//!
//! [`memory`]: crate::memory
//! [`DataType`]: crate::data::managed::datatype::DataType
//! [`Array`]: crate::data::managed::array::Array

macro_rules! impl_construct_type_managed {
    ($ty:ident, 1, $jl_ty:expr) => {
        unsafe impl crate::data::types::construct_type::ConstructType for $ty<'_> {
            type Static = $ty<'static>;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, 'current, 'borrow, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                unsafe {
                    target.data_from_ptr(
                        NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>()),
                        $crate::private::Private,
                    )
                }
            }

            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _env: &$crate::data::types::construct_type::TypeVarEnv,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target> {
                    unsafe {
                        target.data_from_ptr(
                            NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>()),
                            $crate::private::Private,
                        )
                    }
            }

            #[inline]
            fn base_type<'target, Tgt>(_target: &Tgt) -> Option<$crate::data::managed::value::Value<'target, 'static>>
            where
                Tgt: crate::memory::target::Target<'target>,
            {
                unsafe {
                    let ptr = NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>());
                    Some(<$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                        ptr,
                        $crate::private::Private,
                    ))
                }
            }
        }
    };
    ($ty:ident, 2, $jl_ty:expr) => {
        unsafe impl crate::data::types::construct_type::ConstructType for $ty<'_, '_> {
            type Static = $ty<'static, 'static>;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, 'current, 'borrow, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                unsafe {
                    target.data_from_ptr(
                        NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>()),
                        $crate::private::Private,
                    )
                }
            }

            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _env: &$crate::data::types::construct_type::TypeVarEnv,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target> {
                    unsafe {
                        target.data_from_ptr(
                            NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>()),
                            $crate::private::Private,
                        )
                    }
            }

            #[inline]
            fn base_type<'target, Tgt>(_target: &Tgt) -> Option<$crate::data::managed::value::Value<'target, 'static>>
            where
                Tgt: crate::memory::target::Target<'target>,
            {
                unsafe {
                    let ptr = NonNull::new_unchecked($jl_ty.cast::<::jl_sys::jl_value_t>());
                    Some(<$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                        ptr,
                        $crate::private::Private,
                    ))
                }
            }
        }
    };
}

macro_rules! impl_ccall_arg_managed {
    ($ty:ident, 1) => {
        unsafe impl<'scope> $crate::convert::ccall_types::CCallArg for $ty<'scope> {
            type CCallArgType = $crate::data::managed::value::Value<'scope, 'static>;
            type FunctionArgType = $ty<'scope>;
        }

        unsafe impl $crate::convert::ccall_types::CCallReturn
            for $crate::data::managed::Ref<'static, 'static, $ty<'static>>
        {
            type CCallReturnType = $crate::data::managed::value::Value<'static, 'static>;
            type FunctionReturnType = $ty<'static>;
            type ReturnAs = Self;

            #[inline]
            unsafe fn return_or_throw(self) -> Self::ReturnAs {
                self
            }
        }
    };

    ($ty:ident, 2) => {
        unsafe impl<'scope, 'data> $crate::convert::ccall_types::CCallArg for $ty<'scope, 'data> {
            type CCallArgType = $crate::data::managed::value::Value<'static, 'static>;
            type FunctionArgType = $ty<'scope, 'data>;
        }

        unsafe impl $crate::convert::ccall_types::CCallReturn
            for $crate::data::managed::Ref<'static, 'static, $ty<'static, 'static>>
        {
            type CCallReturnType = $crate::data::managed::value::Value<'static, 'static>;
            type FunctionReturnType = $ty<'static, 'static>;
            type ReturnAs = Self;

            #[inline]
            unsafe fn return_or_throw(self) -> Self::ReturnAs {
                self
            }
        }
    };
}

macro_rules! impl_into_typed {
    ($ty:ident) => {
        impl<'scope, 'data> $crate::data::managed::value::typed::AsTyped<'scope, 'data>
            for $ty<'scope>
        {
            #[inline]
            fn as_typed(
                self,
            ) -> $crate::error::JlrsResult<
                $crate::data::managed::value::typed::TypedValue<'scope, 'data, Self>,
            > {
                unsafe {
                    Ok(
                        $crate::data::managed::value::typed::TypedValue::wrap_non_null(
                            self.unwrap_non_null($crate::private::Private).cast(),
                            $crate::private::Private,
                        ),
                    )
                }
            }
        }
    };
}

macro_rules! impl_valid_layout {
    ($ref_type:ident, $type:ident, $type_obj:ident) => {
        unsafe impl $crate::data::layout::valid_layout::ValidLayout for $ref_type<'_> {
            fn valid_layout(ty: $crate::data::managed::value::Value) -> bool {
                if ty.is::<$crate::data::managed::datatype::DataType>() {
                    let dt = unsafe { ty.cast_unchecked::<$crate::data::managed::datatype::DataType>() };
                    dt.is::<$type>()
                } else {
                    false
                }
            }

            fn type_object<'target, Tgt>(
                _: &Tgt
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>
            {
                unsafe {
                    <$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                        std::ptr::NonNull::new_unchecked($type_obj.cast()),
                        $crate::private::Private
                    )
                }
            }

            const IS_REF: bool = true;
        }

        unsafe impl $crate::data::layout::valid_layout::ValidField for Option<$ref_type<'_>> {
            #[inline]
            fn valid_field(ty: $crate::data::managed::value::Value) -> bool {
                if ty.is::<$crate::data::managed::datatype::DataType>() {
                    let dt = unsafe { ty.cast_unchecked::<$crate::data::managed::datatype::DataType>() };
                    dt.is::<$type>()
                } else {
                    false
                }
            }
        }

        unsafe impl $crate::data::layout::valid_layout::ValidLayout for $type<'_> {
            #[inline]
            fn valid_layout(ty: $crate::data::managed::value::Value) -> bool {
                if ty.is::<$crate::data::managed::datatype::DataType>() {
                    let dt = unsafe { ty.cast_unchecked::<$crate::data::managed::datatype::DataType>() };
                    dt.is::<$type>()
                } else {
                    false
                }
            }

            fn type_object<'target, Tgt>(
                _: &Tgt
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>
            {
                unsafe {
                    <$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                        std::ptr::NonNull::new_unchecked($type_obj.cast()),
                        $crate::private::Private
                    )
                }
            }

            const IS_REF: bool = true;
        }

        unsafe impl $crate::data::layout::valid_layout::ValidField for Option<$type<'_>> {
            #[inline]
            fn valid_field(ty: $crate::data::managed::value::Value) -> bool {
                if ty.is::<$crate::data::managed::datatype::DataType>() {
                    let dt = unsafe { ty.cast_unchecked::<$crate::data::managed::datatype::DataType>() };
                    dt.is::<$type>()
                } else {
                    false
                }
            }
        }
    };
}

macro_rules! impl_debug {
    ($type:ty) => {
        impl ::std::fmt::Debug for $type {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match <Self as $crate::data::managed::Managed>::display_string(*self) {
                    Ok(s) => f.write_str(&s),
                    Err(e) => f.write_fmt(format_args!("<Cannot display value: {}>", e)),
                }
            }
        }
    };
}

pub mod array;
pub mod background_task;
pub mod ccall_ref;
pub mod datatype;
#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
pub mod delegated_task;
pub mod expr;
pub mod function;
pub mod module;
pub mod parachute;
pub mod simple_vector;
pub mod string;
pub mod symbol;
pub mod type_name;
pub mod type_var;
pub mod union;
pub mod union_all;
pub mod value;

use std::{
    ffi::c_void,
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::{null_mut, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use jl_sys::{jl_stderr_obj, jlrs_gc_wb};

use self::{module::JlrsCore, private::ManagedPriv};
use crate::{
    call::Call,
    data::{
        layout::valid_layout::{ValidField, ValidLayout},
        managed::{module::Module, string::JuliaString, value::Value},
    },
    error::{JlrsError, JlrsResult, CANNOT_DISPLAY_VALUE},
    memory::target::{unrooted::Unrooted, Target},
    prelude::TargetType,
    private::Private,
};

/// Trait implemented by `Ref`.
pub trait ManagedRef<'scope, 'data>:
    private::ManagedRef<'scope, 'data> + Copy + Debug + ValidLayout
{
    /// The managed type associated with this `Ref`.
    type Managed: Managed<'scope, 'data>;

    // Convert `self` to an explicit `Ref` type.
    fn into_ref(self) -> Ref<'scope, 'data, Self::Managed>;
}

impl<'scope, 'data, T> ManagedRef<'scope, 'data> for Ref<'scope, 'data, T>
where
    T: ManagedPriv<'scope, 'data>,
    Self: Copy + ValidLayout,
    Option<Self>: ValidField,
{
    type Managed = T;

    fn into_ref(self) -> Ref<'scope, 'data, Self::Managed> {
        self
    }
}

/// Trait implemented by all managed types.
pub trait Managed<'scope, 'data>: private::ManagedPriv<'scope, 'data> {
    /// `Self`, but with an arbitrary `'target` lifetime instead of `'scope`.
    type InScope<'target>: Managed<'target, 'data>;

    /// `Self`, but with an arbitrary `'da` lifetime instead of `'data`.
    type WithData<'da>: Managed<'scope, 'da>;

    /// Convert `self` to a `Ref`.
    #[inline]
    fn as_ref(self) -> Ref<'scope, 'data, Self> {
        Ref::wrap(self.unwrap_non_null(Private))
    }

    /// Convert `self` to a `Value`.
    #[inline]
    fn as_value(self) -> Value<'scope, 'data> {
        // Safety: Managed types can always be converted to a Value
        unsafe { Value::wrap_non_null(self.unwrap_non_null(Private).cast(), Private) }
    }

    /// Use the target to reroot `self`.
    #[inline]
    fn root<'target, Tgt>(self, target: Tgt) -> Tgt::Data<'data, Self::InScope<'target>>
    where
        Tgt: Target<'target>,
    {
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private).cast(), Private) }
    }

    /// Returns a new `Unrooted`.
    #[inline]
    fn unrooted_target(self) -> Unrooted<'scope> {
        unsafe { Unrooted::new() }
    }

    /// Convert `self` to its display string, i.e. the string that is shown when calling
    /// `Base.show`.
    fn display_string(self) -> JlrsResult<String> {
        // Safety: all Julia data that is accessed is globally rooted, the result is converted
        // to a String before the GC can free it.
        let global = self.unrooted_target();

        let s = unsafe {
            JlrsCore::value_string(&global)
                .call1(&global, self.as_value())
                .map_err(|e| e.as_value().error_string_or(CANNOT_DISPLAY_VALUE))
                .map_err(|e| JlrsError::exception(format!("JlrsCore.valuestring failed: {}", e)))?
                .as_value()
                .cast::<JuliaString>()?
                .as_str()?
                .to_string()
        };

        Ok(s)
    }

    /// Convert `self` to its error string, i.e. the string that is shown when calling
    /// `Base.showerror`. This string can contain ANSI color codes if this is enabled by calling
    /// [`Julia::error_color`] or [`AsyncHandle::error_color`].
    ///
    /// [`Julia::error_color`]: crate::runtime::sync_rt::Julia::error_color
    /// [`AsyncHandle::error_color`]: crate::runtime::handle::async_handle::AsyncHandle::error_color
    fn error_string(self) -> JlrsResult<String> {
        // Safety: all Julia data that is accessed is globally rooted, the result is converted
        // to a String before the GC can free it.
        let global = self.unrooted_target();

        let s = unsafe {
            JlrsCore::error_string(&global)
                .call1(&global, self.as_value())
                .map_err(|e| e.as_value().error_string_or(CANNOT_DISPLAY_VALUE))
                .map_err(|e| JlrsError::exception(format!("JlrsCore.errorstring failed: {}", e)))?
                .as_value()
                .cast::<JuliaString>()?
                .as_str()?
                .to_string()
        };

        Ok(s)
    }

    #[doc(hidden)]
    unsafe fn print_error(self) {
        let unrooted = Unrooted::new();
        let stderr = Value::wrap_non_null(NonNull::new_unchecked(jl_stderr_obj()), Private);
        let showerror = Module::base(&unrooted)
            .global(unrooted, "showerror")
            .unwrap()
            .as_value();
        showerror.call2(unrooted, stderr, self.as_value()).ok();
    }

    /// Convert `self` to its display string, i.e. the string that is shown by calling
    /// `Base.display`, or some default value.
    fn display_string_or<S: Into<String>>(self, default: S) -> String {
        self.display_string().unwrap_or(default.into())
    }

    /// Convert `self` to its error string, i.e. the string that is shown when this value is
    /// thrown as an exception, or some default value.
    fn error_string_or<S: Into<String>>(self, default: S) -> String {
        self.error_string().unwrap_or(default.into())
    }

    /// Extends the `'scope` lifetime to `'static` and converts it to a `Ref`, which allows this
    /// managed data to be leaked from a scope.
    ///
    /// This method only extends the `'scope` lifetime. This method should only be used to return
    /// managed data from a `ccall`ed function, and in combination with the `ForeignType` trait to
    /// store references to managed data in types that that implement that trait.
    #[inline]
    fn leak(self) -> Ref<'static, 'data, Self::InScope<'static>> {
        self.as_ref().leak()
    }
}

impl<'scope, 'data, W> Managed<'scope, 'data> for W
where
    W: private::ManagedPriv<'scope, 'data>,
{
    type InScope<'target> = Self::WithLifetimes<'target, 'data>;
    type WithData<'da> = Self::WithLifetimes<'scope, 'da>;
}

pub type ManagedData<'target, 'data, Tgt, T> = <Tgt as TargetType<'target>>::Data<'data, T>;

/// A reference to Julia data that is not guaranteed to be rooted.
///
/// Managed types are generally guaranteed to wrap valid, rooted data. In some cases this
/// guarantee is too strong. The garbage collector uses the roots as a starting point to
/// determine what values can be reached, as long as you can guarantee a value is reachable it's
/// safe to use. Whenever data is not rooted jlrs returns a `Ref`. Because it's not rooted it's
/// unsafe to use.
#[repr(transparent)]
pub struct Ref<'scope, 'data, T: ManagedPriv<'scope, 'data>>(
    NonNull<T::Wraps>,
    PhantomData<&'scope ()>,
    PhantomData<&'data ()>,
);

impl<'scope, 'data, T> Clone for Ref<'scope, 'data, T>
where
    T: ManagedPriv<'scope, 'data>,
{
    #[inline]
    fn clone(&self) -> Self {
        Ref(self.0, PhantomData, PhantomData)
    }
}

impl<'scope, 'data, T> Copy for Ref<'scope, 'data, T> where T: ManagedPriv<'scope, 'data> {}

impl<'scope, 'data, T: Managed<'scope, 'data>> Debug for Ref<'scope, 'data, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Ref<{}>", T::NAME)
    }
}

impl<'scope, 'data, W: Managed<'scope, 'data>> Ref<'scope, 'data, W> {
    /// Use `target` to root `self`.
    ///
    /// Safety: The data pointed to by `self` must not have been freed by the GC yet.
    #[inline]
    pub unsafe fn root<'target, Tgt>(self, target: Tgt) -> Tgt::Data<'data, W::InScope<'target>>
    where
        Tgt: Target<'target>,
    {
        target.data_from_ptr(self.ptr().cast(), Private)
    }

    #[inline]
    pub(crate) fn wrap(ptr: NonNull<W::Wraps>) -> Self {
        Ref(ptr, PhantomData, PhantomData)
    }

    /// Assume the reference still points to valid managed data and convert it to its managed type.
    ///
    /// Safety: a reference is only guaranteed to be valid as long as it's reachable from some
    /// GC root. If the reference is unreachable, the GC can free it. The GC can run whenever a
    /// safepoint is reached, this is typically the case when new Julia data is allocated.
    #[inline]
    pub unsafe fn as_managed(self) -> W {
        W::wrap_non_null(self.ptr(), Private)
    }

    /// Assume the reference still points to valid managed data and convert it to a `Value`.
    ///
    /// Safety: a reference is only guaranteed to be valid as long as it's reachable from some
    /// GC root. If the reference is unreachable, the GC can free it. The GC can run whenever a
    /// safepoint is reached, this is typically the case when new Julia data is allocated.
    #[inline]
    pub unsafe fn as_value(self) -> Value<'scope, 'data> {
        Value::wrap_non_null(self.data_ptr().cast(), Private)
    }

    /// Extends the `'data` lifetime to `'static`.
    ///
    /// Safety: this method should only be used when no data borrowed from Rust is referenced by
    /// this Julia data.
    #[inline]
    pub unsafe fn assume_owned(self) -> Ref<'scope, 'static, W::WithData<'static>> {
        Ref::wrap(self.ptr().cast())
    }

    /// Extends the `'scope` lifetime to `'static`, which allows this reference to Julia data to
    /// be leaked from a scope.
    ///
    /// Safety: this method should only be called to return Julia data from a `ccall`ed function
    /// or when storing Julia data in a foreign type.
    #[inline]
    pub fn leak(self) -> Ref<'static, 'data, W::InScope<'static>> {
        Ref::wrap(self.ptr().cast())
    }

    /// Returns a pointer to the data.
    #[inline]
    pub fn data_ptr(self) -> NonNull<c_void> {
        self.ptr().cast()
    }

    #[inline]
    pub(crate) fn ptr(self) -> NonNull<W::Wraps> {
        self.0
    }
}

/// Alias to convert a managed type `V` to its `Ret`-alias.
pub type Ret<'scope, V> = Ref<'static, 'static, <V as Managed<'scope, 'static>>::InScope<'static>>;

/// Alias to convert a `Ref`-type `V` to its `Ret`-alias.
pub type RefRet<'scope, V> = Ref<
    'static,
    'static,
    <<V as ManagedRef<'scope, 'static>>::Managed as Managed<'scope, 'static>>::InScope<'static>,
>;

/// Atomic pointer field.
#[repr(transparent)]
pub struct Atomic<'scope, 'data, T: Managed<'scope, 'data>> {
    ptr: AtomicPtr<T::Wraps>,
    _marker: PhantomData<T>,
}

impl<'scope, 'data, T: Managed<'scope, 'data>> Atomic<'scope, 'data, T> {
    #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
    pub(crate) fn new() -> Self {
        Atomic {
            ptr: AtomicPtr::new(null_mut()),
            _marker: PhantomData,
        }
    }

    /// Load the value with ordering `order`.
    pub fn load<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
        order: Ordering,
    ) -> Option<ManagedData<'target, 'data, Tgt, T::InScope<'target>>> {
        let ptr = self.ptr.load(order);
        let nn = NonNull::new(ptr)?;
        unsafe { Some(T::wrap_non_null(nn, Private).root(target)) }
    }

    /// Load the underlying pointer with ordering `order`.
    pub fn load_ptr(&self, order: Ordering) -> *mut c_void {
        self.ptr.load(order).cast()
    }

    /// Load the value with relaxed ordering.
    pub fn load_relaxed<'target, Tgt: Target<'target>>(
        &self,
        target: Tgt,
    ) -> Option<ManagedData<'target, 'data, Tgt, T::InScope<'target>>> {
        self.load(target, Ordering::Relaxed)
    }

    pub(crate) unsafe fn store(&self, data: Option<T::InScope<'_>>, order: Ordering) {
        if let Some(data) = data {
            self.ptr.store(data.unwrap(Private).cast(), order);
        } else {
            self.ptr.store(null_mut(), order)
        }
    }
}

/// A slice of atomic pointer fields.
pub struct AtomicSlice<'borrow, 'data, T: Managed<'borrow, 'data>, P: Managed<'borrow, 'data>> {
    parent: P,
    slice: &'borrow [Atomic<'borrow, 'data, T>],
}

impl<'borrow, 'data, T: Managed<'borrow, 'data>, P: Managed<'borrow, 'data>>
    AtomicSlice<'borrow, 'data, T, P>
{
    fn new(parent: P, slice: &'borrow [Atomic<'borrow, 'data, T>]) -> Self {
        AtomicSlice { parent, slice }
    }

    /// Returns the underlying slice.
    pub fn into_slice(self) -> &'borrow [Atomic<'borrow, 'data, T>] {
        self.slice
    }

    /// Returns the underlying slice, assuming the data is immutable and non-null.
    ///
    /// Safety: you must guarantee the content of this slice is never changed and contains no
    /// undefined references.
    pub unsafe fn assume_immutable_non_null(self) -> &'borrow [T] {
        std::mem::transmute(self.slice)
    }

    /// Returns the underlying slice, assuming the data is immutable but possibly null.
    ///
    /// Safety: you must guarantee the content of this slice is never changed
    pub unsafe fn assume_immutable(self) -> &'borrow [Option<T>] {
        std::mem::transmute(self.slice)
    }

    /// Atomically sets the element at position `index` to `data` with ordering `order`.
    ///
    /// Safety: Mutating Julia data is generally unsafe. You must guarantee that you're allowed to
    /// mutate this data.
    pub unsafe fn store(&self, index: usize, data: Option<T::InScope<'_>>, order: Ordering) {
        self.slice[index].store(data, order);
        jlrs_gc_wb(
            self.parent.unwrap(Private).cast(),
            data.map(|x| x.unwrap(Private)).unwrap_or(null_mut()).cast(),
        )
    }

    /// Atomically sets the element at position `index` to `data` with relaxed ordering.
    ///
    /// Safety: Mutating Julia data is generally unsafe. You must guarantee that you're allowed to
    /// mutate this data.
    pub unsafe fn store_relaxed(&self, index: usize, data: Option<T::InScope<'_>>) {
        self.store(index, data, Ordering::Relaxed)
    }
}

/// Erase the scope lifetime of managed data by replacing it with `'static`.
///
/// Safety: the returned data must never be used after it has become unrooted.
#[inline]
pub unsafe fn erase_scope_lifetime<'scope, 'data, M: Managed<'scope, 'data>>(
    data: M,
) -> M::InScope<'static> {
    data.leak().as_managed()
}

pub(crate) mod private {
    use std::{fmt::Debug, ptr::NonNull};

    use crate::{
        data::managed::{value::Value, Ref},
        private::Private,
    };

    pub trait ManagedPriv<'scope, 'data>: Copy + Debug {
        type Wraps;
        type WithLifetimes<'target, 'da>: ManagedPriv<'target, 'da>;
        const NAME: &'static str;

        // Safety: `inner` must point to valid data. If it is not
        // rooted, it must never be used after becoming unreachable.
        unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self;

        // Safety: `Self` must be the correct type for `value`.
        #[inline]
        unsafe fn from_value_unchecked(value: Value<'scope, 'data>, _: Private) -> Self {
            Self::wrap_non_null(value.unwrap_non_null(Private).cast(), Private)
        }

        fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps>;

        #[inline]
        fn unwrap(self, _: Private) -> *mut Self::Wraps {
            self.unwrap_non_null(Private).as_ptr()
        }
    }

    pub trait ManagedRef<'scope, 'data> {}

    impl<'scope, 'data, T> ManagedRef<'scope, 'data> for Ref<'scope, 'data, T> where
        T: ManagedPriv<'scope, 'data>
    {
    }
}
