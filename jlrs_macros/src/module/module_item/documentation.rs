use quote::{format_ident, ToTokens};
use syn::{parse_quote, Expr, Ident, ItemFn, Result};

use super::{
    init_fn::InitFn, item_with_attrs::ItemWithAttrs, override_module_fragment, ModuleItem,
};
use crate::JuliaModule;

pub struct DocFragments {
    pub init_docs_fn_ident: Ident,
    pub init_docs_fn: ItemFn,
}

impl DocFragments {
    pub fn generate(module: &JuliaModule, init_fn: &InitFn) -> Result<Self> {
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
            unsafe fn #init_docs_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                array: &mut ::jlrs::data::managed::array::Vector<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                doc_item_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                use ::jlrs::data::managed::array::{data::accessor::{AccessorMut1D as _, AccessorMut as _, Accessor as _}, dimensions::Dims as _};

                unsafe {
                    let mut accessor = array.indeterminate_data_mut();
                    accessor.grow_end_unchecked(#n_docs);

                    #(#fragments)*
                }
            }
        };

        Ok(DocFragments {
            init_docs_fn_ident,
            init_docs_fn,
        })
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
                    frame.local_scope::<3>(|mut frame| {
                        unsafe {
                            let module = #override_module_fragment;
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value(&mut frame, #index, doc_it).unwrap().into_jlrs_result().unwrap();
                        }
                    });
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
                    frame.local_scope::<3>(|mut frame| {
                        unsafe {
                            let module = #override_module_fragment;
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value(&mut frame, #index, doc_it).unwrap().into_jlrs_result().unwrap();
                        }
                    });
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
                    frame.local_scope::<3>(|mut frame| {
                        unsafe {
                            let module = #override_module_fragment;
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value(&mut frame, #index, doc_it).unwrap().into_jlrs_result().unwrap();
                        }
                    });
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
                    frame.local_scope::<3>(|mut frame| {
                        unsafe {
                            let item = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
                            let signature = ::jlrs::data::managed::value::Value::bottom_type(&frame);
                            let doc = ::jlrs::data::managed::string::JuliaString::new(&mut frame, #doc);

                            let doc_it = doc_item_ty.instantiate_unchecked(&mut frame, [module.as_value(), item.as_value(), signature, doc.as_value()]);
                            accessor.set_value(&mut frame, #index, doc_it).unwrap().into_jlrs_result().unwrap();
                        }
                    });
                }

            };

            Ok(q)
        }
        ModuleItem::ExportedAlias(a) => Err(syn::Error::new_spanned(
            a.name.to_token_stream(),
            "type alias cannot be documented",
        ))?,
        ModuleItem::ItemWithAttrs(_) => unreachable!(),
        ModuleItem::ExportedGenerics(_) => unreachable!(),
    }
}
