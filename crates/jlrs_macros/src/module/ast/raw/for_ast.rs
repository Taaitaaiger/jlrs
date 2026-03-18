//! `for <ident> in [<type_paths>] { <generic_items>; }`

use syn::{
    Ident, Path, Result, Token, braced, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Bracket},
};

use crate::module::ast::raw::{
    fn_ast::FnAst, in_ast::InAst, pub_ast::PubAst, struct_ast::StructAst,
    with_attributes::WithAttributes,
};

pub enum ForItemInner {
    Fn(FnAst),
    In(InAst),
    Struct(StructAst),
    Pub(PubAst),
    WithAttributes(WithAttributes),
}

#[cfg(test)]
impl ForItemInner {
    fn is_fn(&self) -> bool {
        match self {
            ForItemInner::Fn(_) => true,
            _ => false,
        }
    }

    fn is_in(&self) -> bool {
        match self {
            ForItemInner::In(_) => true,
            _ => false,
        }
    }

    fn is_struct(&self) -> bool {
        match self {
            ForItemInner::Struct(_) => true,
            _ => false,
        }
    }

    fn is_pub(&self) -> bool {
        match self {
            ForItemInner::Pub(_) => true,
            _ => false,
        }
    }
}

impl Parse for ForItemInner {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![fn]) {
            let ast = input.parse()?;
            Ok(ForItemInner::Fn(ast))
        } else if lookahead.peek(Token![in]) {
            let ast = input.parse()?;
            Ok(ForItemInner::In(ast))
        } else if lookahead.peek(Token![struct]) {
            let ast = input.parse()?;
            Ok(ForItemInner::Struct(ast))
        } else if lookahead.peek(Token![pub]) {
            let ast = input.parse()?;
            Ok(ForItemInner::Pub(ast))
        } else if lookahead.peek(Token![#]) {
            let ast = input.parse()?;
            Ok(ForItemInner::WithAttributes(ast))
        } else {
            Err(input.error("Cannot parse iterated item; expected `fn`, `in`, or `struct`."))
        }
    }
}

pub enum ForItem {
    Entry(ForItemInner),
    Nested(ForAst),
}

impl Parse for ForItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![for]) {
            let ast = input.parse()?;
            Ok(ForItem::Nested(ast))
        } else {
            let inner = input.parse()?;
            Ok(ForItem::Entry(inner))
        }
    }
}

pub struct ForAst {
    pub _for_token: Token![for],
    pub type_param: Ident,
    pub _in_token: Token![in],
    pub _bracket: Bracket,
    pub types: Punctuated<Path, Token![,]>,
    pub _brace: Brace,
    pub items: Punctuated<ForItem, Token![;]>,
}

impl Parse for ForAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let for_token = input.parse()?;
        let type_param = input.parse()?;
        let in_token = input.parse()?;

        let content;
        let bracket = bracketed!(content in input);
        let types = content.parse_terminated(Path::parse, Token![,])?;

        let content;
        let brace = braced!(content in input);
        let items = content.parse_terminated(ForItem::parse, Token![;])?;

        Ok(ForAst {
            _for_token: for_token,
            type_param,
            _in_token: in_token,
            _bracket: bracket,
            types,
            _brace: brace,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::{ForAst, ForItem, ForItemInner};

    #[test]
    fn parse_fn_inner() {
        let ast: ForItemInner = parse_quote! { fn foo() };
        assert!(ast.is_fn())
    }

    #[test]
    fn parse_in_inner() {
        let ast: ForItemInner = parse_quote! { in Foo fn foo() };
        assert!(ast.is_in())
    }

    #[test]
    fn parse_struct_inner() {
        let ast: ForItemInner = parse_quote! { struct Foo };
        assert!(ast.is_struct())
    }

    #[test]
    fn parse_item() {
        let ast: ForItem = parse_quote! { struct Foo };
        match ast {
            ForItem::Entry(entry) => {
                assert!(entry.is_struct());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_pub_item() {
        let ast: ForItem = parse_quote! { pub struct Foo };
        match ast {
            ForItem::Entry(entry) => {
                assert!(entry.is_pub());
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn parse_for_ast_one() {
        let ast: ForAst = parse_quote! {
            for T in [f64, f32] {
                fn foo<T>(t: T)
            }
        };

        assert_eq!(ast.types.len(), 2);
        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn parse_for_ast_two() {
        let ast: ForAst = parse_quote! {
            for T in [f64, f32] {
                fn foo<T>(t: T);
                fn bar<T>(t: T);
            }
        };

        assert_eq!(ast.types.len(), 2);
        assert_eq!(ast.items.len(), 2);
    }
}
