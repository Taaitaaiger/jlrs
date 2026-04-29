mod types;

use std::{fs::File, io::Read, path::PathBuf};

use clap::Parser;
use syn::{
    Attribute, File as SynFile, Ident, ItemMod, MetaList, Signature, Type, TypePath, TypePtr,
    meta::ParseNestedMeta,
};

use crate::types::{CType, ItemType};

struct BindingsModule {
    file: SynFile,
}

const PREFIX: &'static str = "julia_1_";
const PREFIX_LEN: usize = PREFIX.len();
const LTS_VERSION: usize = 10;

#[derive(Debug)]
struct VersionRange {
    since: usize,
    until: Option<usize>,
}

fn minor_version_from_ident(ident: &Ident) -> usize {
    let ident = ident.to_string();
    assert!(ident.starts_with(PREFIX));
    let minor = &ident[PREFIX_LEN..];
    minor.parse::<usize>().unwrap()
}

fn parse_any(meta: &ParseNestedMeta, until: &mut Option<usize>) {
    // #[cfg(any(julia_1_x, julia_1_y, ..))] => until max(x, y, ..)
    meta.parse_nested_meta(|meta| {
        if let Some(ident) = meta.path.get_ident() {
            let minor = minor_version_from_ident(ident);
            match until {
                Some(current) if *current < minor => *current = minor,
                None => *until = Some(minor),
                _ => (),
            }
        }
        Ok(())
    })
    .unwrap();
}

fn parse_not_any(meta: &ParseNestedMeta, since: &mut usize) {
    // #[cfg(not(any(julia_1_x, julia_1_y, ..)))] => since max(x, y, ..)+1
    meta.parse_nested_meta(|meta| {
        if let Some(ident) = meta.path.get_ident() {
            let minor = minor_version_from_ident(ident);
            if *since < minor + 1 {
                *since = minor + 1
            }
        }
        Ok(())
    })
    .unwrap();
}

fn parse_not(meta: &ParseNestedMeta, since: &mut usize) {
    meta.parse_nested_meta(|meta| {
        if meta.path.is_ident("any") {
            // #[cfg(not(any(julia_1_x, julia_1_y)))]
            parse_not_any(&meta, since);
        } else if let Some(ident) = meta.path.get_ident() {
            // #[cfg(not(julia_1_x))] => since x+1
            let minor = minor_version_from_ident(ident);
            if *since < minor + 1 {
                *since = minor + 1
            }
        }
        Ok(())
    })
    .unwrap();
}

fn parse_cfg(cfg_list: &MetaList) -> VersionRange {
    let mut since = LTS_VERSION;
    let mut until = None;
    cfg_list
        .parse_nested_meta(|meta| {
            if meta.path.is_ident("not") {
                // #[cfg(not(julia_1_x))]
                // #[cfg(not(any(julia_1_x, julia_1_y)))]
                parse_not(&meta, &mut since);
            } else if meta.path.is_ident("any") {
                // #[cfg(any(julia_1_x, julia_1_y))]
                parse_any(&meta, &mut until);
            } else if let Some(ident) = meta.path.get_ident() {
                // #[cfg(julia_1_x)] => until x
                let minor = minor_version_from_ident(ident);
                until = Some(minor)
            }

            Ok(())
        })
        .unwrap();

    VersionRange { since, until }
}

impl VersionRange {
    fn from_attributes(attrs: &[Attribute]) -> Self {
        for attr in attrs {
            match &attr.meta {
                syn::Meta::List(meta_list) if meta_list.path.is_ident("cfg") => {
                    return parse_cfg(meta_list);
                }
                _ => continue,
            }
        }

        VersionRange {
            since: LTS_VERSION,
            until: None,
        }
    }

    fn wrap_case(&self, test_case: &str) -> String {
        format!(
            "    #if JULIA_VERSION_MINOR >= {}{}\n        {}\n    #endif\n",
            self.since,
            self.until
                .map(|s| format!(" && JULIA_VERSION_MINOR <= {s}"))
                .unwrap_or_default(),
            test_case
        )
    }
}

