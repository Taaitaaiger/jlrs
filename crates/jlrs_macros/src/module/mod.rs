mod module_item;
mod parameters;

use itertools::Itertools;
use module_item::{
    ModuleItem,
    documentation::DocFragments,
    exported_alias::{AliasFragments, ExportedAlias},
    exported_const::{ConstFragments, ExportedConst},
    exported_function::{ExportedFunction, FunctionFragments},
    exported_generics::ExportedGenerics,
    exported_method::{ExportedMethod, MethodFragments},
    exported_type::{ExportedType, TypeFragments},
    init_fn::InitFn,
    item_with_attrs::ItemWithAttrs,
};
use parameters::{ParameterEnvironment, ParameterList};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    Attribute, Error, Ident, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use self::parameters::{Apply, ResolvedParameterList};
use crate::module::parameters::{as_return_as, take_type};

type RenameFragments = Punctuated<Ident, Token![.]>;

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
        let generic_method_fragments = MethodFragments::generate_generic(&self, init_fn)?;
        let type_fragments = TypeFragments::generate(&self, init_fn);
        let generic_type_fragments = TypeFragments::generate_generic(&self, init_fn);
        let const_fragments = ConstFragments::generate(&self, init_fn);
        let alias_fragments = AliasFragments::generate(&self, init_fn);
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
        let const_init_fn = const_fragments.const_init_fn;
        let const_init_fn_ident = const_fragments.const_init_ident;
        let alias_init_fn = alias_fragments.alias_init_fn;
        let alias_init_fn_ident = alias_fragments.alias_init_ident;
        let doc_init_fn = doc_fragments.init_docs_fn;
        let doc_init_fn_ident = doc_fragments.init_docs_fn_ident;

        let generated = quote::quote! {
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn #init_fn_ident(
                module: ::jlrs::data::managed::module::Module,
                precompiling: u8,
            ) -> ::jlrs::data::managed::value::ValueRet {
                unsafe {
                    #type_init_fn

                    #type_reinit_fn

                    #generic_type_init_fn

                    #generic_type_reinit_fn

                    #function_init_fn

                    #generic_function_init_fn

                    #method_init_fn

                    #generic_method_init_fn

                    #const_init_fn

                    #alias_init_fn

                    #doc_init_fn

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
                                    #type_init_fn_ident(&frame, module);
                                    #generic_type_init_fn_ident(&frame, module);
                                    #const_init_fn_ident(&frame, module);
                                    #alias_init_fn_ident(&frame, module);
                                } else {
                                    #type_reinit_fn_ident(&frame, module);
                                    #generic_type_reinit_fn_ident(&frame, module);
                                }

                                let mut arr = ::jlrs::data::managed::array::Vector::new_for_unchecked(&mut frame, function_info_ty.as_value(), 0);
                                #function_init_fn_ident(&frame, &mut arr, module, function_info_ty);
                                #generic_function_init_fn_ident(&frame, &mut arr, module, function_info_ty);
                                #method_init_fn_ident(&frame, &mut arr, module, function_info_ty);
                                #generic_method_init_fn_ident(&frame, &mut arr, module, function_info_ty);

                                let mut doc_items = ::jlrs::data::managed::array::Vector::new_for_unchecked(&mut frame, doc_item_ty.as_value(), 0);
                                if precompiling == 1 {
                                    #doc_init_fn_ident(&frame, &mut doc_items, module, doc_item_ty);
                                }
                                module_info_ty.instantiate_unchecked(&frame, [arr.as_value(), doc_items.as_value()]).leak()
                            })
                        },
                        Err(_) => panic!("Not called from Julia, or Julia is in a GC-safe state"),
                    }
                }
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

    fn get_exported_aliases(&self) -> impl Iterator<Item = &ExportedAlias> {
        self.items
            .iter()
            .filter(|it| it.is_exported_alias())
            .map(|it| it.get_exported_alias())
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
