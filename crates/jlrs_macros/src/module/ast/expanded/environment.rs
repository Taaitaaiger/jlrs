//! Expanded environment
//!
//! The environment contains all requested combinations of generic parameters and is generated
//! from `ForAst` nodes. Generics from outer nodes may be used in inner nodes.
//!
//! For example, if the environment is defined by `for T in [f32, f64] { for U in [T, i32] }`, the
//! expanded environment will contain the following combinations:
//!
//!  - `T = f32, U = f32`
//!  - `T = f32, U = f64`
//!  - `T = f32, U = i32`
//!  - `T = f64, U = f32`
//!  - `T = f64, U = f64`
//!  - `T = f64, U = i32`

use itertools::Itertools;
use syn::{
    AngleBracketedGenericArguments, Ident, Path, PathSegment, Type, TypePath,
    punctuated::Punctuated,
};

use crate::ast::raw::for_ast::ForAst;

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: Ident,
    pub types: Vec<Path>,
}

impl Parameter {
    pub fn from_for_ast(for_ast: &ForAst) -> Self {
        let name = for_ast.type_param.clone();
        let types = for_ast.types.iter().cloned().unique().collect();

        Parameter { name, types }
    }

    pub fn into_environment(self) -> Environment {
        Environment {
            parameters: vec![self],
        }
    }

    pub fn name(&self) -> Option<&Ident> {
        Some(&self.name)
    }

    pub fn n_types(&self) -> usize {
        self.types.len()
    }

    pub fn expand(&mut self, env: &Environment) {
        let mut types = Vec::new();

        for parameter in env.parameters.iter() {
            for path in self.types.iter() {
                types.extend_from_slice(&path.expand_node(parameter));
            }
        }

        self.types = types.into_iter().unique().collect();
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub parameters: Vec<Parameter>,
}

impl Environment {
    pub fn add_parameter(&self, mut parameter: Parameter) -> Self {
        parameter.expand(&self);
        let mut this = self.clone();
        this.parameters.push(parameter);
        this
    }

    pub fn names(&self) -> impl Iterator<Item = Option<&Ident>> {
        self.parameters.iter().map(Parameter::name)
    }

    pub fn n_combinations(&self) -> usize {
        self.parameters.iter().map(Parameter::n_types).product()
    }

    pub fn nth_combination<'a>(&'a self, list: &mut Vec<&'a Path>, mut combination: usize) {
        list.clear();
        for parameter in self.parameters.iter() {
            let variants = parameter.n_types();
            let div = combination / variants;
            let rem = combination % variants;
            combination = div;
            list.push(&parameter.types[rem]);
        }
        assert!(combination == 0)
    }
}

trait ExpandNode: Sized {
    fn expand_node(&self, parameter: &Parameter) -> Vec<Self>;
}

impl ExpandNode for Path {
    fn expand_node(&self, parameter: &Parameter) -> Vec<Self> {
        let parameter_name = &parameter.name;

        let mut paths = Vec::new();
        if let Some(ident) = self.get_ident() {
            if ident == parameter_name {
                paths.extend(parameter.types.clone());
            } else {
                paths.push(self.clone());
            }
        } else if let Some(segment) = self.segments.last() {
            let segments = segment.expand_node(parameter);
            for segment in segments {
                let mut path = self.clone();
                let last = path.segments.last_mut().unwrap();
                *last = segment;
                paths.push(path);
            }
        }

        paths
    }
}

impl ExpandNode for PathSegment {
    fn expand_node(&self, parameter: &Parameter) -> Vec<PathSegment> {
        let mut segments = Vec::new();

        match &self.arguments {
            syn::PathArguments::None => {
                segments.push(self.clone());
            }
            syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                let args = angle_bracketed_generic_arguments
                    .args
                    .iter()
                    .map(|arg| match arg {
                        syn::GenericArgument::Type(ty) => {
                            let tys = ty.expand_node(parameter);
                            tys.into_iter()
                                .map(syn::GenericArgument::Type)
                                .collect::<Vec<_>>()
                        }
                        _ => vec![arg.clone()],
                    })
                    .collect::<Vec<_>>();

                let seg_iter = CombinationIter::new(args).map(|v| {
                    let angle_bracketed = AngleBracketedGenericArguments {
                        args: Punctuated::from_iter(v.into_iter()),
                        ..angle_bracketed_generic_arguments.clone()
                    };

                    PathSegment {
                        arguments: syn::PathArguments::AngleBracketed(angle_bracketed),
                        ..self.clone()
                    }
                });

                segments.reserve(seg_iter.len());
                segments.extend(seg_iter);
            }
            syn::PathArguments::Parenthesized(_) => unreachable!(),
        }

        segments
    }
}

impl ExpandNode for Type {
    fn expand_node(&self, parameter: &Parameter) -> Vec<Self> {
        let mut tys = Vec::new();

        match self {
            Type::Path(type_path) => {
                let paths = type_path.path.expand_node(parameter);
                tys.extend(paths.into_iter().map(|path| {
                    Type::Path(TypePath {
                        path,
                        qself: type_path.qself.clone(),
                    })
                }));
            }
            // Type::Ptr(type_ptr) => todo!(),
            // Type::Array(type_array) => todo!(),
            // Type::BareFn(type_bare_fn) => todo!(),
            // Type::Group(type_group) => todo!(),
            // Type::ImplTrait(type_impl_trait) => todo!(),
            // Type::Macro(type_macro) => todo!(),
            // Type::Never(type_never) => todo!(),
            // Type::Paren(type_paren) => todo!(),
            // Type::Reference(type_reference) => todo!(),
            // Type::Slice(type_slice) => todo!(),
            // Type::TraitObject(type_trait_object) => todo!(),
            // Type::Tuple(type_tuple) => todo!(),
            // Type::Verbatim(token_stream) => todo!(),
            _ => todo!(),
        }

        tys
    }
}

