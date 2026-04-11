use quote::format_ident;
use syn::{
    Expr, FnArg, Ident, ItemFn, ReturnType, Token, Type, parse_quote, parse_quote_spanned,
    punctuated::Punctuated, spanned::Spanned,
};

use crate::{
    codegen::name_codegen::module_codegen,
    ir::function_ir::{FunctionIR, FunctionsIR},
    model::function_model::FunctionKind,
};

pub struct FunctionCodegen<'a> {
    init_fn_name: &'a Ident,
    ir: &'a FunctionsIR<'a>,
}

impl<'a> FunctionCodegen<'a> {
    pub fn new(init_fn_name: &'a Ident, ir: &'a FunctionsIR) -> Self {
        FunctionCodegen { init_fn_name, ir }
    }

    pub fn functions_init_fn(&self) -> ItemFn {
        let fn_ident = format_ident!("{}_functions", self.init_fn_name);
        let accessor = format_ident!("accessor");
        let module = format_ident!("module");
        let function_info_ty = format_ident!("function_info_ty");

        let n_functions = self.ir.n_exported_functions();
        let fragments = self
            .ir
            .functions
            .iter()
            .enumerate()
            .map(|(offset, ir)| function_info_fragment(ir, &module, &accessor, offset));

        parse_quote! {
            unsafe fn #fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                array: &mut ::jlrs::data::managed::array::Vector<'_, 'static>,
                #module: ::jlrs::data::managed::module::Module,
                #function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                use ::jlrs::data::managed::array::data::accessor::{AccessorMut1D as _, AccessorMut as _, AccessorMut as _};

                unsafe {
                    let mut #accessor = array.indeterminate_data_mut();
                    #accessor.grow_end_unchecked(#n_functions);
                    #(#fragments;)*
                }
            }
        }
    }
}

fn function_info_fragment(
    ir: &FunctionIR,
    module: &Ident,
    accessor: &Ident,
    offset: usize,
) -> Expr {
    let n_inputs = ir.signature.inputs.len();
    let get_module = module_codegen(module, &ir.export_name);
    let name = ir.export_name.name_string();
    let env = format_ident!("env");

    let env_expr = type_var_env_expr(ir.type_var_env);
    let (ccall_inputs, function_inputs) = arg_type_fragments(&ir.signature.inputs, &env);
    let (ccall_output, julia_output) = return_type_fragments(&ir.signature.output, &env);

    let ccall_input_idx = 0..n_inputs;
    let julia_input_idx = 0..n_inputs;

    let invoke_fn = invoke_fn_template(ir);

    let expr = parse_quote! {
        frame.local_scope::<_, 9>(#[inline(never)] |mut frame| {
            // Root 1
            let mut output = frame.output();

            let instance = {
                let name = ::jlrs::data::managed::symbol::Symbol::new(&frame, #name);
                let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&frame).as_value();
                let any_type = ::jlrs::data::managed::datatype::DataType::any_type(&frame).as_value();

                #invoke_fn

                // Root #2
                let func = ::jlrs::data::managed::value::Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                unsafe {
                    // Root #3
                    let #env = #env_expr;

                    // Root #4
                    let mut ccall_arg_types = ::jlrs::data::managed::array::Vector::new_for_unchecked(
                        &mut frame,
                        type_type,
                        #n_inputs,
                    );

                    let mut ccall_arg_types_ref = ccall_arg_types.indeterminate_data_mut();

                    // Root #5
                    let mut julia_arg_types = ::jlrs::data::managed::array::Vector::new_for_unchecked(
                        &mut frame,
                        any_type,
                        #n_inputs,
                    );

                    let mut julia_arg_types_ref = julia_arg_types.indeterminate_data_mut();

                    #(
                        {
                            frame.local_scope::<_, 3>(#[inline(never)] |mut frame|{
                                let ccall_input = #ccall_inputs.as_value();
                                ccall_arg_types_ref.set_value(&mut output, #ccall_input_idx, ccall_input).unwrap().unwrap();
                                let julia_input = #function_inputs.as_value();
                                if julia_input.is::<::jlrs::data::managed::type_var::TypeVar>() {
                                    let julia_input = ::jlrs::data::managed::union_all::tvar_to_unionall(&mut frame, julia_input);
                                    julia_arg_types_ref.set_value(&mut output, #julia_input_idx, julia_input).unwrap().unwrap();
                                } else {
                                    julia_arg_types_ref.set_value(&mut output, #julia_input_idx, julia_input).unwrap().unwrap();
                                }
                            });
                        }
                    )*

                    // Root #6
                    let ccall_return_type = #ccall_output;

                    // Root #7
                    let julia_return_type = #julia_output;

                    let #module = #get_module;

                    // Root #8
                    let #env = #env.filter(&mut frame, julia_arg_types);

                    // Root #9
                    function_info_ty.instantiate_unchecked(&mut frame, [
                        name.as_value(),
                        ccall_arg_types.as_value(),
                        julia_arg_types.as_value(),
                        ccall_return_type.as_value(),
                        julia_return_type.as_value(),
                        func,
                        #module.as_value(),
                        #env.to_svec().as_value(),
                    ])
                }
            };

            #accessor.set_value(&mut output, #offset, instance).unwrap().unwrap();
        })
    };

    expr
}

