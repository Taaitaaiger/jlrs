use itertools::Itertools;
use quote::format_ident;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Expr, FnArg, Ident, ItemFn, Result, Signature, Token,
};

use super::{
    arg_type_fragments,
    generics::{GenericEnvironment, MacroOrType, TypeVarEnv},
    has_outer_path_attr,
    init_fn::InitFn,
    override_module_fragment, return_type_fragments,
};
use crate::{
    module::{
        as_return_as, take_type, Apply, ParameterEnvironment, ParameterList, RenameFragments,
    },
    JuliaModule,
};

pub struct ExportedFunction {
    pub is_pub: bool,
    pub func: Signature,
    pub _as_token: Option<Token![as]>,
    pub name_override: Option<RenameFragments>,
    pub exclamation_mark_token: Option<Token![!]>,
    pub type_var_env: Option<TypeVarEnv>,
}

impl Parse for ExportedFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let func = input.parse()?;

        let lookahead = input.lookahead1();
        let (as_token, name_override, exclamation_mark_token) = if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = RenameFragments::parse_separated_nonempty(input)?;
            let exclamation_mark_token = input.parse()?;

            (as_token, Some(name_override), exclamation_mark_token)
        } else {
            (None, None, None)
        };

        let lookahead = input.lookahead1();
        let type_var_env = if lookahead.peek(Token![use]) {
            let i = Some(input.parse()?);
            i
        } else {
            None
        };

        Ok(ExportedFunction {
            is_pub: false,
            func,
            _as_token: as_token,
            name_override: name_override,
            exclamation_mark_token,
            type_var_env,
        })
    }
}

