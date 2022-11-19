//! While bindgen is awesome, there are some issues with the generated bindings that have to be
//! fixed:
//!
//!  - The generated bindings lack atomic fields and static atomic variables. These have to be
//!    replaced with an appropriate atomic type.
//!
//!  - The generated bindings for Windows lack annotations, everything defined in Julia has to be
//!    annotated with `#[link(name = "libjulia", kind = "raw-dylib")]`. This is necessary because
//!    Julia is distributed without any lib files.
//!
//! The content of julia.h is scanned to detect everything that has to be converted to an atomic.
//! This information is used update the generated bindings.

use std::{
    borrow::Cow,
    fmt::Debug,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use proc_macro2::TokenTree;
use quote::{quote, ToTokens};

#[cfg(any(feature = "windows", windows))]
use syn::{parse::Parser, Attribute, ItemForeignMod};
use syn::{Field, ForeignItem, ItemStruct, ItemUnion, Type, TypePath};

#[derive(Clone, Debug)]
struct AtomicField<'a> {
    name: &'a str,
    ty: &'a str,
}

#[derive(Clone)]
struct HasAtomics<'a> {
    type_name: Cow<'a, str>,
    atomics: Vec<AtomicField<'a>>,
}

impl<'a> HasAtomics<'a> {
    fn get_atomic<S: AsRef<str>>(&self, name: S) -> Option<&AtomicField<'a>> {
        let name = name.as_ref();
        for ty in self.atomics.iter() {
            if ty.name == name {
                return Some(ty);
            }
        }

        None
    }
}

impl Debug for HasAtomics<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.type_name)?;
        writeln!(f, "  atomics:")?;

        if self.atomics.len() > 0 {
            for atomic in self.atomics.iter() {
                writeln!(f, "    {}: {}", atomic.name, atomic.ty)?;
            }
        }
        Ok(())
    }
}

impl<'a> HasAtomics<'a> {
    fn new(struct_def: &'a str) -> (Self, Vec<Self>) {
        let type_name = find_name(struct_def);
        let atomics = find_atomic_fields(struct_def);
        let unions = find_unions_with_atomics(struct_def, type_name);

        (
            HasAtomics {
                type_name: Cow::Borrowed(type_name),
                atomics,
            },
            unions,
        )
    }
}

#[derive(Debug)]
struct StructsWithAtomicFields<'a> {
    data: Vec<HasAtomics<'a>>,
}

impl<'a> StructsWithAtomicFields<'a> {
    fn get<S: AsRef<str>>(&self, name: S) -> Option<&HasAtomics<'a>> {
        let name = name.as_ref();
        for ty in self.data.iter() {
            if ty.type_name == name {
                return Some(ty);
            }
        }

        None
    }
}

fn find_unions_with_atomics<'a>(def: &'a str, type_name: &'a str) -> Vec<HasAtomics<'a>> {
    let mut unions = vec![];
    let mut atomic_fields = vec![];
    let mut parsing_union = false;
    let mut union_name = "";

    for line in def.lines() {
        if line.len() < 4 {
            continue;
        }

        let line = &line[4..];
        if line.starts_with("union") {
            parsing_union = true;
            if line.len() > 8 {
                union_name = &line[6..line.len() - 2];
            }

            continue;
        }

        if line.len() <= 2 {
            continue;
        }

        if parsing_union && line.starts_with("}") {
            parsing_union = false;

            if atomic_fields.len() > 0 {
                let name = format!("{}_{}", type_name, union_name);

                unions.push(HasAtomics {
                    type_name: Cow::Owned(name),
                    atomics: atomic_fields.clone(),
                });

                atomic_fields.clear();
            }

            continue;
        }

        if line.len() < 4 {
            continue;
        }
        let line = &line[4..];
        if parsing_union && line.starts_with("_Atomic") {
            let line = &line[8..];
            let atomic_inner_end = line
                .find(')')
                .expect("Can't find closing delimited of _Atomic field");
            let mut ty = &line[..atomic_inner_end];
            if ty.starts_with("struct ") {
                ty = &ty[7..];
            }
            let name_end = line.find(';').expect("Can't find semicolon");
            let name = &line[atomic_inner_end + 2..name_end];

            atomic_fields.push(AtomicField { ty, name })
        }
    }

    unions
}

fn find_atomic_fields<'a>(def: &'a str) -> Vec<AtomicField<'a>> {
    let mut atomic_fields = vec![];

    for line in def.lines() {
        if line.starts_with("    _Atomic") {
            // len("    _Atomic(") == 12
            let def = &line[12..];
            let atomic_inner_end = def
                .find(')')
                .expect("Can't find closing delimited of _Atomic field");
            let mut ty = &def[..atomic_inner_end];
            if ty.starts_with("struct ") {
                ty = &ty[7..];
            }
            let name_end = def.find(';').expect("Can't find semicolon");
            let name = &def[atomic_inner_end + 2..name_end];

            atomic_fields.push(AtomicField { ty, name: name })
        }
    }

    atomic_fields
}

