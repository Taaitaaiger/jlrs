/// Foreign types wrap Rust data and are opaque to Julia.
///
/// All data in Julia is an instance of some [`DataType`], and `DataType`s typically have layout
/// requirements that are not compatible with arbitrary Rust data. However, it is possible to
/// create new foreign types. Julia makes no assumptions about the layout of foreign types, which
/// means it's possible to move arbitrary data from Rust to Julia.
///
/// In order to create a new foreign type, you must implement the [`ForeignType`] trait. Before
/// this type can be used, [`create_foreign_type`] must be called.
///
/// While foreign types can contain Julia data, the [`ForeignType`] trait requires that
/// `Self: 'static`. This means you'll need to erase the lifetimes of this data, or store them as
/// raw pointers. If a foreign type contains Julia data, [`ForeignType::mark`] must be
/// implemented. Whenever Julia data in an instance of a foreign type is mutated,
/// [`write_barrier`] must be called if the foreign data is owned by Julia.
///
/// It's recommended that `ForeignType` is only implemented for types that are thread-safe, and
/// if a foreign type needs to be mutable this should be achieved through interior mutability.
///
/// [`write_barrier`]: crate::memory::gc::write_barrier
use std::{
    any::TypeId,
    ffi::c_void,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::{Arc, RwLock},
};

#[julia_version(since = "1.9")]
use jl_sys::jl_reinit_foreign_type;
use jl_sys::{
    jl_gc_alloc_typed, jl_gc_schedule_foreign_sweepfunc, jl_new_foreign_type, jl_value_t,
};
use jlrs_macros::julia_version;
use once_cell::sync::OnceCell;

use crate::{
    convert::{construct_type::ConstructType, into_julia::IntoJulia, unbox::Unbox},
    data::{
        layout::valid_layout::ValidLayout,
        managed::{
            datatype::{DataType, DataTypeData},
            module::Module,
            private::ManagedPriv,
            symbol::Symbol,
            value::{Value, ValueData},
        },
    },
    memory::{
        get_tls,
        target::{ExtendedTarget, Target},
        PTls,
    },
    prelude::Managed,
    private::Private,
};

static FOREIGN_TYPE_REGISTRY: OnceCell<Arc<ForeignTypes>> = OnceCell::new();

pub(crate) unsafe extern "C" fn init_foreign_type_registry(registry_ref: &mut *mut c_void) {
    if registry_ref.is_null() {
        FOREIGN_TYPE_REGISTRY.get_or_init(|| {
            let registry = Arc::new(ForeignTypes {
                data: RwLock::new(Vec::new()),
            });
            let cloned = registry.clone();
            *registry_ref = Arc::into_raw(registry) as *mut c_void;
            cloned
        });
    } else {
        FOREIGN_TYPE_REGISTRY.get_or_init(|| {
            std::mem::transmute::<&mut *mut c_void, &Arc<ForeignTypes>>(registry_ref).clone()
        });
    }
}

struct ForeignTypes {
    data: RwLock<Vec<(TypeId, DataType<'static>)>>,
}

impl ForeignTypes {
    fn find<T: 'static>(&self) -> Option<DataType> {
        let tid = TypeId::of::<T>();
        self.data
            .read()
            .expect("Lock poisoned")
            .iter()
            .find_map(|s| match s {
                &(type_id, ty) if type_id == tid => Some(ty),
                _ => None,
            })
    }
}

unsafe impl Sync for ForeignTypes {}
unsafe impl Send for ForeignTypes {}

/// A trait that allows arbitrary Rust data to be converted to Julia.
pub unsafe trait ForeignType: Sized + Send + Sync + 'static {
    #[doc(hidden)]
    const TYPE_FN: Option<unsafe fn() -> DataType<'static>> = None;

    const LARGE: bool = false;
    const HAS_POINTERS: bool = false;

    fn super_type<'target, T>(target: T) -> DataTypeData<'target, T>
    where
        T: Target<'target>,
    {
        DataType::any_type(&target).root(target)
    }

    /// Mark all references to Julia data.
    ///
    /// If a foreign type contains references to Julia data, this method must be overridden.
    /// For each reference to Julia data, you must call [`mark_queue_obj`], if `self` constains a
    /// slice of references, [`mark_queue_objarray`] can be used instead. This method should
    /// return the number of times `mark_queue_obj` returned `true`.
    ///
    /// [`mark_queue_obj`]: crate::memory::gc::mark_queue_obj
    /// [`mark_queue_objarray`]: crate::memory::gc::mark_queue_objarray
    fn mark(_ptls: PTls, _data: &Self) -> usize {
        0
    }
}

