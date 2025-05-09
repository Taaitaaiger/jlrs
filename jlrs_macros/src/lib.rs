mod constant_bytes;
#[cfg(feature = "derive")]
mod derive;
#[cfg(feature = "ccall")]
mod module;
mod version;

use proc_macro::TokenStream;

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
///     // You can loop over types to export types and functions multiple times with
///     // different type parameters.
///     for T in [f32, f64] {
///         fn has_generic(t: T) -> T;
///
///         // POpaque<T> must implement `OpaqueType`.
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
/// certain versions of Julia. For example, to enable a function when Julia 1.10 or 1.12 is
/// used:
///
/// `#[julia_version(since = "1.10", until = "1.12", except = ["1.11"])]`
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
    use derive::into_julia::impl_into_julia;

    let ast = syn::parse(input).unwrap();
    match impl_into_julia(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `IsBits`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(IsBits, attributes(jlrs))]
pub fn is_bits_derive(input: TokenStream) -> TokenStream {
    use derive::is_bits::impl_is_bits;

    let ast = syn::parse(input).unwrap();
    match impl_is_bits(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `HasLayout`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(HasLayout, attributes(jlrs))]
pub fn is_has_layout(input: TokenStream) -> TokenStream {
    use derive::has_layout::impl_has_layout;

    let ast = syn::parse(input).unwrap();
    match impl_has_layout(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `Unbox`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(Unbox, attributes(jlrs))]
pub fn unbox_derive(input: TokenStream) -> TokenStream {
    use derive::unbox::impl_unbox;

    let ast = syn::parse(input).unwrap();
    match impl_unbox(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `Typecheck`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(Typecheck, attributes(jlrs))]
pub fn typecheck_derive(input: TokenStream) -> TokenStream {
    use derive::typecheck::impl_typecheck;

    let ast = syn::parse(input).unwrap();
    match impl_typecheck(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `ValidLayout`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(ValidLayout, attributes(jlrs))]
pub fn valid_layout_derive(input: TokenStream) -> TokenStream {
    use derive::valid_layout::impl_valid_layout;

    let ast = syn::parse(input).unwrap();
    match impl_valid_layout(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `ValidField`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(ValidField, attributes(jlrs))]
pub fn valid_field_derive(input: TokenStream) -> TokenStream {
    use derive::valid_field::impl_valid_field;

    let ast = syn::parse(input).unwrap();
    match impl_valid_field(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `ConstructType`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(ConstructType, attributes(jlrs))]
pub fn construct_type_derive(input: TokenStream) -> TokenStream {
    use derive::construct_type::impl_construct_type;

    let ast = syn::parse(input).unwrap();
    match impl_construct_type(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `CCallArg`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(CCallArg, attributes(jlrs))]
pub fn ccall_arg_derive(input: TokenStream) -> TokenStream {
    use derive::ccall_arg::impl_ccall_arg;

    let ast = syn::parse(input).unwrap();
    match impl_ccall_arg(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `CCallReturn`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(CCallReturn, attributes(jlrs))]
pub fn ccall_return_derive(input: TokenStream) -> TokenStream {
    use derive::ccall_return::impl_ccall_return;

    let ast = syn::parse(input).unwrap();
    match impl_ccall_return(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `Enum`.
///
/// Should only be used in combination with layouts generated by JlrsReflect.jl
#[cfg(feature = "derive")]
#[proc_macro_derive(Enum, attributes(jlrs))]
pub fn enum_derive(input: TokenStream) -> TokenStream {
    use derive::enum_impl::impl_enum;

    let ast = syn::parse(input).unwrap();
    match impl_enum(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `OpaqueType`.
#[cfg(feature = "derive")]
#[proc_macro_derive(OpaqueType, attributes(jlrs))]
pub fn opaque_type_derive(input: TokenStream) -> TokenStream {
    use derive::opaque_type::impl_opaque_type;

    let ast = syn::parse(input).unwrap();
    match impl_opaque_type(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive `ForeignType`.
#[cfg(feature = "derive")]
#[proc_macro_derive(ForeignType, attributes(jlrs))]
pub fn foreign_type_derive(input: TokenStream) -> TokenStream {
    use derive::foreign_type::impl_foreign_type;

    let ast = syn::parse(input).unwrap();
    match impl_foreign_type(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}
