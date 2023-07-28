use syn::{
    parse_quote, punctuated::Punctuated, FnArg, GenericArgument, Ident, PatType, Path,
    PathArguments, ReturnType, Type, TypePath,
};

use super::GenericEnvironment;

#[derive(Debug)]
pub(super) struct ParameterEnvironment<'a> {
    parameter: &'a Ident,
    paths: &'a Vec<&'a Path>,
    super_env: Option<&'a ParameterEnvironment<'a>>,
}

impl<'a> ParameterEnvironment<'a> {
    pub(super) fn new(
        generic_env: &'a GenericEnvironment<'a>,
        super_env: Option<&'a ParameterEnvironment<'a>>,
    ) -> Self {
        let parameter = generic_env.parameter;
        let paths = &generic_env.values;
        ParameterEnvironment {
            paths,
            parameter,
            super_env,
        }
    }

    pub(super) fn n_parameters(&self) -> usize {
        if let Some(env) = self.super_env {
            return 1 + env.n_parameters();
        }

        1
    }

    pub(super) fn n_combinations(&self) -> usize {
        if let Some(super_env) = self.super_env {
            return self.paths.len() * super_env.n_combinations();
        }

        self.paths.len()
    }

    pub(super) fn nth_combination(&self, list: &mut ParameterList<'a>, nth: usize) {
        list.clear_paths();
        self.nth_combination_inner(list, nth, 1);
    }

    fn nth_combination_inner(&self, list: &mut ParameterList<'a>, nth: usize, prod: usize) {
        let n_values = self.paths.len();
        let mth = (nth / prod) % n_values;

        let path = self.paths[mth];
        list.push_path(path);

        if let Some(env) = self.super_env {
            env.nth_combination_inner(list, nth, prod * n_values)
        }
    }
}

#[derive(Debug)]
pub(super) struct ParameterList<'a> {
    parameters: Vec<&'a Ident>,
    paths: Vec<&'a Path>,
}

impl<'a> ParameterList<'a> {
    pub(super) fn new(env: &'a ParameterEnvironment) -> Self {
        let n_params = env.n_parameters();
        let mut list = ParameterList {
            parameters: Vec::with_capacity(n_params),
            paths: Vec::with_capacity(n_params),
        };

        list.insert_parameters(env);
        list
    }

    pub(super) fn resolver(&self) -> ResolvedParameterList<'a> {
        ResolvedParameterList {
            parameters: self.parameters.clone(),
            paths: Vec::with_capacity(self.n_parameters()),
        }
    }

    pub(super) fn clear_paths(&mut self) {
        self.paths.clear();
    }

    pub(super) fn push_path(&mut self, path: &'a Path) {
        self.paths.push(path)
    }

    pub(super) fn resolve(&self, resolver: &mut ResolvedParameterList<'a>) {
        resolver.clear();
        let n_params = self.n_parameters();

        for i in 0..n_params {
            let mut path = self.paths[i].clone();
            for j in i + 1..n_params {
                apply_parameter(&mut path, self.parameters[j], self.paths[j]);
            }

            resolver.push(path);
        }
    }

    fn insert_parameters(&mut self, env: &'a ParameterEnvironment) {
        let parameter = env.parameter;
        self.parameters.push(parameter);

        if let Some(env) = env.super_env {
            self.insert_parameters(env);
        }
    }

    fn n_parameters(&self) -> usize {
        self.parameters.len()
    }
}

#[derive(Clone, Debug)]
pub(super) struct ResolvedParameterList<'a> {
    parameters: Vec<&'a Ident>,
    paths: Vec<Path>,
}

pub(super) trait Apply<To> {
    fn apply(&self, to: &To) -> To;
}

impl<'a> Apply<Path> for ResolvedParameterList<'a> {
    fn apply(&self, to: &Path) -> Path {
        self.assert_resolved();

        for (parameter, parameter_path) in self.parameters.iter().copied().zip(self.paths.iter()) {
            if to.is_ident(parameter) {
                return parameter_path.clone();
            }
        }

        let mut path = to.clone();
        for (parameter, parameter_path) in self.parameters.iter().copied().zip(self.paths.iter()) {
            apply_parameter(&mut path, parameter, parameter_path)
        }

        path
    }
}

impl<'a> Apply<Type> for ResolvedParameterList<'a> {
    fn apply(&self, to: &Type) -> Type {
        self.assert_resolved();

        match to {
            Type::Path(TypePath { path, .. }) => {
                for (parameter, parameter_path) in
                    self.parameters.iter().copied().zip(self.paths.iter())
                {
                    if path.is_ident(parameter) {
                        return Type::Path(TypePath {
                            path: parameter_path.clone(),
                            qself: None,
                        });
                    }
                }

                Type::Path(TypePath {
                    path: self.apply(path),
                    qself: None,
                })
            }
            _ => todo!(),
        }
    }
}

impl<'a> Apply<ReturnType> for ResolvedParameterList<'a> {
    fn apply(&self, to: &ReturnType) -> ReturnType {
        match to {
            ReturnType::Default => ReturnType::Default,
            ReturnType::Type(arr, ty) => {
                ReturnType::Type(arr.clone(), Box::new(self.apply(ty.as_ref())))
            }
        }
    }
}

impl<'a> Apply<PatType> for ResolvedParameterList<'a> {
    fn apply(&self, to: &PatType) -> PatType {
        PatType {
            attrs: to.attrs.clone(),
            pat: to.pat.clone(),
            colon_token: to.colon_token.clone(),
            ty: Box::new(self.apply(to.ty.as_ref())),
        }
    }
}

impl<'a> Apply<FnArg> for ResolvedParameterList<'a> {
    fn apply(&self, to: &FnArg) -> FnArg {
        match to {
            FnArg::Receiver(_) => todo!(),
            FnArg::Typed(pat) => FnArg::Typed(self.apply(pat)),
        }
    }
}
impl<'a, T, P> Apply<Punctuated<T, P>> for ResolvedParameterList<'a>
where
    Self: Apply<T>,
    P: Default,
{
    fn apply(&self, to: &Punctuated<T, P>) -> Punctuated<T, P> {
        to.iter().map(|arg| self.apply(arg)).collect()
    }
}

impl<'a> ResolvedParameterList<'a> {
    fn clear(&mut self) {
        self.paths.clear();
    }

    fn push(&mut self, path: Path) {
        self.paths.push(path);
    }

    fn assert_resolved(&self) {
        if self.parameters.len() != self.paths.len() {
            panic!("Parameters are unresolved")
        }
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

pub(super) fn as_return_as(ret_ty: &ReturnType) -> ReturnType {
    let mut new_ty = ret_ty.clone();

    if let ReturnType::Type(_, ty) = &mut new_ty {
        let new_ty: Type = parse_quote! {
            <#ty as ::jlrs::convert::ccall_types::CCallReturn>::ReturnAs
        };
        **ty = new_ty;
    }

    new_ty
}

pub(super) fn take_type(ty: ReturnType) -> Type {
    match ty {
        ReturnType::Default => parse_quote! { () },
        ReturnType::Type(_, ty) => *ty,
    }
}
