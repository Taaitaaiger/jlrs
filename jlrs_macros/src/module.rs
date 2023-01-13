use std::iter::FromIterator;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::format_ident;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    token::Comma,
    Attribute, Error, Expr, FnArg, Ident, ItemFn, Result, ReturnType, Signature, Token, Type,
};

type RenameFragments = Punctuated<Ident, Token![.]>;

// TODO: Doc fragments

struct InitFn {
    _become_token: Token![become],
    init_fn: Ident,
}

impl Parse for InitFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let init_fn_token = input.parse()?;
        let init_fn = input.parse()?;

        Ok(InitFn {
            _become_token: init_fn_token,
            init_fn,
        })
    }
}

struct ExportedType {
    _struct_token: Token![struct],
    name: Ident,
    _as_token: Option<Token![as]>,
    name_override: Option<RenameFragments>,
}

impl Parse for ExportedType {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_token = input.parse()?;
        let name = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = RenameFragments::parse_separated_nonempty(input)?;

            Ok(ExportedType {
                _struct_token: struct_token,
                name,
                _as_token: Some(as_token),
                name_override: Some(name_override),
            })
        } else {
            Ok(ExportedType {
                _struct_token: struct_token,
                name,
                _as_token: None,
                name_override: None,
            })
        }
    }
}

struct ExportedFunction {
    func: Signature,
    _as_token: Option<Token![as]>,
    name_override: Option<RenameFragments>,
    exclamation_mark_token: Option<Token![!]>,
}

impl Parse for ExportedFunction {
    fn parse(input: ParseStream) -> Result<Self> {
        let func = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = RenameFragments::parse_separated_nonempty(input)?;
            let exclamation_mark_token = input.parse()?;

            Ok(ExportedFunction {
                func,
                _as_token: Some(as_token),
                name_override: Some(name_override),
                exclamation_mark_token,
            })
        } else {
            Ok(ExportedFunction {
                func,
                _as_token: None,
                name_override: None,
                exclamation_mark_token: None,
            })
        }
    }
}

struct ExportedMethod {
    _in_token: Token![in],
    parent: Type,
    func: Signature,
    _as_token: Option<Token![as]>,
    name_override: Option<RenameFragments>,
    exclamation_mark_token: Option<Token![!]>,
}

impl Parse for ExportedMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        let in_token = input.parse()?;
        let parent = input.parse()?;
        let func = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = RenameFragments::parse_separated_nonempty(input)?;
            let exclamation_mark_token = input.parse()?;

            Ok(ExportedMethod {
                _in_token: in_token,
                parent,
                func,
                _as_token: Some(as_token),
                name_override: Some(name_override),
                exclamation_mark_token,
            })
        } else {
            Ok(ExportedMethod {
                _in_token: in_token,
                parent,
                func,
                _as_token: None,
                name_override: None,
                exclamation_mark_token: None,
            })
        }
    }
}

struct ExportedConst {
    _const_token: Token![const],
    name: Ident,
    _colon: Token![:],
    ty: Type,
    _as_token: Option<Token![as]>,
    name_override: Option<Ident>,
}

impl Parse for ExportedConst {
    fn parse(input: ParseStream) -> Result<Self> {
        let const_token = input.parse()?;
        let name = input.parse()?;
        let colon = input.parse()?;
        let ty = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = input.parse()?;

            Ok(ExportedConst {
                _const_token: const_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: Some(as_token),
                name_override: Some(name_override),
            })
        } else {
            Ok(ExportedConst {
                _const_token: const_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: None,
                name_override: None,
            })
        }
    }
}

struct ExportedGlobal {
    _static_token: Token![static],
    name: Ident,
    _colon: Token![:],
    ty: Type,
    _as_token: Option<Token![as]>,
    name_override: Option<Ident>,
}

impl Parse for ExportedGlobal {
    fn parse(input: ParseStream) -> Result<Self> {
        let static_token = input.parse()?;
        let name = input.parse()?;
        let colon = input.parse()?;
        let ty = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![as]) {
            let as_token = input.parse()?;
            let name_override = input.parse()?;

            Ok(ExportedGlobal {
                _static_token: static_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: Some(as_token),
                name_override: Some(name_override),
            })
        } else {
            Ok(ExportedGlobal {
                _static_token: static_token,
                name: name,
                _colon: colon,
                ty: ty,
                _as_token: None,
                name_override: None,
            })
        }
    }
}
struct ItemWithAttrs {
    _attrs: Vec<Attribute>,
    item: Box<ModuleItem>,
}
impl Parse for ItemWithAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let attr: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let item: ModuleItem = input.parse()?;
        Ok(ItemWithAttrs {
            _attrs: attr,
            item: Box::new(item),
        })
    }
}

