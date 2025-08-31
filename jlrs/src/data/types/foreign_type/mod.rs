//! Expose Rust types to Julia.
//!
//! This functionality is intended to be used with the [`julia_module`] macro.
//!
//! All data in Julia is an instance of some [`DataType`]. While `DataType`s typically have layout
//! requirements that are not compatible with arbitrary Rust data, it is possible to create types
//! that have opaque layouts, allowing many types of Rust data to be exposed to Julia.
//!
//! Two ways to expose a Rust type are provided, either the `OpaqueType` or the `ForeignType`
//! trait can be implemented. The difference is that an opaque type is a normal, mutable type with
//! no fields in Julia and can't contain references to Julia data, while a foreign type has a
//! custom mark function that is used by the GC to mark all internal references to Julia data.
//! Unlike foreign types, opaque types may have type parameters.
//!
//! These traits are not intended to be implemented manually, use the derive macros [`OpaqueType`]
//! and [`ForeignType`] instead. Unless the type contains references to Julia data you should
//! derive `OpaqueType`. Types that implement either of these traits and their methods can be
//! exported with the `julia_module` macro as follows:
//!
//! ```
//! use std::iter::Sum;
//!
//! use jlrs::prelude::*;
//!
//! #[derive(OpaqueType)]
//! struct ExportedType {
//!     vector: Vec<u8>,
//! }
//!
//! impl ExportedType {
//!     pub fn takes_mut_self(&mut self, arg2: usize) -> JlrsResult<Nothing> {
//!         Ok(Nothing)
//!     }
//!
//!     pub fn doesnt_take_self(arg: usize) -> u32 {
//!         arg as u32
//!     }
//! }
//!
//! #[derive(OpaqueType)]
//! struct GenericExportedType<T> {
//!     vector: Vec<T>,
//! }
//!
//! impl<T: Sum + Copy> GenericExportedType<T> {
//!     fn sum(&self) -> T {
//!         self.vector.iter().copied().sum()
//!     }
//! }
//!
//! #[derive(ForeignType)]
//! struct ForeignExportedType {
//!     #[jlrs(mark)]
//!     data: Vec<Option<WeakValue<'static, 'static>>>,
//! }
//!
//! impl ForeignExportedType {
//!     // Safety: This method assumes this instance is managed by Julia. It's unsound to
//!     // call this method on an inlined instance of ForeignExportedType. It's safe to call
//!     // from Julia because `self` won't be inlined.
//!     unsafe fn push(&mut self, data: Value<'_, 'static>) {
//!         let leaked = data.as_weak().leak();
//!         self.data.push(Some(leaked));
//!
//!         // Safety: the safety requirements of this method guarantee that `self` is the
//!         // correct parent. We must insert a write barrier because `self` may be old and
//!         // `data` young.
//!         unsafe { self.write_barrier(leaked, self) };
//!     }
//! }
//!
//! // Creating a (mutable) reference to a foreign type from managed data involves tracking,
//! // which guarantees thread-safety and aliasing requirements are upheld.
//! unsafe impl Send for ForeignExportedType {}
//! unsafe impl Sync for ForeignExportedType {}
//!
//! julia_module! {
//!     become module_jl_init;
//!
//!     struct ExportedType;
//!     in ExportedType fn takes_mut_self(&mut self, arg2: usize) -> JlrsResult<Nothing>;
//!     in ExportedType fn doesnt_take_self(arg: usize) -> u32;
//!
//!     for T in [u32, u64] {
//!         struct GenericExportedType<T>;
//!         in GenericExportedType <T> fn sum(&self) -> T;
//!     };
//!
//!     struct ForeignExportedType;
//!     in ForeignExportedType fn push(&mut self, data: Value<'_, 'static>);
//! }
//! ```
//!
//! Implementations of these traits additionally implement `IntoJulia`, `ValidLayout`,
//! `Typecheck`, `Unbox` and `ConstructType`. (Mutable) references to their content can be
//! obtained by tracking the data. If you want to use an opaque or foreign type in an exported
//! method or function as a named argument, a `Value` or `TypedValue` must be used.
//!
//! [`julia_module`]: jlrs_macros::julia_module
//! [`OpaqueType`]: jlrs_macros::OpaqueType
//! [`ForeignType`]: jlrs_macros::ForeignType
pub mod mark;

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    ffi::c_void,
    marker::PhantomData,
    ptr::NonNull,
};

