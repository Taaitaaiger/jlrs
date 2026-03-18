//! Expanded `ConstAst`-node

use syn::{Attribute, Ident, Result, Token, Type};

use crate::{
    ast::expanded::expanded_as::ExpandedAs,
    module::ast::{expanded::expanded_as::LocalName, raw::const_ast::ConstAst},
};

pub struct ExpandedConst {
    pub public: Option<Token![pub]>,
    pub name: Ident,
    pub attrs: Vec<Attribute>,
    pub ty: Type,
    pub name_override: Option<ExpandedAs>,
}

impl ExpandedConst {
    pub fn from_ast(
        ast: ConstAst,
        public: Option<Token![pub]>,
        attrs: Vec<Attribute>,
    ) -> Result<Self> {
        let ConstAst {
            const_token: _,
            name,
            colon_token: _,
            ty,
            name_override,
        } = ast;

        let name_override = name_override
            .map(LocalName::from_ast)
            .transpose()?
            .map(ExpandedAs::Local);

        Ok(ExpandedConst {
            public,
            name,
            attrs,
            ty,
            name_override,
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::module::ast::{expanded::expanded_const::ExpandedConst, raw::const_ast::ConstAst};

    #[test]
    fn expand_renamed_const() {
        let ast: ConstAst = parse_quote! {
            const FOO: usize as BAR
        };

        let expanded = ExpandedConst::from_ast(ast, None, vec![]);
        assert!(expanded.is_ok());
    }

    #[test]
    fn expand_renamed_const_global_name_error() {
        let ast: ConstAst = parse_quote! {
            const FOO: usize as Main.BAR
        };

        let expanded = ExpandedConst::from_ast(ast, None, vec![]);
        assert!(expanded.is_err());
    }
}
