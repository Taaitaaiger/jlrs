use syn::{
    parse::{Parse, ParseStream},
    AttrStyle, Attribute, Expr, Lit, Meta, PatLit as ExprLit, Result,
};

use super::ModuleItem;

pub struct ItemWithAttrs {
    pub attrs: Vec<Attribute>,
    pub item: Box<ModuleItem>,
}

impl ItemWithAttrs {
    pub fn has_docstr(&self) -> bool {
        for attr in self.attrs.iter() {
            match attr.style {
                AttrStyle::Outer => (),
                _ => continue,
            }

            match &attr.meta {
                Meta::NameValue(kv) => {
                    if kv.path.is_ident("doc") {
                        return true;
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };
        }

        false
    }

    pub fn get_docstr(&self) -> Result<String> {
        let mut doc = String::new();
        for attr in self.attrs.iter() {
            match attr.style {
                AttrStyle::Outer => (),
                _ => continue,
            }

            let line = match &attr.meta {
                Meta::NameValue(kv) => {
                    if kv.path.is_ident("doc") {
                        match &kv.value {
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(s), ..
                            }) => s.value(),
                            _ => continue,
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };

            match doc.len() {
                0 => doc.push_str(&line),
                _ => {
                    doc.push('\n');
                    doc.push_str(&line);
                }
            }
        }

        Ok(doc)
    }
}

impl Parse for ItemWithAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let attr: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let item: ModuleItem = input.parse()?;
        Ok(ItemWithAttrs {
            attrs: attr,
            item: Box::new(item),
        })
    }
}
