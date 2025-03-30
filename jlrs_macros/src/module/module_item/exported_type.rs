use itertools::Itertools;
use quote::format_ident;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Expr, Ident, ItemFn, Path, Result, Token,
};

use super::{generics::GenericEnvironment, init_fn::InitFn};
use crate::{
    module::{
        module_item::override_module_fragment,
        parameters::{Apply, ParameterEnvironment, ParameterList},
        RenameFragments,
    },
    JuliaModule,
};

pub struct ExportedType {
    pub is_pub: bool,
    pub _struct_token: Token![struct],
    pub name: Path,
    pub _as_token: Option<Token![as]>,
    pub name_override: Option<RenameFragments>,
}

impl ExportedType {
    pub fn init_with_env(
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
                let ty = ::jlrs::data::managed::erase_scope_lifetime(ty).rewrap(&mut output);
                module.set_const_unchecked(sym, ty);

                #(
                    #variants;
                )*
            }
        }
    }

    pub fn reinit_with_env(
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
                        let params = params.data();
                        let param_slice = params.as_atomic_slice().assume_immutable_non_null();
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
                is_pub: false,
                _struct_token: struct_token,
                name,
                _as_token: Some(as_token),
                name_override: Some(name_override),
            })
        } else {
            Ok(ExportedType {
                is_pub: false,
                _struct_token: struct_token,
                name,
                _as_token: None,
                name_override: None,
            })
        }
    }
}
pub struct TypeFragments {
    pub type_init_fn: ItemFn,
    pub type_init_ident: Ident,
    pub type_reinit_fn: ItemFn,
    pub type_reinit_ident: Ident,
}

impl TypeFragments {
    pub fn generate(info: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_types_fn_ident = format_ident!("{}_types", init_fn.init_fn);
        let init_types_fragments = info.get_exported_types().map(init_type_fragment);

        let type_init_fn = parse_quote! {
            unsafe fn #init_types_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                module: ::jlrs::data::managed::module::Module,
            ) {
                frame.local_scope::<1>(|mut frame| {
                    let mut output = frame.output();

                    #(
                        #init_types_fragments
                    )*
                });
            }
        };

        let reinit_types_fn_ident = format_ident!("{}_reinittypes", init_fn.init_fn);
        let reinit_types_fragments = info.get_exported_types().map(reinit_type_fragment);

        let type_reinit_fn = parse_quote! {
            unsafe fn #reinit_types_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                module: jlrs::data::managed::module::Module
            ) {

                frame.local_scope::<1>(|mut frame| {
                    let mut output = frame.output();

                    #(
                        #reinit_types_fragments
                    )*
                });
            }
        };

        TypeFragments {
            type_init_fn,
            type_init_ident: init_types_fn_ident,
            type_reinit_fn,
            type_reinit_ident: reinit_types_fn_ident,
        }
    }

    pub fn generate_generic(info: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_types_fn_ident = format_ident!("{}_generic_types", init_fn.init_fn);
        let init_types_fragments = info
            .get_exported_generics()
            .map(|g| g.to_generic_environment().init_type_fragments())
            .flatten();

        let type_init_fn = parse_quote! {
            unsafe fn #init_types_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                module: ::jlrs::data::managed::module::Module,
            ) {

                frame.local_scope::<1>(|mut frame| {
                    let mut output = frame.output();

                    #(
                        #init_types_fragments
                    )*
                });
            }
        };

        let reinit_types_fn_ident = format_ident!("{}_reinit_generic_types", init_fn.init_fn);
        let reinit_types_fragments = info
            .get_exported_generics()
            .map(|g| g.to_generic_environment().reinit_type_fragments())
            .flatten();

        let type_reinit_fn = parse_quote! {
            unsafe fn #reinit_types_fn_ident<'target, Tgt: ::jlrs::memory::target::Target<'target>>(
                frame: &Tgt,
                module: jlrs::data::managed::module::Module
            ) {

                frame.local_scope::<1>(|mut frame| {
                    let mut output = frame.output();

                    #(
                        #reinit_types_fragments
                    )*
                });
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
