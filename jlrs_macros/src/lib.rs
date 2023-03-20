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
use self::version::emit_if_compatible;

/// Export functions, types and constants defined in Rust as a Julia module.
///
/// This macro generates an initialization function. This function is used in combination with the
/// macros provided by the `Jlrs.Wrap` module to automatically generate the content of that
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
///     // Exports the function `foo` as `bar`.
///     //
///     // The `unsafe extern "C" part of the signature must be elided, the signature is verified
///     // in the generated code to ensure it's correct and that the function uses the C ABI.
///     //
///     // The `as <exposed_name>` part is optional, by default the function is exported with the
///     // name it has in Rust, the exposed name can end in an exclamation mark.
///     //
///     // A docstring can be provided with the doc attribute; if multiple functions are exported
///     // with the same name it shoud only be documented once. A multi-line docstring can be
///     // created by providing multiple doc attributes for the same item:
///     #[doc = "    bar(arr::Array)"]
///     #[doc = ""]
///     #[doc = "Documentation for this function"]
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
///     //
///     // A Rust function is generated to call this method, so unlike free-standing functions
///     // exported methods don't have to use the C ABI.
///     in MyType fn new(arg0: TypedValue<u32>) -> TypedValueRet<MyType> as MyForeignType;
///
///     // Exports `MyType::add` as the function `increment!`.
///     //
///     // Methods that take `self` in some way must return a `RustResultRet` because the
///     // generated function tracks the borrow of `self` before calling the exported method. If
///     // `self` is taken by value, it's cloned after being tracked.
///     in MyType fn add(&mut self, incr: u32) -> RustResultRet<u32>  as increment!;
///
///     // Exports the function `long_running_func`, the returned `AsyncCallback` is executed on
///     // another thread.
///     //
///     // `AsyncCallback` is essentially a trait alias for a closure with a specific signature:
///     //
///     // ```
///     // impl<T, U> AsyncCallback<T> for U
///     // where
///     //     T: IntoJulia + Send + ConstructType,
///     //     U: 'static + Send + FnOnce() -> JlrsResult<T>,
///     // {
///     // }
///     // ```
///     //
///     // I.e., the exported function must return a closure. The generated Julia function waits
///     // for the closure to finish using an `AsyncCondition`. Because the closure is executed on
///     // another thread you can't call Julia functions or allocate Julia data from it, but it is
///     // possible to (mutably) access Julia data by tracking it.
///     //
///     // In order to be able to use tracked data from the closure,  `Unbound` managed types must
///     // be used. Only `(Typed)ValueUnbound` and `(Typed)ArrayUnbound` exist,  they're aliases
///     // for `(Typed)Value` and `(Typed)Array` with static lifetimes. The generated Julia
///     // function guarantees all data passed as an argument lives at least until the closure has
///     // finished, the tracked data must only be shared with that closure.
///     async fn long_running_func(
///         array: ArrayUnbound
///     ) -> JlrsResult<impl AsyncCallback<i32>>;
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
/// }
/// ```
///
/// And this is all you need to do in Julia:
///
/// ```julia
/// module MyRustModule
/// using Jlrs.Wrap
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
#[proc_macro]
#[cfg(feature = "ccall")]
pub fn julia_module(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as JuliaModule);
    input.generate_init_code().unwrap()
}

/// Conditional compilation depending on the used version of Julia.
///
/// This macro can be used instead of a custom `cfg` to conditionally compile code for
/// certain versions of Julia. For example, to enable a function when Julia 1.6, 1.7 or 1.10 is
/// used on Linux, or when Julia 1.7 or 1.10 is used on Windows:
///
/// `#[julia_version(since = "1.6", until = "1.10", except = ["1.8", "1.9"], windows_lts = false)]`
///
/// By default, `since = "1.6"`, `until = "1.10"`, `except = []`, and `windows_lts = true`, so the
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
