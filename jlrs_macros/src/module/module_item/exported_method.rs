use itertools::Itertools;
use quote::format_ident;
use syn::{
    Attribute, Expr, FnArg, Ident, ItemFn, Result, Signature, Token, Type,
    parse::{Parse, ParseStream},
    parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Pub},
};

use super::{
    generics::{GenericEnvironment, MacroOrType, TypeVarEnv},
    init_fn::InitFn,
    override_module_fragment, return_type_fragments,
};
use crate::{
    JuliaModule,
    module::{
        Apply, ParameterEnvironment, ParameterList, RenameFragments, ResolvedParameterList,
        as_return_as, module_item::has_outer_path_attr, take_type,
    },
};

pub struct ExportedMethod {
    pub _in_token: Token![in],
    pub parent: Type,
    pub _is_pub: bool,
    pub func: Signature,
    pub _as_token: Option<Token![as]>,
    pub name_override: Option<RenameFragments>,
    pub exclamation_mark_token: Option<Token![!]>,
    pub type_var_env: Option<TypeVarEnv>,
}

impl Parse for ExportedMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let in_token = input.parse()?;
        let parent = input.parse()?;

        let lookahead = input.lookahead1();
        let is_pub = lookahead.peek(Token![pub]);
        if is_pub {
            let _: Pub = input.parse()?;
        }

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
            Some(input.parse()?)
        } else {
            None
        };

        Ok(ExportedMethod {
            _in_token: in_token,
            parent,
            _is_pub: is_pub,
            func,
            _as_token: as_token,
            name_override: name_override,
            exclamation_mark_token,
            type_var_env,
        })
    }
}