use fnv::{FnvBuildHasher, FnvHashMap};
use jl_sys::{
    jl_emptysvec, jl_gc_add_ptr_finalizer, jl_gc_alloc_typed, jl_gc_schedule_foreign_sweepfunc,
    jl_new_datatype, jl_new_foreign_type, jl_reinit_foreign_type, jl_value_t, 
};
use jlrs_sys::jlrs_gc_wb;

use super::typecheck::Typecheck;
use crate::{
    convert::{into_julia::IntoJulia, unbox::Unbox},
    data::{
        cache::Cache,
        layout::valid_layout::ValidLayout,
        managed::{
            Managed, Weak,
            datatype::{DataType, DataTypeData},
            erase_scope_lifetime,
            module::Module,
            private::ManagedPriv,
            simple_vector::{SimpleVector, SimpleVectorData},
            symbol::Symbol,
            value::{Value, ValueData},
        },
        types::construct_type::ConstructType,
    },
    memory::{PTls, get_tls, scope::LocalScopeExt, target::Target},
    private::Private,
};

static FOREIGN_TYPE_REGISTRY: ForeignTypes = ForeignTypes::new();

/// Define a type whose layout is invisible to Julia.
///
/// This trait is used to export Rust types to Julia in combination with the `julia_module!`
/// macro. It should not be implemented manually, but derived with the [`OpaqueType`] derive
/// macro.
///
/// Derive:
///
/// The implementation generated by the custom derive uses the following defaults:
///
/// - The `Key` type is the implenting type with all generics set to `()`.
/// - The super-type is Julia's `Any` type.
/// - All generic types are converted to type parameters, with the same name and position, without
///   any bounds.
///
/// All these defaults can be adjusted with attributes.
///
/// - The `Key` type can be set with `#[jlrs(key = "KeyType")]`, where `KeyType` is a path to a
/// Rust type that implements `Any`. This is necessary if a generic cannot be replaced with `()`
/// due to trait bounds. The key type must be unique and not depend on any generics.
/// - The super-type can be set with `#[jlrs(super = "SuperType")]`, where `SuperType` is a path
/// to a Rust type that implements `ConstructType`. The constructed type must be an abstract
/// `DataType` that doesn't depend on any of the parameters of the type.
/// - The bounds can be set with `#[jlrs(bounds = "T1 <: SuperType1, ...")]`,
/// where `T` is a generic of the type and `SuperType` is a path to a type that implements
/// `ConstructType`. These bounds are checked when the type is constructed, but are otherwise
/// invisible to Rust.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// # use std::iter::Sum;
///
/// struct GenericExportedTypeKey;
///
/// #[derive(OpaqueType)]
/// #[jlrs(key = "GenericExportedTypeKey")]
/// #[jlrs(bounds = "T <: ::jlrs::data::types::abstract_type::AnyType")]
/// #[jlrs(super_type = "::jlrs::data::types::abstract_type::AnyType")]
/// struct GenericExportedType<T: Sum + Copy> {
///     vector: Vec<T>,
/// }
///
/// impl<T: Sum + Copy> GenericExportedType<T> {
///     fn sum(&self) -> T {
///         self.vector.iter().copied().sum()
///     }
/// }
///
/// julia_module! {
///     become module_jl_init;
///
///     for T in [u32, u64] {
///         struct GenericExportedType<T>;
///         in GenericExportedType <T> fn sum(&self) -> T;
///     };
/// }
/// ```
///
/// Safety:
///
/// A type that implement this trait must not contain any managed data unless it also implements
/// [`ForeignType`]. A type that implements this trait must be exported with `julia_module`, and
/// must not be accessed outside the library that exports it. An opaque type must be
/// (re-)initialized before it may be used, the initialization function generated with
/// `julia_module` must be used to do so. Unsafe code may assume this trait has been implemented
/// correctly.
///
/// [`OpaqueType`]: jlrs_macros::OpaqueType
pub unsafe trait OpaqueType: Sized + Send + Sync + 'static {
    #[doc(hidden)]
    const IS_FOREIGN: bool = false;

    #[doc(hidden)]
    const TYPE_FN: Option<unsafe fn() -> DataType<'static>> = None;

    /// The number of type parameters.
    const N_PARAMS: usize;

    /// Identifier for this type, must be unique and not use any generic type.
    type Key: Any;

    /// The super-type of this type, `Core.Any` by default.
    #[inline]
    fn super_type<'target, Tgt>(target: Tgt) -> DataTypeData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        DataType::any_type(&target).root(target)
    }

    /// The type parameters of this type.
    ///
    /// This method returns a [`SimpleVector`] with all [`TypeVar`]s of the type, in the same
    /// order and with the same name as they appear in the generics of the type.
    ///
    /// [`TypeVar`]: crate::data::managed::type_var::TypeVar
    fn type_parameters<'target, Tgt>(target: Tgt) -> SimpleVectorData<'target, Tgt>
    where
        Tgt: Target<'target>;

    /// The variant type parameters of this type.
    ///
    /// This method returns a [`SimpleVector`] with all values of the type parameters. For
    /// example, if a type has two parameters `T` and `U`, and the variant sets `T` to `f32` and
    /// `U` to `f64`, the elements must be the `Float32` and `Float64` `DataType`s.
    fn variant_parameters<'target, Tgt>(target: Tgt) -> SimpleVectorData<'target, Tgt>
    where
        Tgt: Target<'target>;

    /// Creates a new opaque type named `name` in `module`.
    ///
    /// An opaque type must be created if it doesn't exist yet in `module`. This method is called
    /// automatically by init functions generated with the `julia_module` macro.
    ///
    /// Safety:
    ///
    /// The new type is not set as a constant in `module`, this must be done manually after
    /// calling this method. The default implementation must not be overridden, it cannot be
    /// implemented correctly without using internal functionality.
    #[inline]
    unsafe fn create_type<'target, Tgt>(
        target: Tgt,
        name: Symbol,
        module: Module,
    ) -> DataTypeData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            if Self::N_PARAMS == 0 {
                create_opaque_type::<Self, Tgt>(target, name, module)
            } else {
                create_parametric_opaque_type::<Self, Tgt>(target, name, module)
            }
        }
    }

    /// Reinitializes a previously created type.
    ///
    /// An opaque type must be reinitialized if it has been created in a precompiled module and
    /// this module is loaded. This method is called automatically by init functions generated
    /// with the `julia_module` macro.
    ///
    /// Safety:
    ///
    /// The type must have been originally created by calling `OpaqueType::create_type`. The
    /// default implementation must not be overridden, it cannot be implemented correctly without
    /// using internal functionality.
    unsafe fn reinit_type(ty: DataType) -> bool {
        unsafe {
            if Self::N_PARAMS == 0 {
                if let Some(_) = FOREIGN_TYPE_REGISTRY.find::<Self>() {
                    return true;
                }

                FOREIGN_TYPE_REGISTRY.insert::<Self>(erase_scope_lifetime(ty));

                true
            } else {
                if let Some(_) = FOREIGN_TYPE_REGISTRY.find::<Key<Self::Key>>() {
                    return true;
                }

                FOREIGN_TYPE_REGISTRY.insert::<Key<Self::Key>>(erase_scope_lifetime(ty));

                true
            }
        }
    }

    /// Creates a new variant of an opaque type named `name`.
    ///
    /// This method is called automatically by init functions generated with the `julia_module`
    /// macro.
    ///
    /// Safety:
    ///
    /// The default implementation must not be overridden, it cannot be implemented correctly
    /// without using internal functionality.
    unsafe fn create_variant<'target, Tgt>(target: Tgt, name: Symbol) -> DataTypeData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            if let Some(ty) = FOREIGN_TYPE_REGISTRY.find::<Self>() {
                return target.data_from_ptr(ty.unwrap_non_null(Private), Private);
            }

            let base_ty = FOREIGN_TYPE_REGISTRY.find::<Key<Self::Key>>();

            if base_ty.is_none() {
                panic!("Type {} was not initialized", name.as_str().unwrap());
            }

            target.with_local_scope::<_, 3>(|target, mut frame| {
                let params = Self::variant_parameters(&mut frame);
                let params = params.data();
                let params_slice = params.as_atomic_slice().assume_immutable_non_null();

                let ty = base_ty
                    .unwrap_unchecked()
                    .rewrap(&mut frame)
                    .apply_type(&mut frame, params_slice)
                    .unwrap()
                    .cast::<DataType>()
                    .unwrap();

                FOREIGN_TYPE_REGISTRY.insert::<Self>(erase_scope_lifetime(ty));

                ty.root(target)
            })
        }
    }

    /// Reinitializes the previously created variant `ty`.
    ///
    /// An opaque type must be reinitialized if it has been created in a precompiled module and
    /// this module is loaded. This method is called automatically by init functions generated
    /// with the `julia_module` macro.
    ///
    /// Safety:
    ///
    /// The datatype must have been originally created by calling `OpaqueType::create_type`. The
    /// default implementation must not be overridden, it cannot be implemented correctly without
    /// using internal functionality.
    unsafe fn reinit_variant(ty: DataType) -> bool {
        unsafe {
            if let Some(_) = FOREIGN_TYPE_REGISTRY.find::<Self>() {
                return true;
            }

            FOREIGN_TYPE_REGISTRY.insert::<Self>(erase_scope_lifetime(ty));

            true
        }
    }
}