/// Create a new foreign type `U`. This method must be called before `U` can be used as a foreign
/// type.
///
/// Safety:
///
/// The new type is not set as a constant in `module`, you must do this manually after calling
/// this function. `large` must be `true` if the size of `U` is larger than 2032 bytes.
/// `has_pointers` must be only be `true` if `U` contains references to Julia data.
pub unsafe fn create_foreign_type<'target, U, T>(
    target: T,
    name: Symbol,
    module: Module,
) -> DataTypeData<'target, T>
where
    U: ForeignType,
    T: Target<'target>,
{
    if let Some(ty) = FOREIGN_TYPE_REGISTRY.get().unwrap().find::<U>() {
        return target.data_from_ptr(ty.unwrap_non_null(Private), Private);
    }

    let large = U::LARGE as _;
    let has_pointers = U::HAS_POINTERS as _;

    unsafe extern "C" fn mark<T: ForeignType>(ptls: PTls, value: *mut jl_value_t) -> usize {
        T::mark(ptls, NonNull::new_unchecked(value.cast()).as_ref())
    }

    unsafe extern "C" fn sweep<T: ForeignType>(value: *mut jl_value_t) {
        do_sweep::<T>(&mut *value.cast())
    }

    let super_type = U::super_type(&target).ptr().as_ptr();

    let ty = jl_new_foreign_type(
        name.unwrap(Private),
        module.unwrap(Private),
        super_type,
        Some(mark::<U>),
        Some(sweep::<U>),
        has_pointers,
        large,
    );

    debug_assert!(!ty.is_null());
    FOREIGN_TYPE_REGISTRY
        .get()
        .unwrap()
        .data
        .write()
        .expect("Foreign type lock was poisoned")
        .push((
            TypeId::of::<U>(),
            DataType::wrap_non_null(NonNull::new_unchecked(ty), Private),
        ));

    target.data_from_ptr(NonNull::new_unchecked(ty), Private)
}

pub(crate) unsafe fn create_foreign_type_internal<'target, U, T>(
    target: T,
    name: Symbol,
    module: Module,
) -> DataTypeData<'target, T>
where
    U: ForeignType,
    T: Target<'target>,
{
    let large = U::LARGE as _;
    let has_pointers = U::HAS_POINTERS as _;

    unsafe extern "C" fn mark<T: ForeignType>(ptls: PTls, value: *mut jl_value_t) -> usize {
        T::mark(ptls, NonNull::new_unchecked(value.cast()).as_ref())
    }

    unsafe extern "C" fn sweep<T: ForeignType>(value: *mut jl_value_t) {
        do_sweep::<T>(NonNull::new_unchecked(value.cast()).as_mut())
    }

    let super_type = U::super_type(&target).ptr().as_ptr();

    let ty = jl_new_foreign_type(
        name.unwrap(Private),
        module.unwrap(Private),
        super_type,
        Some(mark::<U>),
        Some(sweep::<U>),
        has_pointers,
        large,
    );

    target.data_from_ptr(NonNull::new_unchecked(ty), Private)
}

