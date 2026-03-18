//! `type <export_name> = <type_path>`

use quote::ToTokens;
use syn::{
    Ident, Result, Token, Type,
    parse::{Parse, ParseStream},
};

#[derive(Clone)]
pub struct TypeAst {
    pub type_token: Token![type],
    pub name: Ident,
    pub is: Token![=],
    pub ty: Type,
}

impl Parse for TypeAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_token = input.parse()?;
        let name = input.parse()?;
        let is = input.parse()?;
        let ty = input.parse()?;

        Ok(TypeAst {
            type_token,
            name,
            is,
            ty,
        })
    }
}

impl ToTokens for TypeAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.type_token.to_tokens(tokens);
        self.name.to_tokens(tokens);
        self.is.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::TypeAst;

    #[test]
    fn parse_alias() {
        let alias: TypeAst = parse_quote! { type Foo = Bar };
        assert_eq!(alias.name.to_string(), "Foo");
        match alias.ty {
            syn::Type::Path(type_path) => {
                let ty = type_path.path.get_ident().unwrap().to_string();
                assert_eq!(ty, "Bar")
            }
            _ => assert!(false),
        }
    }
}
