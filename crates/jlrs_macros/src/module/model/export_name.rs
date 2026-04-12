use quote::ToTokens;
use syn::Ident;

use crate::module::ast::expanded::expanded_as::ExpandedAs;

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum ExportName<'a> {
    Name(&'a Ident),
    Override(&'a ExpandedAs),
}

impl<'a> ExportName<'a> {
    pub fn from_name_or_local(name: &'a Ident, name_override: Option<&'a ExpandedAs>) -> Self {
        match name_override {
            Some(name) => ExportName::Override(name),
            None => ExportName::Name(name),
        }
    }

    pub fn from_name_or_override(name: &'a Ident, name_override: Option<&'a ExpandedAs>) -> Self {
        match name_override {
            Some(name) => ExportName::Override(name),
            None => ExportName::Name(name),
        }
    }

    pub fn from_name(name: &'a ExpandedAs) -> Self {
        ExportName::Override(name)
    }

    pub fn name_string(&self) -> String {
        match self {
            ExportName::Name(ident) => ident.to_string(),
            ExportName::Override(name_override) => name_override.name_string(),
        }
    }
}

impl<'a> ToTokens for ExportName<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ExportName::Name(ident) => ident.to_tokens(tokens),
            ExportName::Override(name_override) => name_override.to_tokens(tokens),
        }
    }
}