fn type_var_env_expr(type_var_env: Option<&Type>) -> Expr {
    if let Some(x) = type_var_env {
        parse_quote! { <#x as ::jlrs::data::types::construct_type::TypeVars>::into_env(&mut frame) }
    } else {
        parse_quote! { ::jlrs::data::types::construct_type::TypeVarEnv::empty(&frame) }
    }
}

fn arg_type_fragments<'a>(
    inputs: &'a Punctuated<FnArg, Token![,]>,
    env: &'a Ident,
) -> (
    impl 'a + Iterator<Item = Expr>,
    impl 'a + Iterator<Item = Expr>,
) {
    let arg_types = inputs.iter().map(|arg| match arg {
        FnArg::Receiver(_receiver) => {
            unreachable!("julia_module internal error: unexpected receiver in FunctionIR")
        }
        FnArg::Typed(pat_type) => pat_type.ty.as_ref(),
    });

    let ccall_arg_types = arg_types.clone()
        .map(move |ty| {
            parse_quote! {
                if #env.is_empty() {
                    <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                } else {
                    <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &#env)
                }
            }
        });

    let julia_arg_types = arg_types
        .map(move |ty| {
            parse_quote! {
                if #env.is_empty() {
                    <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                } else {
                    <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &#env)
                }
            }
        });

    (ccall_arg_types, julia_arg_types)
}

fn return_type_fragments(ret_ty: &ReturnType, env: &Ident) -> (Expr, Expr) {
    match ret_ty {
        ReturnType::Default => {
            let ccall_ret_type: Expr = parse_quote! {
                ::jlrs::data::managed::datatype::DataType::nothing_type(&frame).as_value()
            };

            let julia_ret_type = ccall_ret_type.clone();
            (ccall_ret_type, julia_ret_type)
        }
        ReturnType::Type(_, ty) => {
            let span = ty.span();
            let ccall_ret_type = parse_quote_spanned! {
                span=> if #env.is_empty() {
                    <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::CCallReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                } else {
                    <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::CCallReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &#env)
                }

            };
            let julia_ret_type = parse_quote_spanned! {
                span=> if #env.is_empty() {
                    <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::FunctionReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                } else {
                    <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::FunctionReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &#env)
                }

            };

            (ccall_ret_type, julia_ret_type)
        }
    }
}

fn as_return_as(ret_ty: &ReturnType) -> ReturnType {
    let mut new_ty = ret_ty.clone();

    if let ReturnType::Type(_, ty) = &mut new_ty {
        let new_ty: Type = parse_quote! {
            <#ty as ::jlrs::convert::ccall_types::CCallReturn>::ReturnAs
        };
        **ty = new_ty;
    }

    new_ty
}

