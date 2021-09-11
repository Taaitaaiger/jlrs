#include <julia.h>

//! The Julia C API can throw exceptions when used incorrectly, whenever this happens the code
//! will try to jump to the nearest enclosing catch-block. If no enclosing catch-block exists the
//! program is aborted. Because the JULIA_TRY and JULIA_CATCH macros can't be expressed in Rust
//! without depending on undefined behaviour, this small C library provides a few functions that
//! wrap the functions from the C API that jlrs uses and can throw exceptions in such blocks.

/// Flag used by `jlrs_result_t` that indicates what the union field of that struct contains.
typedef enum
{
    JLRS_RESULT_VOID = 0,
    JLRS_RESULT_VALUE = 1,
    JLRS_RESULT_ERR = 2,
} jlrs_result_tag_t;

/// Container for the result of some function called in a JULIA_TRY block. The flag indicates what
/// the union field contains. If the flag is `JLRS_RESULT_VOID` `data` is set to a null
/// pointer, if it's `JLRS_RESULT_ERR` `data` is set to the pointer to the exception.
typedef struct
{
    jlrs_result_tag_t flag;
    jl_value_t *data;
} jlrs_result_t;

jlrs_result_t jlrs_alloc_array_1d(jl_value_t *atype, size_t nr);
jlrs_result_t jlrs_alloc_array_2d(jl_value_t *atype, size_t nr, size_t nc);
jlrs_result_t jlrs_alloc_array_3d(jl_value_t *atype, size_t nr, size_t nc, size_t z);
jlrs_result_t jlrs_apply_array_type(jl_value_t *ty, size_t dim);
jlrs_result_t jlrs_apply_type(jl_value_t *tc, jl_value_t **params, size_t n);
jlrs_result_t jlrs_new_array(jl_value_t *atype, jl_value_t *dims);
jlrs_result_t jlrs_new_structv(jl_datatype_t *type, jl_value_t **args, uint32_t na);
jlrs_result_t jlrs_new_typevar(jl_sym_t *name, jl_value_t *lb, jl_value_t *ub);
jlrs_result_t jlrs_set_const(jl_module_t *m JL_ROOTING_ARGUMENT, jl_sym_t *var, jl_value_t *val JL_ROOTED_ARGUMENT);
jlrs_result_t jlrs_set_global(jl_module_t *m JL_ROOTING_ARGUMENT, jl_sym_t *var, jl_value_t *val JL_ROOTED_ARGUMENT);
jlrs_result_t jlrs_set_nth_field(jl_value_t *v, size_t i, jl_value_t *rhs);
jlrs_result_t jlrs_type_union(jl_value_t **ts, size_t n);
jlrs_result_t jlrs_type_unionall(jl_tvar_t *v, jl_value_t *body);
jlrs_result_t jlrs_reshape_array(jl_value_t *atype, jl_array_t *data, jl_value_t *_dims);
jlrs_result_t jlrs_array_grow_end(jl_array_t *a, size_t inc);
jlrs_result_t jlrs_array_del_end(jl_array_t *a, size_t dec);
jlrs_result_t jlrs_array_grow_beg(jl_array_t *a, size_t inc);
jlrs_result_t jlrs_array_del_beg(jl_array_t *a, size_t dec);

jl_task_t *jlrs_current_task(void);