enum ModuleItem {
    InitFn(InitFn),
    ExportedType(ExportedType),
    ExportedFunction(ExportedFunction),
    ExportedMethod(ExportedMethod),
    ExportedConst(ExportedConst),
    ExportedGlobal(ExportedGlobal),
    ItemWithAttrs(ItemWithAttrs),
}

impl ModuleItem {
    fn is_init_fn(&self) -> bool {
        match self {
            ModuleItem::InitFn(_) => true,
            _ => false,
        }
    }

    fn get_init_fn(&self) -> &InitFn {
        match self {
            ModuleItem::InitFn(ref init_fn) => init_fn,
            _ => panic!(),
        }
    }

    fn is_exported_fn(&self) -> bool {
        match self {
            ModuleItem::ExportedFunction(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_fn() => true,
            _ => false,
        }
    }

    fn get_exported_fn(&self) -> &ExportedFunction {
        match self {
            ModuleItem::ExportedFunction(ref exported_fn) => exported_fn,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_fn() => {
                item.get_exported_fn()
            }
            _ => panic!(),
        }
    }

    fn is_exported_method(&self) -> bool {
        match self {
            ModuleItem::ExportedMethod(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_method() => {
                true
            }
            _ => false,
        }
    }

    fn get_exported_method(&self) -> &ExportedMethod {
        match self {
            ModuleItem::ExportedMethod(ref exported_method) => exported_method,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_method() => {
                item.get_exported_method()
            }
            _ => panic!(),
        }
    }

    fn is_exported_type(&self) -> bool {
        match self {
            ModuleItem::ExportedType(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_type() => {
                true
            }
            _ => false,
        }
    }

    fn get_exported_type(&self) -> &ExportedType {
        match self {
            ModuleItem::ExportedType(ref exported_type) => exported_type,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_type() => {
                item.get_exported_type()
            }
            _ => panic!(),
        }
    }

    fn is_exported_const(&self) -> bool {
        match self {
            ModuleItem::ExportedConst(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_const() => {
                true
            }
            _ => false,
        }
    }

    fn get_exported_const(&self) -> &ExportedConst {
        match self {
            ModuleItem::ExportedConst(ref exported_const) => exported_const,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_const() => {
                item.get_exported_const()
            }
            _ => panic!(),
        }
    }

    fn is_exported_global(&self) -> bool {
        match self {
            ModuleItem::ExportedGlobal(_) => true,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_global() => {
                true
            }
            _ => false,
        }
    }

    fn get_exported_global(&self) -> &ExportedGlobal {
        match self {
            ModuleItem::ExportedGlobal(ref exported_global) => exported_global,
            ModuleItem::ItemWithAttrs(ItemWithAttrs { item, .. }) if item.is_exported_global() => {
                item.get_exported_global()
            }
            _ => panic!(),
        }
    }

    fn _get_attrs(&self) -> Option<_Attributes> {
        match self {
            ModuleItem::ItemWithAttrs(ItemWithAttrs { _attrs: attrs, .. }) => {
                Some(_Attributes { attrs })
            }
            _ => None,
        }
    }
}

impl Parse for ModuleItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![become]) {
            input.parse().map(ModuleItem::InitFn)
        } else if lookahead.peek(Token![struct]) {
            input.parse().map(ModuleItem::ExportedType)
        } else if lookahead.peek(Token![fn]) {
            input.parse().map(ModuleItem::ExportedFunction)
        } else if lookahead.peek(Token![in]) {
            input.parse().map(ModuleItem::ExportedMethod)
        } else if lookahead.peek(Token![const]) {
            input.parse().map(ModuleItem::ExportedConst)
        } else if lookahead.peek(Token![static]) {
            input.parse().map(ModuleItem::ExportedGlobal)
        } else if lookahead.peek(Token![#]) {
            input.parse().map(ModuleItem::ItemWithAttrs)
        } else {
            Err(Error::new(
                input.span(),
                "Expected `become`, `fn`, `in`, `struct`, `const`, or `static`.",
            ))
        }
    }
}

struct _Attributes<'a> {
    attrs: &'a [Attribute],
}

pub(crate) struct JuliaModule {
    items: Punctuated<ModuleItem, Token![;]>,
}

impl Parse for JuliaModule {
    fn parse(input: ParseStream) -> Result<Self> {
        let content = input;

        Ok(JuliaModule {
            items: content.parse_terminated(ModuleItem::parse)?,
        })
    }
}

impl JuliaModule {
    pub(crate) fn generate_init_code(self) -> Result<TokenStream> {
        let init_fn = self.get_init_fn()?;
        let init_fn_ident = &init_fn.init_fn;

        let fn_fragments = FunctionFragments::generate(&self, init_fn);
        let method_fragments = MethodFragments::generate(&self, init_fn);
        let type_fragments = TypeFragments::generate(&self, init_fn);
        let const_fragments = ConstFragments::generate(&self, init_fn);
        let global_fragments = GlobalFragments::generate(&self, init_fn);

        // DocFragments::generate

        let type_init_fn = type_fragments.type_init_fn;
        let type_init_fn_ident = type_fragments.type_init_ident;
        let type_reinit_fn = type_fragments.type_reinit_fn;
        let type_reinit_fn_ident = type_fragments.type_reinit_ident;
        let function_init_fn = fn_fragments.init_functions_fn;
        let function_init_fn_ident = fn_fragments.init_functions_fn_ident;
        let method_init_fn = method_fragments.init_methods_fn;
        let method_init_fn_ident = method_fragments.init_methods_fn_ident;
        let const_init_fn = const_fragments.const_init_fn;
        let const_init_fn_ident = const_fragments.const_init_ident;
        let global_init_fn = global_fragments.global_init_fn;
        let global_init_fn_ident = global_fragments.global_init_ident;

        let invoke_type_init: Expr = if type_reinit_fn_ident.is_none() {
            parse_quote! {
                {
                    #type_init_fn_ident(&mut frame, module);
                }
            }
        } else {
            parse_quote! {
                if precompiling == 1 {
                    #type_init_fn_ident(&mut frame, module);
                } else {
                    #type_reinit_fn_ident(&mut frame, module);
                }
            }
        };

        let invoke_const_init: Expr = parse_quote! {
            if precompiling == 1 {
                #const_init_fn_ident(&mut frame, module);
            }
        };

        let invoke_global_init: Expr = parse_quote! {
            if precompiling == 1 {
                #global_init_fn_ident(&mut frame, module);
            }
        };

        let generated = quote::quote! {

            #[no_mangle]
            pub unsafe extern "C" fn #init_fn_ident(
                module: ::jlrs::data::managed::module::Module,
                precompiling: u8,
            ) -> ::jlrs::data::managed::value::ValueRet {
                #type_init_fn

                #type_reinit_fn

                #function_init_fn

                #method_init_fn

                #const_init_fn

                #global_init_fn

                static IS_INIT: ::std::sync::atomic::AtomicBool = ::std::sync::atomic::AtomicBool::new(false);
                if IS_INIT.compare_exchange(false, true, ::std::sync::atomic::Ordering::Relaxed, ::std::sync::atomic::Ordering::Relaxed).is_err() {
                    let unrooted = <::jlrs::data::managed::module::Module as ::jlrs::data::managed::Managed>::unrooted_target(module);
                    return ::jlrs::data::managed::value::Value::nothing(&unrooted).as_ref().leak();
                }

                let mut stack_frame = ::jlrs::memory::stack_frame::StackFrame::new();
                let mut ccall = ::jlrs::ccall::CCall::new(&mut stack_frame);

                ccall.init_jlrs();

                ccall.scope(|mut frame| {
                    let function_info_ty = ::jlrs::data::managed::module::Module::main(&frame)
                        .submodule(&frame, "Jlrs")
                        .unwrap()
                        .as_managed()
                        .submodule(&frame, "Wrap")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "JlrsFunctionInfo")
                        .unwrap()
                        .as_value()
                        .cast_unchecked::<::jlrs::data::managed::datatype::DataType>();

                    #invoke_type_init;
                    #invoke_const_init;
                    #invoke_global_init;

                    let mut arr = ::jlrs::data::managed::array::Array::new_for_unchecked(frame.as_extended_target(), 0, function_info_ty.as_value());
                    #function_init_fn_ident(&mut frame, &mut arr, module, function_info_ty);
                    #method_init_fn_ident(&mut frame, &mut arr, module, function_info_ty);

                    Ok(arr.as_value().as_ref().leak())
                }).unwrap()
            }
        };