fn maybe_make_gc_safe(ir: &FunctionIR, expr: Expr) -> Expr {
    if ir.gc_safe {
        parse_quote! {
            ::jlrs::memory::gc::gc_safe(|| {
                #expr
            })
        }
    } else {
        expr
    }
}

fn call_method_template(ir: &FunctionIR) -> Expr {
    let out_type = return_type_into_type(ir.signature.output.clone());
    let name = &ir.signature.ident;
    let names = {
        let invoke_names_iter = ir.signature.inputs.iter().map(|arg| match arg {
            FnArg::Receiver(_) => {
                unreachable!("julia_module internal error: unexpected receiver in FunctionIR")
            }
            FnArg::Typed(pat_type) => match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => &pat_ident.ident,
                _ => todo!(),
            },
        });
        Punctuated::<_, Token![,]>::from_iter(invoke_names_iter)
    };

    let gc_unsafe_call_expr: Expr = match &ir.kind {
        FunctionKind::SelfMethod { parent, .. }
        | FunctionKind::RefSelfMethod { parent, .. }
        | FunctionKind::MutRefSelfMethod { parent, .. }
        | FunctionKind::AssocFunction { parent } => {
            parse_quote! {
                <#parent>::#name(#names)
            }
        }
        FunctionKind::Function => parse_quote! {
            #name(#names)
        },
    };

    let call_expr = maybe_make_gc_safe(ir, gc_unsafe_call_expr);

    parse_quote! {
        {
            let res = #call_expr;
            <#out_type as ::jlrs::convert::ccall_types::CCallReturn>::return_or_throw(res)
        }
    }
}

fn invoke_block_template(ir: &FunctionIR) -> Expr {
    let call_expr = call_method_template(ir);

    match &ir.kind {
        FunctionKind::SelfMethod {
            untracked_self,
            parent,
        } => {
            if *untracked_self {
                parse_quote! {
                    {
                        let this = (&this).data_ptr().cast::<#parent>().as_ref().clone();
                        #call_expr
                    }
                }
            } else {
                parse_quote! {
                    match (&this).track_shared() {
                        Ok(this) => {
                            let this = (&*this).clone();
                            #call_expr
                        },
                        Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                    }
                }
            }
        }
        FunctionKind::RefSelfMethod {
            untracked_self,
            parent,
        } => {
            if *untracked_self {
                parse_quote! {
                    {
                        let this = (&this).data_ptr().cast::<#parent>().as_ref();
                        #call_expr
                    }
                }
            } else {
                parse_quote! {
                    match (&this).track_shared() {
                        Ok(this) => {
                            let this = &*this;
                            #call_expr
                        },
                        Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                    }
                }
            }
        }
        FunctionKind::MutRefSelfMethod {
            untracked_self,
            parent,
        } => {
            if *untracked_self {
                parse_quote! {
                    {
                        let this = (&mut this).data_ptr().cast::<#parent>().as_mut();
                        #call_expr
                    }
                }
            } else {
                parse_quote! {
                    match (&mut this).track_exclusive() {
                        Ok(mut this) => {
                            use std::ops::DerefMut as _;
                            let this = this.deref_mut();
                            #call_expr
                        },
                        Err(_) => ::jlrs::runtime::handle::ccall::throw_borrow_exception()
                    }
                }
            }
        }
        FunctionKind::AssocFunction { .. } | FunctionKind::Function => call_expr,
    }
}

fn invoke_fn_template(ir: &FunctionIR) -> ItemFn {
    let inputs = &ir.signature.inputs;
    let return_ty = as_return_as(&ir.signature.output);
    let invoke_block = invoke_block_template(ir);

    parse_quote! {
        unsafe extern "C" fn invoke(#inputs) #return_ty {
            unsafe {
                #invoke_block
            }
        }
    }
}

fn return_type_into_type(ty: ReturnType) -> Type {
    match ty {
        ReturnType::Default => parse_quote! { () },
        ReturnType::Type(_, ty) => *ty,
    }
}