/// Define a type whose layout is invisible to Julia with a custom mark function.
///
/// This trait is used to export Rust types to Julia in combination with the `julia_module`
/// macro. It should not be implemented manually, but derived with the [`ForeignType`] derive
/// macro.
///
/// A `ForeignType` can contain references to Julia data because it has a custom mark function.
/// This function is called by the GC to mark all internal references during its marking phase,
/// types that contain no references to Julia data should implement the `OpaqueType` trait
/// instead.
///
/// Because this trait has a `'static` lifetime bound, it's necessary to erase the `'scope`
/// lifetime of Julia data present in the implementor. This can be done by leaking data with
/// [`Weak::leak`] or [`Managed::leak`].
///
/// Be aware that whenever fields are mutated to contain new Julia data, a write barrier must be
/// inserted. See [`ForeignType::write_barrier`] for more information.
///
/// Derive:
///
/// The implementation generated by the custom derive uses the following defaults:
///
/// - The super-type is Julia's `Any` type.
///
/// This default can be adjusted with an attribute.
///
/// - The super-type can be set with `#[jlrs(super = "SuperType")]`, where `SuperType` is a path
/// to a Rust type that implements `ConstructType`. The constructed type must be an abstract
/// `DataType` that doesn't depend on any of the parameters of the type.
///
/// All fields that reference Julia data must implement [`Mark`] and be annotated with the
/// `#[mark]` attribute, fields that reference, or a custom marking function for that field must
/// be provided with `#[mark_with = mark_fn]`, where `mark_fn` is a function with the same
/// signature as [`Mark::mark`]
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// # use std::collections::HashMap;
/// use jlrs::data::types::foreign_type::{ForeignType as FT, mark::Mark};
///
/// unsafe fn mark_map<M: Mark, P: FT>(
///     data: &HashMap<(), M>,
///     ptls: jlrs::memory::PTls,
///     parent: &P,
/// ) -> usize {
///     data.values().map(|v| unsafe { v.mark(ptls, parent) }).sum()
/// }
///
/// #[derive(ForeignType)]
/// struct ForeignExportedType {
///     #[jlrs(mark)]
///     data: Vec<Option<WeakValue<'static, 'static>>>,
///     #[jlrs(mark_with = mark_map)]
///     map: HashMap<(), WeakValue<'static, 'static>>,
/// }
///
/// impl ForeignExportedType {
///     // Safety: This method assumes this instance is managed by Julia. It's unsound to
///     // call this method on an inlined instance of ForeignExportedType. It's safe to call
///     // from Julia because `self` won't be inlined.
///     unsafe fn push(&mut self, data: Value<'_, 'static>) {
///         let leaked = data.as_weak().leak();
///         self.data.push(Some(leaked));
///
///         // Safety: the safety requirements of this method guarantee that `self` is the
///         // correct parent. We must insert a write barrier because `self` may be old and
///         // `data` young.
///         unsafe { self.write_barrier(leaked, self) };
///
///         self.map.insert((), leaked);
///
///         // NB: We've already inserted a write barrier for `leaked`. We can guarantee the GC
///         // won't run during this function (no new Julia data is allocated), so we don't need
///         // to insert it again.
///     }
/// }
///
/// // Creating a (mutable) reference to a foreign type from managed data involves tracking,
/// // which guarantees thread-safety and aliasing requirements are upheld.
/// unsafe impl Send for ForeignExportedType {}
/// unsafe impl Sync for ForeignExportedType {}
///
/// julia_module! {
///     become module_jl_init;
///
///     struct ForeignExportedType;
///     in ForeignExportedType fn push(&mut self, data: Value<'_, 'static>);
/// }
/// ```
///
/// Safety:
///
/// All implementations of this trait implement `OpaqueType`, the same safety rules apply.
///
/// The implementor should reference managed data, The implementation of the `mark` method must
/// mark all these references. If it doesn't reference managed data, `OpaqueType` should be
/// implemented instead.
///
/// Whenever an internal reference to Julia data is changed while an instance of a foreign type is
/// managed by Julia, [`ForeignType::write_barrier`] must be called to ensure GC invariants are
/// maintained.
///
/// [`Weak::leak`]: crate::data::managed::Weak::leak
/// [`ForeignType`]: jlrs_macros::ForeignType
/// [`write_barrier`]: crate::memory::gc::write_barrier
/// [`Mark`]: crate::data::types::foreign_type::mark::Mark
/// [`Mark::mark`]: crate::data::types::foreign_type::mark::Mark::mark

