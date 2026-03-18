//! Expanded `AliasAst`-node

use syn::{Attribute, Token, Type};

use crate::{
    ast::expanded::expanded_as::ExpandedAs,
    module::ast::{expanded::expanded_as::LocalName, raw::type_ast::TypeAst},
};

#[derive(Clone)]
pub struct ExpandedAlias {
    pub public: Option<Token![pub]>,
    pub name: ExpandedAs,
    pub attrs: Vec<Attribute>,
    pub ty: Type,
}

impl ExpandedAlias {
    pub fn from_ast(ast: TypeAst, public: Option<Token![pub]>, attrs: Vec<Attribute>) -> Self {
        let TypeAst {
            type_token: _type_token,
            name,
            is: _is,
            ty,
        } = ast;

        ExpandedAlias {
            public,
            name: ExpandedAs::Local(LocalName::from_ident(name)),
            attrs,
            ty,
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::module::ast::{expanded::expanded_alias::ExpandedAlias, raw::type_ast::TypeAst};

    #[test]
    fn expand_alias() {
        let ast: TypeAst = parse_quote! { type Foo = Bar };
        let expanded = ExpandedAlias::from_ast(ast, None, vec![]);
        assert!(expanded.public.is_none());
    }
}