        Ok(generated.into())
    }

    fn get_init_fn(&self) -> Result<&InitFn> {
        let n_init_fns = self.items.iter().filter(|it| it.is_init_fn()).count();
        if n_init_fns != 1 {
            let msg = format!("Expected 1 init fn, found {}", n_init_fns);
            Err(Error::new(Span::call_site(), msg))?;
        }

        let init_fn = self
            .items
            .iter()
            .find(|it| it.is_init_fn())
            .unwrap()
            .get_init_fn();

        Ok(init_fn)
    }

    fn get_exported_functions(&self) -> impl Iterator<Item = &ExportedFunction> {
        self.items
            .iter()
            .filter(|it| it.is_exported_fn())
            .map(|it| it.get_exported_fn())
    }

    fn get_exported_methods(&self) -> impl Iterator<Item = &ExportedMethod> {
        self.items
            .iter()
            .filter(|it| it.is_exported_method())
            .map(|it| it.get_exported_method())
    }

    fn get_exported_types(&self) -> impl Iterator<Item = &ExportedType> {
        self.items
            .iter()
            .filter(|it| it.is_exported_type())
            .map(|it| it.get_exported_type())
    }

    fn get_exported_consts(&self) -> impl Iterator<Item = &ExportedConst> {
        self.items
            .iter()
            .filter(|it| it.is_exported_const())
            .map(|it| it.get_exported_const())
    }

    fn get_exported_globals(&self) -> impl Iterator<Item = &ExportedGlobal> {
        self.items
            .iter()
            .filter(|it| it.is_exported_global())
            .map(|it| it.get_exported_global())
    }
}

