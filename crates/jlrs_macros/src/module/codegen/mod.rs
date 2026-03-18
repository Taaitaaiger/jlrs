//! Codegen for `julia_module!`

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::{
        alias_codegen::AliasCodegen, const_codegen::ConstCodegen, docs_codegen::DocsCodegen,
        function_codegen::FunctionCodegen, struct_codegen::StructCodegen,
    },
    ir::JuliaModuleIR,
};

mod alias_codegen;
mod const_codegen;
mod docs_codegen;
mod function_codegen;
mod name_codegen;
mod struct_codegen;

pub fn codegen(ir: JuliaModuleIR) -> TokenStream {
    let init_fn_ident = &ir.init_fn;

    let struct_codegen = StructCodegen::new(init_fn_ident, &ir.structs_ir);
    let struct_init_fn = struct_codegen.init_fn();
    let struct_init_fn_ident = &struct_init_fn.sig.ident;
    let struct_reinit_fn = struct_codegen.reinit_fn();
    let struct_reinit_fn_ident = &struct_reinit_fn.sig.ident;

    let function_codegen = FunctionCodegen::new(init_fn_ident, &ir.functions_ir);
    let function_init_fn = function_codegen.functions_init_fn();
    let function_init_fn_ident = &function_init_fn.sig.ident;

    let alias_codegen = AliasCodegen::new(init_fn_ident, &ir.aliases_ir);
    let alias_init_fn = alias_codegen.init_fn();
    let alias_init_fn_ident = &alias_init_fn.sig.ident;

    let const_codegen = ConstCodegen::new(init_fn_ident, &ir.consts_ir);
    let const_init_fn = const_codegen.init_fn();
    let const_init_fn_ident = &const_init_fn.sig.ident;

    let docs_codegen = DocsCodegen::new(init_fn_ident, &ir.docs_ir);
    let docs_init_fn = docs_codegen.init_fn();
    let docs_init_fn_ident = &docs_init_fn.sig.ident;

    quote! {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn #init_fn_ident(
            module: ::jlrs::data::managed::module::Module,
            precompiling: u8,
        ) -> ::jlrs::data::managed::value::ValueRet {
            unsafe {
                #struct_init_fn

                #struct_reinit_fn

                #function_init_fn

                #const_init_fn

                #alias_init_fn

                #docs_init_fn

                static IS_INIT: ::std::sync::atomic::AtomicBool = ::std::sync::atomic::AtomicBool::new(false);
                if IS_INIT.compare_exchange(false, true, ::std::sync::atomic::Ordering::Relaxed, ::std::sync::atomic::Ordering::Relaxed).is_err() {
                    let unrooted = <::jlrs::data::managed::module::Module as ::jlrs::data::managed::Managed>::unrooted_target(module);
                    return ::jlrs::data::managed::value::Value::nothing(&unrooted).as_weak().leak();
                }

                ::jlrs::runtime::handle::ccall::init_jlrs_wrapped(&::jlrs::InstallJlrsCore::No);

                match ::jlrs::weak_handle!() {
                    Ok(handle) => {
                        handle.local_scope::<_, 2>(|mut frame| {
                            let wrap_mod = ::jlrs::data::managed::module::Module::jlrs_core(&frame)
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

                            if precompiling == 1 {
                                #struct_init_fn_ident(&frame, module);
                                #const_init_fn_ident(&frame, module);
                                #alias_init_fn_ident(&frame, module);
                            } else {
                                #struct_reinit_fn_ident(&frame, module);
                            }

                            let mut arr = ::jlrs::data::managed::array::Vector::new_for_unchecked(&mut frame, function_info_ty.as_value(), 0);
                            #function_init_fn_ident(&frame, &mut arr, module, function_info_ty);

                            let mut doc_items = ::jlrs::data::managed::array::Vector::new_for_unchecked(&mut frame, doc_item_ty.as_value(), 0);
                            if precompiling == 1 {
                                #docs_init_fn_ident(&frame, &mut doc_items, module, doc_item_ty);
                            }
                            module_info_ty.instantiate_unchecked(&frame, [arr.as_value(), doc_items.as_value()]).leak()
                        })
                    },
                    Err(_) => panic!("Not called from Julia, or Julia is in a GC-safe state"),
                }
            }
        }
    }
}
