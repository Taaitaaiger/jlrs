use crate::layout::valid_layout::ValidLayout;

/// A marker trait implemented by all wrapper types.
pub trait Wrapper<'scope, 'data>: private::Wrapper<'scope, 'data> + Copy + ValidLayout {}
impl<'scope, 'data, W: private::Wrapper<'scope, 'data> + Copy + ValidLayout> Wrapper<'scope, 'data>
    for W
{
}

pub(crate) mod private {
    use jl_sys::{
        jl_array_t, jl_code_instance_t, jl_datatype_t, jl_expr_t, jl_method_instance_t,
        jl_method_match_t, jl_method_t, jl_methtable_t, jl_module_t, jl_svec_t, jl_sym_t,
        jl_task_t, jl_tvar_t, jl_typemap_entry_t, jl_typemap_level_t, jl_typename_t, jl_unionall_t,
        jl_uniontype_t, jl_value_t, jl_weakref_t,
    };

    use crate::{
        private::Private,
        value::{
            array::Array, code_instance::CodeInstance, datatype::DataType, expr::Expr,
            method::Method, method_instance::MethodInstance, method_match::MethodMatch,
            method_table::MethodTable, module::Module, simple_vector::SimpleVector,
            string::JuliaString, symbol::Symbol, task::Task, type_name::TypeName,
            type_var::TypeVar, typemap_entry::TypeMapEntry, typemap_level::TypeMapLevel,
            union::Union, union_all::UnionAll, weak_ref::WeakRef, wrapper_ref::WrapperRef, Value,
        },
    };

    pub trait Wrapper<'scope, 'data> {
        type Internal: Copy;
        unsafe fn wrap(ptr: *mut Self::Internal, _: Private) -> Self;

        unsafe fn assume_valid_unchecked(
            value_ref: WrapperRef<'scope, 'data, Self>,
            _: Private,
        ) -> Self
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Self::wrap(value_ref.ptr(), Private)
        }

        unsafe fn assume_valid(
            value_ref: WrapperRef<'scope, 'data, Self>,
            _: Private,
        ) -> Option<Self>
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            let ptr = value_ref.ptr();
            if ptr.is_null() {
                return None;
            }

            Some(Self::wrap(ptr, Private))
        }

        unsafe fn assume_valid_value_unchecked(
            value_ref: WrapperRef<'scope, 'data, Self>,
            _: Private,
        ) -> Value<'scope, 'data>
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            Value::wrap(value_ref.ptr().cast())
        }

        unsafe fn assume_valid_value(
            value_ref: WrapperRef<'scope, 'data, Self>,
            _: Private,
        ) -> Option<Value<'scope, 'data>>
        where
            Self: Sized + super::Wrapper<'scope, 'data>,
        {
            let ptr = value_ref.ptr();
            if ptr.is_null() {
                return None;
            }

            Some(Value::wrap(ptr.cast()))
        }
    }

    macro_rules! impl_wrap {
        ($scope:lifetime, $data:lifetime, $tp:tt, $internal_tp:ty) => {
            impl<$scope, $data> Wrapper<$scope, $data> for $tp<$scope, $data> {
                type Internal = $internal_tp;
                unsafe fn wrap(ptr: *mut Self::Internal, _: Private) -> Self {
                    $tp::wrap(ptr)
                }
            }
        };
        ($scope:lifetime, $tp:tt, $internal_tp:ty) => {
            impl<$scope> Wrapper<$scope, '_> for $tp<$scope> {
                type Internal = $internal_tp;
                unsafe fn wrap(ptr: *mut Self::Internal, _: Private) -> Self {
                    $tp::wrap(ptr)
                }
            }
        };
    }

    impl_wrap!('scope, 'data, Value, jl_value_t);
    impl_wrap!('scope, 'data, Array, jl_array_t);
    impl_wrap!('scope, CodeInstance, jl_code_instance_t);
    impl_wrap!('scope, DataType, jl_datatype_t);
    impl_wrap!('scope, Expr, jl_expr_t);
    impl_wrap!('scope, JuliaString, u8);
    impl_wrap!('scope, Method, jl_method_t);
    impl_wrap!('scope, MethodInstance, jl_method_instance_t);
    impl_wrap!('scope, MethodMatch, jl_method_match_t);
    impl_wrap!('scope, MethodTable, jl_methtable_t);
    impl_wrap!('scope, Module, jl_module_t);
    impl_wrap!('scope, SimpleVector, jl_svec_t);
    impl_wrap!('scope, Symbol, jl_sym_t);
    impl_wrap!('scope, Task, jl_task_t);
    impl_wrap!('scope, TypeName, jl_typename_t);
    impl_wrap!('scope, TypeVar, jl_tvar_t);
    impl_wrap!('scope, TypeMapEntry, jl_typemap_entry_t);
    impl_wrap!('scope, TypeMapLevel, jl_typemap_level_t);
    impl_wrap!('scope, Union, jl_uniontype_t);
    impl_wrap!('scope, UnionAll, jl_unionall_t);
    impl_wrap!('scope, WeakRef, jl_weakref_t);
}