struct FunctionFragments {
    init_functions_fn_ident: Ident,
    init_functions_fn: ItemFn,
}

impl FunctionFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_functions_fn_ident = format_ident!("{}_functions", init_fn.init_fn);
        let n_functions = module.get_exported_functions().count();

        let function_init_fragments = module
            .get_exported_functions()
            .enumerate()
            .map(function_info_fragment);

        let init_functions_fn = parse_quote! {
            unsafe fn #init_functions_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(|mut frame| {
                    array.grow_end_unchecked(#n_functions);
                    let mut accessor = array.value_data_mut().unwrap();

                    #(
                        #function_init_fragments
                    )*

                    Ok(())
                }).unwrap()
            }
        };

        FunctionFragments {
            init_functions_fn_ident,
            init_functions_fn,
        }
    }
}

struct MethodFragments {
    init_methods_fn_ident: Ident,
    init_methods_fn: ItemFn,
}

impl MethodFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_methods_fn_ident = format_ident!("{}_methods", init_fn.init_fn);
        let n_methods = module.get_exported_methods().count();

        let method_init_fragments = module
            .get_exported_methods()
            .enumerate()
            .map(method_info_fragment);

        let init_methods_fn = parse_quote! {
            unsafe fn #init_methods_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                array: &mut ::jlrs::data::managed::array::Array<'_, 'static>,
                module: ::jlrs::data::managed::module::Module,
                function_info_ty: ::jlrs::data::managed::datatype::DataType,
            ) {
                frame.scope(|mut frame| {
                    array.grow_end_unchecked(#n_methods);
                    let mut accessor = array.value_data_mut().unwrap();
                    let offset = ::jlrs::data::managed::array::dimensions::Dims::size(&accessor.dimensions()) - #n_methods;

                    #(
                        #method_init_fragments
                    )*

                    Ok(())
                }).unwrap()
            }
        };

        MethodFragments {
            init_methods_fn_ident,
            init_methods_fn,
        }
    }
}

struct TypeFragments {
    type_init_fn: ItemFn,
    type_init_ident: Ident,
    type_reinit_fn: Option<ItemFn>,
    type_reinit_ident: Option<Ident>,
}