impl ExportedMethod {
    pub fn init_with_env(
        &self,
        generic: &GenericEnvironment,
        env: Option<&ParameterEnvironment>,
        offset: &mut usize,
        untracked_self: bool,
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

            let ret_ty = resolver.apply(&self.func.output);
            let (ccall_ret_type, julia_ret_type) = return_type_fragments(&ret_ty);

            let ccall_arg_idx = 0..n_args;
            let julia_arg_idx = 0..n_args;

            let (ccall_arg_types, julia_arg_types, invoke_fn) = method_arg_type_fragments_in_env(self, &resolver, untracked_self, gc_safe);

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

                    #invoke_fn;

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
                            frame.local_scope::<_, 2>(|mut frame|{
                                let t1 = #ccall_arg_types.as_value();
                                ccall_arg_types_ref.set_value(&mut frame, #ccall_arg_idx, t1).unwrap().unwrap();
                                let t2 = #julia_arg_types.as_value();
                                julia_arg_types_ref.set_value(&mut frame, #julia_arg_idx, t2).unwrap().unwrap();
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
                            ccall_return_type,
                            julia_return_type,
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
                        // Root #1
                        let mut output = frame.output();
                        let instance = #expr;
                        let start = #start;
                        let idx = #idx;
                        let n = offset + start + idx;
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

pub struct MethodFragments {
    pub init_methods_fn_ident: Ident,
    pub init_methods_fn: ItemFn,
}

impl MethodFragments {
    pub fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_methods_fn_ident = format_ident!("{}_methods", init_fn.init_fn);
        let n_methods = module.get_exported_methods().count();

        let method_init_fragments = module
            .get_exported_methods()
            .enumerate()
            .map(method_info_fragment);

        let init_methods_fn = parse_quote! {
            unsafe fn #init_methods_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                array: &mut ::jlrs::data::managed::array::Vector<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                use ::jlrs::data::managed::array::{data::accessor::{AccessorMut1D as _, AccessorMut as _, Accessor as _}, dimensions::Dims as _};

                unsafe {
                    let mut accessor = array.indeterminate_data_mut();
                    let offset = accessor.array().dimensions().size();
                    accessor.grow_end_unchecked(#n_methods);
                    #(#method_init_fragments)*
                }
            }
        };

        MethodFragments {
            init_methods_fn_ident,
            init_methods_fn,
        }
    }

    pub fn generate_generic(info: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
        let mut offset = 0;

        let init_methods_fn_ident = format_ident!("{}_generic_methods", init_fn.init_fn);
        let init_methods_fragments = info
            .get_exported_generics()
            .map(|g| {
                g.to_generic_environment()
                    .init_method_fragments_env(None, &mut offset)
            })
            .collect::<Result<Vec<_>>>()?;

        let init_methods_fn = parse_quote! {
            unsafe fn #init_methods_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                array: &mut ::jlrs::data::managed::array::Vector<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                use ::jlrs::data::managed::array::{data::accessor::{AccessorMut1D as _, AccessorMut as _, Accessor as _}, dimensions::Dims as _};

                unsafe {
                    let mut accessor = array.indeterminate_data_mut();
                    let offset = accessor.array().dimensions().size();
                    #(#init_methods_fragments)*
                }
            }
        };

        let fragments = MethodFragments {
            init_methods_fn_ident,
            init_methods_fn,
        };

        Ok(fragments)
    }
}
fn method_info_fragment<'a>(
    (index, (info, attrs)): (usize, (&'a ExportedMethod, Option<&'a [Attribute]>)),
) -> Expr {
    let n_args = info.func.inputs.len();
    let name_ident = &info.func.ident;

    let mut untracked_self = false;
    let mut gc_safe = false;

    if let Some(attrs) = attrs {
        untracked_self = has_outer_path_attr(attrs, "untracked_self");
        gc_safe = has_outer_path_attr(attrs, "gc_safe");
    }

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
    let (ccall_ret_type, julia_ret_type) = return_type_fragments(ret_ty);

    let ccall_arg_idx = 0..n_args;

    let (ccall_arg_types, julia_arg_types, invoke_fn) =
        method_arg_type_fragments(info, untracked_self, gc_safe);

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

    parse_quote! {
        {
            frame.local_scope::<_, 8>(|mut frame| {
                // Root #1
                let mut output = frame.output();
                let name = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&frame).as_value();
                let any_type = ::jlrs::data::managed::datatype::DataType::any_type(&frame).as_value();

                #invoke_fn;

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
                            let idx = #ccall_arg_idx;
                            let ccall_arg_type = #ccall_arg_types.as_value();
                            ccall_arg_types_ref.set_value(&mut output, idx, ccall_arg_type).unwrap().unwrap();
                            let julia_arg_type = #julia_arg_types.as_value();
                            julia_arg_types_ref.set_value(&mut output, idx, julia_arg_type).unwrap().unwrap();
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
                        ccall_return_type,
                        julia_return_type,
                        func,
                        module.as_value(),
                        env.to_svec().as_value(),
                    ]);

                    let n = #index + offset;
                    accessor.set_value(&mut output, n, instance).unwrap().unwrap();
                }
            });
        }
    }
}

