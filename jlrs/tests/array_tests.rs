pub(crate) mod array_tests_impl;
pub(crate) mod util;
use array_tests_impl::*;

#[test]
fn array_tests() {
    array_bits_data_mut_tests();
    array_bits_data_tests();
    array_constructor_tests();
    array_conversion_tests();
    array_fields_and_flags_tests();
    array_grow_del_tests();
    array_inline_data_mut_tests();
    array_inline_data_tests();
    array_layouts_tests();
    array_managed_data_mut_tests();
    array_managed_data_tests();
    array_type_constructor_tests();
    array_union_data_mut_tests();
    array_union_data_tests();
    array_value_data_mut_tests();
    array_value_data_tests();
    ranked_array_constructors_tests();
    typed_array_constructors_tests();
    typed_ranked_array_constructors_tests();
    typed_vector_constructors_test();
    typed_vector_fields_tests();
    vector_any_tests();
}