#[derive(Debug)]
struct Global<'a> {
    name: &'a Ident,
    ty: &'a Type,
    version_range: VersionRange,
}

impl<'a> Global<'a> {
    fn c_type(&self) -> String {
        ItemType::from(self.ty).c_type()
    }

    fn test_case(&self) -> String {
        let case = format!(
            "{} tmp_{} = {}; assert(&tmp_{} != NULL);",
            self.c_type(),
            self.name,
            self.name,
            self.name
        );

        self.version_range.wrap_case(&case)
    }

    fn print_type(&self) {
        let s = match self.ty {
            Type::Path(type_path) => type_path_to_string(type_path),
            Type::Ptr(type_ptr) => type_ptr_to_string(type_ptr),
            _ => unreachable!(),
        };

        println!("{s}")
    }
}

struct GlobalsMod<'a> {
    globals: &'a ItemMod,
}

impl<'a> GlobalsMod<'a> {
    fn globals(&'a self) -> impl Iterator<Item = Global<'a>> {
        let items = &self.globals.content.as_ref().unwrap().1;

        assert_eq!(items.len(), 1);
        let inner = &items[0];
        if let syn::Item::ForeignMod(foreign) = inner {
            foreign.items.iter().map(|item| match item {
                syn::ForeignItem::Static(item_static) => {
                    let name = &item_static.ident;
                    let ty = item_static.ty.as_ref();
                    let version_range = VersionRange::from_attributes(&item_static.attrs);
                    Global {
                        name,
                        ty,
                        version_range,
                    }
                }
                _ => unreachable!(),
            })
        } else {
            unreachable!()
        }
    }

    fn print_types(&self) {
        for global in self.globals() {
            global.print_type();
        }
    }
}

struct Function<'a> {
    signature: &'a Signature,
    version_range: VersionRange,
}

impl Function<'_> {
    fn test_case(&self) -> String {
        let ret = match &self.signature.output {
            syn::ReturnType::Default => "",
            syn::ReturnType::Type(_, ty) => match ty.as_ref() {
                Type::Never(..) => "",
                _ => "return",
            },
        };
        let fn_name = &self.signature.ident;
        let ret_type = ItemType::from(&self.signature.output).c_type();
        let arg_types = self.signature.inputs.iter().map(|s| ItemType::from(s));
        let arg_names = self.signature.inputs.iter().map(|i| match i {
            syn::FnArg::Receiver(_) => unreachable!(),
            syn::FnArg::Typed(pat_type) => match pat_type.pat.as_ref() {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                _ => unreachable!(),
            },
        });

        let typed_args = arg_names
            .clone()
            .zip(arg_types)
            .map(|(name, ty)| format!("{} {}", ty.c_type(), name))
            .collect::<Vec<_>>()
            .join(", ");

        let names = arg_names.collect::<Vec<_>>().join(", ");

        let case =
            format!("{ret_type } tmp_{fn_name}({typed_args}) {{ {ret} {fn_name}({names}); }}");

        self.version_range.wrap_case(&case)
    }

    fn print_return_type(&self) {
        let s = match &self.signature.output {
            syn::ReturnType::Default => "DEFAULT".to_string(),
            syn::ReturnType::Type(_, ty) => match ty.as_ref() {
                Type::Never(_type_never) => "NEVER".to_string(),
                Type::Path(type_path) => type_path_to_string(type_path),
                Type::Ptr(type_ptr) => type_ptr_to_string(type_ptr),
                _ => unreachable!(),
            },
        };

        println!("{s}")
    }

    fn print_arg_types(&self) {
        for input in self.signature.inputs.iter() {
            let s = match input {
                syn::FnArg::Receiver(_) => unreachable!(),
                syn::FnArg::Typed(pat_type) => match pat_type.ty.as_ref() {
                    Type::Path(type_path) => type_path_to_string(type_path),
                    Type::Ptr(type_ptr) => type_ptr_to_string(type_ptr),
                    _ => unreachable!(),
                },
            };

            println!("{s}")
        }
    }
}

