//! Raw AST of `julia_module!`
//!
//! The content of `julia_module` is parsed as a list of `ModuleItem`s, separated by semicolons.
//! Each item is identified by a keyword, an item's AST type is generally named `<keyword>Ast`.

use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::module::ast::raw::{
    become_ast::BecomeAst, const_ast::ConstAst, fn_ast::FnAst, for_ast::ForAst, in_ast::InAst,
    pub_ast::PubAst, struct_ast::StructAst, type_ast::TypeAst, with_attributes::WithAttributes,
};

pub mod as_ast;
pub mod become_ast;
pub mod const_ast;
pub mod fn_ast;
pub mod for_ast;
pub mod in_ast;
pub mod pub_ast;
pub mod struct_ast;
pub mod type_ast;
pub mod use_ast;
pub mod with_attributes;

pub enum ModuleItem {
    Const(ConstAst),
    Fn(FnAst),
    In(InAst),
    Struct(StructAst),
    Type(TypeAst),
    For(ForAst),
    Pub(PubAst),
    WithAttributes(WithAttributes),
}

#[cfg(test)]
impl ModuleItem {
    fn is_const(&self) -> bool {
        match self {
            ModuleItem::Const(_) => true,
            _ => false,
        }
    }

    fn is_fn(&self) -> bool {
        match self {
            ModuleItem::Fn(_) => true,
            _ => false,
        }
    }

    fn is_in(&self) -> bool {
        match self {
            ModuleItem::In(_) => true,
            _ => false,
        }
    }

    fn is_struct(&self) -> bool {
        match self {
            ModuleItem::Struct(_) => true,
            _ => false,
        }
    }

    fn is_type(&self) -> bool {
        match self {
            ModuleItem::Type(_) => true,
            _ => false,
        }
    }

    fn is_for(&self) -> bool {
        match self {
            ModuleItem::For(_) => true,
            _ => false,
        }
    }

    fn is_pub(&self) -> bool {
        match self {
            ModuleItem::Pub(_) => true,
            _ => false,
        }
    }

    fn is_with_attributes(&self) -> bool {
        match self {
            ModuleItem::WithAttributes(_) => true,
            _ => false,
        }
    }
}

impl Parse for ModuleItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![fn]) {
            let ast = input.parse()?;
            Ok(ModuleItem::Fn(ast))
        } else if lookahead.peek(Token![in]) {
            let ast = input.parse()?;
            Ok(ModuleItem::In(ast))
        } else if lookahead.peek(Token![struct]) {
            let ast = input.parse()?;
            Ok(ModuleItem::Struct(ast))
        } else if lookahead.peek(Token![pub]) {
            let ast = input.parse()?;
            Ok(ModuleItem::Pub(ast))
        } else if lookahead.peek(Token![for]) {
            let ast = input.parse()?;
            Ok(ModuleItem::For(ast))
        } else if lookahead.peek(Token![const]) {
            let ast = input.parse()?;
            Ok(ModuleItem::Const(ast))
        } else if lookahead.peek(Token![type]) {
            let ast = input.parse()?;
            Ok(ModuleItem::Type(ast))
        } else if lookahead.peek(Token![#]) {
            let ast = input.parse()?;
            Ok(ModuleItem::WithAttributes(ast))
        } else {
            Err(input.error("Cannot parse module item; expected `fn`, `in`, `struct`, `pub`, `for`, `const` or `type`."))
        }
    }
}

pub struct JuliaModuleAst {
    pub init_fn: BecomeAst,
    pub _semicolon: Token![;],
    pub items: Punctuated<ModuleItem, Token![;]>,
}

impl Parse for JuliaModuleAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let init_fn = input.parse()?;
        let semicolon = input.parse()?;
        let items = Punctuated::parse_terminated(input)?;
        Ok(JuliaModuleAst {
            init_fn,
            _semicolon: semicolon,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::{JuliaModuleAst, ModuleItem};

    #[test]
    fn parse_fn_module_item() {
        let ast: ModuleItem = parse_quote! { fn foo() };
        assert!(ast.is_fn())
    }

    #[test]
    fn parse_in_module_item() {
        let ast: ModuleItem = parse_quote! { in Foo fn foo() };
        assert!(ast.is_in())
    }

    #[test]
    fn parse_struct_module_item() {
        let ast: ModuleItem = parse_quote! { struct Foo };
        assert!(ast.is_struct())
    }

    #[test]
    fn parse_pub_module_item() {
        let ast: ModuleItem = parse_quote! { pub struct Foo };
        assert!(ast.is_pub())
    }

    #[test]
    fn parse_const_module_item() {
        let ast: ModuleItem = parse_quote! { const FOO: usize };
        assert!(ast.is_const())
    }

    #[test]
    fn parse_type_module_item() {
        let ast: ModuleItem = parse_quote! { type Bar = Baz };
        assert!(ast.is_type())
    }

    #[test]
    fn parse_for_module_item() {
        let ast: ModuleItem = parse_quote! { for T in [] {} };
        assert!(ast.is_for())
    }

    #[test]
    fn parse_module_item_with_attributes() {
        let ast: ModuleItem = parse_quote! {
           #[foo]
           pub type Bar = Baz
        };
        assert!(ast.is_with_attributes())
    }

    #[test]
    fn parse_module() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;
            fn foo();
        };
        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn parse_module_two_items() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;
            const FOO: usize;
            pub fn foo();
        };
        assert_eq!(ast.items.len(), 2);
    }
}
