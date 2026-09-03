use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use jl_sys::jl_new_struct_uninit;

use crate::{
    convert::{
        ccall_types::{CCallArg, CCallReturn},
        into_julia::IntoJulia,
        unbox::Unbox,
    },
    data::{
        layout::{
            is_bits::IsBits,
            static_arrays::dims::Dims1D,
            valid_layout::{ValidField, ValidLayout},
        },
        managed::{
            Managed,
            datatype::{DataType, DataTypeData},
            private::ManagedPriv,
            union_all::UnionAll,
            value::{Value, ValueData},
        },
        types::{
            abstract_type::RefTypeConstructor,
            construct_type::{ConstructType, TypeVarEnv},
            typecheck::Typecheck,
        },
    },
    inline_static_ref,
    memory::{
        scope::{LocalScope, LocalScopeExt},
        target::Target,
    },
    private::Private,
    weak_handle_unchecked,
};

#[derive(Clone)]
#[repr(C)]
pub struct SVector<T, const N: usize> {
    data: [T; N],
    _d: PhantomData<Dims1D<N>>,
}

impl<T, const N: usize> SVector<T, N> {
    pub const fn new(data: [T; N]) -> Self {
        SVector {
            data,
            _d: PhantomData,
        }
    }
    pub const fn data(&self) -> &[T; N] {
        &self.data
    }
}

unsafe impl<T: ConstructType, const N: usize> Typecheck for SVector<T, N> {
    fn typecheck(t: DataType) -> bool {
        Self::valid_layout(t.as_value())
    }
}

unsafe impl<T: ConstructType, const N: usize> ValidLayout for SVector<T, N> {
    fn valid_layout(ty: Value) -> bool {
        Self::valid_field(ty)
    }

    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        unsafe { Self::construct_type(target).as_value() }
    }
}

unsafe impl<T: ConstructType, const N: usize> ValidField for SVector<T, N> {
    fn valid_field(ty: Value) -> bool {
        unsafe {
            let handle = weak_handle_unchecked!();
            handle.local_scope::<_, 1>(|mut frame| {
                let t = Self::construct_type(&mut frame);
                t == ty
            })
        }
    }
}

unsafe impl<T: ConstructType + Clone, const N: usize> Unbox for SVector<T, N> {
    type Output = Self;
}

unsafe impl<T: 'static + ConstructType + Clone, const N: usize> IntoJulia for SVector<T, N> {
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        unsafe {
            Self::construct_type(&target)
                .as_managed()
                .cast::<DataType>()
                .unwrap()
                .root(target)
        }
    }
}

unsafe impl<'a, T: 'static + ConstructType + Clone, const N: usize> IntoJulia for &SVector<T, N> {
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        unsafe {
            SVector::<T, N>::construct_type(&target)
                .as_managed()
                .cast::<DataType>()
                .unwrap()
                .root(target)
        }
    }

    fn into_julia<'scope, Tgt>(self, target: Tgt) -> ValueData<'scope, 'static, Tgt>
    where
        Tgt: Target<'scope>,
    {
        unsafe {
            let ty = Self::julia_type(&target).as_managed();
            debug_assert!(ty.is_bits());

            let container = jl_new_struct_uninit(ty.unwrap(Private));
            debug_assert!(!container.is_null());
            let container = NonNull::new_unchecked(container);
            container
                .cast::<MaybeUninit<SVector<T, N>>>()
                .copy_from(std::mem::transmute(self), 1);
            target.data_from_ptr(container, Private)
        }
    }
}

unsafe impl<T: ConstructType, const N: usize> ConstructType for SVector<T, N> {
    type Static = SVector<T::Static, N>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 4>(|target, mut frame| unsafe {
            let t = T::construct_type(&mut frame);
            let n = (N as isize).into_julia(&mut frame);

            let svector = Self::base_type(&mut frame).unwrap();
            assert!(svector.is::<UnionAll>());
            let svector_ua = svector.cast_unchecked::<UnionAll>();

            svector_ua
                .apply_types_unchecked(&mut frame, [n, t])
                .cast_unchecked::<DataType>()
                .rewrap(target)
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 4>(|target, mut frame| unsafe {
            let t = T::construct_type_with_env(&mut frame, env);
            let n = (N as isize).into_julia(&mut frame);

            let svector = Self::base_type(&mut frame).unwrap();
            assert!(svector.is::<UnionAll>());
            let svector_ua = svector.cast_unchecked::<UnionAll>();

            svector_ua
                .apply_types_unchecked(&mut frame, [n, t])
                .cast_unchecked::<DataType>()
                .wrap_with_env(target, env)
        })
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let var = inline_static_ref!(STATIC, Value, "StaticArrays.SVector", target);
        Some(var)
    }
}

unsafe impl<T: IsBits, const N: usize> IsBits for SVector<T, N> {}

unsafe impl<T: ConstructType, const N: usize> CCallArg for SVector<T, N> {
    type CCallArgType = Self;
    type FunctionArgType = Self;
}

unsafe impl<T: ConstructType, const N: usize> CCallArg for &SVector<T, N> {
    type CCallArgType = RefTypeConstructor<SVector<T, N>>;
    type FunctionArgType = SVector<T, N>;
}

unsafe impl<T: ConstructType, const N: usize> CCallReturn for SVector<T, N> {
    type FunctionReturnType = Self;
    type CCallReturnType = Self;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}
