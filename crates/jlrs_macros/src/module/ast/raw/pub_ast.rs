//! `pub <pub_item>`

use quote::ToTokens;
use syn::{
    Result, Token,
    parse::{Parse, ParseStream},
};

use crate::module::ast::raw::{
    const_ast::ConstAst, fn_ast::FnAst, struct_ast::StructAst, type_ast::TypeAst,
};

pub enum PubItem {
    Const(ConstAst),
    Fn(FnAst),
    Struct(StructAst),
    Type(TypeAst),
}

#[cfg(test)]
impl PubItem {
    fn is_const(&self) -> bool {
        match self {
            PubItem::Const(_) => true,
            _ => false,
        }
    }

    fn is_fn(&self) -> bool {
        match self {
            PubItem::Fn(_) => true,
            _ => false,
        }
    }

    fn is_struct(&self) -> bool {
        match self {
            PubItem::Struct(_) => true,
            _ => false,
        }
    }

    fn is_type(&self) -> bool {
        match self {
            PubItem::Type(_) => true,
            _ => false,
        }
    }
}

impl Parse for PubItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![const]) {
            let ast: ConstAst = input.parse()?;
            Ok(PubItem::Const(ast))
        } else if lookahead.peek(Token![fn]) {
            let ast: FnAst = input.parse()?;
            Ok(PubItem::Fn(ast))
        } else if lookahead.peek(Token![struct]) {
            let ast: StructAst = input.parse()?;
            Ok(PubItem::Struct(ast))
        } else if lookahead.peek(Token![type]) {
            let ast: TypeAst = input.parse()?;
            Ok(PubItem::Type(ast))
        } else if lookahead.peek(Token![in]) {
            Err(input.error("Exported public method syntax is `in <type> pub <signature>"))
        } else {
            Err(input.error("Cannot parse pub item; expected `const`, `fn`, `struct`, or `type`."))
        }
    }
}

pub struct PubAst {
    pub pub_token: Token![pub],
    pub item: PubItem,
}

impl Parse for PubAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let pub_token = input.parse()?;
        let item = input.parse()?;

        Ok(PubAst { pub_token, item })
    }
}

impl ToTokens for PubAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.pub_token.to_tokens(tokens);
        match &self.item {
            PubItem::Const(const_ast) => const_ast.to_tokens(tokens),
            PubItem::Fn(fn_ast) => fn_ast.to_tokens(tokens),
            PubItem::Struct(struct_ast) => struct_ast.to_tokens(tokens),
            PubItem::Type(type_ast) => type_ast.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::PubAst;

    #[test]
    fn parse_pub_const() {
        let ast: PubAst = parse_quote! { pub const FOO: usize };
        assert!(ast.item.is_const())
    }

    #[test]
    fn parse_pub_fn() {
        let ast: PubAst = parse_quote! { pub fn foo() };
        assert!(ast.item.is_fn())
    }

    #[test]
    fn parse_pub_struct() {
        let ast: PubAst = parse_quote! { pub struct Foo };
        assert!(ast.item.is_struct())
    }

    #[test]
    fn parse_pub_type() {
        let ast: PubAst = parse_quote! { pub type Foo = Bar };
        assert!(ast.item.is_type())
    }
}
