//! `const <export_name> = <rust_global>`

use quote::ToTokens;
use syn::{Ident, Token, Type, parse::Parse};

use crate::module::ast::raw::as_ast::AsAst;

pub struct ConstAst {
    pub const_token: Token![const],
    pub name: Ident,
    pub colon_token: Token![:],
    pub ty: Type,
    pub name_override: Option<AsAst>,
}

impl Parse for ConstAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let const_token = input.parse()?;
        let name = input.parse()?;
        let colon_token = input.parse()?;
        let ty = input.parse()?;
        let name_override = input
            .lookahead1()
            .peek(Token![as])
            .then(|| input.parse())
            .transpose()?;

        Ok(ConstAst {
            const_token,
            name,
            colon_token,
            ty,
            name_override,
        })
    }
}

impl ToTokens for ConstAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.const_token.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        self.name_override.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::ConstAst;

    #[test]
    fn parse_const() {
        let const_ast: ConstAst = parse_quote! { const FOO: usize };
        assert_eq!(const_ast.name.to_string(), "FOO");
        match const_ast.ty {
            syn::Type::Path(type_path) => {
                let ty = type_path.path.get_ident().unwrap().to_string();
                assert_eq!(ty, "usize")
            }
            _ => assert!(false),
        }
        assert!(const_ast.name_override.is_none())
    }

    #[test]
    fn parse_renamed_const() {
        let const_ast: ConstAst = parse_quote! { const FOO: usize as BAR };
        assert_eq!(const_ast.name.to_string(), "FOO");
        match const_ast.ty {
            syn::Type::Path(type_path) => {
                let ty = type_path.path.get_ident().unwrap().to_string();
                assert_eq!(ty, "usize")
            }
            _ => assert!(false),
        }
        assert!(const_ast.name_override.is_some())
    }
}