struct CombinationIter<T> {
    src: Vec<Vec<T>>,
    idx: usize,
    n_combinations: usize,
    window_sz: usize,
}

impl<T: Clone> CombinationIter<T> {
    fn new(src: Vec<Vec<T>>) -> Self {
        let window_sz = src.len();
        let n_combinations = src.iter().map(|v| v.len()).product();
        let idx = 0;

        CombinationIter {
            src,
            idx,
            n_combinations,
            window_sz,
        }
    }
}

impl<T: Clone> Iterator for CombinationIter<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.n_combinations {
            return None;
        }

        let mut out = Vec::with_capacity(self.window_sz);
        let mut combination = self.idx;
        for i in 0..self.window_sz {
            let sz = self.src[i].len();
            let div = combination / sz;
            let rem = combination % sz;
            combination = div;
            out.push(self.src[i][rem].clone());
        }

        self.idx += 1;
        Some(out)
    }
}

impl<T: Clone> ExactSizeIterator for CombinationIter<T> {
    fn len(&self) -> usize {
        self.n_combinations
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use crate::ast::{expanded::environment::Parameter, raw::for_ast::ForAst};

    #[test]
    fn expand_environment() {
        let for_ast_outer: ForAst = parse_quote! {
            for T in [f64] { }
        };
        let for_ast_inner: ForAst = parse_quote! {
            for U in [T] { }
        };

        let param_t = Parameter::from_for_ast(&for_ast_outer);
        let param_u = Parameter::from_for_ast(&for_ast_inner);

        let env = param_t.into_environment();
        let env2 = env.add_parameter(param_u);

        let Parameter { name, types } = &env2.parameters[1];
        assert_eq!(name.to_string(), "U");

        assert_eq!(types.len(), 1);
        assert_eq!(types[0].get_ident().unwrap(), "f64");
    }

    #[test]
    fn expand_environment_path() {
        let for_ast_outer: ForAst = parse_quote! {
            for T in [f64] { }
        };
        let for_ast_inner: ForAst = parse_quote! {
            for U in [Foo<T>] { }
        };

        let param_t = Parameter::from_for_ast(&for_ast_outer);
        let param_u = Parameter::from_for_ast(&for_ast_inner);

        let env = param_t.into_environment();
        let env2 = env.add_parameter(param_u);

        let Parameter { name, types } = &env2.parameters[1];
        assert_eq!(name, "U");

        assert_eq!(types.len(), 1);
        let last_segment = types[0].segments.last().unwrap();
        assert_eq!(last_segment.ident, "Foo");
        assert!(!last_segment.arguments.is_empty());

        match &last_segment.arguments {
            syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                assert_eq!(angle_bracketed_generic_arguments.args.len(), 1);
                let arg = &angle_bracketed_generic_arguments.args[0];
                match arg {
                    syn::GenericArgument::Type(ty) => match ty {
                        syn::Type::Path(type_path) => {
                            assert_eq!(type_path.path.get_ident().unwrap(), "f64")
                        }
                        _ => assert!(false),
                    },
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn expand_environment_multiple() {
        let for_ast_outer: ForAst = parse_quote! {
            for T in [f32, f64] { }
        };
        let for_ast_inner: ForAst = parse_quote! {
            for U in [Foo<T>] { }
        };

        let param_t = Parameter::from_for_ast(&for_ast_outer);
        let param_u = Parameter::from_for_ast(&for_ast_inner);

        let env = param_t.into_environment();
        let env2 = env.add_parameter(param_u);

        let Parameter { name: _, types } = &env2.parameters[1];

        assert_eq!(types.len(), 2);
        {
            let last_segment = types[0].segments.last().unwrap();
            assert_eq!(last_segment.ident, "Foo");
            assert!(!last_segment.arguments.is_empty());

            match &last_segment.arguments {
                syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                    assert_eq!(angle_bracketed_generic_arguments.args.len(), 1);
                    let arg = &angle_bracketed_generic_arguments.args[0];
                    match arg {
                        syn::GenericArgument::Type(ty) => match ty {
                            syn::Type::Path(type_path) => {
                                assert_eq!(type_path.path.get_ident().unwrap(), "f32")
                            }
                            _ => assert!(false),
                        },
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            }
        }
        {
            let last_segment = types[1].segments.last().unwrap();
            assert_eq!(last_segment.ident, "Foo");
            assert!(!last_segment.arguments.is_empty());

            match &last_segment.arguments {
                syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                    assert_eq!(angle_bracketed_generic_arguments.args.len(), 1);
                    let arg = &angle_bracketed_generic_arguments.args[0];
                    match arg {
                        syn::GenericArgument::Type(ty) => match ty {
                            syn::Type::Path(type_path) => {
                                assert_eq!(type_path.path.get_ident().unwrap(), "f64")
                            }
                            _ => assert!(false),
                        },
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            }
        }
    }
}
