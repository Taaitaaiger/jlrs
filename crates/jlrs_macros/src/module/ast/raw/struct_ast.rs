//! `struct <path> [<as_ast>]`

use quote::ToTokens;
use syn::{
    Path, Result, Token,
    parse::{Parse, ParseStream},
};

use crate::module::ast::raw::as_ast::AsAst;

#[derive(Clone)]
pub struct StructAst {
    pub struct_token: Token![struct],
    pub path: Path,
    pub name_override: Option<AsAst>,
}

impl Parse for StructAst {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_token = input.parse()?;
        let path = input.parse()?;
        let name_override = input
            .lookahead1()
            .peek(Token![as])
            .then(|| input.parse())
            .transpose()?;

        Ok(StructAst {
            struct_token,
            path,
            name_override,
        })
    }
}

impl ToTokens for StructAst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.struct_token.to_tokens(tokens);
        self.path.to_tokens(tokens);
        self.name_override.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::StructAst;

    #[test]
    fn parse_struct() {
        let ast: StructAst = parse_quote! { struct Foo };
        assert_eq!(ast.path.get_ident().unwrap().to_string(), "Foo");
        assert!(ast.name_override.is_none())
    }

    #[test]
    fn parse_renamed_struct() {
        let ast: StructAst = parse_quote! { struct Foo as Bar };
        assert_eq!(ast.path.get_ident().unwrap().to_string(), "Foo");
        assert_eq!(
            ast.name_override
                .unwrap()
                .name_override
                .first()
                .unwrap()
                .to_string(),
            "Bar"
        );
    }

    #[test]
    fn parse_generic_struct() {
        let ast: StructAst = parse_quote! { struct Foo<T> };
        assert_eq!(ast.path.segments.len(), 1);
        let segment = ast.path.segments.first().unwrap();
        assert!(!segment.arguments.is_empty());
        assert!(ast.name_override.is_none())
    }

    #[test]
    fn parse_renamed_generic_struct() {
        let ast: StructAst = parse_quote! { struct Foo<T> as Bar };
        assert_eq!(ast.path.segments.len(), 1);
        let segment = ast.path.segments.first().unwrap();
        assert!(!segment.arguments.is_empty());
        assert_eq!(
            ast.name_override
                .unwrap()
                .name_override
                .first()
                .unwrap()
                .to_string(),
            "Bar"
        );
    }

    #[test]
    fn parse_struct_with_multiple_path_segments() {
        let ast: StructAst = parse_quote! { struct crate::foo::Foo<<[f32; 1] as Iterator>::Item> };
        assert_eq!(ast.path.segments.len(), 3);
        let segment = ast.path.segments.last().unwrap();
        assert!(!segment.arguments.is_empty());
        assert!(ast.name_override.is_none())
    }
}
