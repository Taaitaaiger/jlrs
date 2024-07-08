mod constant_bytes;
#[cfg(feature = "derive")]
mod derive;
#[cfg(feature = "ccall")]
mod module;
mod version;

use proc_macro::TokenStream;

#[cfg(feature = "derive")]
use self::derive::*;
#[cfg(feature = "ccall")]
use self::module::*;
use self::{constant_bytes::*, version::emit_if_compatible};

/// Export functions, types and constants defined in Rust as a Julia module.
///
/// This macro generates an initialization function. This function is used in combination with the
/// macros provided by the `JlrsCore.Wrap` module to automatically generate the content of that
/// module.
///
/// The syntax is as follows:
///
/// ```ignore
/// julia_macro! {
///     // init_function_name is the name of the generated initialization function.
///     //
///     // The name of the generated function must be unique, it's recommended you prefix it with
///     // the crate name. If your crate is named foo-jl, you should use a name like
///     // `foo_jl_init`.
///     become init_function_name;
///
///     // Exports the function `foo` as `bar` with documentation.
///     //
///     // The `as <exposed_name>` part is optional, by default the function is exported with the
///     // name it has in Rust, the exposed name can end in an exclamation mark.
///     //
///     // A docstring can be provided with a doc comment; if multiple functions are exported
///     // with the same name it shoud only be documented once. All exported items can be
///     // documented.
///     //
///     // If the function doesn't need to call into Julia, you can annotate it with `#[gc_safe]`
///     // to allow the GC to run without having to wait until the function has returned.
///
///     ///     bar(arr::Array)
///     ///
///     /// Documentation for this function"]
///     #[gc_safe]
///     fn foo(arr: Array) -> usize as bar;
///
///     // Exports the function `foo` as `bar!` in the `Base` module.
///     //
///     // This syntax can be used to extend existing functions.
///     fn foo(arr: Array) -> usize as Base.bar!;
///
///     // Exports the struct `MyType` as `MyForeignType`. `MyType` must implement `OpaqueType`
///     // or `ForeignType`.
///     struct MyType as MyForeignType;
///
///     // Exports `MyType::new` as `MyForeignType`, turning it into a constructor for that type.
///     in MyType fn new(arg0: TypedValue<u32>) -> TypedValueRet<MyType> as MyForeignType;
///
///     // Exports `MyType::add` as the function `increment!`.
///     //
///     // If a method takes `self` in some way, it is tracked by default. You can opt out of this
///     // behavior with the `#[untracked_self]` attribute.
///     #[untracked_self]
///     in MyType fn add(&mut self, incr: u32) -> JlrsResult<u32>  as increment!;
///
///     // Exports the alias `MyTypeAlias` for `MyType`.
///     //
///     // This is exposes as `const MyTypeAlias = MyType`.
///     type MyTypeAlias = MyType;
///
///     // Exports `MY_CONST` as the constant `MY_CONST`, its type must implement `IntoJulia`.
///     // `MY_CONST` can be defined in Rust as either static or constant data, i.e. both
///     // `static MY_CONST: u8 = 1` and `const MY_CONST: u8 = 1` can be exposed this way.
///     const MY_CONST: u8;
///
///     // Exports `MY_CONST` as the global `MY_GLOBAL`, its type must implement `IntoJulia`.
///     // `MY_CONST` can be defined in Rust as either static or constant data, i.e. both
///     // `static MY_CONST: u8 = 1` and `const MY_CONST: u8 = 1` can be exposed this way.
///     static MY_CONST: u8 as MY_GLOBAL;
///
///     // You can loop over types to export types and functions multiple times with
///     // different type parameters.
///     for T in [f32, f64] {
///         fn has_generic(t: T) -> T;
///
///         // POpaque<T> must implement `ParametricBase` and `ParametricVariant`.
///         struct POpaque<T>;
///
///         in POpaque<T> fn new(value: T) -> TypedValueRet<POpaque<T>> as POpaque;
///     }
///
///     // You can use an environment of type parameters to define generic functions.
///     // type GenericEnv = tvars!(tvar!('T'; AbstractFloat), tvar!('N'), tvar!('A'; AbstractArray<tvar!('T'), tvar!('N')>));
///     fn takes_generics_from_env(array: TypedValue<tvar!('A')>, data: TypedValue<tvar!('T')>) use GenericEnv;
/// }
/// ```
///
/// And this is all you need to do in Julia:
///
/// ```julia
/// module MyRustModule
/// using JlrsCore.Wrap
///
/// @wrapmodule("path/to/lib", :init_function_name)
///
/// function __init__()
///     @initjlrs
/// end
/// end
/// ```
///
/// It can be rather tricky to figure out how data is passed from Julia to Rust when `ccall`ing
/// a function written in Rust. Primitive and `isbits` types are passed by value, managed types
/// provided directly by jlrs are guaranteed to be boxed, all other types might be passed by
/// value or be boxed.
///
/// In order to avoid figuring out how such data is passed you can work with `TypedValue`, which
/// ensures the data is boxed by using `Any` in the signature of the generated `ccall` invocation,
/// but restricts the type of the data in the generated function to the type constructed from the
/// `TypedValue`'s type parameter.
///
/// [`AsyncCondition`]: https://docs.julialang.org/en/v1/base/base/#Base.AsyncCondition
#[proc_macro]
#[cfg(feature = "ccall")]
pub fn julia_module(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as JuliaModule);
    match input.generate_init_code() {
        Ok(a) => a,
        Err(b) => b.to_compile_error().into(),
    }
}

