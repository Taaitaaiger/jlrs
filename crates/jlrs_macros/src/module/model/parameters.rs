use syn::{
    FnArg, GenericArgument, Ident, PatType, Path, PathArguments, Result, ReturnType, Signature,
    Type, TypePath, parse_quote, punctuated::Punctuated,
};

use crate::ast::expanded::environment::Environment;

#[derive(Clone, Debug)]
pub struct ResolvedParameterList<'a> {
    env: &'a Environment,
    parameters: Vec<&'a Ident>,
    paths: Vec<&'a Path>,
    n_combinations: usize,
}

impl<'a> ResolvedParameterList<'a> {
    pub fn new(env: &'a Environment) -> Self {
        let parameters = env.names().map(|n| n.unwrap()).collect::<Vec<_>>();
        let paths = Vec::with_capacity(parameters.len());
        let n_combinations = env.n_combinations();

        ResolvedParameterList {
            env,
            parameters,
            paths,
            n_combinations,
        }
    }

    fn assert_resolved(&self) {
        if self.parameters.len() != self.paths.len() {
            panic!("Parameters are unresolved")
        }
    }

    pub fn resolve(&mut self, i: usize) {
        debug_assert!(i < self.n_combinations);
        self.env.nth_combination(&mut self.paths, i);
    }
}

pub trait Apply<To> {
    fn apply(&self, to: &To) -> Result<To> {
        Self::apply_with_parent(&self, to, None)
    }

    fn apply_with_parent(&self, to: &To, parent: Option<&Path>) -> Result<To>;
}

impl<'a> Apply<Path> for ResolvedParameterList<'a> {
    fn apply_with_parent(&self, to: &Path, _parent: Option<&Path>) -> Result<Path> {
        self.assert_resolved();

        for (parameter, parameter_path) in self.parameters.iter().copied().zip(self.paths.iter()) {
            if to.is_ident(parameter) {
                return Ok((*parameter_path).clone());
            }
        }

        let mut path = to.clone();
        for (parameter, parameter_path) in self.parameters.iter().copied().zip(self.paths.iter()) {
            apply_parameter(&mut path, parameter, parameter_path)
        }

        Ok(path)
    }
}

impl<'a> Apply<Type> for ResolvedParameterList<'a> {
    fn apply_with_parent(&self, to: &Type, parent: Option<&Path>) -> Result<Type> {
        self.assert_resolved();

        match to {
            Type::Path(TypePath { path, .. }) => {
                for (parameter, parameter_path) in
                    self.parameters.iter().copied().zip(self.paths.iter())
                {
                    if path.is_ident(parameter) {
                        return Ok(Type::Path(TypePath {
                            path: (*parameter_path).clone(),
                            qself: None,
                        }));
                    }
                }

                Ok(Type::Path(TypePath {
                    path: self.apply_with_parent(path, parent)?,
                    qself: None,
                }))
            }
            _ => todo!(),
        }
    }
}

impl<'a> Apply<ReturnType> for ResolvedParameterList<'a> {
    fn apply_with_parent(&self, to: &ReturnType, parent: Option<&Path>) -> Result<ReturnType> {
        match to {
            ReturnType::Default => Ok(ReturnType::Default),
            ReturnType::Type(arr, ty) => Ok(ReturnType::Type(
                arr.clone(),
                Box::new(self.apply_with_parent(ty.as_ref(), parent)?),
            )),
        }
    }
}

impl<'a> Apply<PatType> for ResolvedParameterList<'a> {
    fn apply_with_parent(&self, to: &PatType, parent: Option<&Path>) -> Result<PatType> {
        Ok(PatType {
            attrs: to.attrs.clone(),
            pat: to.pat.clone(),
            colon_token: to.colon_token.clone(),
            ty: Box::new(self.apply_with_parent(to.ty.as_ref(), parent)?),
        })
    }
}

impl<'a> Apply<FnArg> for ResolvedParameterList<'a> {
    fn apply_with_parent(&self, to: &FnArg, parent: Option<&Path>) -> Result<FnArg> {
        match to {
            FnArg::Receiver(receiver) => {
                assert!(parent.is_some());
                let mutability = receiver.mutability.as_ref();
                Ok(
                    parse_quote! { #mutability this: ::jlrs::data::managed::value::typed::TypedValue<#parent> },
                )
            }
            FnArg::Typed(pat) => Ok(FnArg::Typed(self.apply_with_parent(pat, parent)?)),
        }
    }
}

impl<'a, T, P> Apply<Punctuated<T, P>> for ResolvedParameterList<'a>
where
    Self: Apply<T>,
    P: Default,
{
    fn apply_with_parent(
        &self,
        to: &Punctuated<T, P>,
        parent: Option<&Path>,
    ) -> Result<Punctuated<T, P>> {
        to.iter()
            .map(|arg| self.apply_with_parent(arg, parent))
            .collect()
    }
}

impl<'a> Apply<Signature> for ResolvedParameterList<'a> {
    fn apply_with_parent(&self, to: &Signature, parent: Option<&Path>) -> Result<Signature> {
        let inputs = self.apply_with_parent(&to.inputs, parent)?;
        let output = self.apply_with_parent(&to.output, parent)?;

        Ok(Signature {
            inputs,
            output,
            ..to.clone()
        })
    }
}

fn apply_parameter(path: &mut Path, parameter: &Ident, parameter_path: &Path) {
    if path.is_ident(parameter) {
        *path = parameter_path.clone();
        return;
    }

    let segment = path.segments.last_mut().unwrap();
    if let PathArguments::AngleBracketed(bracketed) = &mut segment.arguments {
        for arg in bracketed.args.iter_mut() {
            if let GenericArgument::Type(Type::Path(ty)) = arg {
                apply_parameter(&mut ty.path, parameter, parameter_path)
            }
        }
    }
}