impl TypeFragments {
    fn generate(info: &JuliaModule, init_fn: &InitFn) -> Self {
        let init_types_fn_ident = format_ident!("{}_types", init_fn.init_fn);
        let init_types_fragments = info.get_exported_types().map(init_type_fragment);

        let type_init_fn = parse_quote! {
            unsafe fn #init_types_fn_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: ::jlrs::data::managed::module::Module,
            ) {
                frame.scope(|frame| {
                    let mut output = frame.output();

                    #(
                        #init_types_fragments
                    )*

                    Ok(())
                }).unwrap();
            }
        };

        #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
        {
            let reinit_types_fn_ident = format_ident!("{}_reinittypes", init_fn.init_fn);
            let reinit_types_fragments = info.get_exported_types().map(reinit_type_fragment);

            let type_reinit_fn = parse_quote! {
                unsafe fn #reinit_types_fn_ident(
                    frame: &mut ::jlrs::memory::target::frame::GcFrame,
                    module: jlrs::data::managed::module::Module
                ) {
                    frame.scope(|frame| {
                        let mut output = frame.output();

                        #(
                            #reinit_types_fragments
                        )*

                        Ok(())
                    }).unwrap();
                }
            };

            TypeFragments {
                type_init_fn,
                type_init_ident: init_types_fn_ident,
                type_reinit_fn: Some(type_reinit_fn),
                type_reinit_ident: Some(reinit_types_fn_ident),
            }
        }

        #[cfg(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8"))]
        {
            TypeFragments {
                type_init_fn,
                type_init_ident: init_types_fn_ident,
                type_reinit_fn: None,
                type_reinit_ident: None,
            }
        }
    }
}

struct ConstFragments {
    const_init_fn: ItemFn,
    const_init_ident: Ident,
}

impl ConstFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let const_init_ident = format_ident!("{}_consts", init_fn.init_fn);

        let const_init_fragments = module.get_exported_consts().map(const_info_fragment);

        let const_init_fn = parse_quote! {
            unsafe fn #const_init_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: ::jlrs::data::managed::module::Module,
            ) {
                #(
                    #const_init_fragments
                )*
            }
        };

        ConstFragments {
            const_init_ident,
            const_init_fn,
        }
    }
}

struct GlobalFragments {
    global_init_fn: ItemFn,
    global_init_ident: Ident,
}

impl GlobalFragments {
    fn generate(module: &JuliaModule, init_fn: &InitFn) -> Self {
        let global_init_ident = format_ident!("{}_globals", init_fn.init_fn);

        let global_init_fragments = module.get_exported_globals().map(global_info_fragment);

        let global_init_fn = parse_quote! {
            unsafe fn #global_init_ident(
                frame: &mut ::jlrs::memory::target::frame::GcFrame,
                module: ::jlrs::data::managed::module::Module,
            ) {
                #(
                    #global_init_fragments
                )*
            }
        };

        GlobalFragments {
            global_init_ident,
            global_init_fn,
        }
    }
}