struct FunctionsMod<'a> {
    functions: &'a ItemMod,
}

impl<'a> FunctionsMod<'a> {
    fn functions(&'a self) -> impl Iterator<Item = Function<'a>> {
        let items = &self.functions.content.as_ref().unwrap().1;

        assert_eq!(items.len(), 1);
        let inner = &items[0];
        if let syn::Item::ForeignMod(foreign) = inner {
            foreign.items.iter().map(|item| match item {
                syn::ForeignItem::Fn(item_fn) => {
                    let signature = &item_fn.sig;
                    let version_range = VersionRange::from_attributes(&item_fn.attrs);
                    Function {
                        signature,
                        version_range,
                    }
                }
                _ => unreachable!(),
            })
        } else {
            unreachable!()
        }
    }

    fn print_return_types(&self) {
        for func in self.functions() {
            func.print_return_type();
        }
    }

    fn print_arg_types(&self) {
        for func in self.functions() {
            func.print_arg_types();
        }
    }
}

impl BindingsModule {
    fn globals_module(&self) -> GlobalsMod<'_> {
        for item in self.file.items.iter() {
            match item {
                syn::Item::Mod(item_mod) if item_mod.ident == "globals" => {
                    return GlobalsMod { globals: item_mod };
                }
                _ => (),
            }
        }

        unreachable!()
    }

    fn functions_module(&self) -> FunctionsMod<'_> {
        for item in self.file.items.iter() {
            match item {
                syn::Item::Mod(item_mod) if item_mod.ident == "functions" => {
                    return FunctionsMod {
                        functions: item_mod,
                    };
                }
                _ => (),
            }
        }

        unreachable!()
    }
}

fn globals_test_fn<'a>(globals: impl Iterator<Item = Global<'a>>) -> String {
    let global_cases = globals.map(|global| global.test_case()).collect::<String>();
    format!("void globals_test_fn(void) {{\n{global_cases}}}",)
}

fn functions_test_fns<'a>(functions: impl Iterator<Item = Function<'a>>) -> String {
    functions
        .map(|func| func.test_case())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn type_ptr_to_string(type_ptr: &TypePtr) -> String {
    let s = match type_ptr.elem.as_ref() {
        Type::Path(type_path) => type_path_to_string(type_path),
        Type::Ptr(type_ptr) => type_ptr_to_string(type_ptr),
        _ => todo!(),
    };

    if type_ptr.mutability.is_some() {
        format!("*mut {s}")
    } else {
        format!("*const {s}")
    }
}

fn type_path_to_string(type_path: &TypePath) -> String {
    type_path
        .path
        .segments
        .iter()
        .map(|x| x.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn print_all_types(bindings_module: &BindingsModule) {
    let globals_mod = bindings_module.globals_module();
    let functions_mod = bindings_module.functions_module();

    globals_mod.print_types();
    functions_mod.print_arg_types();
    functions_mod.print_return_types();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    file: PathBuf,
    #[arg(short, long, help = "Print all types used by the bindings and exit")]
    print_types: bool,
}

fn main() {
    let args = Args::parse();

    let bindings_module = {
        let mut file = File::open(args.file).unwrap();
        let mut unparsed_bindings_module = String::new();
        file.read_to_string(&mut unparsed_bindings_module).unwrap();
        BindingsModule {
            file: syn::parse_file(&unparsed_bindings_module).unwrap(),
        }
    };

    if args.print_types {
        print_all_types(&bindings_module);
        return;
    }

    let globals = bindings_module.globals_module();
    let globals_test_fn = globals_test_fn(globals.globals());

    let functions = bindings_module.functions_module();
    let functions_test_fn = functions_test_fns(functions.functions());

    let c_file = format!(
        "#include <julia.h>
#include <julia_gcext.h>
#include <assert.h>

// Internal but necessary functions.
void jl_enter_threaded_region(void);
void jl_exit_threaded_region(void);

{globals_test_fn}

{functions_test_fn}

int main() {{
    globals_test_fn();
}}"
    );

    println!("{c_file}");
}