/// Encode the literal string passed to this macro as [`ConstantBytes`].
///
/// [`ConstantBytes`]: jlrs::data::types::construct_type::ConstantBytes
#[proc_macro]
pub fn encode_as_constant_bytes(item: TokenStream) -> TokenStream {
    let s: syn::LitStr = syn::parse_macro_input!(item as syn::LitStr);
    let input = s.value();
    convert_to_constant_bytes(input)
}

/// Conditional compilation depending on the used version of Julia.
///
/// This macro can be used instead of a custom `cfg` to conditionally compile code for
/// certain versions of Julia. For example, to enable a function when Julia 1.6, 1.7 or 1.10 is
/// used on Linux, or when Julia 1.7 or 1.10 is used on Windows:
///
/// `#[julia_version(since = "1.6", until = "1.10", except = ["1.8", "1.9"], windows_lts = false)]`
///
/// By default, `since = "1.6"`, `until = "1.10"`, `except = []`, and `windows_lts = None`, so the
/// above can be written more compactly as:
///
/// `#[julia_version(except = ["1.8", "1.9"], windows_lts = false)]`.
#[proc_macro_attribute]
pub fn julia_version(attr: TokenStream, item: TokenStream) -> TokenStream {
    emit_if_compatible(attr, item)
}

/// Derive `IntoJulia`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(IntoJulia, attributes(jlrs))]
pub fn into_julia_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_into_julia(&ast)
}

/// Derive `IsBits`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(IsBits, attributes(jlrs))]
pub fn is_bits_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_is_bits(&ast)
}

/// Derive `HasLayout`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(HasLayout, attributes(jlrs))]
pub fn is_has_layout(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_has_layout(&ast)
}

/// Derive `Unbox`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(Unbox, attributes(jlrs))]
pub fn unbox_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_unbox(&ast)
}

/// Derive `Typecheck`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(Typecheck, attributes(jlrs))]
pub fn typecheck_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_typecheck(&ast)
}

/// Derive `ValidLayout`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(ValidLayout, attributes(jlrs))]
pub fn valid_layout_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_valid_layout(&ast)
}

/// Derive `ValidField`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(ValidField, attributes(jlrs))]
pub fn valid_field_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_valid_field(&ast)
}

/// Derive `ConstructType`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(ConstructType, attributes(jlrs))]
pub fn construct_type_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_construct_type(&ast)
}

/// Derive `CCallArg`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(CCallArg, attributes(jlrs))]
pub fn ccall_arg_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_ccall_arg(&ast)
}

/// Derive `CCallReturn`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(CCallReturn, attributes(jlrs))]
pub fn ccall_return_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_ccall_return(&ast)
}

/// Derive `Enum`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(Enum, attributes(jlrs))]
pub fn enum_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_enum(&ast)
}