fn method_arg_type_fragments<'a>(
    info: &'a ExportedMethod,
    untracked_self: bool,
    gc_safe: bool,
) -> (
    impl 'a + Iterator<Item = Expr>,
    impl 'a + Iterator<Item = Expr>,
    ItemFn,
) {
    let inputs = &info.func.inputs;

    let takes_self = match inputs.first() {
        Some(FnArg::Receiver(r)) => Some((r.mutability.is_some(), r.reference.is_some())),
        _ => None,
    };

    let invoke_fn = match takes_self {
        None => invoke_fn_no_self_method_fragment(info, gc_safe),
        Some((true, true)) => invoke_fn_mut_self_method_fragment(info, untracked_self, gc_safe),
        Some((false, true)) => invoke_fn_ref_self_method_fragment(info, untracked_self, gc_safe),
        Some((_, false)) => invoke_fn_move_self_method_fragment(info, untracked_self, gc_safe),
    };

    let parent = &info.parent;
    let ccall_arg_types = inputs
        .iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = &ty.ty;
                    let span = ty.span();
                    parse_quote_spanned! {
                        span=> if env.is_empty() {
                            <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                        } else {
                            <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)
                        }
                    }
                },
                _ => {
                    let span = parent.span();
                    parse_quote_spanned! {
                        span=> if env.is_empty() {
                            <<::jlrs::data::managed::value::typed::TypedValue::<#parent> as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                        } else {
                            <<::jlrs::data::managed::value::typed::TypedValue::<#parent> as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)
                        }
                    }
                },
            }
        });

    let julia_arg_types = inputs
        .iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = &ty.ty;
                    let span = ty.span();
                    parse_quote_spanned! {
                        span=> if env.is_empty() {
                            <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                        } else{
                            <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)
                        }
                    }
                },
                _ => {
                    let span = parent.span();
                    parse_quote_spanned! {
                        span=> if env.is_empty() {
                            <<::jlrs::data::managed::value::typed::TypedValue<#parent> as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                        } else{
                            <<::jlrs::data::managed::value::typed::TypedValue<#parent> as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)
                        }
                    }
                },
            }
        });

    (ccall_arg_types, julia_arg_types, invoke_fn)
}

fn method_arg_type_fragments_in_env<'a>(
    info: &'a ExportedMethod,
    resolver: &'a ResolvedParameterList,
    untracked_self: bool,
    gc_safe: bool,
) -> (
    impl 'a + Iterator<Item = Expr>,
    impl 'a + Iterator<Item = Expr>,
    ItemFn,
) {
    let inputs = &info.func.inputs;

    let takes_self = match inputs.first() {
        Some(FnArg::Receiver(r)) => Some((r.mutability.is_some(), r.reference.is_some())),
        _ => None,
    };

    let invoke_fn = match takes_self {
        None => invoke_fn_no_self_method_fragment_in_env(info, resolver, gc_safe),
        Some((true, true)) => {
            invoke_fn_mut_self_method_fragment_in_env(info, resolver, untracked_self, gc_safe)
        }
        Some((false, true)) => {
            invoke_fn_ref_self_method_fragment_in_env(info, resolver, untracked_self, gc_safe)
        }
        Some((_, false)) => {
            invoke_fn_move_self_method_fragment_in_env(info, resolver, untracked_self, gc_safe)
        }
    };

    let parent = resolver.apply(&info.parent);
    let parent2 = parent.clone();

    let ccall_arg_types = inputs
        .iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = resolver.apply(ty.ty.as_ref());
                    let span = ty.span();
                    parse_quote_spanned! {
                        span=> if env.is_empty() {

                            <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                        } else {
                            <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)

                        }
                    }
                },
                _ => {
                    let span = parent.span();
                    parse_quote_spanned! {
                        span=> if env.is_empty() {

                            <<::jlrs::data::managed::value::typed::TypedValue::<#parent> as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                        } else {
                            <<::jlrs::data::managed::value::typed::TypedValue::<#parent> as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)

                        }
                    }
                },
            }
        });

    let julia_arg_types = inputs
        .iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = resolver.apply(ty.ty.as_ref());
                    let span = ty.span();
                    parse_quote_spanned! {
                        span=> if env.is_empty() {

                            <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                        } else {
                            <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)

                        }
                    }
                },
                _ => {
                    let span = parent2.span();
                    parse_quote_spanned! {
                        span=> if env.is_empty() {

                            <<::jlrs::data::managed::value::typed::TypedValue<#parent2> as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                        } else {
                            <<::jlrs::data::managed::value::typed::TypedValue<#parent2> as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)

                        }
                    }
                },
            }
        });

    (ccall_arg_types, julia_arg_types, invoke_fn)
}

