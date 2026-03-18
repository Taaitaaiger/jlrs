use quote::format_ident;
use syn::{Expr, Ident, ItemFn, parse_quote};

use crate::{
    codegen::name_codegen::module_codegen,
    ir::struct_ir::{StructIR, StructsIR},
};

pub struct StructCodegen<'a> {
    init_fn_name: &'a Ident,
    ir: &'a StructsIR<'a>,
}

impl<'a> StructCodegen<'a> {
    pub fn new(init_fn_name: &'a Ident, ir: &'a StructsIR) -> Self {
        StructCodegen { init_fn_name, ir }
    }

    pub fn init_fn(&self) -> ItemFn {
        let fn_ident = format_ident!("{}_types", self.init_fn_name);
        let module = format_ident!("module");

        let fragments = self
            .ir
            .structs
            .iter()
            .map(|ir| init_type_fragment(ir, &module))
            .collect::<Vec<_>>();

        init_fn(fn_ident, fragments)
    }

    pub fn reinit_fn(&self) -> ItemFn {
        let fn_ident = format_ident!("{}_reinit_types", self.init_fn_name);
        let module = format_ident!("module");

        let fragments = self
            .ir
            .structs
            .iter()
            .map(|ir| reinit_type_fragment(ir, &module))
            .collect::<Vec<_>>();

        init_fn(fn_ident, fragments)
    }
}

fn init_fn(ident: Ident, fragments: Vec<Expr>) -> ItemFn {
    parse_quote! {
        unsafe fn #ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
            frame: &Tgt,
            module: ::jlrs::data::managed::module::Module,
        ) {
            frame.local_scope::<_, 1>(|mut frame| {
                let mut output = frame.output();
                unsafe {
                    #(#fragments)*
                }
            });
        }
    }
}

fn init_type_fragment(struct_ir: &StructIR, module: &Ident) -> Expr {
    let get_module = module_codegen(module, &struct_ir.export_name);
    let name = struct_ir.export_name.name_string();
    let path = &struct_ir.key;
    let variants = &struct_ir.paths;

    parse_quote! {
        {
            let sym = ::jlrs::data::managed::symbol::Symbol::new(&frame, #name);
            let #module = #get_module;
            let ty = <#path as ::jlrs::data::types::foreign_type::OpaqueType>::create_type(&mut output, sym, #module);
            let ty = ::jlrs::data::managed::erase_scope_lifetime(ty).rewrap(&mut output);
            #module.set_const_unchecked(sym, ty);

            #(
                <#variants as ::jlrs::data::types::foreign_type::OpaqueType>::create_variant(&mut output, sym);
            )*
        }
    }
}

fn reinit_type_fragment(struct_ir: &StructIR, module: &Ident) -> Expr {
    let module = module_codegen(module, &struct_ir.export_name);
    let name = struct_ir.export_name.name_string();
    let path = &struct_ir.key;

    let variant_reinit = struct_ir.paths.iter().map(|variant| {
        let expr: Expr = parse_quote! {
            {
                let params = <#variant as ::jlrs::data::types::foreign_type::OpaqueType>::variant_parameters(&mut output);
                let params = ::jlrs::data::managed::erase_scope_lifetime(params);
                let params = params.data();
                let param_slice = params.as_atomic_slice().assume_immutable_non_null();
                let dt = ua.apply_types_unchecked(&mut output, param_slice).cast::<::jlrs::data::managed::datatype::DataType>().unwrap();
                let dt = ::jlrs::data::managed::erase_scope_lifetime(dt);

                <#variant as ::jlrs::data::types::foreign_type::OpaqueType>::reinit_variant(dt);
            }
        };

        expr
    });

    parse_quote! {
        {
            let ty = #module
                .global(&frame, #name)
                .unwrap()
                .as_value();

            if let Ok(dt) = ty.cast::<::jlrs::data::managed::datatype::DataType>() {
                <#path as ::jlrs::data::types::foreign_type::OpaqueType>::reinit_type(dt);
            } else if let Ok(ua) = ty.cast::<::jlrs::data::managed::union_all::UnionAll>() {
                let dt = ua.base_type();
                <#path as ::jlrs::data::types::foreign_type::OpaqueType>::reinit_type(dt);

                #(#variant_reinit;)*
            } else {
                panic!()
            }

        }
    }
}
