use syn::{FnArg, Ident, ReturnType, Type, TypePath, TypePtr};

pub struct JlType<'a> {
    name: &'a Ident,
}

pub struct FFIType<'a> {
    name: &'a Ident,
}

pub struct PtrType<'a> {
    mutability: bool,
    inner: Box<ItemType<'a>>,
}

pub struct UnsafeCellType<'a> {
    inner: Box<ItemType<'a>>,
}

pub struct AtomicType<'a> {
    name: &'a Ident,
}

pub enum NumberType {
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
    F32,
    F64,
}

pub enum ItemType<'a> {
    JlType(JlType<'a>),
    FfiType(FFIType<'a>),
    PtrType(PtrType<'a>),
    UnsafeCellType(UnsafeCellType<'a>),
    AtomicType(AtomicType<'a>),
    NumberType(NumberType),
    Default,
    Never,
}

pub trait CType {
    fn c_type(&self) -> String;
}

impl<'a> CType for JlType<'a> {
    fn c_type(&self) -> String {
        self.name.to_string()
    }
}

impl<'a> CType for FFIType<'a> {
    fn c_type(&self) -> String {
        let s = self.name
            .to_string()
            .strip_prefix("c_")
            .unwrap()
            .to_string();

        if s != "uint" {
            s
        } else {
            "unsigned int".into()
        }
    }
}

impl<'a> CType for AtomicType<'a> {
    fn c_type(&self) -> String {
        match self.name.to_string().strip_prefix("Atomic") {
            Some("I32") => "_Atomic(int32_t)".into(),
            _ => todo!("Only AtomicI32 has currently been implemented"),
        }
    }
}

impl<'a> CType for NumberType {
    fn c_type(&self) -> String {
        match self {
            NumberType::I8 => "int8_t",
            NumberType::I16 => "int16_t",
            NumberType::I32 => "int32_t",
            NumberType::I64 => "int64_t",
            NumberType::Isize => "ssize_t",
            NumberType::U8 => "uint8_t",
            NumberType::U16 => "uint16_t",
            NumberType::U32 => "uint32_t",
            NumberType::U64 => "uint64_t",
            NumberType::Usize => "size_t",
            NumberType::F32 => "float",
            NumberType::F64 => "double",
        }
        .into()
    }
}

impl<'a> CType for UnsafeCellType<'a> {
    fn c_type(&self) -> String {
        self.inner.c_type()
    }
}

impl<'a> CType for PtrType<'a> {
    fn c_type(&self) -> String {
        let inner = self.inner.c_type();
        if self.mutability {
            format!("{} *", inner)
        } else {
            format!("const {} *", inner)
        }
    }
}

impl<'a> CType for ItemType<'a> {
    fn c_type(&self) -> String {
        match self {
            ItemType::JlType(jl_type) => jl_type.c_type(),
            ItemType::FfiType(ffi_type) => ffi_type.c_type(),
            ItemType::PtrType(ptr_type) => ptr_type.c_type(),
            ItemType::UnsafeCellType(unsafe_cell_type) => unsafe_cell_type.c_type(),
            ItemType::AtomicType(atomic_type) => atomic_type.c_type(),
            ItemType::NumberType(number_type) => number_type.c_type(),
            ItemType::Default => "void".into(),
            ItemType::Never => "void".into(),
        }
    }
}

impl<'a> From<&'a TypePath> for ItemType<'a> {
    fn from(value: &'a TypePath) -> Self {
        if let Some(ident) = value.path.get_ident() {
            let number_type = NumberType::from(ident);
            ItemType::NumberType(number_type)
        } else {
            let n_segments = value.path.segments.len();

            let first_segment = &value.path.segments[0];
            if first_segment.ident == "crate" {
                assert_eq!(n_segments, 3, "Expected 3 segments");
                let second_segment = &value.path.segments[1];
                assert_eq!(second_segment.ident, "types");

                ItemType::JlType(JlType {
                    name: &value.path.segments[2].ident,
                })
            } else if first_segment.ident == "std" {
                assert!(n_segments >= 3, "Expected at least 3 segments");
                let second_segment = &value.path.segments[1];
                if second_segment.ident == "ffi" {
                    // must be a type in std::ffi
                    ItemType::FfiType(FFIType {
                        name: &value.path.segments[2].ident,
                    })
                } else if second_segment.ident == "sync" {
                    // must be std::sync::atomic::AtomicI32
                    assert!(n_segments == 4, "Expected 4 segments");
                    assert_eq!(value.path.segments[2].ident, "atomic");
                    ItemType::AtomicType(AtomicType {
                        name: &value.path.segments[3].ident,
                    })
                } else if second_segment.ident == "cell" {
                    // must be UnsafeCell
                    let third_segment = &value.path.segments[2];
                    assert_eq!(third_segment.ident, "UnsafeCell");

                    match &third_segment.arguments {
                        syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                            match angle_bracketed_generic_arguments.args.first().unwrap() {
                                syn::GenericArgument::Type(ty) => {
                                    let inner = ItemType::from(ty);
                                    ItemType::UnsafeCellType(UnsafeCellType { inner: Box::new(inner) })
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                } else {
                    todo!()
                }
            } else {
                todo!()
            }
        }
    }
}

impl<'a> From<&'a TypePtr> for PtrType<'a> {
    fn from(value: &'a TypePtr) -> Self {
        let mutability = value.mutability.is_some();
        let inner = ItemType::from(value.elem.as_ref());

        PtrType {
            mutability,
            inner: Box::new(inner),
        }
    }
}

impl<'a> From<&'a TypePtr> for ItemType<'a> {
    fn from(value: &'a TypePtr) -> Self {
        let ptr_type = PtrType::from(value);
        ItemType::PtrType(ptr_type)
    }
}

impl<'a> From<&'a FnArg> for ItemType<'a> {
    fn from(value: &'a FnArg) -> Self {
        match value {
            FnArg::Receiver(_) => todo!(),
            FnArg::Typed(pat_type) => pat_type.ty.as_ref().into(),
        }
    }
}

impl<'a> From<&'a ReturnType> for ItemType<'a> {
    fn from(value: &'a ReturnType) -> Self {
        match value {
            ReturnType::Default => ItemType::Default,
            ReturnType::Type(_, ty) => ty.as_ref().into(),
        }
    }
}

impl<'a> From<&'a Type> for ItemType<'a> {
    fn from(value: &'a Type) -> Self {
        match value {
            syn::Type::Never(_) => ItemType::Never,
            syn::Type::Path(type_path) => type_path.into(),
            syn::Type::Ptr(type_ptr) => type_ptr.into(),
            _ => todo!(),
        }
    }
}

impl<'a> From<&'a Ident> for NumberType {
    fn from(value: &'a Ident) -> Self {
        if value == "i8" {
            NumberType::I8
        } else if value == "i16" {
            NumberType::I16
        } else if value == "i32" {
            NumberType::I32
        } else if value == "i64" {
            NumberType::I64
        } else if value == "isize" {
            NumberType::Isize
        } else if value == "u8" {
            NumberType::U8
        } else if value == "u16" {
            NumberType::U16
        } else if value == "u32" {
            NumberType::U32
        } else if value == "u64" {
            NumberType::U64
        } else if value == "usize" {
            NumberType::Usize
        } else if value == "f32" {
            NumberType::F32
        } else if value == "f64" {
            NumberType::F64
        } else {
            unreachable!()
        }
    }
}