fn find_name<'a>(def: &'a str) -> &'a str {
    if def.starts_with('t') {
        let def = &def[15..];
        if def.starts_with('{') {
            let sep = def.rfind(' ').expect("Can't detect final space");
            &def[sep + 1..def.len() - 2]
        } else {
            let sep = def.find(' ').expect("Can't detect struct name");
            &def[..sep]
        }
    } else {
        let def = &def[7..];
        let sep = def.find(' ').expect("Can't detect struct name");
        &def[..sep]
    }
}

fn parse<'a>(header: &'a str) -> (Vec<HasAtomics<'a>>, Vec<AtomicField<'a>>) {
    let mut offset = 0;
    let mut struct_start = 0;
    let mut atomics = Vec::new();
    let mut atomic_statics = vec![];
    let mut p = false;

    for line in header.lines() {
        if !line.starts_with(" ")
            && line.contains("struct")
            && line.contains("{")
            && !line.contains("JL_DLL")
        {
            p = true;
            struct_start = offset;
        } else if p && line.starts_with("}") {
            p = false;
            let struct_end = offset + line.len() + 1;
            let bytes = header.as_bytes();
            let data = &bytes[struct_start..struct_end];
            let struct_def = unsafe { std::str::from_utf8_unchecked(data) };

            if struct_def.contains("_Atomic") {
                let (s, e) = HasAtomics::new(struct_def);
                atomics.push(s);
                atomics.extend(e);
            }
        } else if line.starts_with("extern") {
            let line = &line[7..];
            let split_line = line.split(" ").collect::<Vec<_>>();
            let n_terms = split_line.len();
            let ty = split_line[n_terms - 2];
            if ty.starts_with("_Atomic") {
                let name = split_line[n_terms - 1];
                let name_len = name.len();
                let name = &name[..name_len - 1];

                let ty = &ty[8..];
                let atomic_inner_end = ty
                    .find(')')
                    .expect("Can't find closing delimited of _Atomic field");

                let ty = &ty[..atomic_inner_end];
                atomic_statics.push(AtomicField { name, ty })
            }
        }

        offset += line.len() + 1;
    }

    (atomics, atomic_statics)
}

fn read_header<P: AsRef<Path>>(include_dir: P) -> String {
    let mut path_buf = PathBuf::new();
    path_buf.push(include_dir);
    path_buf.push("julia.h");

    if path_buf.exists() {
        return fs::read_to_string(path_buf).expect("Cannot read Julia header");
    } else {
        panic!("Cannot find Julia header.")
    }
}

fn clear_derives(struct_def: &mut ItemStruct) {
    let mut remove_nth_attr = 0;
    for (idx, attr) in struct_def.attrs.iter_mut().enumerate() {
        let mut derives_debug = false;

        if attr.path.segments[0].ident.to_string() == "derive" {
            let cloned = attr.tokens.clone().into_iter().collect::<Vec<_>>();
            if let TokenTree::Group(group) = &cloned[0] {
                for tt in group.stream().into_iter() {
                    if let TokenTree::Ident(i) = tt {
                        if i == "Debug" {
                            derives_debug = true;
                            break;
                        }
                    }
                }
            }

            if derives_debug {
                let s = quote! {
                    (Debug)
                };
                attr.tokens = s;

                return;
            } else {
                remove_nth_attr = idx;
                break;
            }
        }
    }

    struct_def.attrs.remove(remove_nth_attr);
}

fn clear_derives_union(struct_def: &mut ItemUnion) {
    let mut n = None;
    for (idx, attr) in struct_def.attrs.iter_mut().enumerate() {
        if attr.path.segments[0].ident.to_string() == "derive" {
            n = Some(idx);
            break;
        }
    }

    if let Some(n) = n {
        struct_def.attrs.remove(n);
    }
}

fn convert_to_atomic(field_def: &mut Field, info: &AtomicField) {
    if let Type::Path(p) = &mut field_def.ty {
        *p = if info.ty == "void*" {
            syn::parse_str::<TypePath>("::std::sync::atomic::AtomicPtr<::std::ffi::c_void>")
                .unwrap()
        } else if info.ty.ends_with("*") {
            let new_ty = format!(
                "::std::sync::atomic::AtomicPtr<{}>",
                &info.ty[..info.ty.len() - 1]
            );
            syn::parse_str::<TypePath>(&new_ty).unwrap()
        } else if info.ty == "uint8_t" {
            syn::parse_str::<TypePath>("::std::sync::atomic::AtomicU8").unwrap()
        } else if info.ty == "uint32_t" {
            syn::parse_str::<TypePath>("::std::sync::atomic::AtomicU32").unwrap()
        } else if info.ty == "int16_t" {
            syn::parse_str::<TypePath>("::std::sync::atomic::AtomicI16").unwrap()
        } else if info.ty == "jl_callptr_t" {
            syn::parse_str::<TypePath>("::atomic::Atomic<jl_callptr_t>").unwrap()
        } else if info.ty == "jl_fptr_args_t" {
            syn::parse_str::<TypePath>("::atomic::Atomic<jl_fptr_args_t>").unwrap()
        } else if info.ty == "jl_fptr_sparam_t" {
            syn::parse_str::<TypePath>("::atomic::Atomic<jl_fptr_sparam_t>").unwrap()
        } else {
            panic!("Unsupported type: {}", info.ty);
        };
    }
}