// TODO: docs
#[julia_version(since = "1.9")]
pub unsafe fn reinit_foreign_type<U>(datatype: DataType) -> bool
where
    U: ForeignType,
{
    if let Some(_) = FOREIGN_TYPE_REGISTRY.get().unwrap().find::<U>() {
        return true;
    }

    unsafe extern "C" fn mark<T: ForeignType>(ptls: PTls, value: *mut jl_value_t) -> usize {
        T::mark(ptls, NonNull::new_unchecked(value.cast()).as_ref())
    }

    unsafe extern "C" fn sweep<T: ForeignType>(value: *mut jl_value_t) {
        do_sweep::<T>(NonNull::new_unchecked(value.cast()).as_mut())
    }

    let ty = datatype.unwrap(Private);
    let ret = jl_reinit_foreign_type(ty, Some(mark::<U>), Some(sweep::<U>));
    if ret != 0 {
        FOREIGN_TYPE_REGISTRY
            .get()
            .unwrap()
            .data
            .write()
            .expect("Foreign type lock was poisoned")
            .push((
                TypeId::of::<U>(),
                DataType::wrap_non_null(NonNull::new_unchecked(ty), Private),
            ));
        true
    } else {
        false
    }
}

#[inline(always)]
unsafe fn do_sweep<T>(data: &mut ForeignValue<T>)
where
    T: ForeignType,
{
    data.data.assume_init_drop();
}

unsafe impl<F: ForeignType> IntoJulia for F {
    fn julia_type<'scope, T>(target: T) -> DataTypeData<'scope, T>
    where
        T: Target<'scope>,
    {
        let ty = FOREIGN_TYPE_REGISTRY
            .get()
            .unwrap()
            .find::<F>()
            .expect("Doesn't exist");
        unsafe { target.data_from_ptr(ty.unwrap_non_null(Private), Private) }
    }

    fn into_julia<'scope, T>(self, target: T) -> ValueData<'scope, 'static, T>
    where
        T: Target<'scope>,
    {
        unsafe {
            let ptls = get_tls();
            let sz = std::mem::size_of::<Self>();
            let maybe_ty = FOREIGN_TYPE_REGISTRY.get().unwrap().find::<F>();

            let ty = match maybe_ty {
                None => {
                    if let Some(func) = Self::TYPE_FN {
                        let mut guard = FOREIGN_TYPE_REGISTRY
                            .get()
                            .unwrap()
                            .data
                            .write()
                            .expect("Foreign type lock was poisoned");

                        // Check again
                        let tid = TypeId::of::<Self>();
                        if let Some(ty) = guard.iter().find_map(|s| match s {
                            &(type_id, ty) if type_id == tid => Some(ty),
                            _ => None,
                        }) {
                            ty
                        } else {
                            let ty = func();
                            guard.push((TypeId::of::<Self>(), ty));
                            ty
                        }
                    } else {
                        maybe_ty.expect("Doesn't exist")
                    }
                }
                Some(t) => t,
            };

            let ptr: *mut Self = jl_gc_alloc_typed(ptls, sz, ty.unwrap(Private).cast()).cast();
            ptr.write(self);
            let res = target.data_from_ptr(NonNull::new_unchecked(ptr.cast()), Private);
            jl_gc_schedule_foreign_sweepfunc(ptls, ptr.cast());

            res
        }
    }
}

unsafe impl<T: ForeignType> ValidLayout for T {
    fn valid_layout(ty: Value) -> bool {
        if let Ok(dt) = ty.cast::<DataType>() {
            if let Some(ty) = FOREIGN_TYPE_REGISTRY.get().unwrap().find::<T>() {
                dt.unwrap(Private) == ty.unwrap(Private)
            } else {
                false
            }
        } else {
            false
        }
    }
}

unsafe impl<T: ForeignType + Clone> Unbox for T {
    type Output = T;
}

#[repr(transparent)]
struct ForeignValue<T: ForeignType> {
    pub data: MaybeUninit<T>,
}

unsafe impl<U: ForeignType> ConstructType for U {
    fn base_type<'target, T>(target: &T) -> crate::data::managed::value::Value<'target, 'static>
    where
        T: Target<'target>,
    {
        unsafe { <U as crate::convert::into_julia::IntoJulia>::julia_type(target).as_value() }
    }

    fn construct_type<'target, 'current, 'borrow, T>(
        target: ExtendedTarget<'target, 'current, 'borrow, T>,
    ) -> DataTypeData<'target, T>
    where
        T: Target<'target>,
    {
        let (target, _) = target.split();
        <U as crate::convert::into_julia::IntoJulia>::julia_type(target)
    }
}
