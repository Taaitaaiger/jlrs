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
/// [`InlineLayout::write_barrier`] must be called if the foreign data is owned by Julia.
///
/// It's recommended that `ForeignType` is only implemented for types that are thread-safe, and
/// if a foreign type needs to be mutable this should be achieved through interior mutability.
use std::{
    any::TypeId,
    mem::MaybeUninit,
    ptr::{null_mut, NonNull},
    sync::RwLock,
};

use jl_sys::{
    jl_gc_alloc_typed, jl_gc_schedule_foreign_sweepfunc, jl_new_foreign_type, jl_value_t,
};

use crate::{
    convert::into_julia::IntoJulia,
    memory::{get_tls, PTls},
    prelude::{DataType, Module, Symbol, Target},
    private::Private,
};

use super::ptr::private::WrapperPriv;

static FOREIGN_TYPES: ForeignTypes = ForeignTypes {
    data: RwLock::new(Vec::new()),
};

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

/// A trait that allows arbitrary Rust data to be converted to Julia.
pub unsafe trait ForeignType: Sized + 'static {
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

pub unsafe fn create_foreign_type<'target, U, T>(
    target: T,
    name: Symbol,
    module: Module,
    super_type: Option<DataType>,
    has_pointers: bool,
    large: bool,
) -> T::Data
where
    U: ForeignType,
    T: Target<'target, 'static, DataType<'target>>,
{
    if let Some(ty) = FOREIGN_TYPES.find::<U>() {
        return target.data_from_ptr(ty.unwrap_non_null(Private), Private);
    }

    let large = large as _;
    let has_pointers = has_pointers as _;

    unsafe extern "C" fn mark<T: ForeignType>(ptls: PTls, value: *mut jl_value_t) -> usize {
        T::mark(ptls, NonNull::new_unchecked(value.cast()).as_ref())
    }

    unsafe extern "C" fn sweep<T: ForeignType>(value: *mut jl_value_t) {
        do_sweep::<T>(NonNull::new_unchecked(value.cast()).as_mut())
    }

    let ty = jl_new_foreign_type(
        name.unwrap(Private),
        module.unwrap(Private),
        super_type.map_or(null_mut(), |s| s.unwrap(Private)),
        Some(mark::<U>),
        Some(sweep::<U>),
        has_pointers,
        large,
    );

    debug_assert!(!ty.is_null());
    FOREIGN_TYPES
        .data
        .write()
        .expect("Foreign type lock was poisoned")
        .push((
            TypeId::of::<U>(),
            DataType::wrap_non_null(NonNull::new_unchecked(ty), Private),
        ));

    target.data_from_ptr(NonNull::new_unchecked(ty), Private)
}

#[inline(always)]
unsafe fn do_sweep<T>(data: &mut ForeignValue<T>)
where
    T: ForeignType,
{
    data.data.assume_init_drop();
}

unsafe impl<F: ForeignType> IntoJulia for F {
    fn julia_type<'scope, T>(target: T) -> T::Data
    where
        T: Target<'scope, 'static, DataType<'scope>>,
    {
        let ty = FOREIGN_TYPES.find::<F>().expect("Doesn't exist");
        unsafe { target.data_from_ptr(ty.unwrap_non_null(Private), Private) }
    }

    fn into_julia<'scope, T>(self, target: T) -> T::Data
    where
        T: Target<'scope, 'static>,
    {
        unsafe {
            let ptls = get_tls();
            let sz = std::mem::size_of::<Self>();
            let ty = FOREIGN_TYPES.find::<F>().expect("Doesn't exist");

            let ptr: *mut Self = jl_gc_alloc_typed(ptls, sz, ty.unwrap(Private).cast()).cast();
            ptr.write(self);
            let res = target.data_from_ptr(NonNull::new_unchecked(ptr.cast()), Private);
            jl_gc_schedule_foreign_sweepfunc(ptls, ptr.cast());

            res
        }
    }
}

#[repr(transparent)]
pub struct ForeignValue<T: ForeignType> {
    pub data: MaybeUninit<T>,
}