#[diagnostic::on_unimplemented(
    message = "the trait bound `{Self}: ForeignType` is not satisfied",
    label = "the trait `ForeignType` is not implemented for `{Self}`",
    note = "Unless you are calling a function that explicitly takes an implementation of `ForeignType`, this diagnostic is likely incorrect",
    note = "It is more likely that the issue lies with not implementing `ValidLayout`, `IntoJulia`, `Typecheck`, `Unbox` or `ConstructType`",
    note = "Custom types that implement the traits mentioned in the previous note should be generated with JlrsCore.reflect",
    note = "Do not implement `ForeignType` or `OpaqueType` unless this type is exported to Julia with `julia_module!`"
)]
pub unsafe trait ForeignType: Sized + Send + Sync + 'static {
    #[doc(hidden)]
    const TYPE_FN: Option<unsafe fn() -> DataType<'static>> = None;

    /// Whether or not this type should be considered to be large.
    ///
    /// If the size of an instance of this type is larger than 2032 bytes this constant must be
    /// set to `true`, otherwise it can be `false`. This is the default. If this constant is
    /// `true` Julia will internally use `malloc` when allocating a value of this type, otherwise
    /// a preallocated pool is used.
    const LARGE: bool = ::std::mem::size_of::<Self>() > 2032;

    /// The super-type of this type, `Core.Any` by default.
    #[inline]
    fn super_type<'target, Tgt>(target: Tgt) -> DataTypeData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        DataType::any_type(&target).root(target)
    }

    /// Insert a write barrier.
    ///
    /// A write barrier must be inserted whenever new managed data is referenced by a foreign
    /// type. Examples include setting a field to `value`, or inserting it into a hash map. It
    /// must only be inserted if this instance is managed by Julia (i.e. it has been allocated
    /// with `(Typed)Value::new`), or if it's referenced by a managed instance (e.g. if it has
    /// been inlined into another foreign type which is managed).
    ///
    /// A write barrier checks if the newly referenced data is young and the parent is old, if
    /// this is the case the parent is guaranteed to be scanned during the next GC cycle.
    /// If this didn't happen, an incremental collection cycle would free the young data if its
    /// old parent is the only live object that references it.
    ///
    /// The parent is `&self` if this instance is managed by Julia, otherwise it's a reference to
    /// the instance of the foreign type which references it.
    ///
    /// Safety:
    ///
    /// The parent must be correct, see the explanation above. `value` must reference existing
    /// data.
    unsafe fn write_barrier<T: Managed<'static, 'static>, P: ForeignType>(
        &self,
        value: Weak<'static, 'static, T>,
        parent: &P,
    ) {
        unsafe {
            jlrs_gc_wb(parent as *const _ as *mut _, value.ptr().as_ptr().cast());
        }
    }

    /// Mark all references to Julia data.
    ///
    /// For each reference to Julia data you must call [`mark_queue_obj`], if `self` contains an
    /// array-like object with references [`mark_queue_objarray`] can be used instead for that
    /// object. This method should return the number of times `mark_queue_obj` returned `true`,
    /// or 0 if only `mark_queue_objarray` is called.
    ///
    /// The parent is `&self` if this instance is managed by Julia, otherwise it's a reference to
    /// the instance of the foreign type which references it.
    ///
    /// Safety:
    ///
    /// This method must only be called by the garbage collector, and must mark all managed data
    /// referenced by the implementor. The parent must be correct.
    ///
    /// [`mark_queue_obj`]: crate::memory::gc::mark_queue_obj
    /// [`mark_queue_objarray`]: crate::memory::gc::mark_queue_objarray
    unsafe fn mark<P: ForeignType>(ptls: PTls, data: &Self, parent: &P) -> usize;
}

