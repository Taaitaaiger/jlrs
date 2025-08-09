use syn::{
    braced, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Bracket},
    Ident, Path, Result, Token,
};

use super::{generics::GenericEnvironment, ModuleItem};

pub struct ExportedGenerics {
    pub _for: Token![for],
    pub type_param: Ident,
    pub _in: Token![in],
    pub _bracket: Bracket,
    pub types: Punctuated<Path, Token![,]>,
    pub _brace: Brace,
    pub items: Punctuated<ModuleItem, Token![;]>,
}

impl ExportedGenerics {
    pub fn to_generic_environment(&self) -> GenericEnvironment<'_> {
        GenericEnvironment::new(self)
    }
}

impl Parse for ExportedGenerics {
    fn parse(input: ParseStream) -> Result<Self> {
        let for_token = input.parse()?;
        let type_param = input.parse()?;
        let in_token = input.parse()?;

        let content;
        let bracket = bracketed!(content in input);
        let types = content.parse_terminated(Path::parse, Token![,])?;

        let content;
        let brace = braced!(content in input);
        let items = content.parse_terminated(ModuleItem::parse, Token![;])?;

        Ok(ExportedGenerics {
            _for: for_token,
            type_param,
            _in: in_token,
            _bracket: bracket,
            types,
            _brace: brace,
            items,
        })
    }
}