fn function_info_fragment((index, info): (usize, &ExportedFunction)) -> Expr {
    let n_args = info.func.inputs.len();
    let name_ident = &info.func.ident;

    let override_module_fragment = override_module_fragment(&info.name_override);
    let mut rename = info
        .name_override
        .as_ref()
        .map(|parts| parts.last())
        .flatten()
        .unwrap_or(name_ident)
        .to_string();

    if info.exclamation_mark_token.is_some() {
        rename.push('!')
    }

    let tys = info.func.inputs.iter().map(|x| match x {
        FnArg::Typed(pat) => &pat.ty,
        _ => unreachable!(),
    });

    let punctuated_tys = Punctuated::<_, Comma>::from_iter(tys);
    let ret_ty = &info.func.output;

    let (ccall_ret_type, julia_ret_type) = return_type_fragments(&info.func.output);

    let ccall_arg_idx = 0..n_args;
    let julia_arg_idx = 0..n_args;

    let (ccall_arg_types, julia_arg_types) = arg_type_fragments(info);

    parse_quote! {
        {
            frame.scope(|mut frame| {
                let name = Symbol::new(&frame, #rename);
                let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&frame).as_value();
                // Ensure a compile error happens if the signatures of the function don't match.
                let func: unsafe extern "C" fn(#punctuated_tys) #ret_ty = #name_ident;
                let func = Value::new(&mut frame, func as *mut ::std::ffi::c_void);

                unsafe {
                    let mut ccall_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        frame.as_extended_target(),
                        #n_args,
                        type_type);

                    let mut ccall_arg_types_ref = ccall_arg_types.value_data_mut().unwrap();

                    let mut julia_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        frame.as_extended_target(),
                        #n_args,
                        type_type);

                    let mut julia_arg_types_ref = julia_arg_types.value_data_mut().unwrap();

                    #(
                        ccall_arg_types_ref.set(#ccall_arg_idx, Some(#ccall_arg_types.as_value())).unwrap();
                        julia_arg_types_ref.set(#julia_arg_idx, Some(#julia_arg_types.as_value())).unwrap();
                    )*

                    let ccall_return_type = #ccall_ret_type;
                    let julia_return_type = #julia_ret_type;

                    let module = #override_module_fragment;

                    let instance = function_info_ty.instantiate_unchecked(&mut frame, [
                        name.as_value(),
                        ccall_arg_types.as_value(),
                        julia_arg_types.as_value(),
                        ccall_return_type.as_value(),
                        julia_return_type.as_value(),
                        func,
                        module.as_value()
                    ]);

                    accessor.set(#index, Some(instance)).unwrap();
                }

                Ok(())
            }).unwrap();
        }
    }
}

fn override_module_fragment(name_override: &Option<RenameFragments>) -> Expr {
    let name_override = name_override.as_ref();
    if name_override.is_none() {
        return parse_quote! { { module } };
    }
    let name_override = name_override.unwrap();
    let n_parts = name_override.len();
    if n_parts == 1 {
        return parse_quote! { { module } };
    }

    let modules = name_override
        .iter()
        .take(n_parts - 1)
        .map(|ident| ident.to_string());

    let parsed = parse_quote! {
        {
            let mut module = Module::main(&frame);

            #(
                module = module
                .submodule(&frame, #modules)
                .expect("Submodule does not exist")
                .as_managed();
            )*

            module
        }
    };

    parsed
}

fn return_type_fragments(ret_ty: &ReturnType) -> (Expr, Expr) {
    match ret_ty {
        ReturnType::Default => {
            let ccall_ret_type: Expr = parse_quote! {
                ::jlrs::data::managed::datatype::DataType::nothing_type(&frame)
            };

            let julia_ret_type = ccall_ret_type.clone();
            (ccall_ret_type, julia_ret_type)
        }
        ReturnType::Type(_, ref ty) => {
            let ccall_ret_type = parse_quote! {
                <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::CCallReturnType as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target())
            };
            let julia_ret_type = parse_quote! {
                <<#ty as ::jlrs::convert::ccall_types::CCallReturn>::FunctionReturnType as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target())
            };

            (ccall_ret_type, julia_ret_type)
        }
    }
}

fn arg_type_fragments<'a>(
    info: &'a ExportedFunction,
) -> (
    impl 'a + Iterator<Item = Expr>,
    impl 'a + Iterator<Item = Expr>,
) {
    let inputs = &info.func.inputs;
    let n_args = inputs.len();

    if n_args > 0 {
        match inputs.first().unwrap() {
            FnArg::Receiver(_) => panic!(),
            _ => (),
        }
    }

    let ccall_arg_types = inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(ty) => &ty.ty,
            _ => unreachable!(),
        })
        .map(|ty| {
            parse_quote! {
                <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target())
            }
        });

    let julia_arg_types = inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(ty) => &ty.ty,
            _ => unreachable!(),
        })
        .map(|ty| {
            parse_quote! {
                <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target())
            }
        });

    (ccall_arg_types, julia_arg_types)
}