unsafe impl<T: ForeignType> OpaqueType for T {
    type Key = Self;

    const IS_FOREIGN: bool = true;
    const TYPE_FN: Option<unsafe fn() -> DataType<'static>> = <T as ForeignType>::TYPE_FN;
    const N_PARAMS: usize = 0;

    #[inline]
    fn super_type<'target, Tgt>(target: Tgt) -> DataTypeData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        <Self as ForeignType>::super_type(target)
    }

    unsafe fn create_type<'target, Tgt>(
        target: Tgt,
        name: Symbol,
        module: Module,
    ) -> DataTypeData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            if let Some(ty) = FOREIGN_TYPE_REGISTRY.find::<Self>() {
                return target.data_from_ptr(ty.unwrap_non_null(Private), Private);
            }

            let large = Self::LARGE as _;
            let has_pointers = true as _;

            unsafe extern "C" fn mark<T: ForeignType>(ptls: PTls, value: *mut jl_value_t) -> usize {
                unsafe {
                    let data = NonNull::new_unchecked(value.cast()).as_ref();
                    T::mark(ptls, data, data)
                }
            }

            unsafe extern "C" fn sweep<T: ForeignType>(value: *mut jl_value_t) {
                unsafe { do_sweep::<T>(value.cast()) }
            }

            target.with_local_scope::<_, 1>(|target, mut frame| {
                let super_type = Self::super_type(&mut frame).unwrap(Private);

                let ty = jl_new_foreign_type(
                    name.unwrap(Private),
                    module.unwrap(Private),
                    super_type,
                    mark::<Self>,
                    sweep::<Self>,
                    has_pointers,
                    large,
                );

                assert!(
                    !ty.is_null(),
                    "Unable to create foreign type {}",
                    type_name::<Self>()
                );
                FOREIGN_TYPE_REGISTRY
                    .insert::<Self>(DataType::wrap_non_null(NonNull::new_unchecked(ty), Private));

                target.data_from_ptr(NonNull::new_unchecked(ty), Private)
            })
        }
    }

    unsafe fn reinit_type(datatype: DataType) -> bool {
        unsafe {
            if let Some(_) = FOREIGN_TYPE_REGISTRY.find::<Self>() {
                return true;
            }

            unsafe extern "C" fn mark<T: ForeignType>(ptls: PTls, value: *mut jl_value_t) -> usize {
                unsafe {
                    let data = NonNull::new_unchecked(value.cast()).as_ref();
                    T::mark(ptls, data, data)
                }
            }

            unsafe extern "C" fn sweep<T: ForeignType>(value: *mut jl_value_t) {
                unsafe { do_sweep::<T>(value.cast()) }
            }

            let ty = datatype.unwrap(Private);
            let ret = jl_reinit_foreign_type(ty, mark::<Self>, sweep::<Self>);
            if ret != 0 {
                FOREIGN_TYPE_REGISTRY
                    .insert::<Self>(DataType::wrap_non_null(NonNull::new_unchecked(ty), Private));

                true
            } else {
                panic!("Unable to reinit type {}", type_name::<Self>())
            }
        }
    }

    fn variant_parameters<'target, Tgt>(target: Tgt) -> SimpleVectorData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        SimpleVector::emptysvec(&target).root(target)
    }

    fn type_parameters<'target, Tgt>(target: Tgt) -> SimpleVectorData<'target, Tgt>
    where
        Tgt: Target<'target>,
    {
        SimpleVector::emptysvec(&target).root(target)
    }
}

