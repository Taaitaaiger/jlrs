mod parameters;
use std::iter::FromIterator;

use itertools::Itertools;
use parameters::{ParameterEnvironment, ParameterList};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, ToTokens};
use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Bracket, Comma},
    AttrStyle, Attribute, Error, Expr, ExprLit, FnArg, GenericArgument, Ident, ItemFn, Lit, Meta,
    Path, PathArguments, Result, ReturnType, Signature, Token, Type, TypeImplTrait, TypeParamBound,
};

use self::parameters::{Apply, ResolvedParameterList};
use crate::module::parameters::{as_return_as, take_type};

type RenameFragments = Punctuated<Ident, Token![.]>;

// todo: generic docs

struct InitFn {
    _become_token: Token![become],
    init_fn: Ident,
}

impl Parse for InitFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let init_fn_token = input.parse()?;
        let init_fn = input.parse()?;

        Ok(InitFn {
            _become_token: init_fn_token,
            init_fn,
        })
    }
}

struct ExportedType {
    _struct_token: Token![struct],
    name: Path,
    _as_token: Option<Token![as]>,
    name_override: Option<RenameFragments>,
}

impl ExportedType {
    fn init_with_env(
        &self,
        generic: &GenericEnvironment,
        env: Option<&ParameterEnvironment>,
    ) -> Expr {
        let override_module_fragment = override_module_fragment(&self.name_override);
        let name = &self.name;
        let name_ident = &name.segments.last().unwrap().ident;

        let rename = self
            .name_override
            .as_ref()
            .map(|parts| parts.last())
            .flatten()
            .unwrap_or(name_ident)
            .to_string();

        let env = ParameterEnvironment::new(generic, env);
        let mut list = ParameterList::new(&env);
        let mut resolver = list.resolver();

        env.nth_combination(&mut list, 0);
        list.resolve(&mut resolver);
        let ty = resolver.apply(name);

        let variants = (0..env.n_combinations()).map(|i| -> Expr {
            env.nth_combination(&mut list, i);
            list.resolve(&mut resolver);
            let ty = resolver.apply(name);

            parse_quote! {
                <#ty as ::jlrs::data::types::foreign_type::ParametricVariant>::create_variant(&mut output, sym)
            }
        }).unique();

        parse_quote! {
            {
                let sym = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                let module = #override_module_fragment;
                let ty = <#ty as ::jlrs::data::types::foreign_type::ParametricBase>::create_type(&mut output, sym, module);
                let ty = ::jlrs::data::managed::erase_scope_lifetime(ty);
                let ty = ::jlrs::data::managed::union_all::UnionAll::rewrap(&mut output, ty);
                module.set_const_unchecked(sym, ty);

                #(
                    #variants;
                )*
            }
        }
    }

    fn reinit_with_env(
        &self,
        generic: &GenericEnvironment,
        env: Option<&ParameterEnvironment>,
    ) -> Expr {
        {
            let override_module_fragment = override_module_fragment(&self.name_override);
            let name = &self.name;
            let name_ident = &name.segments.last().unwrap().ident;

            let rename = self
                .name_override
                .as_ref()
                .map(|parts| parts.last())
                .flatten()
                .unwrap_or(name_ident)
                .to_string();

            let env = ParameterEnvironment::new(generic, env);
            let mut list = ParameterList::new(&env);
            let mut resolver = list.resolver();

            env.nth_combination(&mut list, 0);
            list.resolve(&mut resolver);
            let ty = resolver.apply(name);

            let variants = (0..env.n_combinations()).map(|i| -> Expr {
                env.nth_combination(&mut list, i);
                list.resolve(&mut resolver);
                let ty = resolver.apply(name);

                parse_quote! {
                    {
                        let params = <#ty as ::jlrs::data::types::foreign_type::ParametricVariant>::variant_parameters(&mut output);
                        let params = ::jlrs::data::managed::erase_scope_lifetime(params);
                        let param_slice = params.value_slice_unchecked();
                        let dt = ua.apply_types_unchecked(&mut output, param_slice).cast::<::jlrs::data::managed::datatype::DataType>().unwrap();
                        let dt = ::jlrs::data::managed::erase_scope_lifetime(dt);

                        <#ty as ::jlrs::data::types::foreign_type::ParametricVariant>::reinit_variant(dt);
                    }
                }
            }).unique();

            parse_quote! {
                {
                    let module = #override_module_fragment;

                    let ua = module
                        .global(&frame, #rename)
                        .unwrap()
                        .as_value()
                        .cast::<::jlrs::data::managed::union_all::UnionAll>()
                        .unwrap();

                    let dt = ua.base_type();

                    <#ty as ::jlrs::data::types::foreign_type::ParametricBase>::reinit_type(dt);

                    #(
                        #variants;
                    )*
                }
            }
        }
    }
}

impl Parse for ExportedType {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_token = input.parse()?;
        let name = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = RenameFragments::parse_separated_nonempty(input)?;

            Ok(ExportedType {
                _struct_token: struct_token,
                name,
                _as_token: Some(as_token),
                name_override: Some(name_override),
            })
        } else {
            Ok(ExportedType {
                _struct_token: struct_token,
                name,
                _as_token: None,
                name_override: None,
            })
        }
    }
}

struct ExportedFunction {
    func: Signature,
    _as_token: Option<Token![as]>,
    name_override: Option<RenameFragments>,
    exclamation_mark_token: Option<Token![!]>,
}

impl Parse for ExportedFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let func = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = RenameFragments::parse_separated_nonempty(input)?;
            let exclamation_mark_token = input.parse()?;

            Ok(ExportedFunction {
                func,
                _as_token: Some(as_token),
                name_override: Some(name_override),
                exclamation_mark_token,
            })
        } else {
            Ok(ExportedFunction {
                func,
                _as_token: None,
                name_override: None,
                exclamation_mark_token: None,
            })
        }
    }
}

