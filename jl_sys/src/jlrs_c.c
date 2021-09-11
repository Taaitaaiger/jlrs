#include "jlrs_c.h"
#include <julia.h>

jlrs_result_t jlrs_alloc_array_1d(jl_value_t *atype, size_t nr)
{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = (jl_value_t *)jl_alloc_array_1d(atype, nr);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_alloc_array_2d(jl_value_t *atype, size_t nr, size_t nc)
{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = (jl_value_t *)jl_alloc_array_2d(atype, nr, nc);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_alloc_array_3d(jl_value_t *atype, size_t nr, size_t nc, size_t z)
{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = (jl_value_t *)jl_alloc_array_3d(atype, nr, nc, z);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_apply_array_type(jl_value_t *ty, size_t dim)

{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = jl_apply_array_type(ty, dim);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_apply_type(jl_value_t *tc, jl_value_t **params, size_t n)

{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = jl_apply_type(tc, params, n);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_new_array(jl_value_t *atype, jl_value_t *dims)
{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = (jl_value_t *)jl_new_array(atype, dims);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_new_structv(jl_datatype_t *type, jl_value_t **args, uint32_t na)
{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = jl_new_structv(type, args, na);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_new_typevar(jl_sym_t *name, jl_value_t *lb, jl_value_t *ub)
{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = (jl_value_t *)jl_new_typevar(name, lb, ub);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_set_const(jl_module_t *m JL_ROOTING_ARGUMENT, jl_sym_t *var, jl_value_t *val JL_ROOTED_ARGUMENT)
{
    jlrs_result_t out;

    JL_TRY
    {
        jl_set_const(m, var, val);
        out.data = NULL;
        out.flag = JLRS_RESULT_VOID;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_set_global(jl_module_t *m JL_ROOTING_ARGUMENT, jl_sym_t *var, jl_value_t *val JL_ROOTED_ARGUMENT)
{
    jlrs_result_t out;

    JL_TRY
    {
        jl_set_global(m, var, val);
        out.data = NULL;
        out.flag = JLRS_RESULT_VOID;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_set_nth_field(jl_value_t *v, size_t i, jl_value_t *rhs)
{
    jlrs_result_t out;

    JL_TRY
    {
        jl_set_nth_field(v, i, rhs);
        out.data = NULL;
        out.flag = JLRS_RESULT_VOID;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_type_union(jl_value_t **ts, size_t n)
{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = jl_type_union(ts, n);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_type_unionall(jl_tvar_t *v, jl_value_t *body)
{
    jlrs_result_t out;

    JL_TRY
    {
        out.data = jl_type_unionall(v, body);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_reshape_array(jl_value_t *atype, jl_array_t *data, jl_value_t *_dims)
{
    jlrs_result_t out;

    JL_TRY
    {

        out.data = (jl_value_t *)jl_reshape_array(atype, data, _dims);
        out.flag = JLRS_RESULT_VALUE;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_array_grow_end(jl_array_t *a, size_t inc)
{
    jlrs_result_t out;

    JL_TRY
    {
        jl_array_grow_end(a, inc);
        out.data = NULL;
        out.flag = JLRS_RESULT_VOID;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_array_del_end(jl_array_t *a, size_t dec)
{
    jlrs_result_t out;

    JL_TRY
    {
        jl_array_del_end(a, dec);
        out.data = NULL;
        out.flag = JLRS_RESULT_VOID;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_array_grow_beg(jl_array_t *a, size_t inc)
{
    jlrs_result_t out;

    JL_TRY
    {
        jl_array_grow_beg(a, inc);
        out.data = NULL;
        out.flag = JLRS_RESULT_VOID;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jlrs_result_t jlrs_array_del_beg(jl_array_t *a, size_t dec)
{
    jlrs_result_t out;

    JL_TRY
    {
        jl_array_del_beg(a, dec);
        out.data = NULL;
        out.flag = JLRS_RESULT_VOID;
    }
    JL_CATCH
    {
        out.data = jl_current_exception();
        out.flag = JLRS_RESULT_ERR;
    }
    jl_exception_clear();

    return out;
}

jl_task_t *jlrs_current_task(void)
{
    return jl_current_task;
}
