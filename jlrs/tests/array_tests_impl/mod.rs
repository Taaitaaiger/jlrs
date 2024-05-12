pub(crate) mod array_bits_data_mut;
pub(crate) use array_bits_data_mut::tests::*;
pub(crate) mod array_conversions;
pub(crate) use array_conversions::tests::*;
pub(crate) mod array_inline_data_mut;
pub(crate) use array_inline_data_mut::tests::*;
pub(crate) mod array_managed_data_mut;
pub(crate) use array_managed_data_mut::tests::*;
pub(crate) mod array_type_constructor;
pub(crate) use array_type_constructor::tests::*;
pub(crate) mod array_value_data_mut;
pub(crate) use array_value_data_mut::tests::*;
pub(crate) mod array_bits_data;
pub(crate) use array_bits_data::tests::*;
pub(crate) mod array_fields_and_flags;
pub(crate) use array_fields_and_flags::tests::*;
pub(crate) mod array_inline_data;
pub(crate) use array_inline_data::tests::*;
pub(crate) mod array_managed_data;
pub(crate) use array_managed_data::tests::*;
pub(crate) mod array_union_data_mut;
pub(crate) use array_union_data_mut::tests::*;
pub(crate) mod array_value_data;
pub(crate) use array_value_data::tests::*;
pub(crate) mod array_constructors;
pub(crate) use array_constructors::tests::*;
pub(crate) mod array_grow_del;
// pub(crate) use array_grow_del::tests::*;
pub(crate) mod array_layouts;
pub(crate) use array_layouts::tests::*;
pub(crate) mod array_union_data;
pub(crate) use array_union_data::tests::*;
pub(crate) mod reshape_array;
#[cfg(any(
    feature = "julia-1-6",
    feature = "julia-1-7",
    feature = "julia-1-8",
    feature = "julia-1-9",
    feature = "julia-1-10"
))]
pub(crate) use reshape_array::tests::*;
pub(crate) mod ranked_array_constructors;
pub(crate) use ranked_array_constructors::tests::*;
mod typed_array_constructors;
pub(crate) use typed_array_constructors::tests::*;
mod typed_ranked_array_constructors;
pub(crate) use typed_ranked_array_constructors::tests::*;
mod typed_vector_constructors;
pub(crate) use typed_vector_constructors::tests::*;
mod typed_vector_fields;
pub(crate) use typed_vector_fields::tests::*;

mod vector_any_constructors;
pub(crate) use vector_any_constructors::tests::*;