impl ExportedFunction {
    pub fn init_with_env(
        &self,
        generic: &GenericEnvironment,
        env: Option<&ParameterEnvironment>,
        offset: &mut usize,
        gc_safe: bool,
    ) -> Result<Expr> {
        let n_args = self.func.inputs.len();
        let name_ident = &self.func.ident;
        let start = *offset;

        let override_module_fragment = override_module_fragment(&self.name_override);
        let mut rename = self
            .name_override
            .as_ref()
            .map(|parts| parts.last())
            .flatten()
            .unwrap_or(name_ident)
            .to_string();

        if self.exclamation_mark_token.is_some() {
            rename.push('!')
        }

        let env = ParameterEnvironment::new(generic, env);
        let n_combinations = env.n_combinations();

        let mut list = ParameterList::new(&env);
        let mut resolver = list.resolver();

        let exprs = (0..n_combinations).map(|i| -> Result<Expr> {
            env.nth_combination(&mut list, i);
            list.resolve(&mut resolver);

            let inputs = resolver.apply(&self.func.inputs);
            let (ccall_arg_types, function_arg_types) = arg_type_fragments(&inputs)?;
            let ret_ty = resolver.apply(&self.func.output);
            let (ccall_ret_type, julia_ret_type) = return_type_fragments(&ret_ty);
            let new_ret_ty = as_return_as(&ret_ty);
            let ret_ty = take_type(ret_ty.clone());

            let ccall_arg_idx = 0..n_args;
            let julia_arg_idx = 0..n_args;
            let args = resolver.apply(&self.func.inputs);

            let names = args.iter().map(|arg| match arg {
                FnArg::Typed(ty) => &ty.pat,
                _ => unreachable!(),
            });
            let names = Punctuated::<_, Comma>::from_iter(names);

            let call_expr: Expr = if gc_safe {
                parse_quote! {  ::jlrs::memory::gc::gc_safe(|| #name_ident(#names)) }
            } else {
                parse_quote! { #name_ident(#names) }
            };

            let span = self.func.span();
            let invoke_fn: ItemFn = parse_quote_spanned! {
                span=> unsafe extern "C" fn invoke(#args) #new_ret_ty {
                    let res = #call_expr;
                    <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                }
            };

            let env_expr: Expr = if let Some(x) = self.type_var_env.as_ref() {
                match &x.macro_or_type {
                    MacroOrType::Macro(m) => {
                        parse_quote! { <#m as ::jlrs::data::types::construct_type::TypeVars>::into_env(&mut frame) }
                    }
                    MacroOrType::Type(t) => {
                        parse_quote! { <#t as ::jlrs::data::types::construct_type::TypeVars>::into_env(&mut frame) }
                    }
                }
            } else {
                parse_quote! { ::jlrs::data::types::construct_type::TypeVarEnv::empty(&frame) }
            };

            let ex = parse_quote! {
                {
                    let name = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                    let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&frame).as_value();
                    let any_type = ::jlrs::data::managed::datatype::DataType::any_type(&frame).as_value();

                    #invoke_fn

                    // Root #2
                    let func = ::jlrs::data::managed::value::Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                    unsafe {
                        // Root #3
                        let env = #env_expr;
                        // Root #4
                        let mut ccall_arg_types = ::jlrs::data::managed::array::Vector::new_for_unchecked(
                            &mut frame,
                            type_type,
                            #n_args,
                        );

                        let mut ccall_arg_types_ref = ccall_arg_types.indeterminate_data_mut();

                        // Root #5
                        let mut julia_arg_types = ::jlrs::data::managed::array::Vector::new_for_unchecked(
                            &mut frame,
                            any_type,
                            #n_args,
                        );

                        let mut julia_arg_types_ref = julia_arg_types.indeterminate_data_mut();

                        #(
                            frame.local_scope::<_, 2>(|mut frame| {
                                let t1 = #ccall_arg_types.as_value();
                                ccall_arg_types_ref.set_value(&mut output, #ccall_arg_idx, t1).unwrap().unwrap();
                                let t2 = #function_arg_types.as_value();
                                julia_arg_types_ref.set_value(&mut output, #julia_arg_idx, t2).unwrap().unwrap();
                            });
                        )*

                        // Root #6
                        let ccall_return_type = #ccall_ret_type;
                        // Root #7
                        let julia_return_type = #julia_ret_type;

                        let module = #override_module_fragment;

                        let false_v = ::jlrs::data::managed::value::Value::false_v(&frame);
                        // Root #8
                        function_info_ty.instantiate_unchecked(&mut frame, [
                            name.as_value(),
                            ccall_arg_types.as_value(),
                            julia_arg_types.as_value(),
                            ccall_return_type.as_value(),
                            julia_return_type.as_value(),
                            func,
                            module.as_value(),
                            env.to_svec().as_value(),
                        ])
                    }
                }
            };

            Ok(ex)
        }).collect::<Result<Vec<_>>>()?
        .into_iter()
        .unique()
        .enumerate()
        .map(|(idx, expr)| -> Expr {
            parse_quote! {
                {
                    frame.local_scope::<_, 8>(|mut frame| {
                        // Root 1
                        let mut output = frame.output();
                        let instance = #expr;
                        let n = offset + #start + #idx;
                        accessor.set_value(&mut output, n, instance).unwrap().unwrap();
                    });
                }
            }
        }).collect::<Vec<_>>();

        let n_unique = exprs.len();
        *offset += n_unique;
        let ex = parse_quote! {
            {
                accessor.grow_end_unchecked(#n_unique);
                #(#exprs)*
            }
        };

        Ok(ex)
    }
}

pub struct FunctionFragments {
    pub init_functions_fn_ident: Ident,
    pub init_functions_fn: ItemFn,
}

impl FunctionFragments {
    pub fn generate(module: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
        let init_functions_fn_ident = format_ident!("{}_functions", init_fn.init_fn);
        let n_functions = module.get_exported_functions().count();

        let fragments = module
            .get_exported_functions()
            .enumerate()
            .map(function_info_fragment)
            .collect::<Result<Vec<_>>>()?;

        let init_functions_fn = parse_quote! {
            unsafe fn #init_functions_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                array: &mut ::jlrs::data::managed::array::Vector<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                use ::jlrs::data::managed::array::data::accessor::{AccessorMut1D as _, AccessorMut as _, AccessorMut as _};

                unsafe {
                    let mut accessor = array.indeterminate_data_mut();
                    accessor.grow_end_unchecked(#n_functions);
                    #(#fragments)*
                }
            }
        };

        Ok(FunctionFragments {
            init_functions_fn_ident,
            init_functions_fn,
        })
    }

    pub fn generate_generic(info: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
        let mut offset = 0;

        let init_functions_fn_ident = format_ident!("{}_generic_functions", init_fn.init_fn);
        let init_functions_fragments = info
            .get_exported_generics()
            .map(|g| {
                g.to_generic_environment()
                    .init_function_fragments_env(None, &mut offset)
            })
            .collect::<Result<Vec<_>>>()?;

        let init_functions_fn = parse_quote! {
            unsafe fn #init_functions_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                array: &mut ::jlrs::data::managed::array::Vector<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                use ::jlrs::data::managed::array::{data::accessor::{AccessorMut1D as _, AccessorMut as _, Accessor as _}, dimensions::Dims as _};
                unsafe {
                    let mut accessor = array.indeterminate_data_mut();
                    let offset = accessor.array().dimensions().size();
                    #(#init_functions_fragments)*
                }
            }
        };

        let fragments = FunctionFragments {
            init_functions_fn_ident,
            init_functions_fn,
        };

        Ok(fragments)
    }
}

fn function_info_fragment(
    (index, (info, attrs)): (usize, (&ExportedFunction, Option<&[Attribute]>)),
) -> Result<Expr> {
    let n_args = info.func.inputs.len();
    let name_ident = &info.func.ident;

    let override_module_fragment = override_module_fragment(&info.name_override);
    let mut rename = info
        .name_override
        .as_ref()
        .map(|parts| parts.last())
        .flatten()
        .unwrap_or(name_ident)
        .to_string();

    if info.exclamation_mark_token.is_some() {
        rename.push('!')
    }

    let ret_ty = &info.func.output;
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty.clone());

    let (ccall_ret_type, julia_ret_type) = return_type_fragments(&info.func.output);

    let ccall_arg_idx = 0..n_args;
    let julia_arg_idx = 0..n_args;

    let (ccall_arg_types, julia_arg_types) = arg_type_fragments(&info.func.inputs)?;
    let args = &info.func.inputs;

    let names = args.iter().map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });
    let names = Punctuated::<_, Comma>::from_iter(names);

    let mut gc_safe = false;
    if let Some(attrs) = attrs {
        gc_safe = has_outer_path_attr(attrs, "gc_safe");
    }

    let call_expr: Expr = if gc_safe {
        parse_quote! {  ::jlrs::memory::gc::gc_safe(|| #name_ident(#names)) }
    } else {
        parse_quote! { #name_ident(#names) }
    };

    let span = info.func.span();
    let invoke_fn: ItemFn = parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args) #new_ret_ty {
            let res = #call_expr;
            <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
        }
    };

    let env_expr: Expr = if let Some(x) = info.type_var_env.as_ref() {
        match &x.macro_or_type {
            MacroOrType::Macro(m) => {
                parse_quote! { <#m as ::jlrs::data::types::construct_type::TypeVars>::into_env(&mut frame) }
            }
            MacroOrType::Type(t) => {
                parse_quote! { <#t as ::jlrs::data::types::construct_type::TypeVars>::into_env(&mut frame) }
            }
        }
    } else {
        parse_quote! { ::jlrs::data::types::construct_type::TypeVarEnv::empty(&frame) }
    };

    let expr = parse_quote! {
        {
            frame.local_scope::<_, 8>(|mut frame| {
                // Root #1
                let mut output = frame.output();
                let name = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&frame).as_value();
                let any_type = ::jlrs::data::managed::datatype::DataType::any_type(&frame).as_value();
                // Ensure a compile error happens if the signatures of the function don't match.

                #invoke_fn

                // Root #2
                let func = Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                unsafe {
                    // Root #3
                    let env = #env_expr;

                    // Root #4
                    let mut ccall_arg_types = ::jlrs::data::managed::array::Vector::new_for_unchecked(
                        &mut frame,
                        type_type,
                        #n_args,
                    );

                    let mut ccall_arg_types_ref = ccall_arg_types.indeterminate_data_mut();

                    // Root #5
                    let mut julia_arg_types = ::jlrs::data::managed::array::Vector::new_for_unchecked(
                        &mut frame,
                        any_type,
                        #n_args,
                    );

                    let mut julia_arg_types_ref = julia_arg_types.indeterminate_data_mut();

                    #(
                        frame.local_scope::<_, 2>(|mut frame| {
                            let t1 = #ccall_arg_types.as_value();
                            ccall_arg_types_ref.set_value(&mut output, #ccall_arg_idx, t1).unwrap().unwrap();
                            let t2 = #julia_arg_types.as_value();
                            julia_arg_types_ref.set_value(&mut output, #julia_arg_idx, t2).unwrap().unwrap();
                        });
                    )*

                    // Root #6
                    let ccall_return_type = #ccall_ret_type;
                    // Root #7
                    let julia_return_type = #julia_ret_type;

                    let module = #override_module_fragment;
                    let false_v = ::jlrs::data::managed::value::Value::false_v(&frame);

                    // Root #8
                    let instance = function_info_ty.instantiate_unchecked(&mut frame, [
                        name.as_value(),
                        ccall_arg_types.as_value(),
                        julia_arg_types.as_value(),
                        ccall_return_type.as_value(),
                        julia_return_type.as_value(),
                        func,
                        module.as_value(),
                        env.to_svec().as_value(),
                    ]);

                    let n = #index;
                    accessor.set_value(&mut output, n, instance).unwrap().unwrap();
                }
            });
        }
    };

    Ok(expr)
}