impl ExportedFunction {
    fn init_with_env(
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

            let ex = parse_quote! {
                {
                    let name = Symbol::new(&frame, #rename);
                    let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&frame).as_value();

                    #invoke_fn

                    let func = Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                    unsafe {
                        let mut ccall_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                            &mut frame,
                            #n_args,
                            type_type);

                        let mut ccall_arg_types_ref = ccall_arg_types.value_data_mut().unwrap();

                        let mut julia_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                            &mut frame,
                            #n_args,
                            type_type);

                        let mut julia_arg_types_ref = julia_arg_types.value_data_mut().unwrap();

                        #(
                            ccall_arg_types_ref.set(#ccall_arg_idx, Some(#ccall_arg_types.as_value())).unwrap();
                            julia_arg_types_ref.set(#julia_arg_idx, Some(#function_arg_types.as_value())).unwrap();
                        )*

                        let ccall_return_type = #ccall_ret_type;
                        let julia_return_type = #julia_ret_type;

                        let module = #override_module_fragment;
                        let nothing = ::jlrs::data::managed::value::Value::nothing(&frame);

                        let false_v = ::jlrs::data::managed::value::Value::false_v(&frame);
                        function_info_ty.instantiate_unchecked(&mut frame, [
                            name.as_value(),
                            ccall_arg_types.as_value(),
                            julia_arg_types.as_value(),
                            ccall_return_type.as_value(),
                            julia_return_type.as_value(),
                            func,
                            module.as_value(),
                            false_v
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
                    frame.scope(|mut frame| {
                        let instance = #expr;
                        let n = offset + #start + #idx;
                        accessor.set(n, Some(instance)).unwrap();

                        Ok(())
                    }).unwrap();
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

struct ExportedMethod {
    _in_token: Token![in],
    parent: Type,
    func: Signature,
    _as_token: Option<Token![as]>,
    name_override: Option<RenameFragments>,
    exclamation_mark_token: Option<Token![!]>,
}

impl Parse for ExportedMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let in_token = input.parse()?;
        let parent = input.parse()?;
        let func = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = RenameFragments::parse_separated_nonempty(input)?;
            let exclamation_mark_token = input.parse()?;

            Ok(ExportedMethod {
                _in_token: in_token,
                parent,
                func,
                _as_token: Some(as_token),
                name_override: Some(name_override),
                exclamation_mark_token,
            })
        } else {
            Ok(ExportedMethod {
                _in_token: in_token,
                parent,
                func,
                _as_token: None,
                name_override: None,
                exclamation_mark_token: None,
            })
        }
    }
}

impl ExportedMethod {
    fn init_with_env(
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

            let ex = parse_quote! {
                {
                    let unrooted = frame.unrooted();
                    let name = Symbol::new(&frame, #rename);
                    let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&unrooted).as_value();

                    #invoke_fn;

                    let func = Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                    unsafe {
                        let mut ccall_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                            &mut frame,
                            #n_args,
                            type_type);

                        let mut ccall_arg_types_ref = ccall_arg_types.value_data_mut().unwrap();

                        let mut julia_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                            &mut frame,
                            #n_args,
                            type_type);

                        let mut julia_arg_types_ref = julia_arg_types.value_data_mut().unwrap();

                        #(
                            ccall_arg_types_ref.set(#ccall_arg_idx, Some(#ccall_arg_types.as_value())).unwrap();
                            julia_arg_types_ref.set(#julia_arg_idx, Some(#julia_arg_types.as_value())).unwrap();
                        )*

                        let ccall_return_type = #ccall_ret_type;
                        let julia_return_type = #julia_ret_type;

                        let module = #override_module_fragment;
                        let nothing = ::jlrs::data::managed::value::Value::nothing(&frame);

                        let false_v = ::jlrs::data::managed::value::Value::false_v(&frame);
                        function_info_ty.instantiate_unchecked(&mut frame, [
                            name.as_value(),
                            ccall_arg_types.as_value(),
                            julia_arg_types.as_value(),
                            ccall_return_type,
                            julia_return_type,
                            func,
                            module.as_value(),
                            false_v
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
                    frame.scope(|mut frame| {
                        let instance = #expr;
                        let start = #start;
                        let idx = #idx;
                        let n = offset + start + idx;
                        accessor.set(n, Some(instance)).unwrap();

                        Ok(())
                    }).unwrap();
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

struct ExportedAsyncCallback {
    _async_token: Token![async],
    func: Signature,
    _as_token: Option<Token![as]>,
    name_override: Option<RenameFragments>,
    exclamation_mark_token: Option<Token![!]>,
}

impl Parse for ExportedAsyncCallback {
    fn parse(input: ParseStream) -> Result<Self> {
        let async_token = input.parse()?;
        let func = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = RenameFragments::parse_separated_nonempty(input)?;
            let exclamation_mark_token = input.parse()?;

            Ok(ExportedAsyncCallback {
                _async_token: async_token,
                func,
                _as_token: Some(as_token),
                name_override: Some(name_override),
                exclamation_mark_token,
            })
        } else {
            Ok(ExportedAsyncCallback {
                _async_token: async_token,
                func,
                _as_token: None,
                name_override: None,
                exclamation_mark_token: None,
            })
        }
    }
}

impl ExportedAsyncCallback {
    fn init_with_env(
        &self,
        generic: &GenericEnvironment,
        env: Option<&ParameterEnvironment>,
        offset: &mut usize,
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

            let (inner_ret_ty, ccall_arg_types, julia_arg_types, invoke_fn) =
                async_callback_arg_type_fragments_in_env(self,  &resolver)?;

            let ccall_arg_idx = 0..n_args;
            let julia_arg_idx = 0..n_args;

            let ccall_ret_type: Expr = parse_quote! {
                <::jlrs::ccall::AsyncCCall as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
            };
            let julia_ret_type: Expr = parse_quote! {
                <#inner_ret_ty as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
            };

            let first_arg: Expr = syn::parse_quote! {
                <*mut ::std::ffi::c_void as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
            };

            let ex = parse_quote! {
                {
                    let unrooted = frame.unrooted();
                    let name = Symbol::new(&frame, #rename);
                    let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&unrooted).as_value();

                    #invoke_fn;

                    let func = Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                    unsafe {
                        let mut ccall_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                            &mut frame,
                            #n_args + 1,
                            type_type);

                        let mut ccall_arg_types_ref = ccall_arg_types.value_data_mut().unwrap();

                        let mut julia_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                            &mut frame,
                            #n_args,
                            type_type);

                        let mut julia_arg_types_ref = julia_arg_types.value_data_mut().unwrap();

                        ccall_arg_types_ref.set(0, Some(#first_arg)).unwrap();
                        #(
                            ccall_arg_types_ref.set(#ccall_arg_idx + 1, Some(#ccall_arg_types.as_value())).unwrap();
                            julia_arg_types_ref.set(#julia_arg_idx, Some(#julia_arg_types.as_value())).unwrap();
                        )*

                        let ccall_return_type = #ccall_ret_type;
                        let julia_return_type = #julia_ret_type;

                        let module = #override_module_fragment;
                        let nothing = ::jlrs::data::managed::value::Value::nothing(&frame);

                        let true_v = ::jlrs::data::managed::value::Value::true_v(&frame);
                        function_info_ty.instantiate_unchecked(&mut frame, [
                            name.as_value(),
                            ccall_arg_types.as_value(),
                            julia_arg_types.as_value(),
                            ccall_return_type,
                            julia_return_type,
                            func,
                            module.as_value(),
                            true_v
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
                    frame.scope(|mut frame| {
                        let instance = #expr;
                        let n = offset + #start + #idx;
                        accessor.set(n, Some(instance)).unwrap();

                        Ok(())
                    }).unwrap();
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

struct ExportedConst {
    _const_token: Token![const],
    name: Ident,
    _colon: Token![:],
    ty: Type,
    _as_token: Option<Token![as]>,
    name_override: Option<Ident>,
}

impl Parse for ExportedConst {
    fn parse(input: ParseStream) -> Result<Self> {
        let const_token = input.parse()?;
        let name = input.parse()?;
        let colon = input.parse()?;
        let ty = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = input.parse()?;

            Ok(ExportedConst {
                _const_token: const_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: Some(as_token),
                name_override: Some(name_override),
            })
        } else {
            Ok(ExportedConst {
                _const_token: const_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: None,
                name_override: None,
            })
        }
    }
}

struct ExportedGlobal {
    _static_token: Token![static],
    name: Ident,
    _colon: Token![:],
    ty: Type,
    _as_token: Option<Token![as]>,
    name_override: Option<Ident>,
}

impl Parse for ExportedGlobal {
    fn parse(input: ParseStream) -> Result<Self> {
        let static_token = input.parse()?;
        let name = input.parse()?;
        let colon = input.parse()?;
        let ty = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = input.parse()?;

            Ok(ExportedGlobal {
                _static_token: static_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: Some(as_token),
                name_override: Some(name_override),
            })
        } else {
            Ok(ExportedGlobal {
                _static_token: static_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: None,
                name_override: None,
            })
        }
    }
}

struct ItemWithAttrs {
    attrs: Vec<Attribute>,
    item: Box<ModuleItem>,
}

impl ItemWithAttrs {
    fn has_docstr(&self) -> bool {
        for attr in self.attrs.iter() {
            match attr.style {
                AttrStyle::Outer => (),
                _ => continue,
            }

            match &attr.meta {
                Meta::NameValue(kv) => {
                    if kv.path.is_ident("doc") {
                        return true;
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };
        }

        false
    }

    fn get_docstr(&self) -> Result<String> {
        let mut doc = String::new();
        for attr in self.attrs.iter() {
            match attr.style {
                AttrStyle::Outer => (),
                _ => continue,
            }

            let line = match &attr.meta {
                Meta::NameValue(kv) => {
                    if kv.path.is_ident("doc") {
                        match &kv.value {
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(s), ..
                            }) => s.value(),
                            _ => continue,
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };

            match doc.len() {
                0 => doc.push_str(&line),
                _ => {
                    doc.push('\n');
                    doc.push_str(&line);
                }
            }
        }

        Ok(doc)
    }
}

impl Parse for ItemWithAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let attr: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let item: ModuleItem = input.parse()?;
        Ok(ItemWithAttrs {
            attrs: attr,
            item: Box::new(item),
        })
    }
}

struct ExportedGenerics {
    _for: Token![for],
    type_param: Ident,
    _in: Token![in],
    _bracket: Bracket,
    types: Punctuated<Path, Token![,]>,
    _brace: Brace,
    items: Punctuated<ModuleItem, Token![;]>,
}

impl ExportedGenerics {
    fn to_generic_environment(&self) -> GenericEnvironment {
        GenericEnvironment::new(self)
    }
}

struct GenericEnvironment<'a> {
    parameter: &'a Ident,
    values: Vec<&'a Path>,
    items: Vec<&'a ModuleItem>,
    subenvs: Vec<GenericEnvironment<'a>>,
}

impl<'a> GenericEnvironment<'a> {
    fn new(generics: &'a ExportedGenerics) -> Self {
        let parameter = &generics.type_param;
        let values: Vec<_> = generics.types.iter().collect();

        let n_globals = generics
            .items
            .iter()
            .filter(|f| f.is_exported_const() || f.is_exported_global())
            .count();

        if n_globals != 0 {
            panic!("Globals and constants must be defined outside a `for` block.")
        }

        let items: Vec<_> = generics
            .items
            .iter()
            .filter(|f| {
                f.is_exported_fn()
                    || f.is_exported_method()
                    || f.is_exported_type()
                    || f.is_exported_async_callback()
            })
            .collect();

        let subenvs: Vec<_> = generics
            .items
            .iter()
            .filter(|f| f.is_exported_generics())
            .map(|f| f.get_exported_generics())
            .map(GenericEnvironment::new)
            .collect();

        GenericEnvironment {
            parameter,
            values,
            items,
            subenvs,
        }
    }

    fn init_type_fragments(&self) -> impl Iterator<Item = Expr> {
        let mut out = vec![];
        self.init_type_fragments_env(None, &mut out);
        out.into_iter()
    }

    fn init_type_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        out: &mut Vec<Expr>,
    ) {
        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            sub_env.init_type_fragments_env(Some(&env), out);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_type())
            .map(|it| it.get_exported_type())
            .map(|it| it.init_with_env(self, env));

        out.extend(exprs);
    }

    fn reinit_type_fragments(&self) -> impl Iterator<Item = Expr> {
        let mut out = vec![];
        self.reinit_type_fragments_env(None, &mut out);
        out.into_iter()
    }

    fn reinit_type_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        out: &mut Vec<Expr>,
    ) {
        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            sub_env.reinit_type_fragments_env(Some(&env), out);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_type())
            .map(|it| it.get_exported_type())
            .map(|it| it.reinit_with_env(self, env));

        out.extend(exprs);
    }

    fn init_function_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        offset: &mut usize,
    ) -> Result<Expr> {
        let mut sup_exprs = vec![];

        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            let ex = sub_env.init_function_fragments_env(Some(&env), offset)?;
            sup_exprs.push(ex);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_fn())
            .map(|it| it.get_exported_fn())
            .map(|it| {
                let mut gc_safe = false;
                if let Some(attrs) = it.1 {
                    gc_safe = has_outer_path_attr(attrs, "gc_safe");
                }
                it.0.init_with_env(self, env, offset, gc_safe)
            })
            .collect::<Result<Vec<_>>>()?;

        let ex = parse_quote! {
            {
                #(#sup_exprs;)*
                #(#exprs;)*
            }
        };

        Ok(ex)
    }

    fn init_method_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        offset: &mut usize,
    ) -> Result<Expr> {
        let mut sup_exprs = vec![];

        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            let ex = sub_env.init_method_fragments_env(Some(&env), offset)?;
            sup_exprs.push(ex);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_method())
            .map(|it| it.get_exported_method())
            .map(|it| {
                let mut untracked_self = false;
                let mut gc_safe = false;
                if let Some(attrs) = it.1 {
                    untracked_self = has_outer_path_attr(attrs, "untracked_self");
                    gc_safe = has_outer_path_attr(attrs, "gc_safe");
                }
                it.0.init_with_env(self, env, offset, untracked_self, gc_safe)
            }) // TODO: attrs
            .collect::<Result<Vec<_>>>()?;

        let ex = parse_quote! {
            {
                #(#sup_exprs;)*
                #(#exprs;)*
            }
        };

        Ok(ex)
    }

    fn init_async_callback_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        offset: &mut usize,
    ) -> Result<Expr> {
        let mut sup_exprs = vec![];

        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            let ex = sub_env.init_async_callback_fragments_env(Some(&env), offset)?;
            sup_exprs.push(ex);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_async_callback())
            .map(|it| it.get_exported_async_callback())
            .map(|it| it.init_with_env(self, env, offset))
            .collect::<Result<Vec<_>>>()?;

        let ex = parse_quote! {
            {
                #(#sup_exprs;)*
                #(#exprs;)*
            }
        };

        Ok(ex)
    }
}

impl Parse for ExportedGenerics {
    fn parse(input: ParseStream) -> Result<Self> {
        let for_token = input.parse()?;
        let type_param = input.parse()?;
        let in_token = input.parse()?;

        let content;
        let bracket = bracketed!(content in input);
        let types = content.parse_terminated(Path::parse, Token![,])?;

        let content;
        let brace = braced!(content in input);
        let items = content.parse_terminated(ModuleItem::parse, Token![;])?;

        Ok(ExportedGenerics {
            _for: for_token,
            type_param,
            _in: in_token,
            _bracket: bracket,
            types,
            _brace: brace,
            items,
        })
    }
}

enum ModuleItem {
    InitFn(InitFn),
    ExportedType(ExportedType),
    ExportedFunction(ExportedFunction),
    ExportedMethod(ExportedMethod),
    ExportedAsyncCallback(ExportedAsyncCallback),
    ExportedConst(ExportedConst),
    ExportedGlobal(ExportedGlobal),
    ItemWithAttrs(ItemWithAttrs),
    ExportedGenerics(ExportedGenerics),
}

impl ModuleItem {
    fn is_init_fn(&self) -> bool {
        match self {
            ModuleItem::InitFn(_) => true,
            _ => false,
        }
    }

    fn get_init_fn(&self) -> &InitFn {
        match self {
            ModuleItem::InitFn(ref init_fn) => init_fn,
            _ => panic!(),
        }
    }

    fn is_exported_fn(&self) -> bool {
        match self {
            ModuleItem::ExportedFunction(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_fn() => true,
            _ => false,
        }
    }

    fn get_exported_fn(&self) -> (&ExportedFunction, Option<&[Attribute]>) {
        match self {
            ModuleItem::ExportedFunction(ref exported_fn) => (exported_fn, None),
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, attrs }) if item.is_exported_fn() => {
                (item.get_exported_fn().0, Some(attrs))
            }
            _ => panic!(),
        }
    }

    fn is_exported_method(&self) -> bool {
        match self {
            ModuleItem::ExportedMethod(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_method() => {
                true
            }
            _ => false,
        }
    }

    fn get_exported_method(&self) -> (&ExportedMethod, Option<&[Attribute]>) {
        match self {
            ModuleItem::ExportedMethod(ref exported_method) => (exported_method, None),
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, ref attrs })
                if item.is_exported_method() =>
            {
                (item.get_exported_method().0, Some(attrs.as_ref()))
            }
            _ => panic!(),
        }
    }

    fn is_exported_async_callback(&self) -> bool {
        match self {
            ModuleItem::ExportedAsyncCallback(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. })
                if item.is_exported_async_callback() =>
            {
                true
            }
            _ => false,
        }
    }

    fn get_exported_async_callback(&self) -> &ExportedAsyncCallback {
        match self {
            ModuleItem::ExportedAsyncCallback(ref exported_async_callback) => {
                exported_async_callback
            }
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. })
                if item.is_exported_async_callback() =>
            {
                item.get_exported_async_callback()
            }
            _ => panic!(),
        }
    }

    fn is_exported_type(&self) -> bool {
        match self {
            ModuleItem::ExportedType(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_type() => {
                true
            }
            _ => false,
        }
    }

    fn get_exported_type(&self) -> &ExportedType {
        match self {
            ModuleItem::ExportedType(ref exported_type) => exported_type,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_type() => {
                item.get_exported_type()
            }
            _ => panic!(),
        }
    }

    fn is_exported_const(&self) -> bool {
        match self {
            ModuleItem::ExportedConst(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_const() => {
                true
            }
            _ => false,
        }
    }

    fn get_exported_const(&self) -> &ExportedConst {
        match self {
            ModuleItem::ExportedConst(ref exported_const) => exported_const,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_const() => {
                item.get_exported_const()
            }
            _ => panic!(),
        }
    }

    fn is_exported_global(&self) -> bool {
        match self {
            ModuleItem::ExportedGlobal(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_global() => {
                true
            }
            _ => false,
        }
    }

    fn get_exported_global(&self) -> &ExportedGlobal {
        match self {
            ModuleItem::ExportedGlobal(ref exported_global) => exported_global,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_global() => {
                item.get_exported_global()
            }
            _ => panic!(),
        }
    }

    fn is_exported_generics(&self) -> bool {
        match self {
            ModuleItem::ExportedGenerics(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. })
                if item.is_exported_generics() =>
            {
                true
            }
            _ => false,
        }
    }

    fn get_exported_generics(&self) -> &ExportedGenerics {
        match self {
            ModuleItem::ExportedGenerics(ref exported_generics) => exported_generics,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. })
                if item.is_exported_generics() =>
            {
                item.get_exported_generics()
            }
            _ => panic!(),
        }
    }

    fn get_all_with_docs(&self) -> Vec<&ItemWithAttrs> {
        let mut items = vec![];
        match self {
            ModuleItem::ExportedGenerics(ref exported_generics) => {
                for item in exported_generics.items.iter() {
                    item.get_all_with_docs_inner(&mut items);
                }
            }
            ModuleItem::ItemWithAttrs(item) => {
                if item.has_docstr() {
                    items.push(item)
                }
            }
            _ => (),
        }

        items
    }

    fn get_all_with_docs_inner<'a>(&'a self, items: &mut Vec<&'a ItemWithAttrs>) {
        match self {
            ModuleItem::ExportedGenerics(ref exported_generics) => {
                for item in exported_generics.items.iter() {
                    item.get_all_with_docs_inner(items);
                }
            }
            ModuleItem::ItemWithAttrs(item) => {
                if item.has_docstr() {
                    items.push(item)
                }
            }
            _ => (),
        }
    }
}

