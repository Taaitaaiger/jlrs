/// Use `num::Complex` as bindings for Julia's `Complex` types.
pub use num_complex::Complex;

use crate::{
    convert::{
        ccall_types::{CCallArg, CCallReturn},
        unbox::Unbox,
    },
    data::{
        layout::{
            is_bits::IsBits,
            valid_layout::{ValidField, ValidLayout},
        },
        managed::{datatype::DataType, union_all::UnionAll, value::Value, Managed},
        types::{construct_type::ConstructType, typecheck::Typecheck},
    },
    define_fast_key, define_static_ref,
    memory::target::Target,
    static_ref,
};

define_fast_key!(
    /// A fast type constructor for `Complex<f32>`
    pub ComplexF32,
    Complex<f32>
);
define_fast_key!(
    /// A fast type constructor for `Complex<f64>`
    pub ComplexF64,
    Complex<f64>
);

define_static_ref!(COMPLEX_UNION_ALL, UnionAll, "Base.Complex");

unsafe impl<T: ValidField> ValidLayout for Complex<T> {
    fn valid_layout(ty: Value) -> bool {
        if !ty.is::<DataType>() {
            return false;
        }

        unsafe {
            let ty = ty.cast_unchecked::<DataType>();
            if ty.n_fields() != Some(2) {
                return false;
            }

            let field_tys = ty.field_types();
            let field_tys = field_tys.data();
            let field_tys = field_tys.as_atomic_slice().assume_immutable_non_null();

            if !T::valid_field(field_tys[0]) {
                return false;
            }

            if !T::valid_field(field_tys[1]) {
                return false;
            }
        }

        true
    }

    fn type_object<'target, Tgt: Target<'target>>(target: &Tgt) -> Value<'target, 'static> {
        static_ref!(COMPLEX_UNION_ALL, target).as_value()
    }
}

unsafe impl<T: ValidField> Typecheck for Complex<T> {
    fn typecheck(t: DataType) -> bool {
        Self::valid_layout(t.as_value())
    }
}

unsafe impl<T: Clone> Unbox for Complex<T> {
    type Output = Self;
}

unsafe impl<T: ValidField> ValidField for Complex<T> {
    fn valid_field(ty: Value) -> bool {
        Self::valid_layout(ty)
    }
}

unsafe impl<T: IsBits + ValidField> IsBits for Complex<T> {}

unsafe impl<T: ConstructType> ConstructType for Complex<T> {
    type Static = Complex<T::Static>;

    fn construct_type_uncached<'target, Tgt>(
        target: Tgt,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let t = T::construct_type(&mut frame);
            let complex_ua = static_ref!(COMPLEX_UNION_ALL, &frame);
            let complex_t = unsafe { complex_ua.apply_types_unchecked(target, [t]) };

            complex_t
        })
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &crate::data::types::construct_type::TypeVarEnv,
    ) -> crate::prelude::ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        target.with_local_scope::<_, _, 1>(|target, mut frame| {
            let t = T::construct_type_with_env(&mut frame, env);
            let complex_ua = static_ref!(COMPLEX_UNION_ALL, &frame);
            let complex_t = unsafe { complex_ua.apply_types_unchecked(target, [t, t]) };

            complex_t
        })
    }

    fn base_type<'target, Tgt>(target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        Some(static_ref!(COMPLEX_UNION_ALL, target).as_value())
    }
}

unsafe impl<T: IsBits + ConstructType> CCallArg for Complex<T> {
    type CCallArgType = Self;
    type FunctionArgType = Self;
}

unsafe impl<T: IsBits + ConstructType> CCallReturn for Complex<T> {
    type FunctionReturnType = Self;
    type CCallReturnType = Self;
    type ReturnAs = Self;

    unsafe fn return_or_throw(self) -> Self::ReturnAs {
        self
    }
}
