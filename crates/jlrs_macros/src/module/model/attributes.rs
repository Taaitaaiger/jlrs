use itertools::Itertools;
use syn::{AttrStyle, Attribute, Error, Expr, Lit, Meta, PatLit, Result, spanned::Spanned};

pub struct Documentation<'a> {
    _original: Vec<&'a Attribute>,
    lines: Vec<String>,
}

impl<'a> Documentation<'a> {
    pub fn from_attributes(attrs: Vec<&'a Attribute>) -> Result<Option<Self>> {
        if attrs.len() == 0 {
            return Ok(None);
        }

        let mut lines = Vec::with_capacity(attrs.len());
        for attr in &attrs {
            match &attr.meta {
                Meta::NameValue(nv) => match &nv.value {
                    Expr::Lit(PatLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) => {
                        debug_assert!(nv.path.is_ident("doc"));
                        lines.push(lit_str.value());
                    }
                    _ => Err(Error::new(
                        attr.span(),
                        "Unexpected literal in documentation",
                    ))?,
                },
                _ => unreachable!(),
            }
        }

        Ok(Some(Documentation {
            _original: attrs,
            lines,
        }))
    }

    pub fn to_string(&self) -> String {
        self.lines.iter().map(|line| if line.starts_with(' ') {
            &line[1..]
        } else {
            line
        }).join("\n")
    }
}

pub struct Attributes<'a> {
    pub attrs: Vec<&'a Attribute>,
    pub gc_safe: bool,
    pub untracked_self: bool,
    pub documentation: Option<Documentation<'a>>,
}

impl<'a> Attributes<'a> {
    pub fn from_attributes(attrs: &'a [Attribute]) -> Result<Self> {
        let inner_attributes: Vec<&Attribute> = attrs
            .into_iter()
            .map(|attr| match &attr.style {
                AttrStyle::Outer => Ok(attr),
                _ => Err(Error::new(attr.span(), "Unexpected inner attribute")),
            })
            .collect::<Result<_>>()?;

        let mut attrs = Vec::new();
        let mut doc_attrs = Vec::new();
        let mut gc_safe = false;
        let mut untracked_self = false;

        for attr in inner_attributes {
            match &attr.meta {
                Meta::NameValue(kv) if kv.path.is_ident("doc") => doc_attrs.push(attr),
                Meta::Path(p) if p.is_ident("gc_safe") => gc_safe = true,
                Meta::Path(p) if p.is_ident("untracked_self") => untracked_self = true,
                _ => attrs.push(attr),
            }
        }

        let documentation = Documentation::from_attributes(doc_attrs)?;

        Ok(Attributes {
            attrs,
            gc_safe,
            untracked_self,
            documentation,
        })
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::{
        ast::expanded::ExpandedModuleItem,
        module::{
            ast::{expanded::ExpandedModule, raw::JuliaModuleAst},
            model::attributes::Attributes,
        },
    };

    #[test]
    fn single_line_fn_doc() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            fn foo();
        };

        let expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match &expanded.items[0] {
            ExpandedModuleItem::Function(expanded_function) => {
                let attrs = Attributes::from_attributes(&expanded_function.attrs).unwrap();
                assert_eq!(attrs.attrs.len(), 0);
                assert!(!attrs.gc_safe);
                let lines = &attrs.documentation.as_ref().unwrap().lines;
                assert_eq!(lines.len(), 1);
                assert_eq!(lines[0].as_str(), " Foo");
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn single_line_gc_safe_fn_doc() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            #[gc_safe]
            /// Foo
            fn foo();
        };

        let expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match &expanded.items[0] {
            ExpandedModuleItem::Function(expanded_function) => {
                let attrs = Attributes::from_attributes(&expanded_function.attrs).unwrap();
                assert_eq!(attrs.attrs.len(), 0);
                assert!(attrs.gc_safe);
                let lines = &attrs.documentation.as_ref().unwrap().lines;
                assert_eq!(lines.len(), 1);
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn single_line_gc_safe_fn_doc_and_others() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            #[gc_safe]
            /// Foo
            #[egc_safe]
            fn foo();
        };

        let expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match &expanded.items[0] {
            ExpandedModuleItem::Function(expanded_function) => {
                let attrs = Attributes::from_attributes(&expanded_function.attrs).unwrap();
                assert_eq!(attrs.attrs.len(), 1);
                assert!(attrs.gc_safe);
                let lines = &attrs.documentation.as_ref().unwrap().lines;
                assert_eq!(lines.len(), 1);
            }
            _ => assert!(false),
        };
    }

    #[test]
    fn multi_line_doc() {
        let ast: JuliaModuleAst = parse_quote! {
            become init_fn;

            /// Foo
            /// Bar
            fn foo();
        };

        let expanded = ExpandedModule::from_ast(ast).unwrap();
        assert_eq!(expanded.items.len(), 1);

        match &expanded.items[0] {
            ExpandedModuleItem::Function(expanded_function) => {
                assert_eq!(expanded_function.attrs.len(), 2);
                let attrs = Attributes::from_attributes(&expanded_function.attrs).unwrap();
                assert_eq!(attrs.attrs.len(), 0);
                let lines = &attrs.documentation.as_ref().unwrap().lines;
                assert_eq!(lines.len(), 2);
            }
            _ => assert!(false),
        };
    }
}
