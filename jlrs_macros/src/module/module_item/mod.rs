use exported_alias::ExportedAlias;
use exported_const::ExportedConst;
use exported_function::ExportedFunction;
use exported_generics::ExportedGenerics;
use exported_method::ExportedMethod;
use exported_type::ExportedType;
use init_fn::InitFn;
use item_with_attrs::ItemWithAttrs;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Pub},
    AttrStyle, Attribute, Error, Expr, FnArg, Meta, Result, ReturnType, Token,
};

use super::RenameFragments;

pub mod documentation;
pub mod exported_alias;
pub mod exported_const;
pub mod exported_function;
pub mod exported_generics;
pub mod exported_method;
pub mod exported_type;
pub mod generics;
pub mod init_fn;
pub mod item_with_attrs;

pub enum ModuleItem {
    InitFn(InitFn),
    ExportedType(ExportedType),
    ExportedFunction(ExportedFunction),
    ExportedMethod(ExportedMethod),
    ExportedConst(ExportedConst),
    ItemWithAttrs(ItemWithAttrs),
    ExportedGenerics(ExportedGenerics),
    ExportedAlias(ExportedAlias),
}

impl ModuleItem {
    pub fn is_init_fn(&self) -> bool {
        match self {
            ModuleItem::InitFn(_) => true,
            _ => false,
        }
    }

    pub fn get_init_fn(&self) -> &InitFn {
        match self {
            ModuleItem::InitFn(ref init_fn) => init_fn,
            _ => panic!(),
        }
    }

    pub fn is_exported_fn(&self) -> bool {
        match self {
            ModuleItem::ExportedFunction(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_fn() => true,
            _ => false,
        }
    }

    pub fn get_exported_fn(&self) -> (&ExportedFunction, Option<&[Attribute]>) {
        match self {
            ModuleItem::ExportedFunction(ref exported_fn) => (exported_fn, None),
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, attrs }) if item.is_exported_fn() => {
                (item.get_exported_fn().0, Some(attrs))
            }
            _ => panic!(),
        }
    }

    pub fn is_exported_method(&self) -> bool {
        match self {
            ModuleItem::ExportedMethod(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_method() => {
                true
            }
            _ => false,
        }
    }

    pub fn get_exported_method(&self) -> (&ExportedMethod, Option<&[Attribute]>) {
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

    pub fn is_exported_type(&self) -> bool {
        match self {
            ModuleItem::ExportedType(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_type() => {
                true
            }
            _ => false,
        }
    }

    pub fn get_exported_type(&self) -> &ExportedType {
        match self {
            ModuleItem::ExportedType(ref exported_type) => exported_type,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_type() => {
                item.get_exported_type()
            }
            _ => panic!(),
        }
    }

    pub fn is_exported_const(&self) -> bool {
        match self {
            ModuleItem::ExportedConst(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_const() => {
                true
            }
            _ => false,
        }
    }

    pub fn get_exported_const(&self) -> &ExportedConst {
        match self {
            ModuleItem::ExportedConst(ref exported_const) => exported_const,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_const() => {
                item.get_exported_const()
            }
            _ => panic!(),
        }
    }

    pub fn is_exported_alias(&self) -> bool {
        match self {
            ModuleItem::ExportedAlias(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_alias() => {
                true
            }
            _ => false,
        }
    }

    pub fn get_exported_alias(&self) -> &ExportedAlias {
        match self {
            ModuleItem::ExportedAlias(ref exported_alias) => exported_alias,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_alias() => {
                item.get_exported_alias()
            }
            _ => panic!(),
        }
    }

    pub fn is_exported_generics(&self) -> bool {
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

    pub fn get_exported_generics(&self) -> &ExportedGenerics {
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

    pub fn get_all_with_docs(&self) -> Vec<&ItemWithAttrs> {
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

    pub fn get_all_with_docs_inner<'a>(&'a self, items: &mut Vec<&'a ItemWithAttrs>) {
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
        let mut lookahead = input.lookahead1();

        let is_pub = lookahead.peek(Token![pub]);
        if is_pub {
            let _: Pub = Pub::parse(input)?;
            lookahead = input.lookahead1();
        }

        if !is_pub && lookahead.peek(Token![become]) {
            input.parse().map(ModuleItem::InitFn)
        } else if lookahead.peek(Token![struct]) {
            input
                .parse()
                .map(|mut s: ExportedType| {
                    s.is_pub = is_pub;
                    s
                })
                .map(ModuleItem::ExportedType)
        } else if lookahead.peek(Token![fn]) {
            input
                .parse()
                .map(|mut s: ExportedFunction| {
                    s.is_pub = is_pub;
                    s
                })
                .map(ModuleItem::ExportedFunction)
        } else if !is_pub && lookahead.peek(Token![in]) {
            input.parse().map(ModuleItem::ExportedMethod)
        } else if lookahead.peek(Token![const]) {
            input
                .parse()
                .map(|mut s: ExportedConst| {
                    s.is_pub = is_pub;
                    s
                })
                .map(ModuleItem::ExportedConst)
        } else if lookahead.peek(Token![type]) {
            input
                .parse()
                .map(|mut s: ExportedAlias| {
                    s.is_pub = is_pub;
                    s
                })
                .map(ModuleItem::ExportedAlias)
        } else if !is_pub && lookahead.peek(Token![#]) {
            input.parse().map(ModuleItem::ItemWithAttrs)
        } else if !is_pub && lookahead.peek(Token![for]) {
            input.parse().map(ModuleItem::ExportedGenerics)
        } else {
            Err(Error::new(
                input.span(),
                "Expected `become`, `fn`, `in`, `struct`, `const`, or `static`.",
            ))
        }
    }
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
            let mut module = ::jlrs::data::managed::module::Module::main(&frame);

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
                span=> if env.is_empty() {
                    <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::CCallReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                } else {
                    <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::CCallReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)
                }

            };
            let julia_ret_type = parse_quote_spanned! {
                span=> if env.is_empty() {
                    <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::FunctionReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                } else {
                    <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::FunctionReturnType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)
                }

            };

            (ccall_ret_type, julia_ret_type)
        }
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
                if env.is_empty() {
                    <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                } else {
                    <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)
                }
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
                if env.is_empty() {
                    <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type(&mut frame)
                } else {
                    <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, &env)
                }
            }
        });

    Ok((ccall_arg_types, julia_arg_types))
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