impl Parse for ModuleItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![become]) {
            input.parse().map(ModuleItem::InitFn)
        } else if lookahead.peek(Token![struct]) {
            input.parse().map(ModuleItem::ExportedType)
        } else if lookahead.peek(Token![fn]) {
            input.parse().map(ModuleItem::ExportedFunction)
        } else if lookahead.peek(Token![in]) {
            input.parse().map(ModuleItem::ExportedMethod)
        } else if lookahead.peek(Token![async]) {
            input.parse().map(ModuleItem::ExportedAsyncCallback)
        } else if lookahead.peek(Token![const]) {
            input.parse().map(ModuleItem::ExportedConst)
        } else if lookahead.peek(Token![static]) {
            input.parse().map(ModuleItem::ExportedGlobal)
        } else if lookahead.peek(Token![#]) {
            input.parse().map(ModuleItem::ItemWithAttrs)
        } else if lookahead.peek(Token![for]) {
            input.parse().map(ModuleItem::ExportedGenerics)
        } else {
            Err(Error::new(
                input.span(),
                "Expected `become`, `fn`, `in`, `struct`, `const`, or `static`.",
            ))
        }
    }
}

pub(crate) struct JuliaModule {
    items: Punctuated<ModuleItem, Token![;]>,
}

impl Parse for JuliaModule {
    fn parse(input: ParseStream) -> Result<Self> {
        let content = input;
        let items = content.parse_terminated(ModuleItem::parse, Token![;])?;

        Ok(JuliaModule { items: items })
    }
}