fn invoke_fn_no_self_method_fragment(info: &ExportedMethod, gc_safe: bool) -> ItemFn {
    let name = &info.func.ident;
    let span = info.func.ident.span();
    let ty = &info.parent;
    let ret_ty = &info.func.output;
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty.clone());

    let args = &info.func.inputs;
    let names = args.iter().map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    let call_expr: Expr = if gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                <#ty>::#name(#names)
            })
        }
    } else {
        parse_quote! { <#ty>::#name(#names) }
    };

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args) #new_ret_ty {
            unsafe {
                let res = #call_expr;
                <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
            }
        }
    }
}

fn invoke_fn_no_self_method_fragment_in_env(
    info: &ExportedMethod,
    resolver: &ResolvedParameterList,
    gc_safe: bool,
) -> ItemFn {
    let name = &info.func.ident;
    let span = info.func.ident.span();
    let ty = resolver.apply(&info.parent);
    let ret_ty = resolver.apply(&info.func.output);
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty);

    let args = resolver.apply(&info.func.inputs);
    let names: Punctuated<_, Comma> = args
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(ty) => &ty.pat,
            _ => unreachable!(),
        })
        .collect();

    let call_expr: Expr = if gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                <#ty>::#name(#names)
            })
        }
    } else {
        parse_quote! { <#ty>::#name(#names) }
    };

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args) #new_ret_ty {
            unsafe {
                let res = #call_expr;
                <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
            }
        }
    }
}

fn invoke_fn_ref_self_method_fragment(
    info: &ExportedMethod,
    untracked_self: bool,
    gc_safe: bool,
) -> ItemFn {
    let name = &info.func.ident;
    let span = info.func.ident.span();
    let ty = &info.parent;
    let ret_ty = &info.func.output;
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty.clone());

    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();

    *first = parse_quote! {
        this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = cloned_args;

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    let to_ref_expr: Expr = if untracked_self {
        parse_quote! { ::std::result::Result::<_, ()>::Ok((&this).data_ptr().cast::<#ty>().as_ref()) }
    } else {
        parse_quote! { (&this).track_shared() }
    };

    let call_expr: Expr = if gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                this.#name(#names)
            })
        }
    } else {
        parse_quote! { this.#name(#names) }
    };

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args_self_renamed) #new_ret_ty {
            unsafe {
                match #to_ref_expr {
                    Ok(this) => {
                        let res = #call_expr;
                        <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                    },
                    Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                }
            }
        }
    }
}

fn invoke_fn_ref_self_method_fragment_in_env(
    info: &ExportedMethod,
    resolver: &ResolvedParameterList,
    untracked_self: bool,
    gc_safe: bool,
) -> ItemFn {
    let name = &info.func.ident;
    let span = info.func.ident.span();
    let ty = resolver.apply(&info.parent);
    let ret_ty = resolver.apply(&info.func.output);
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty.clone());

    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();

    *first = parse_quote! {
        this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = resolver.apply(&cloned_args);

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    let to_ref_expr: Expr = if untracked_self {
        parse_quote! { ::std::result::Result::<_, ()>::Ok((&this).data_ptr().cast::<#ty>().as_ref()) }
    } else {
        parse_quote! { (&this).track_shared() }
    };

    let call_expr: Expr = if gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                this.#name(#names)
            })
        }
    } else {
        parse_quote! { this.#name(#names) }
    };

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args_self_renamed) #new_ret_ty {
            unsafe {
                match #to_ref_expr {
                    Ok(this) => {
                        let res = #call_expr;
                        <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                    },
                    Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                }
            }
        }
    }
}

fn invoke_fn_move_self_method_fragment(
    info: &ExportedMethod,
    untracked_self: bool,
    gc_safe: bool,
) -> ItemFn {
    let name = &info.func.ident;
    let span = info.func.ident.span();
    let ty = &info.parent;
    let ret_ty = &info.func.output;
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty.clone());

    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();

    *first = parse_quote! {
        this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = cloned_args;

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    let to_ref_expr: Expr = if untracked_self {
        parse_quote! { ::std::result::Result::<_, ()>::Ok((&this).data_ptr().cast::<#ty>().as_ref()) }
    } else {
        parse_quote! { (&this).track_shared() }
    };

    let call_expr: Expr = if gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                this.clone().#name(#names)
            })
        }
    } else {
        parse_quote! { this.clone().#name(#names) }
    };

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args_self_renamed) #new_ret_ty {
            unsafe {
                match #to_ref_expr {
                    Ok(this) => {
                        let res = #call_expr;
                        <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                    },
                    Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                }
            }
        }
    }
}

