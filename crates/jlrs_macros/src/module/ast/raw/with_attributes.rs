//! #<attrs...> <attributed_item>

use syn::{
    Attribute, Result, Token,
    parse::{Parse, ParseStream},
};

use crate::module::ast::raw::{
    const_ast::ConstAst, fn_ast::FnAst, in_ast::InAst, pub_ast::PubAst, struct_ast::StructAst,
    type_ast::TypeAst,
};

pub enum Attributed {
    Const(ConstAst),
    Fn(FnAst),
    In(InAst),
    Struct(StructAst),
    Pub(PubAst),
    Type(TypeAst),
}

impl Parse for Attributed {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![fn]) {
            let ast = input.parse()?;
            Ok(Attributed::Fn(ast))
        } else if lookahead.peek(Token![in]) {
            let ast = input.parse()?;
            Ok(Attributed::In(ast))
        } else if lookahead.peek(Token![struct]) {
            let ast = input.parse()?;
            Ok(Attributed::Struct(ast))
        } else if lookahead.peek(Token![pub]) {
            let ast = input.parse()?;
            Ok(Attributed::Pub(ast))
        } else if lookahead.peek(Token![const]) {
            let ast = input.parse()?;
            Ok(Attributed::Const(ast))
        } else if lookahead.peek(Token![type]) {
            let ast = input.parse()?;
            Ok(Attributed::Type(ast))
        } else {
            Err(input.error("Cannot parse attributed item; expected `fn`, `in`, `struct`, `pub`, `const`, or `type`."))
        }
    }
}

pub struct WithAttributes {
    pub attributes: Vec<Attribute>,
    pub item: Attributed,
}

impl Parse for WithAttributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let item: Attributed = input.parse()?;
        Ok(WithAttributes { attributes, item })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::WithAttributes;

    #[test]
    fn parse_attributed_const() {
        let ast: WithAttributes = parse_quote! {
            #[foo]
            const FOO: Foo
        };
        assert_eq!(ast.attributes.len(), 1);
    }

    #[test]
    fn parse_attributed_in() {
        let ast: WithAttributes = parse_quote! {
            #[foo]
            in Foo fn foo()
        };
        assert_eq!(ast.attributes.len(), 1);
    }

    #[test]
    fn parse_attributed_fn() {
        let ast: WithAttributes = parse_quote! {
            #[foo]
            fn foo()
        };
        assert_eq!(ast.attributes.len(), 1);
    }

    #[test]
    fn parse_attributed_struct() {
        let ast: WithAttributes = parse_quote! {
            #[foo]
            struct Foo
        };
        assert_eq!(ast.attributes.len(), 1);
    }

    #[test]
    fn parse_attributed_type() {
        let ast: WithAttributes = parse_quote! {
            #[foo]
            type Foo = Bar
        };
        assert_eq!(ast.attributes.len(), 1);
    }

    #[test]
    fn parse_attributed_pub() {
        let ast: WithAttributes = parse_quote! {
            #[foo]
            pub type Foo = Bar
        };
        assert_eq!(ast.attributes.len(), 1);
    }
}