unsafe fn create_parametric_opaque_type<'target, T, Tgt>(
    target: Tgt,
    name: Symbol,
    module: Module,
) -> DataTypeData<'target, Tgt>
where
    T: OpaqueType,
    Tgt: Target<'target>,
{
    unsafe {
        if let Some(ty) = FOREIGN_TYPE_REGISTRY.find::<Key<T::Key>>() {
            return target.data_from_ptr(ty.unwrap_non_null(Private), Private);
        }

        target.with_local_scope::<_, 2>(|target, mut frame| {
            let super_type = T::super_type(&mut frame);
            let parameters = T::type_parameters(&mut frame);

            let ty = jl_new_datatype(
                name.unwrap(Private),
                module.unwrap(Private),
                super_type.unwrap(Private),
                parameters.unwrap(Private),
                jl_emptysvec,
                jl_emptysvec,
                jl_emptysvec,
                0,
                1,
                0,
            );

            debug_assert!(!ty.is_null());
            FOREIGN_TYPE_REGISTRY.insert::<Key<T::Key>>(DataType::wrap_non_null(
                NonNull::new_unchecked(ty),
                Private,
            ));

            target.data_from_ptr::<DataType>(NonNull::new_unchecked(ty), Private)
        })
    }
}

unsafe fn create_opaque_type<'target, T, Tgt>(
    target: Tgt,
    name: Symbol,
    module: Module,
) -> DataTypeData<'target, Tgt>
where
    T: OpaqueType,
    Tgt: Target<'target>,
{
    unsafe {
        if let Some(ty) = FOREIGN_TYPE_REGISTRY.find::<T>() {
            return target.data_from_ptr(ty.unwrap_non_null(Private), Private);
        }

        target.with_local_scope::<_, 2>(|target, mut frame| {
            let super_type = T::super_type(&mut frame);
            let parameters = T::type_parameters(&mut frame);

            let ty = jl_new_datatype(
                name.unwrap(Private),
                module.unwrap(Private),
                super_type.unwrap(Private),
                parameters.unwrap(Private),
                jl_emptysvec,
                jl_emptysvec,
                jl_emptysvec,
                0,
                1,
                0,
            );

            debug_assert!(!ty.is_null());
            FOREIGN_TYPE_REGISTRY
                .insert::<T>(DataType::wrap_non_null(NonNull::new_unchecked(ty), Private));

            target.data_from_ptr::<DataType>(NonNull::new_unchecked(ty), Private)
        })
    }
}

