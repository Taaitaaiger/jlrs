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
            static_arrays::dims::Dims,
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
pub struct MArray<T, D: Dims<R>, const N: usize, const R: usize> {
    data: [T; N],
    _d: PhantomData<D>,
}

impl<T, D: Dims<R>, const N: usize, const R: usize> MArray<T, D, N, R> {
    pub const fn new(data: [T; N]) -> Self {
        MArray {
            data,
            _d: PhantomData,
        }
    }
    pub const fn data(&self) -> &[T; N] {
        &self.data
    }
}

unsafe impl<T: ConstructType, D: Dims<R>, const N: usize, const R: usize> Typecheck
    for MArray<T, D, N, R>
{
    fn typecheck(t: DataType) -> bool {
        Self::valid_layout(t.as_value())
    }
}

unsafe impl<T: ConstructType, D: Dims<R>, const N: usize, const R: usize> ValidLayout
    for MArray<T, D, N, R>
{
    fn valid_layout(ty: Value) -> bool {
        Self::valid_field(ty)
    }

    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        unsafe { Self::construct_type(target).as_value() }
    }
}

unsafe impl<T: ConstructType, D: Dims<R>, const N: usize, const R: usize> ValidField
    for MArray<T, D, N, R>
{
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

unsafe impl<T: ConstructType + Clone, D: Dims<R>, const N: usize, const R: usize> Unbox
    for MArray<T, D, N, R>
{
    type Output = Self;
}

unsafe impl<T: 'static + ConstructType + Clone, D: Dims<R>, const N: usize, const R: usize>
    IntoJulia for MArray<T, D, N, R>
{
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

unsafe impl<'a, T: 'static + ConstructType + Clone, D: Dims<R>, const N: usize, const R: usize>
    IntoJulia for &MArray<T, D, N, R>
{
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        unsafe {
            MArray::<T, D, N, R>::construct_type(&target)
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
                .cast::<MaybeUninit<MArray<T, D, N, R>>>()
                .copy_from(std::mem::transmute(self), 1);
            target.data_from_ptr(container, Private)
        }
    }
}

unsafe impl<T: ConstructType, D: Dims<R>, const N: usize, const R: usize> ConstructType
    for MArray<T, D, N, R>
{
    type Static = MArray<T::Static, D, N, R>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 6>(|target, mut frame| unsafe {
            let t = T::construct_type(&mut frame);
            let rank = (R as isize).into_julia(&mut frame);
            let n = (N as isize).into_julia(&mut frame);

            let s = (&mut frame).with_local_scope::<_, R>(|target, mut frame| {
                let params = D::PARAMS.map(|p| Value::new(&mut frame, p as isize));
                DataType::anytuple_type(&frame)
                    .as_value()
                    .apply_type(target, params)
                    .unwrap()
            });

            let m_array = Self::base_type(&mut frame).unwrap();
            let m_array_ua = m_array.cast::<UnionAll>().unwrap();

            m_array_ua
                .apply_types_unchecked(&mut frame, [s, t, n, rank])
                .cast::<DataType>()
                .unwrap()
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
        target.with_local_scope::<_, 6>(|target, mut frame| unsafe {
            let t = T::construct_type_with_env(&mut frame, env);
            let rank = (R as isize).into_julia(&mut frame);
            let n = (N as isize).into_julia(&mut frame);

            let s = (&mut frame).with_unsized_local_scope(R, |target, mut frame| {
                let params = D::PARAMS.map(|p| Value::new(&mut frame, p as isize));
                DataType::anytuple_type(&frame)
                    .as_value()
                    .apply_type_unchecked(target, params)
            });

            let m_array = Self::base_type(&mut frame).unwrap();
            assert!(m_array.is::<UnionAll>());
            let m_array_ua = m_array.cast_unchecked::<UnionAll>();

            m_array_ua
                .apply_types_unchecked(&mut frame, [s, t, n, rank])
                .cast_unchecked::<DataType>()
                .wrap_with_env(target, env)
        })
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let var = inline_static_ref!(STATIC, Value, "StaticArrays.MArray", target);
        Some(var)
    }
}

unsafe impl<T: IsBits, D: Dims<R>, const N: usize, const R: usize> IsBits for MArray<T, D, N, R> {}

unsafe impl<T: ConstructType, D: Dims<R>, const N: usize, const R: usize> CCallArg
    for &MArray<T, D, N, R>
{
    type CCallArgType = RefTypeConstructor<MArray<T, D, N, R>>;
    type FunctionArgType = MArray<T, D, N, R>;
}

unsafe impl<T: ConstructType, D: Dims<R>, const N: usize, const R: usize> CCallReturn
    for MArray<T, D, N, R>
{
    type FunctionReturnType = Self;
    type CCallReturnType = Self;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}