fn init_type_fragment(info: &ExportedType) -> Expr {
    let override_module_fragment = override_module_fragment(&info.name_override);
    let name_ident = &info.name;

    let rename = info
        .name_override
        .as_ref()
        .map(|parts| parts.last())
        .flatten()
        .unwrap_or(name_ident)
        .to_string();

    let ty = format_ident!("{}", info.name);

    parse_quote! {
        {
            let sym = ::jlrs::data::managed::symbol::Symbol::new(&frame, #rename);
            let module = #override_module_fragment;
            let ty = ::jlrs::data::layout::foreign::create_foreign_type::<#ty, _>(&mut output, sym, module);
            module.set_const_unchecked(sym, <::jlrs::data::managed::datatype::DataType as ::jlrs::data::managed::Managed>::as_value(ty));
        }
    }
}

#[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
fn reinit_type_fragment(info: &ExportedType) -> Expr {
    {
        let override_module_fragment = override_module_fragment(&info.name_override);
        let name_ident = &info.name;

        let rename = info
            .name_override
            .as_ref()
            .map(|parts| parts.last())
            .flatten()
            .unwrap_or(name_ident)
            .to_string();

        let ty = format_ident!("{}", info.name);

        parse_quote! {
            {
                let module = #override_module_fragment;

                let dt = module
                    .global(&frame, #rename)
                    .unwrap()
                    .as_value()
                    .cast::<::jlrs::data::managed::datatype::DataType>()
                    .unwrap();

                ::jlrs::data::layout::foreign::reinit_foreign_type::<#ty>(dt);
            }
        }
    }
}

fn method_info_fragment((index, info): (usize, &ExportedMethod)) -> Expr {
    let n_args = info.func.inputs.len();
    let name_ident = &info.func.ident;

    let override_module_fragment = override_module_fragment(&info.name_override);
    let mut rename = info
        .name_override
        .as_ref()
        .map(|parts| parts.last())
        .flatten()
        .unwrap_or(name_ident)
        .to_string();

    if info.exclamation_mark_token.is_some() {
        rename.push('!')
    }

    let ret_ty = &info.func.output;
    let (ccall_ret_type, julia_ret_type) = return_type_fragments(ret_ty);

    let ccall_arg_idx = 0..n_args;
    let julia_arg_idx = 0..n_args;

    let (ccall_arg_types, julia_arg_types, invoke_fn) = method_arg_type_fragments(info);

    parse_quote! {
        {
            frame.scope(|mut frame| {
                let unrooted = frame.unrooted();
                let name = Symbol::new(&frame, #rename);
                let type_type = ::jlrs::data::managed::union_all::UnionAll::type_type(&unrooted).as_value();

                #invoke_fn;

                let func = Value::new(&mut frame, invoke as *mut ::std::ffi::c_void);

                unsafe {
                    let mut ccall_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        frame.as_extended_target(),
                        #n_args,
                        type_type);

                    let mut ccall_arg_types_ref = ccall_arg_types.value_data_mut().unwrap();

                    let mut julia_arg_types = ::jlrs::data::managed::array::Array::new_for_unchecked(
                        frame.as_extended_target(),
                        #n_args,
                        type_type);

                    let mut julia_arg_types_ref = julia_arg_types.value_data_mut().unwrap();

                    #(
                        ccall_arg_types_ref.set(#ccall_arg_idx, Some(#ccall_arg_types.as_value())).unwrap();
                        julia_arg_types_ref.set(#julia_arg_idx, Some(#julia_arg_types.as_value())).unwrap();
                    )*

                    let ccall_return_type = #ccall_ret_type;
                    let julia_return_type = #julia_ret_type;

                    let module = #override_module_fragment;

                    let instance = function_info_ty.instantiate_unchecked(&mut frame, [
                        name.as_value(),
                        ccall_arg_types.as_value(),
                        julia_arg_types.as_value(),
                        ccall_return_type.as_value(),
                        julia_return_type.as_value(),
                        func,
                        module.as_value()
                    ]);

                    accessor.set(#index + offset, Some(instance)).unwrap();
                }

                Ok(())
            }).unwrap();
        }
    }
}

fn const_info_fragment(info: &ExportedConst) -> Expr {
    let name = &info.name;
    let rename = info.name_override.as_ref().unwrap_or(name).to_string();
    let ty = &info.ty;

    parse_quote! {
        {
            frame.scope(|mut frame| {
                let v: #ty = #name;
                let value = ::jlrs::data::managed::value::Value::new(&mut frame, v);

                unsafe {
                    module.set_const_unchecked(#rename, value);
                }

                Ok(())

            }).unwrap();
        }
    }
}

fn global_info_fragment(info: &ExportedGlobal) -> Expr {
    let name = &info.name;
    let rename = info.name_override.as_ref().unwrap_or(name).to_string();
    let ty = &info.ty;

    parse_quote! {
        {
            frame.scope(|mut frame| {
                let v: #ty = #name;
                let value = ::jlrs::data::managed::value::Value::new(&mut frame, v);

                unsafe {
                    module.set_global_unchecked(#rename, value);
                }

                Ok(())

            }).unwrap();
        }
    }
}

fn method_arg_type_fragments<'a>(
    info: &'a ExportedMethod,
) -> (
    impl 'a + Iterator<Item = Expr>,
    impl 'a + Iterator<Item = Expr>,
    Option<ItemFn>,
) {
    let inputs = &info.func.inputs;
    let n_args = inputs.len();

    let takes_self = if n_args > 0 {
        match inputs.first().unwrap() {
            FnArg::Receiver(r) => Some((r.mutability.is_some(), r.reference.is_some())),
            FnArg::Typed(_) => None,
        }
    } else {
        None
    };

    let invoke_fn = match takes_self {
        None => Some(invoke_fn_no_self_method_fragment(info)),
        Some((true, true)) => Some(invoke_fn_mut_self_method_fragment(info)),
        Some((false, true)) => Some(invoke_fn_ref_self_method_fragment(info)),
        Some((_, false)) => Some(invoke_fn_move_self_method_fragment(info)),
    };

    let parent = &info.parent;
    let ccall_arg_types = inputs
        .iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = &ty.ty;
                    parse_quote! {
                        <<#ty as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target())
                    }
                },
                _ => {

                    parse_quote! {
                        <<TypedValue::<#parent> as ::jlrs::convert::ccall_types::CCallArg>::CCallArgType as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target())
                    }
                },
            }
        });

    let julia_arg_types = inputs
        .iter()
        .map(move |arg| {
            match arg {
                FnArg::Typed(ty) => {
                    let ty = &ty.ty;
                    parse_quote! {
                        <<#ty as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target())
                    }
                },
                _ => {

                    parse_quote! {
                        <<TypedValue<#parent> as ::jlrs::convert::ccall_types::CCallArg>::FunctionArgType as ::jlrs::convert::construct_type::ConstructType>::construct_type(frame.as_extended_target())
                    }
                },
            }
        });

    (ccall_arg_types, julia_arg_types, invoke_fn)
}

fn invoke_fn_no_self_method_fragment(info: &ExportedMethod) -> ItemFn {
    let name = &info.func.ident;
    let ty = &info.parent;
    let ret_ty = &info.func.output;
    let args = &info.func.inputs;
    let names = args.iter().map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    parse_quote! {
        unsafe extern "C" fn invoke(#args) #ret_ty {
            <#ty>::#name(#names)
        }
    }
}

fn invoke_fn_ref_self_method_fragment(info: &ExportedMethod) -> ItemFn {
    let name = &info.func.ident;
    let ty = &info.parent;
    let ret_ty = &info.func.output;
    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();

    *first = parse_quote! {
        this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = cloned_args;

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    parse_quote! {
        unsafe extern "C" fn invoke(#args_self_renamed) #ret_ty {
            match (&this).track() {
                Ok(this) => this.#name(#names),
                Err(_) => ::jlrs::data::managed::rust_result::RustResult::borrow_err_internal()
            }
        }
    }
}

fn invoke_fn_move_self_method_fragment(info: &ExportedMethod) -> ItemFn {
    let name = &info.func.ident;
    let ty = &info.parent;
    let ret_ty = &info.func.output;
    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();

    *first = parse_quote! {
        this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = cloned_args;

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    parse_quote! {
        unsafe extern "C" fn invoke(#args_self_renamed) #ret_ty {
            match (&this).track() {
                Ok(this) => this.clone().#name(#names),
                Err(_) => ::jlrs::data::managed::rust_result::RustResult::borrow_err_internal()
            }
        }
    }
}

fn invoke_fn_mut_self_method_fragment(info: &ExportedMethod) -> ItemFn {
    let name = &info.func.ident;
    let ty = &info.parent;
    let ret_ty = &info.func.output;
    let args = &info.func.inputs;
    let mut cloned_args = args.clone();
    let first = cloned_args.first_mut().unwrap();

    *first = parse_quote! {
        mut this: ::jlrs::data::managed::value::typed::TypedValue<#ty>
    };

    let args_self_renamed = cloned_args;

    let names = args.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ty) => &ty.pat,
        _ => unreachable!(),
    });

    let names = Punctuated::<_, Comma>::from_iter(names);

    parse_quote! {
        unsafe extern "C" fn invoke(#args_self_renamed) #ret_ty {
            match (&mut this).track_mut() {
                Ok(mut this) => this.#name(#names),
                Err(_) => ::jlrs::data::managed::rust_result::RustResult::borrow_err_internal()
            }
        }
    }
}