impl JuliaModule {
    pub(crate) fn generate_init_code(self) -> Result<TokenStream> {
        let init_fn = self.get_init_fn()?;
        let init_fn_ident = &init_fn.init_fn;

        let fn_fragments = FunctionFragments::generate(&self, init_fn)?;
        let generic_fn_fragments = FunctionFragments::generate_generic(&self, init_fn)?;
        let method_fragments = MethodFragments::generate(&self, init_fn);
        let async_callback_fragments = AsyncCallbackFragments::generate(&self, init_fn)?;
        let generic_async_callback_fragments =
            AsyncCallbackFragments::generate_generic(&self, init_fn)?;
        let generic_method_fragments = MethodFragments::generate_generic(&self, init_fn)?;
        let type_fragments = TypeFragments::generate(&self, init_fn);
        let generic_type_fragments = TypeFragments::generate_generic(&self, init_fn);
        let const_fragments = ConstFragments::generate(&self, init_fn);
        let global_fragments = GlobalFragments::generate(&self, init_fn);
        let doc_fragments = DocFragments::generate(&self, init_fn)?;

        let type_init_fn = type_fragments.type_init_fn;
        let type_init_fn_ident = type_fragments.type_init_ident;
        let type_reinit_fn = type_fragments.type_reinit_fn;
        let type_reinit_fn_ident = type_fragments.type_reinit_ident;
        let generic_type_init_fn = generic_type_fragments.type_init_fn;
        let generic_type_init_fn_ident = generic_type_fragments.type_init_ident;
        let generic_type_reinit_fn = generic_type_fragments.type_reinit_fn;
        let generic_type_reinit_fn_ident = generic_type_fragments.type_reinit_ident;
        let function_init_fn = fn_fragments.init_functions_fn;
        let function_init_fn_ident = fn_fragments.init_functions_fn_ident;
        let generic_function_init_fn = generic_fn_fragments.init_functions_fn;
        let generic_function_init_fn_ident = generic_fn_fragments.init_functions_fn_ident;
        let method_init_fn = method_fragments.init_methods_fn;
        let method_init_fn_ident = method_fragments.init_methods_fn_ident;
        let generic_method_init_fn = generic_method_fragments.init_methods_fn;
        let generic_method_init_fn_ident = generic_method_fragments.init_methods_fn_ident;
        let async_callback_init_fn = async_callback_fragments.init_async_callbacks_fn;
        let async_callback_init_fn_ident = async_callback_fragments.init_async_callbacks_fn_ident;
        let generic_async_callback_init_fn =
            generic_async_callback_fragments.init_async_callbacks_fn;
        let generic_async_callback_init_fn_ident =
            generic_async_callback_fragments.init_async_callbacks_fn_ident;
        let const_init_fn = const_fragments.const_init_fn;
        let const_init_fn_ident = const_fragments.const_init_ident;
        let global_init_fn = global_fragments.global_init_fn;
        let global_init_fn_ident = global_fragments.global_init_ident;
        let doc_init_fn = doc_fragments.init_docs_fn;
        let doc_init_fn_ident = doc_fragments.init_docs_fn_ident;

        let invoke_type_init: Expr = parse_quote! {
            if precompiling == 1 {
                #type_init_fn_ident(&mut frame, module);
            } else {
                #type_reinit_fn_ident(&mut frame, module);
            }
        };

        let invoke_generic_type_init: Expr = parse_quote! {
            if precompiling == 1 {
                #generic_type_init_fn_ident(&mut frame, module);
            } else {
                #generic_type_reinit_fn_ident(&mut frame, module);
            }
        };

        let invoke_const_init: Expr = parse_quote! {
            if precompiling == 1 {
                #const_init_fn_ident(&mut frame, module);
            }
        };

        let invoke_global_init: Expr = parse_quote! {
            if precompiling == 1 {
                #global_init_fn_ident(&mut frame, module);
            }
        };

        let generated = quote::quote! {

            #[no_mangle]
            pub unsafe extern "C" fn #init_fn_ident(
                module: ::jlrs::data::managed::module::Module,
                precompiling: u8,
            ) -> ::jlrs::data::managed::value::ValueRet {
                #type_init_fn

                #type_reinit_fn

                #generic_type_init_fn

                #generic_type_reinit_fn

                #function_init_fn

                #generic_function_init_fn

                #method_init_fn

                #generic_method_init_fn

                #async_callback_init_fn

                #generic_async_callback_init_fn

                #const_init_fn

                #global_init_fn

                #doc_init_fn

                static IS_INIT: ::std::sync::atomic::AtomicBool = ::std::sync::atomic::AtomicBool::new(false);
                if IS_INIT.compare_exchange(false, true, ::std::sync::atomic::Ordering::Relaxed, ::std::sync::atomic::Ordering::Relaxed).is_err() {
                    let unrooted = <::jlrs::data::managed::module::Module as ::jlrs::data::managed::Managed>::unrooted_target(module);
                    return ::jlrs::data::managed::value::Value::nothing(&unrooted).as_ref().leak();
                }

                let mut stack_frame = ::jlrs::memory::stack_frame::StackFrame::new();
                let mut ccall = ::jlrs::ccall::CCall::new(&mut stack_frame);

                ccall.init_jlrs(&::jlrs::InstallJlrsCore::Default, Some(module));

                ccall.scope(|mut frame| {
                    let wrap_mod = ::jlrs::data::managed::module::Module::main(&frame)
                        .submodule(&frame, "JlrsCore")
                        .unwrap()
                        .as_managed()
                        .submodule(&frame, "Wrap")
                        .unwrap()
                        .as_managed();

                    let function_info_ty = wrap_mod
                        .global(&frame, "JlrsFunctionInfo")
                        .unwrap()
                        .as_value()
                        .cast_unchecked::<::jlrs::data::managed::datatype::DataType>();

                    let doc_item_ty = wrap_mod
                        .global(&frame, "DocItem")
                        .unwrap()
                        .as_value()
                        .cast_unchecked::<::jlrs::data::managed::datatype::DataType>();

                    let module_info_ty = wrap_mod
                        .global(&frame, "JlrsModuleInfo")
                        .unwrap()
                        .as_value()
                        .cast_unchecked::<::jlrs::data::managed::datatype::DataType>();

                    #invoke_type_init;
                    #invoke_generic_type_init;
                    #invoke_const_init;
                    #invoke_global_init;

                    let mut arr = ::jlrs::data::managed::array::Array::new_for_unchecked(&mut frame, 0, function_info_ty.as_value());
                    #function_init_fn_ident(&mut frame, &mut arr, module, function_info_ty);
                    #generic_function_init_fn_ident(&mut frame, &mut arr, module, function_info_ty);
                    #method_init_fn_ident(&mut frame, &mut arr, module, function_info_ty);
                    #generic_method_init_fn_ident(&mut frame, &mut arr, module, function_info_ty);
                    #async_callback_init_fn_ident(&mut frame, &mut arr, module, function_info_ty);
                    #generic_async_callback_init_fn_ident(&mut frame, &mut arr, module, function_info_ty);

                    let mut doc_items = ::jlrs::data::managed::array::Array::new_for_unchecked(&mut frame, 0, doc_item_ty.as_value());
                    if precompiling == 1 {
                        #doc_init_fn_ident(&mut frame, &mut doc_items, module, doc_item_ty);
                    }
                    Ok(module_info_ty.instantiate_unchecked(&frame, [arr.as_value(), doc_items.as_value()]).leak())
                }).unwrap()
            }
        };

        Ok(generated.into())
    }

    fn get_init_fn(&self) -> Result<&InitFn> {
        let n_init_fns = self.items.iter().filter(|it| it.is_init_fn()).count();
        if n_init_fns != 1 {
            let msg = format!("Expected 1 init fn, found {}", n_init_fns);
            Err(Error::new(Span::call_site(), msg))?;
        }

        let init_fn = self
            .items
            .iter()
            .find(|it| it.is_init_fn())
            .unwrap()
            .get_init_fn();

        Ok(init_fn)
    }

    fn get_exported_functions(
        &self,
    ) -> impl Iterator<Item = (&ExportedFunction, Option<&[Attribute]>)> {
        self.items
            .iter()
            .filter(|it| it.is_exported_fn())
            .map(|it| it.get_exported_fn())
    }

    fn get_exported_methods(
        &self,
    ) -> impl Iterator<Item = (&ExportedMethod, Option<&[Attribute]>)> {
        self.items
            .iter()
            .filter(|it| it.is_exported_method())
            .map(|it| it.get_exported_method())
    }

    fn get_exported_async_callbacks(&self) -> impl Iterator<Item = &ExportedAsyncCallback> {
        self.items
            .iter()
            .filter(|it| it.is_exported_async_callback())
            .map(|it| it.get_exported_async_callback())
    }

    fn get_exported_types(&self) -> impl Iterator<Item = &ExportedType> {
        self.items
            .iter()
            .filter(|it| it.is_exported_type())
            .map(|it| it.get_exported_type())
    }

    fn get_exported_consts(&self) -> impl Iterator<Item = &ExportedConst> {
        self.items
            .iter()
            .filter(|it| it.is_exported_const())
            .map(|it| it.get_exported_const())
    }

    fn get_exported_globals(&self) -> impl Iterator<Item = &ExportedGlobal> {
        self.items
            .iter()
            .filter(|it| it.is_exported_global())
            .map(|it| it.get_exported_global())
    }

    fn get_exported_generics(&self) -> impl Iterator<Item = &ExportedGenerics> {
        self.items
            .iter()
            .filter(|it| it.is_exported_generics())
            .map(|it| it.get_exported_generics())
    }

    fn get_items_with_docs(&self) -> impl Iterator<Item = &ItemWithAttrs> {
        self.items
            .iter()
            .map(|it| it.get_all_with_docs())
            .concat()
            .into_iter()
    }
}

