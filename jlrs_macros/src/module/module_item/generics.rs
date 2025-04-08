use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Expr, Ident, Macro, Path, Result, Token,
};

use super::{exported_generics::ExportedGenerics, has_outer_path_attr, ModuleItem};
use crate::module::ParameterEnvironment;

#[derive(Debug)]
pub enum MacroOrType {
    Macro(Macro),
    Type(Ident),
}
impl Parse for MacroOrType {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        if let Ok(m) = input.parse() {
            Ok(MacroOrType::Macro(m))
        } else {
            let t = fork.parse()?;
            Ok(MacroOrType::Type(t))
        }
    }
}

#[derive(Debug)]
pub struct TypeVarEnv {
    _use_token: Option<Token![use]>,
    pub macro_or_type: MacroOrType,
}

impl Parse for TypeVarEnv {
    fn parse(input: ParseStream) -> Result<Self> {
        let use_token = input.parse()?;
        let macro_or_type = input.parse()?;

        Ok(Self {
            _use_token: use_token,
            macro_or_type,
        })
    }
}

pub struct GenericEnvironment<'a> {
    pub parameter: &'a Ident,
    pub values: Vec<&'a Path>,
    pub items: Vec<&'a ModuleItem>,
    pub subenvs: Vec<GenericEnvironment<'a>>,
}

impl<'a> GenericEnvironment<'a> {
    pub fn new(generics: &'a ExportedGenerics) -> Self {
        let parameter = &generics.type_param;
        let values: Vec<_> = generics.types.iter().collect();

        let items: Vec<_> = generics
            .items
            .iter()
            .filter(|f| f.is_exported_fn() || f.is_exported_method() || f.is_exported_type())
            .collect();

        let subenvs: Vec<_> = generics
            .items
            .iter()
            .filter(|f| f.is_exported_generics())
            .map(|f| f.get_exported_generics())
            .map(GenericEnvironment::new)
            .collect();

        GenericEnvironment {
            parameter,
            values,
            items,
            subenvs,
        }
    }

    pub fn init_type_fragments(&self) -> impl Iterator<Item = Expr> {
        let mut out = vec![];
        self.init_type_fragments_env(None, &mut out);
        out.into_iter()
    }

    pub fn init_type_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        out: &mut Vec<Expr>,
    ) {
        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            sub_env.init_type_fragments_env(Some(&env), out);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_type())
            .map(|it| it.get_exported_type())
            .map(|it| it.init_with_env(self, env));

        out.extend(exprs);
    }

    pub fn reinit_type_fragments(&self) -> impl Iterator<Item = Expr> {
        let mut out = vec![];
        self.reinit_type_fragments_env(None, &mut out);
        out.into_iter()
    }

    pub fn reinit_type_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        out: &mut Vec<Expr>,
    ) {
        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            sub_env.reinit_type_fragments_env(Some(&env), out);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_type())
            .map(|it| it.get_exported_type())
            .map(|it| it.reinit_with_env(self, env));

        out.extend(exprs);
    }

    pub fn init_function_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        offset: &mut usize,
    ) -> Result<Expr> {
        let mut sup_exprs = vec![];

        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            let ex = sub_env.init_function_fragments_env(Some(&env), offset)?;
            sup_exprs.push(ex);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_fn())
            .map(|it| it.get_exported_fn())
            .map(|it| {
                let mut gc_safe = false;
                if let Some(attrs) = it.1 {
                    gc_safe = has_outer_path_attr(attrs, "gc_safe");
                }
                it.0.init_with_env(self, env, offset, gc_safe)
            })
            .collect::<Result<Vec<_>>>()?;

        let ex = parse_quote! {
            {
                #(#sup_exprs;)*
                #(#exprs;)*
            }
        };

        Ok(ex)
    }

    pub fn init_method_fragments_env(
        &'a self,
        env: Option<&ParameterEnvironment<'a>>,
        offset: &mut usize,
    ) -> Result<Expr> {
        let mut sup_exprs = vec![];

        for sub_env in self.subenvs.iter() {
            let env = ParameterEnvironment::new(self, env);
            let ex = sub_env.init_method_fragments_env(Some(&env), offset)?;
            sup_exprs.push(ex);
        }

        let exprs = self
            .items
            .iter()
            .copied()
            .filter(|it| it.is_exported_method())
            .map(|it| it.get_exported_method())
            .map(|it| {
                let mut untracked_self = false;
                let mut gc_safe = false;
                if let Some(attrs) = it.1 {
                    untracked_self = has_outer_path_attr(attrs, "untracked_self");
                    gc_safe = has_outer_path_attr(attrs, "gc_safe");
                }
                it.0.init_with_env(self, env, offset, untracked_self, gc_safe)
            }) // TODO: attrs
            .collect::<Result<Vec<_>>>()?;

        let ex = parse_quote! {
            {
                #(#sup_exprs;)*
                #(#exprs;)*
            }
        };

        Ok(ex)
    }
}