#[inline]
unsafe fn do_sweep<T>(data: *mut T)
where
    T: ForeignType,
{
    unsafe {
        std::ptr::drop_in_place(data);
    }
}

unsafe impl<F: OpaqueType> IntoJulia for F {
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        FOREIGN_TYPE_REGISTRY
            .find::<F>()
            .expect("Type has not been initialized")
            .root(target)
    }

    fn into_julia<'scope, Tgt>(self, target: Tgt) -> ValueData<'scope, 'static, Tgt>
    where
        Tgt: Target<'scope>,
    {
        unsafe {
            let ty = if let Some(ty) = FOREIGN_TYPE_REGISTRY.find::<F>() {
                ty
            } else {
                if let Some(func) = Self::TYPE_FN {
                    let ty = func();
                    FOREIGN_TYPE_REGISTRY.insert::<Self>(ty);
                    ty
                } else {
                    panic!("Type {} was not initialized", type_name::<Self>());
                }
            };

            let ptls = get_tls();
            let sz = std::mem::size_of::<Self>();
            let ptr = jl_gc_alloc_typed(ptls, sz, ty.unwrap(Private).cast()).cast::<jl_value_t>();

            assert!(!ptr.is_null(), "allocation failed");
            assert!(
                ptr as usize % std::mem::align_of::<Self>() == 0,
                "allocation for {} has insufficient alignnment",
                type_name::<Self>()
            );

            ptr.cast::<Self>().write(self);
            let res = target.data_from_ptr(NonNull::new_unchecked(ptr), Private);

            if Self::IS_FOREIGN {
                jl_gc_schedule_foreign_sweepfunc(ptls, ptr);
            } else {
                jl_gc_add_ptr_finalizer(ptls, ptr, drop_opaque::<Self> as *mut c_void);
            }

            res
        }
    }
}