struct DocFragments {
    init_docs_fn_ident: Ident,
    init_docs_fn: ItemFn,
}

impl DocFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
        let init_docs_fn_ident = format_ident!("{}_docs", init_fn.init_fn);
        let n_docs = module.get_items_with_docs().count();

        let doc_init_fragments = module
            .get_items_with_docs()
            .enumerate()
            .map(doc_info_fragment);

        let mut fragments = Vec::with_capacity(n_docs);
        for fragment in doc_init_fragments {
            fragments.push(fragment?);
        }

        let init_docs_fn = parse_quote! {
            unsafe fn #init_docs_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                doc_item_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(move |mut frame| {
                    array.grow_end_unchecked(#n_docs);
                    let mut accessor = array.indeterminate_data_mut();

                    #(
                        #fragments
                    )*

                    Ok(())
                }).unwrap()
            }
        };

        Ok(DocFragments {
            init_docs_fn_ident,
            init_docs_fn,
        })
    }
}

struct FunctionFragments {
    init_functions_fn_ident: Ident,
    init_functions_fn: ItemFn,
}

impl FunctionFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
        let init_functions_fn_ident = format_ident!("{}_functions", init_fn.init_fn);
        let n_functions = module.get_exported_functions().count();

        let fragments = module
            .get_exported_functions()
            .enumerate()
            .map(function_info_fragment)
            .collect::<Result<Vec<_>>>()?;

        let init_functions_fn = parse_quote! {
            unsafe fn #init_functions_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(move |mut frame| {
                    array.grow_end_unchecked(#n_functions);
                    let mut accessor = array.value_data_mut().unwrap();
                    #(#fragments)*

                    Ok(())
                }).unwrap()
            }
        };

        Ok(FunctionFragments {
            init_functions_fn_ident,
            init_functions_fn,
        })
    }

    fn generate_generic(info: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
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
            unsafe fn #init_functions_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(move |mut frame| {
                    let mut accessor = array.value_data_mut().unwrap();
                    let offset = ::jlrs::data::managed::array::dimensions::Dims::size(&accessor.dimensions());
                    #(#init_functions_fragments)*
                    Ok(())
                }).unwrap()
            }
        };

        let fragments = FunctionFragments {
            init_functions_fn_ident,
            init_functions_fn,
        };

        Ok(fragments)
    }
}

struct MethodFragments {
    init_methods_fn_ident: Ident,
    init_methods_fn: ItemFn,
}

impl MethodFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_methods_fn_ident = format_ident!("{}_methods", init_fn.init_fn);
        let n_methods = module.get_exported_methods().count();

        let method_init_fragments = module
            .get_exported_methods()
            .enumerate()
            .map(method_info_fragment);

        let init_methods_fn = parse_quote! {
            unsafe fn #init_methods_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(move |mut frame| {
                    array.grow_end_unchecked(#n_methods);
                    let mut accessor = array.value_data_mut().unwrap();
                    let offset = ::jlrs::data::managed::array::dimensions::Dims::size(&accessor.dimensions()) - #n_methods;

                    #(
                        #method_init_fragments
                    )*

                    Ok(())
                }).unwrap()
            }
        };

        MethodFragments {
            init_methods_fn_ident,
            init_methods_fn,
        }
    }

    fn generate_generic(info: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
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
            unsafe fn #init_methods_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(move |mut frame| {
                    let mut accessor = array.value_data_mut().unwrap();
                    let offset = ::jlrs::data::managed::array::dimensions::Dims::size(&accessor.dimensions());
                    #(#init_methods_fragments)*
                    Ok(())
                }).unwrap()
            }
        };

        let fragments = MethodFragments {
            init_methods_fn_ident,
            init_methods_fn,
        };

        Ok(fragments)
    }
}

struct AsyncCallbackFragments {
    init_async_callbacks_fn_ident: Ident,
    init_async_callbacks_fn: ItemFn,
}

impl AsyncCallbackFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
        let init_async_callbacks_fn_ident = format_ident!("{}_async_callbacks", init_fn.init_fn);
        let n_async_callbacks = module.get_exported_async_callbacks().count();

        let async_callback_init_fragments = module
            .get_exported_async_callbacks()
            .enumerate()
            .map(async_callback_info_fragment);

        let mut fragments = Vec::with_capacity(n_async_callbacks);
        for fragment in async_callback_init_fragments {
            fragments.push(fragment?);
        }

        let init_async_callbacks_fn = parse_quote! {
            unsafe fn #init_async_callbacks_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(move |mut frame| {
                    array.grow_end_unchecked(#n_async_callbacks);
                    let mut accessor = array.value_data_mut().unwrap();
                    let offset = ::jlrs::data::managed::array::dimensions::Dims::size(&accessor.dimensions()) - #n_async_callbacks;

                    #(
                        #fragments
                    )*

                    Ok(())
                }).unwrap()
            }
        };

        Ok(AsyncCallbackFragments {
            init_async_callbacks_fn_ident,
            init_async_callbacks_fn,
        })
    }

    fn generate_generic(info: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
        let mut offset = 0;

        let init_async_callbacks_fn_ident =
            format_ident!("{}_generic_async_callbacks", init_fn.init_fn);
        let init_async_callback_fragments = info
            .get_exported_generics()
            .map(|g| {
                g.to_generic_environment()
                    .init_async_callback_fragments_env(None, &mut offset)
            })
            .collect::<Result<Vec<_>>>()?;

        let init_async_callbacks_fn = parse_quote! {
            unsafe fn #init_async_callbacks_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(move |mut frame| {
                    let mut accessor = array.value_data_mut().unwrap();
                    let offset = ::jlrs::data::managed::array::dimensions::Dims::size(&accessor.dimensions());
                    #(#init_async_callback_fragments)*
                    Ok(())
                }).unwrap()
            }
        };

        let fragments = AsyncCallbackFragments {
            init_async_callbacks_fn_ident,
            init_async_callbacks_fn,
        };

        Ok(fragments)
    }
}

struct TypeFragments {
    type_init_fn: ItemFn,
    type_init_ident: Ident,
    type_reinit_fn: ItemFn,
    type_reinit_ident: Ident,
}

impl TypeFragments {
    fn generate(info: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_types_fn_ident = format_ident!("{}_types", init_fn.init_fn);
        let init_types_fragments = info.get_exported_types().map(init_type_fragment);

        let type_init_fn = parse_quote! {
            unsafe fn #init_types_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: ::jlrs::data::managed::module::Module,
            ) {
                frame.scope(|mut frame| {
                    let mut output = frame.output();

                    #(
                        #init_types_fragments
                    )*

                    Ok(())
                }).unwrap();
            }
        };

        let reinit_types_fn_ident = format_ident!("{}_reinittypes", init_fn.init_fn);
        let reinit_types_fragments = info.get_exported_types().map(reinit_type_fragment);

        let type_reinit_fn = parse_quote! {
            unsafe fn #reinit_types_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: jlrs::data::managed::module::Module
            ) {
                frame.scope(|frame| {
                    let mut output = frame.output();

                    #(
                        #reinit_types_fragments
                    )*

                    Ok(())
                }).unwrap();
            }
        };

        TypeFragments {
            type_init_fn,
            type_init_ident: init_types_fn_ident,
            type_reinit_fn,
            type_reinit_ident: reinit_types_fn_ident,
        }
    }

    fn generate_generic(info: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_types_fn_ident = format_ident!("{}_generic_types", init_fn.init_fn);
        let init_types_fragments = info
            .get_exported_generics()
            .map(|g| g.to_generic_environment().init_type_fragments())
            .flatten();

        let type_init_fn = parse_quote! {
            unsafe fn #init_types_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: ::jlrs::data::managed::module::Module,
            ) {
                frame.scope(|mut frame| {
                    let mut output = frame.output();

                    #(
                        #init_types_fragments
                    )*

                    Ok(())
                }).unwrap();
            }
        };

        let reinit_types_fn_ident = format_ident!("{}_reinit_generic_types", init_fn.init_fn);
        let reinit_types_fragments = info
            .get_exported_generics()
            .map(|g| g.to_generic_environment().reinit_type_fragments())
            .flatten();

        let type_reinit_fn = parse_quote! {
            unsafe fn #reinit_types_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: jlrs::data::managed::module::Module
            ) {
                frame.scope(|mut frame| {
                    let mut output = frame.output();

                    #(
                        #reinit_types_fragments
                    )*

                    Ok(())
                }).unwrap();
            }
        };

        TypeFragments {
            type_init_fn,
            type_init_ident: init_types_fn_ident,
            type_reinit_fn,
            type_reinit_ident: reinit_types_fn_ident,
        }
    }
}