fn convert_union_to_atomic(field_def: &mut Field, info: &AtomicField) {
    if let Type::Path(p) = &mut field_def.ty {
        *p = if info.ty == "void*" {
            syn::parse_str::<TypePath>(
                "::std::mem::ManuallyDrop<::std::sync::atomic::AtomicPtr<::std::ffi::c_void>>",
            )
            .unwrap()
        } else if info.ty.ends_with("*") {
            let new_ty = format!(
                "::std::mem::ManuallyDrop<::std::sync::atomic::AtomicPtr<{}>>",
                &info.ty[..info.ty.len() - 1]
            );
            syn::parse_str::<TypePath>(&new_ty).unwrap()
        } else if info.ty == "jl_callptr_t" {
            syn::parse_str::<TypePath>("::std::mem::ManuallyDrop<::atomic::Atomic<jl_callptr_t>>")
                .unwrap()
        } else if info.ty == "jl_fptr_args_t" {
            syn::parse_str::<TypePath>("::std::mem::ManuallyDrop<::atomic::Atomic<jl_fptr_args_t>>")
                .unwrap()
        } else if info.ty == "jl_fptr_sparam_t" {
            syn::parse_str::<TypePath>(
                "::std::mem::ManuallyDrop<::atomic::Atomic<jl_fptr_sparam_t>>",
            )
            .unwrap()
        } else {
            panic!("Unsupported type: {}", info.ty);
        };
    }
}

#[cfg(any(feature = "windows", windows))]
fn item_is_in_libjulia(fmod: &mut ItemForeignMod) {
    if let ForeignItem::Static(_) = &fmod.items[0] {
        let attr = Attribute::parse_outer
            .parse_str("#[link(name = \"libjulia\", kind = \"raw-dylib\")]")
            .unwrap()[0]
            .clone();
        fmod.attrs.push(attr);
    }

    if let ForeignItem::Fn(f) = &fmod.items[0] {
        if f.sig.ident.to_string().starts_with("jl_") {
            let attr = Attribute::parse_outer
                .parse_str("#[link(name = \"libjulia\", kind = \"raw-dylib\")]")
                .unwrap()[0]
                .clone();
            fmod.attrs.push(attr);
        }
    }
}

pub fn fix_bindings<P: AsRef<Path>, Q: AsRef<Path>>(header_path: P, bindings: &str, path: Q) {
    let header = read_header(header_path);
    let (structs, statics) = parse(&header);
    let b = StructsWithAtomicFields { data: structs };

    let mut stream = syn::parse_file(bindings).expect("msg");
    for item in &mut stream.items {
        match item {
            syn::Item::Struct(struct_def) => {
                let has_atomics = b.get(struct_def.ident.to_string());
                if let Some(has_atomics) = has_atomics {
                    clear_derives(struct_def);

                    for field in struct_def.fields.iter_mut() {
                        let info =
                            has_atomics.get_atomic(field.ident.as_ref().unwrap().to_string());
                        if let Some(info) = info {
                            convert_to_atomic(field, info)
                        }
                    }
                }
            }
            syn::Item::Union(struct_def) => {
                let has_atomics = b.get(struct_def.ident.to_string());
                if let Some(has_atomics) = has_atomics {
                    clear_derives_union(struct_def);

                    for field in struct_def.fields.named.iter_mut() {
                        let info =
                            has_atomics.get_atomic(field.ident.as_ref().unwrap().to_string());
                        if let Some(info) = info {
                            convert_union_to_atomic(field, info)
                        }
                    }
                }
            }
            syn::Item::ForeignMod(fmod) => {
                #[cfg(any(feature = "windows", windows))]
                item_is_in_libjulia(fmod);

                match &mut fmod.items[0] {
                    ForeignItem::Static(foreign_static) => {
                        let ident = foreign_static.ident.to_string();
                        if let Some(field) = statics.iter().find(|f| f.name == ident) {
                            if let Type::Path(p) = foreign_static.ty.as_mut() {
                                *p = if field.ty == "int" {
                                    syn::parse_str::<TypePath>("::std::sync::atomic::AtomicI32")
                                        .unwrap()
                                } else {
                                    panic!("Unsupported type: {}", field.ty)
                                };
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    }

    let fixed = stream.to_token_stream().to_string();
    let mut output = File::create(path).expect("Cannot create file");
    write!(output, "{}", fixed).expect("Cannot write bindings");
}
