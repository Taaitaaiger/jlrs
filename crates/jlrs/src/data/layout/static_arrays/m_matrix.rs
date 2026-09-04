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
            static_arrays::dims::Dims2D,
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
pub struct MMatrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; ROWS]; COLS],
    _d: PhantomData<Dims2D<ROWS, COLS>>,
}

impl<T, const ROWS: usize, const COLS: usize> MMatrix<T, ROWS, COLS> {
    pub const fn new(data: [[T; ROWS]; COLS]) -> Self {
        MMatrix {
            data,
            _d: PhantomData,
        }
    }
    pub const fn data(&self) -> &[[T; ROWS]; COLS] {
        &self.data
    }
}

unsafe impl<T: ConstructType, const ROWS: usize, const COLS: usize> Typecheck
    for MMatrix<T, ROWS, COLS>
{
    fn typecheck(t: DataType) -> bool {
        Self::valid_layout(t.as_value())
    }
}

unsafe impl<T: ConstructType, const ROWS: usize, const COLS: usize> ValidLayout
    for MMatrix<T, ROWS, COLS>
{
    fn valid_layout(ty: Value) -> bool {
        Self::valid_field(ty)
    }

    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        unsafe { Self::construct_type(target).as_value() }
    }
}

unsafe impl<T: ConstructType, const ROWS: usize, const COLS: usize> ValidField
    for MMatrix<T, ROWS, COLS>
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

unsafe impl<T: ConstructType + Clone, const ROWS: usize, const COLS: usize> Unbox
    for MMatrix<T, ROWS, COLS>
{
    type Output = Self;
}

unsafe impl<T: 'static + ConstructType + Clone, const ROWS: usize, const COLS: usize> IntoJulia
    for MMatrix<T, ROWS, COLS>
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

unsafe impl<'a, T: 'static + ConstructType + Clone, const ROWS: usize, const COLS: usize> IntoJulia
    for &MMatrix<T, ROWS, COLS>
{
    fn julia_type<'scope, Tgt>(target: Tgt) -> DataTypeData<'scope, Tgt>
    where
        Tgt: Target<'scope>,
    {
        unsafe {
            MMatrix::<T, ROWS, COLS>::construct_type(&target)
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
                .cast::<MaybeUninit<MMatrix<T, ROWS, COLS>>>()
                .copy_from(std::mem::transmute(self), 1);
            target.data_from_ptr(container, Private)
        }
    }
}

unsafe impl<T: ConstructType, const ROWS: usize, const COLS: usize> ConstructType
    for MMatrix<T, ROWS, COLS>
{
    type Static = MMatrix<T::Static, ROWS, COLS>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, 6>(|target, mut frame| unsafe {
            let t = T::construct_type(&mut frame);
            let rows = (ROWS as isize).into_julia(&mut frame);
            let cols = (COLS as isize).into_julia(&mut frame);
            let n = ((ROWS * COLS) as isize).into_julia(&mut frame);

            let m_matrix = Self::base_type(&mut frame).unwrap();
            assert!(m_matrix.is::<UnionAll>());
            let m_matrix_ua = m_matrix.cast_unchecked::<UnionAll>();

            m_matrix_ua
                .apply_types_unchecked(&mut frame, [rows, cols, t, n])
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
        target.with_local_scope::<_, 6>(|target, mut frame| unsafe {
            let t = T::construct_type_with_env(&mut frame, env);
            let rows = (ROWS as isize).into_julia(&mut frame);
            let cols = (COLS as isize).into_julia(&mut frame);
            let n = ((ROWS * COLS) as isize).into_julia(&mut frame);

            let m_matrix = Self::base_type(&mut frame).unwrap();
            assert!(m_matrix.is::<UnionAll>());
            let m_matrix_ua = m_matrix.cast_unchecked::<UnionAll>();

            m_matrix_ua
                .apply_types_unchecked(&mut frame, [rows, cols, t, n])
                .cast_unchecked::<DataType>()
                .wrap_with_env(target, env)
        })
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        let var = inline_static_ref!(STATIC, Value, "StaticArrays.MMatrix", target);
        Some(var)
    }
}

unsafe impl<T: IsBits, const ROWS: usize, const COLS: usize> IsBits for MMatrix<T, ROWS, COLS> {}

unsafe impl<T: ConstructType, const ROWS: usize, const COLS: usize> CCallArg
    for &MMatrix<T, ROWS, COLS>
{
    type CCallArgType = RefTypeConstructor<MMatrix<T, ROWS, COLS>>;
    type FunctionArgType = MMatrix<T, ROWS, COLS>;
}

unsafe impl<T: ConstructType, const ROWS: usize, const COLS: usize> CCallArg
    for &mut MMatrix<T, ROWS, COLS>
{
    type CCallArgType = RefTypeConstructor<MMatrix<T, ROWS, COLS>>;
    type FunctionArgType = MMatrix<T, ROWS, COLS>;
}

unsafe impl<T: ConstructType, const ROWS: usize, const COLS: usize> CCallReturn
    for MMatrix<T, ROWS, COLS>
{
    type FunctionReturnType = Self;
    type CCallReturnType = Self;
    type ReturnAs = Self;

    #[inline]
    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}