struct ConstFragments {
    const_init_fn: ItemFn,
    const_init_ident: Ident,
}

impl ConstFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let const_init_ident = format_ident!("{}_consts", init_fn.init_fn);

        let const_init_fragments = module.get_exported_consts().map(const_info_fragment);

        let const_init_fn = parse_quote! {
            unsafe fn #const_init_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: ::jlrs::data::managed::module::Module,
            ) {
                #(
                    #const_init_fragments
                )*
            }
        };

        ConstFragments {
            const_init_ident,
            const_init_fn,
        }
    }
}

struct GlobalFragments {
    global_init_fn: ItemFn,
    global_init_ident: Ident,
}

impl GlobalFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let global_init_ident = format_ident!("{}_globals", init_fn.init_fn);

        let global_init_fragments = module.get_exported_globals().map(global_info_fragment);

        let global_init_fn = parse_quote! {
            unsafe fn #global_init_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: ::jlrs::data::managed::module::Module,
            ) {
                #(
                    #global_init_fragments
                )*
            }
        };

        GlobalFragments {
            global_init_ident,
            global_init_fn,
        }
    }
}

fn doc_info_fragment((index, info): (usize, &ItemWithAttrs)) -> Result<Expr> {
    match info.item.as_ref() {
        ModuleItem::InitFn(i) => Err(syn::Error::new_spanned(
            i.init_fn.to_token_stream(),
            "init function cannot be documented",
        ))?,
        ModuleItem::ExportedType(ty) => {
            let override_module_fragment = override_module_fragment(&ty.name_override);
            let name_ident = &ty.name.segments.last().unwrap().ident;

            let rename = ty
                .name_override
                .as_ref()
                .map(|parts| parts.last())
                .flatten()
                .unwrap_or(name_ident)
                .to_string();

            let doc = info.get_docstr()?;

            let q = parse_quote! {
                {
                    frame.scope(|mut frame| {
                        unsafe {
                            let module = #override_module_fragment;
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value_unchecked(#index, Some(doc_it)).unwrap();
                        }

                        Ok(())
                    }).unwrap();
                }
            };

            Ok(q)
        }
        ModuleItem::ExportedFunction(func) => {
            let name_ident = &func.func.ident;

            let override_module_fragment = override_module_fragment(&func.name_override);
            let mut rename = func
                .name_override
                .as_ref()
                .map(|parts| parts.last())
                .flatten()
                .unwrap_or(name_ident)
                .to_string();

            if func.exclamation_mark_token.is_some() {
                rename.push('!')
            }

            let doc = info.get_docstr()?;

            let q = parse_quote! {
                {
                    frame.scope(|mut frame| {
                        unsafe {
                            let module = #override_module_fragment;
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value_unchecked(#index, Some(doc_it)).unwrap();
                        }

                        Ok(())
                    }).unwrap();
                }

            };

            Ok(q)
        }
        ModuleItem::ExportedMethod(func) => {
            let name_ident = &func.func.ident;

            let override_module_fragment = override_module_fragment(&func.name_override);
            let mut rename = func
                .name_override
                .as_ref()
                .map(|parts| parts.last())
                .flatten()
                .unwrap_or(name_ident)
                .to_string();

            if func.exclamation_mark_token.is_some() {
                rename.push('!')
            }

            let doc = info.get_docstr()?;

            let q = parse_quote! {
                {
                    frame.scope(|mut frame| {
                        unsafe {
                            let module = #override_module_fragment;
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value_unchecked(#index, Some(doc_it)).unwrap();
                        }

                        Ok(())
                    }).unwrap();
                }

            };

            Ok(q)
        }
        ModuleItem::ExportedAsyncCallback(func) => {
            let name_ident = &func.func.ident;

            let override_module_fragment = override_module_fragment(&func.name_override);
            let mut rename = func
                .name_override
                .as_ref()
                .map(|parts| parts.last())
                .flatten()
                .unwrap_or(name_ident)
                .to_string();

            if func.exclamation_mark_token.is_some() {
                rename.push('!')
            }

            let doc = info.get_docstr()?;

            let q = parse_quote! {
                {
                    frame.scope(|mut frame| {
                        unsafe {
                            let module = #override_module_fragment;
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value_unchecked(#index, Some(doc_it)).unwrap();
                        }

                        Ok(())
                    }).unwrap();
                }

            };

            Ok(q)
        }
        ModuleItem::ExportedConst(val) => {
            let name_ident = &val.name;
            let rename = val.name_override.as_ref().unwrap_or(name_ident).to_string();
            let doc = info.get_docstr()?;

            let q = parse_quote! {
                {
                    frame.scope(|mut frame| {
                        unsafe {
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value_unchecked(#index, Some(doc_it)).unwrap();
                        }

                        Ok(())
                    }).unwrap();
                }

            };

            Ok(q)
        }
        ModuleItem::ExportedGlobal(val) => {
            let name_ident = &val.name;
            let rename = val.name_override.as_ref().unwrap_or(name_ident).to_string();
            let doc = info.get_docstr()?;

            let q = parse_quote! {
                {
                    frame.scope(|mut frame| {
                        unsafe {
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value_unchecked(#index, Some(doc_it)).unwrap();
                        }

                        Ok(())
                    }).unwrap();
                }

            };

            Ok(q)
        }
        ModuleItem::ItemWithAttrs(_) => unreachable!(),
        ModuleItem::ExportedGenerics(_) => unreachable!(),
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

    let expr = parse_quote! {
        {
            frame.scope(|mut frame| {
                let name = Symbol::new(&frame, #rename);
                let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&frame).as_value();
                // Ensure a compile error happens if the signatures of the function don't match.

                #invoke_fn

                let func = Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                unsafe {
                    let mut ccall_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        &mut frame,
                        #n_args,
                        type_type);

                    let mut ccall_arg_types_ref = ccall_arg_types.value_data_mut().unwrap();

                    let mut julia_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        &mut frame,
                        #n_args,
                        type_type);

                    let mut julia_arg_types_ref = julia_arg_types.value_data_mut().unwrap();

                    #(
                        ccall_arg_types_ref.set(#ccall_arg_idx, Some(#ccall_arg_types.as_value())).unwrap();
                        julia_arg_types_ref.set(#julia_arg_idx, Some(#julia_arg_types.as_value())).unwrap();
                    )*

                    let ccall_return_type = #ccall_ret_type;
                    let julia_return_type = #julia_ret_type;

                    let module = #override_module_fragment;
                    let nothing = ::jlrs::data::managed::value::Value::nothing(&frame);

                    let false_v = ::jlrs::data::managed::value::Value::false_v(&frame);
                    let instance = function_info_ty.instantiate_unchecked(&mut frame, [
                        name.as_value(),
                        ccall_arg_types.as_value(),
                        julia_arg_types.as_value(),
                        ccall_return_type.as_value(),
                        julia_return_type.as_value(),
                        func,
                        module.as_value(),
                        false_v
                    ]);

                    let n = #index;
                    accessor.set(n, Some(instance)).unwrap();
                }

                Ok(())
            }).unwrap();
        }
    };

    Ok(expr)
}

fn override_module_fragment(name_override: &Option<RenameFragments>) -> Expr {
    let name_override = name_override.as_ref();
    if name_override.is_none() {
        return parse_quote! { { module } };
    }
    let name_override = name_override.unwrap();
    let n_parts = name_override.len();
    if n_parts == 1 {
        return parse_quote! { { module } };
    }

    let modules = name_override
        .iter()
        .take(n_parts - 1)
        .map(|ident| ident.to_string());

    let parsed = parse_quote! {
        {
            let mut module = Module::main(&frame);

            #(
                module = module
                .submodule(&frame, #modules)
                .expect("Submodule does not exist")
                .as_managed();
            )*

            module
        }
    };

    parsed
}

fn return_type_fragments(ret_ty: &ReturnType) -> (Expr, Expr) {
    match ret_ty {
        ReturnType::Default => {
            let ccall_ret_type: Expr = parse_quote! {
                ::jlrs::data::managed::datatype::DataType::nothing_type(&frame).as_value()
            };

            let julia_ret_type = ccall_ret_type.clone();
            (ccall_ret_type, julia_ret_type)
        }
        ReturnType::Type(_, ref ty) => {
            let span = ty.span();
            let ccall_ret_type = parse_quote_spanned! {
                span=> <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::CCallReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
            };
            let julia_ret_type = parse_quote_spanned! {
                span=> <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::FunctionReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
            };

            (ccall_ret_type, julia_ret_type)
        }
    }
}

// FIXME
fn extract_return_type_from_impl_trait(timplt: &TypeImplTrait) -> Option<Type> {
    for bound in timplt.bounds.iter() {
        match bound {
            TypeParamBound::Trait(t) => {
                let segment = t.path.segments.last();
                if segment.is_none() {
                    continue;
                }

                match &segment.unwrap().arguments {
                    PathArguments::AngleBracketed(p) => match p.args.last() {
                        Some(GenericArgument::Type(ref t)) => return Some(t.clone()),
                        _ => continue,
                    },
                    _ => continue,
                }
            }
            _ => continue,
        }
    }

    None
}

fn extract_inner_return_type_from_path(path: &Path) -> Option<Type> {
    if let Some(segment) = path.segments.last() {
        match segment.arguments {
            PathArguments::AngleBracketed(ref args) => {
                if let Some(arg) = args.args.first() {
                    match arg {
                        GenericArgument::Type(t) => match t {
                            &Type::ImplTrait(ref timplt) => {
                                return extract_return_type_from_impl_trait(timplt)
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    None
}

fn extract_inner_return_type(ty: &Type) -> Option<Type> {
    match ty {
        Type::Path(ref p) => extract_inner_return_type_from_path(&p.path),
        _ => None,
    }
}

fn async_callback_return_type_fragments(ret_ty: &ReturnType) -> Option<Type> {
    match ret_ty {
        ReturnType::Type(_, ty) => extract_inner_return_type(&*ty),
        _ => None,
    }
}

fn arg_type_fragments<'a>(
    inputs: &'a Punctuated<FnArg, Comma>,
) -> Result<(
    impl 'a + Iterator<Item = Expr>,
    impl 'a + Iterator<Item = Expr>,
)> {
    let n_args = inputs.len();

    if n_args > 0 {
        if let FnArg::Receiver(r) = inputs.first().unwrap() {
            Err(syn::Error::new_spanned(
                r.to_token_stream(),
                "exported function must be a free-standing function, use `in <struct name> fn ...` to export methods",
            ))?;
        }
    }

    let ccall_arg_types = inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(ty) => &ty.ty,
            _ => unreachable!(),
        })
        .map(|ty| {
            parse_quote! {
                <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
            }
        });

    let julia_arg_types = inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(ty) => &ty.ty,
            _ => unreachable!(),
        })
        .map(|ty| {
            parse_quote! {
                <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
            }
        });

    Ok((ccall_arg_types, julia_arg_types))
}

fn init_type_fragment(info: &ExportedType) -> Expr {
    let override_module_fragment = override_module_fragment(&info.name_override);
    let name_ident = &info.name.segments.last().unwrap().ident;

    let rename = info
        .name_override
        .as_ref()
        .map(|parts| parts.last())
        .flatten()
        .unwrap_or(name_ident)
        .to_string();

    let ty = format_ident!("{}", name_ident);

    parse_quote! {
        {
            let sym = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
            let module = #override_module_fragment;
            let ty = <#ty as ::jlrs::data::types::foreign_type::OpaqueType>::create_type(&mut output, sym, module);
            module.set_const_unchecked(sym, <::jlrs::data::managed::datatype::DataType as ::jlrs::data::managed::Managed>::as_value(ty));
        }
    }
}

fn reinit_type_fragment(info: &ExportedType) -> Expr {
    {
        let override_module_fragment = override_module_fragment(&info.name_override);
        let name_ident = &info.name.segments.last().unwrap().ident;

        let rename = info
            .name_override
            .as_ref()
            .map(|parts| parts.last())
            .flatten()
            .unwrap_or(name_ident)
            .to_string();

        let ty = format_ident!("{}", name_ident);

        parse_quote! {
            {
                let module = #override_module_fragment;

                let dt = module
                    .global(&frame, #rename)
                    .unwrap()
                    .as_value()
                    .cast::<::jlrs::data::managed::datatype::DataType>()
                    .unwrap();

                <#ty as ::jlrs::data::types::foreign_type::OpaqueType>::reinit_type(dt);
            }
        }
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
    let julia_arg_idx = 0..n_args;

    let (ccall_arg_types, julia_arg_types, invoke_fn) =
        method_arg_type_fragments(info, untracked_self, gc_safe);

    parse_quote! {
        {
            frame.scope(|mut frame| {
                let unrooted = frame.unrooted();
                let name = Symbol::new(&frame, #rename);
                let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&unrooted).as_value();

                #invoke_fn;

                let func = Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                unsafe {
                    let mut ccall_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        &mut frame,
                        #n_args,
                        type_type);

                    let mut ccall_arg_types_ref = ccall_arg_types.value_data_mut().unwrap();

                    let mut julia_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        &mut frame,
                        #n_args,
                        type_type);

                    let mut julia_arg_types_ref = julia_arg_types.value_data_mut().unwrap();

                    #(
                        ccall_arg_types_ref.set(#ccall_arg_idx, Some(#ccall_arg_types.as_value())).unwrap();
                        julia_arg_types_ref.set(#julia_arg_idx, Some(#julia_arg_types.as_value())).unwrap();
                    )*

                    let ccall_return_type = #ccall_ret_type;
                    let julia_return_type = #julia_ret_type;

                    let module = #override_module_fragment;
                    let nothing = ::jlrs::data::managed::value::Value::nothing(&frame);

                    let false_v = ::jlrs::data::managed::value::Value::false_v(&frame);
                    let instance = function_info_ty.instantiate_unchecked(&mut frame, [
                        name.as_value(),
                        ccall_arg_types.as_value(),
                        julia_arg_types.as_value(),
                        ccall_return_type,
                        julia_return_type,
                        func,
                        module.as_value(),
                        false_v
                    ]);

                    let n = #index + offset;
                    accessor.set(n, Some(instance)).unwrap();
                }

                Ok(())
            }).unwrap();
        }
    }
}

fn async_callback_info_fragment((index, info): (usize, &ExportedAsyncCallback)) -> Result<Expr> {
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
    let inner_ret_ty = async_callback_return_type_fragments(ret_ty);
    if inner_ret_ty.is_none() {
        return Ok(parse_quote_spanned! {
            name_ident.span() => {
                compile_error!("Async callback must return JlrsResult<impl AsyncCallback<T>> where T is some type that implements IntoJulia");
            }
        });
    }

    let inner_ret_ty = inner_ret_ty.unwrap();
    let ccall_ret_type: Expr = parse_quote! {
        <::jlrs::ccall::AsyncCCall as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
    };
    let julia_ret_type: Expr = parse_quote! {
        <#inner_ret_ty as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
    };

    let (ccall_arg_types, julia_arg_types, invoke_fn) =
        async_callback_arg_type_fragments(info, &inner_ret_ty)?;

    let first_arg: Expr = syn::parse_quote! {
        <*mut ::std::ffi::c_void as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
    };

    let ccall_arg_idx = 0..n_args;
    let julia_arg_idx = 0..n_args;

    let q = parse_quote! {
        {
            frame.scope(|mut frame| {
                let unrooted = frame.unrooted();
                let name = Symbol::new(&frame, #rename);
                let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&unrooted).as_value();

                #invoke_fn;

                let func = Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                unsafe {
                    let mut ccall_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        &mut frame,
                        #n_args + 1,
                        type_type);

                    let mut ccall_arg_types_ref = ccall_arg_types.value_data_mut().unwrap();

                    let mut julia_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        &mut frame,
                        #n_args,
                        type_type);

                    let mut julia_arg_types_ref = julia_arg_types.value_data_mut().unwrap();

                    ccall_arg_types_ref.set(0, Some(#first_arg)).unwrap();
                    #(
                        ccall_arg_types_ref.set(#ccall_arg_idx + 1, Some(#ccall_arg_types.as_value())).unwrap();
                        julia_arg_types_ref.set(#julia_arg_idx, Some(#julia_arg_types.as_value())).unwrap();
                    )*

                    let ccall_return_type = #ccall_ret_type;
                    let julia_return_type = #julia_ret_type;

                    let module = #override_module_fragment;

                    let true_v = ::jlrs::data::managed::value::Value::true_v(&frame);
                    let nothing = ::jlrs::data::managed::value::Value::nothing(&frame);

                    let instance = function_info_ty.instantiate_unchecked(&mut frame, [
                        name.as_value(),
                        ccall_arg_types.as_value(),
                        julia_arg_types.as_value(),
                        ccall_return_type,
                        julia_return_type,
                        func,
                        module.as_value(),
                        true_v
                    ]);


                    let n = #index + offset;
                    accessor.set(n, Some(instance)).unwrap();
                }

                Ok(())
            }).unwrap();
        }
    };

    Ok(q)
}

fn const_info_fragment(info: &ExportedConst) -> Expr {
    let name = &info.name;
    let rename = info.name_override.as_ref().unwrap_or(name).to_string();
    let ty = &info.ty;

    parse_quote! {
        {
            frame.scope(move |mut frame| {
                let v: #ty = #name;
                let value = ::jlrs::data::managed::value::Value::new(&mut frame, v);

                unsafe {
                    module.set_const_unchecked(#rename, value);
                }

                Ok(())

            }).unwrap();
        }
    }
}

fn global_info_fragment(info: &ExportedGlobal) -> Expr {
    let name = &info.name;
    let rename = info.name_override.as_ref().unwrap_or(name).to_string();
    let ty = &info.ty;

    parse_quote! {
        {
            frame.scope(move |mut frame| {
                let v: #ty = #name;
                let value = ::jlrs::data::managed::value::Value::new(&mut frame, v);

                unsafe {
                    module.set_global_unchecked(#rename, value);
                }

                Ok(())

            }).unwrap();
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
                        span=> <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
                _ => {
                    let span = parent.span();
                    parse_quote_spanned! {
                        span=> <<TypedValue::<#parent> as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
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
                        span=> <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
                _ => {
                    let span = parent.span();
                    parse_quote_spanned! {
                        span=> <<::jlrs::data::managed::value::typed::TypedValue<#parent> as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
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
                        span=> <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
                _ => {
                    let span = parent.span();
                    parse_quote_spanned! {
                        span=> <<TypedValue::<#parent> as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
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
                        span=> <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
                _ => {
                    let span = parent2.span();
                    parse_quote_spanned! {
                        span=> <<::jlrs::data::managed::value::typed::TypedValue<#parent2> as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
            }
        });

    (ccall_arg_types, julia_arg_types, invoke_fn)
}

fn async_callback_arg_type_fragments<'a>(
    info: &'a ExportedAsyncCallback,
    inner_ret_ty: &Type,
) -> Result<(
    impl 'a + Iterator<Item = Expr>,
    impl 'a + Iterator<Item = Expr>,
    ItemFn,
)> {
    let inputs = &info.func.inputs;
    let n_args = inputs.len();

    if n_args > 0 {
        if let FnArg::Receiver(r) = inputs.first().unwrap() {
            Err(syn::Error::new_spanned(
                r.to_token_stream(),
                "async callback must be a free-standing function",
            ))?;
        }
    }

    let invoke_fn = invoke_async_callback(info, inner_ret_ty);

    let ccall_arg_types = inputs
        .iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = &ty.ty;
                    parse_quote! {
                        <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
                _ => unreachable!()
            }
        });

    let julia_arg_types = inputs
        .iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = &ty.ty;
                    parse_quote! {
                        <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
                _ => unreachable!()
            }
        });

    Ok((ccall_arg_types, julia_arg_types, invoke_fn))
}

fn async_callback_arg_type_fragments_in_env<'a>(
    info: &'a ExportedAsyncCallback,
    resolver: &'a ResolvedParameterList,
) -> Result<(
    Type,
    impl 'a + Iterator<Item = Expr>,
    impl 'a + Iterator<Item = Expr>,
    ItemFn,
)> {
    let inputs = resolver.apply(&info.func.inputs);
    let n_args = inputs.len();

    if n_args > 0 {
        if let FnArg::Receiver(r) = inputs.first().unwrap() {
            Err(syn::Error::new_spanned(
                r.to_token_stream(),
                "async callback must be a free-standing function",
            ))?;
        }
    }

    let ret_ty: &ReturnType = &info.func.output;
    let inner_ret_ty = async_callback_return_type_fragments(ret_ty);
    let Some(inner_ret_ty) = inner_ret_ty else {
        Err(syn::Error::new_spanned(
            ret_ty.to_token_stream(),
            "Async callback must return JlrsResult<impl AsyncCallback<T>> where T is some type that implements IntoJulia",
        ))?;
        unreachable!()
    };

    let inner_ret_ty = resolver.apply(&inner_ret_ty);
    let name = &info.func.ident;
    let invoke_fn = invoke_async_callback_in_env(name, &inputs, &inner_ret_ty);

    let ccall_arg_types = inputs
        .clone()
        .into_iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = &ty.ty;
                    parse_quote! {
                        <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
                _ => unreachable!()
            }
        });

    let julia_arg_types = inputs
        .into_iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = &ty.ty;
                    parse_quote! {
                        <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                    }
                },
                _ => unreachable!()
            }
        });

    Ok((inner_ret_ty, ccall_arg_types, julia_arg_types, invoke_fn))
}

// TODO: invoke fix exc
// FIXME: require impl AsyncCallback
fn invoke_async_callback(info: &ExportedAsyncCallback, ret_ty: &Type) -> ItemFn {
    let name = &info.func.ident;
    let args = &info.func.inputs;
    let span = info.func.ident.span();
    let mut extended_args = args.clone();
    extended_args.insert(
        0,
        syn::parse_quote! { jlrs_async_condition_handle: ::jlrs::ccall::AsyncConditionHandle },
    );

    let names = args.iter().map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#extended_args) -> ::jlrs::ccall::AsyncCCall {
            let join_handle: ::std::sync::Arc<::jlrs::ccall::DispatchHandle<#ret_ty>> = match #name(#names) {
                Ok(callback) => {
                    ::jlrs::ccall::CCall::dispatch_to_pool(move |dispatch_handle| {
                        let handle = jlrs_async_condition_handle;
                        let res: ::jlrs::error::JlrsResult<#ret_ty> = callback();
                        unsafe { dispatch_handle.set(res); }
                        ::jlrs::ccall::CCall::uv_async_send(handle.0);
                    })
                },
                Err(e) => {
                     ::jlrs::ccall::CCall::dispatch_to_pool(move |dispatch_handle| {
                        let handle = jlrs_async_condition_handle;
                        let res: ::jlrs::error::JlrsResult<#ret_ty> = Err(e);
                        unsafe { dispatch_handle.set(res); }
                        ::jlrs::ccall::CCall::uv_async_send(handle.0);
                    })
                }
            };

            let join_handle = ::std::sync::Arc::into_raw(join_handle);

            unsafe extern "C" fn join_func(
                handle: *mut ::jlrs::ccall::DispatchHandle<#ret_ty>
            ) -> #ret_ty {
                let handle = ::std::sync::Arc::from_raw(handle);
                let res = handle.join();
                <::jlrs::error::JlrsResult<#ret_ty> as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
            }

            ::jlrs::ccall::AsyncCCall {
                join_handle: join_handle as *mut ::jlrs::ccall::DispatchHandle<#ret_ty> as *mut ::std::ffi::c_void,
                join_func: join_func as *mut ::std::ffi::c_void
            }
        }
    }
}

fn invoke_async_callback_in_env<'a>(
    name: &Ident,
    inputs: &Punctuated<FnArg, Token![,]>,
    ret_ty: &Type,
) -> ItemFn {
    let span = name.span();
    let mut extended_args = inputs.clone();
    extended_args.insert(
        0,
        syn::parse_quote! { jlrs_async_condition_handle: ::jlrs::ccall::AsyncConditionHandle },
    );

    let names = inputs.iter().map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    parse_quote_spanned! {
        span=> unsafe extern "C" fn invoke(#extended_args) -> ::jlrs::ccall::AsyncCCall {
            let join_handle: ::std::sync::Arc<::jlrs::ccall::DispatchHandle<#ret_ty>> = match #name(#names) {
                Ok(callback) => {
                    ::jlrs::ccall::CCall::dispatch_to_pool(move |dispatch_handle| {
                        let handle = jlrs_async_condition_handle;
                        let res = callback();
                        unsafe { dispatch_handle.set(res); }
                        ::jlrs::ccall::CCall::uv_async_send(handle.0);
                    })
                },
                Err(e) => {
                     ::jlrs::ccall::CCall::dispatch_to_pool(move |dispatch_handle| {
                        let handle = jlrs_async_condition_handle;
                        let res: ::jlrs::error::JlrsResult<#ret_ty> = Err(e);
                        unsafe { dispatch_handle.set(res); }
                        ::jlrs::ccall::CCall::uv_async_send(handle.0);
                    })
                }
            };

            let join_handle = ::std::sync::Arc::into_raw(join_handle);

            unsafe extern "C" fn join_func(
                handle: *mut ::jlrs::ccall::DispatchHandle<#ret_ty>
            ) -> #ret_ty {
                let handle = ::std::sync::Arc::from_raw(handle);
                let res = handle.join();
                <::jlrs::error::JlrsResult<#ret_ty> as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
            }

            ::jlrs::ccall::AsyncCCall {
                join_handle: join_handle as *mut ::jlrs::ccall::DispatchHandle<#ret_ty> as *mut ::std::ffi::c_void,
                join_func: join_func as *mut ::std::ffi::c_void
            }
        }
    }
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
            let res = #call_expr;
            <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
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
            let res = #call_expr;
            <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
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
            match #to_ref_expr {
                Ok(this) => {
                    let res = #call_expr;
                    <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                },
                Err(_) => ::jlrs::ccall::CCall::throw_borrow_exception()
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
            match #to_ref_expr {
                Ok(this) => {
                    let res = #call_expr;
                    <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                },
                Err(_) => ::jlrs::ccall::CCall::throw_borrow_exception()
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
            match #to_ref_expr {
                Ok(this) => {
                    let res = #call_expr;
                    <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                },
                Err(_) => ::jlrs::ccall::CCall::throw_borrow_exception()
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
            match #to_ref_expr {
                Ok(this) => {
                    let res = #call_expr;
                    <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                },
                Err(_) => ::jlrs::ccall::CCall::throw_borrow_exception()
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
            match #to_ref_expr {
                #[allow(unused_mut)]
                Ok(mut this) => {
                    let res = #call_expr;
                    <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                },
                Err(_) => ::jlrs::ccall::CCall::throw_borrow_exception()
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
            match #to_ref_expr {
                #[allow(unused_mut)]
                Ok(mut this) => {
                    let res = #call_expr;
                    <#ret_ty as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
                },
                Err(_) => ::jlrs::ccall::CCall::throw_borrow_exception()
            }
        }
    }
}

fn has_outer_path_attr(attrs: &[Attribute], name: &str) -> bool {
    for attr in attrs {
        match attr.style {
            AttrStyle::Outer => (),
            _ => continue,
        }

        match attr.meta {
            Meta::Path(ref p) => {
                if p.is_ident(name) {
                    return true;
                }
            }
            _ => continue,
        }
    }

    false
}