fn invoke_fn_move_self_method_fragment_in_env(
    info: &ExportedMethod,
    resolver: &ResolvedParameterList,
    untracked_self: bool,
    gc_safe: bool,
) -> ItemFn {
    let name = &info.func.ident;
    let span = info.func.ident.span();
    let ty = resolver.apply(&info.parent);
    let ret_ty = resolver.apply(&info.func.output);
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty.clone());

    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();

    *first = parse_quote! {
        this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = resolver.apply(&cloned_args);

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    let to_ref_expr: Expr = if untracked_self {
        parse_quote! { ::std::result::Result::<_, ()>::Ok((&this).data_ptr().cast::<#ty>().as_ref()) }
    } else {
        parse_quote! { (&this).track_shared() }
    };

    let call_expr: Expr = if gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                this.clone().#name(#names)
            })
        }
    } else {
        parse_quote! { this.clone().#name(#names) }
    };

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args_self_renamed) #new_ret_ty {
            unsafe {
                match #to_ref_expr {
                    Ok(this) => {
                        let res = #call_expr;
                        <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                    },
                    Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                }
            }
        }
    }
}

fn invoke_fn_mut_self_method_fragment(
    info: &ExportedMethod,
    untracked_self: bool,
    gc_safe: bool,
) -> ItemFn {
    let name = &info.func.ident;
    let span = info.func.ident.span();
    let ty = &info.parent;
    let ret_ty = &info.func.output;
    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty.clone());

    *first = parse_quote! {
        mut this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = cloned_args;

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    let to_ref_expr: Expr = if untracked_self {
        parse_quote! { ::std::result::Result::<_, ()>::Ok((&mut this).data_ptr().cast::<#ty>().as_mut()) }
    } else {
        parse_quote! { (&mut this).track_exclusive() }
    };

    let call_expr: Expr = if gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                this.#name(#names)
            })
        }
    } else {
        parse_quote! { this.#name(#names) }
    };

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args_self_renamed) #new_ret_ty {
            unsafe {
                match #to_ref_expr {
                    #[allow(unused_mut)]
                    Ok(mut this) => {
                        let res = #call_expr;
                        <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                    },
                    Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                }
            }
        }
    }
}

fn invoke_fn_mut_self_method_fragment_in_env(
    info: &ExportedMethod,
    resolver: &ResolvedParameterList,
    untracked_self: bool,
    gc_safe: bool,
) -> ItemFn {
    let name = &info.func.ident;
    let span = info.func.ident.span();
    let ty = resolver.apply(&info.parent);
    let ret_ty = resolver.apply(&info.func.output);
    let new_ret_ty = as_return_as(&ret_ty);
    let ret_ty = take_type(ret_ty.clone());
    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();

    *first = parse_quote! {
        mut this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = resolver.apply(&cloned_args);

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    let to_ref_expr: Expr = if untracked_self {
        parse_quote! { ::std::result::Result::<_, ()>::Ok((&mut this).data_ptr().cast::<#ty>().as_mut()) }
    } else {
        parse_quote! { (&mut this).track_exclusive() }
    };

    let call_expr: Expr = if gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                this.#name(#names)
            })
        }
    } else {
        parse_quote! { this.#name(#names) }
    };

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#args_self_renamed) #new_ret_ty {
            unsafe {
                match #to_ref_expr {
                    #[allow(unused_mut)]
                    Ok(mut this) => {
                        let res = #call_expr;
                        <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                    },
                    Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                }
            }
        }
    }
}