unsafe impl<T: OpaqueType> ValidLayout for T {
    fn valid_layout(ty: Value) -> bool {
        if ty.is::<DataType>() {
            unsafe { T::typecheck(ty.cast_unchecked()) }
        } else {
            false
        }
    }

    fn type_object<'target, Tgt: Target<'target>>(_target: &Tgt) -> Value<'target, 'static> {
        FOREIGN_TYPE_REGISTRY.find::<T>().unwrap().as_value()
    }
}

unsafe impl<T: OpaqueType> Typecheck for T {
    fn typecheck(ty: DataType) -> bool {
        if let Some(found_ty) = FOREIGN_TYPE_REGISTRY.find::<T>() {
            ty.unwrap(Private) == found_ty.unwrap(Private)
        } else {
            false
        }
    }
}

unsafe impl<T: OpaqueType + Clone> Unbox for T {
    type Output = T;
}

unsafe extern "C" fn drop_opaque<T: OpaqueType>(data: *mut T) {
    unsafe {
        std::ptr::drop_in_place(data);
    }
}

unsafe impl<T: OpaqueType> ConstructType for T {
    type Static = T;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        FOREIGN_TYPE_REGISTRY
            .find::<T>()
            .unwrap()
            .as_value()
            .root(target)
    }

    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(FOREIGN_TYPE_REGISTRY.find::<T>()?.as_value())
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        _env: &super::construct_type::TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        FOREIGN_TYPE_REGISTRY
            .find::<T>()
            .unwrap()
            .as_value()
            .root(target)
    }
}

struct Key<K>(PhantomData<K>);

struct ForeignTypes {
    data: Cache<FnvHashMap<TypeId, DataType<'static>>>,
}

impl ForeignTypes {
    const fn new() -> Self {
        let hasher = FnvBuildHasher::new();
        let map = HashMap::with_hasher(hasher);
        ForeignTypes {
            data: Cache::new(map),
        }
    }

    fn find<T: 'static>(&self) -> Option<DataType<'_>> {
        let tid = TypeId::of::<T>();
        unsafe { self.data.read(|cache| cache.cache().get(&tid).copied()) }
    }

    // Safety: ty must be the datatype associated with T.
    unsafe fn insert<T: 'static>(&self, ty: DataType<'static>) {
        let tid = TypeId::of::<T>();
        unsafe {
            self.data.write(|cache| cache.cache_mut().insert(tid, ty));
        }
    }
}

unsafe impl Sync for ForeignTypes {}
unsafe impl Send for ForeignTypes {}
