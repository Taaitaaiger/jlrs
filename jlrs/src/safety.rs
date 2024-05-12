//! Safety
//!
//! The usual goal for crates that provide bindings to libraries written in other languages is
//! safety. While jlrs strives to make as many things as possible safe to do from Rust, Julia and
//! Rust are two very different programming languages, and there are plenty of ways to mess things
//! up if you're not careful. As a result, you won't be able to avoid writing unsafe code if you
//! use this crate. Here you'll find general guidelines to use Rust and Julia together safely.
//!
//! # General rules
//!
//! The first bit of unsafety you'll run into is initializing Julia. Julia can be initialized
//! once, and it's technically possible to cause problems by initializing Julia from two threads
//! by using some other crate unrelated to jlrs.
//!
//! More importantly, mutating Julia data is considered unsafe. The main reason for this is that
//! a lot of things in Julia are mutable that must absolutely not be mutated. An obvious example is
//! the `Core` module, you should never mutate its contents, but there's nothing preventing you from
//! doing so either.
//!
//! Similarly, it's unsafe to call Julia functions. Julia has no unsafe keyword, there's no real
//! way to distinguish between functions like `+` and `unsafe_load`. Julia functions can also have
//! arbitrary side-effects. For example, it's possible to change global values by calling Julia
//! functions; while `Module::set_global` prevents you from setting a global that contains
//! borrowed data, this limitation doesn't exist inside a Julia function. Finally, you have to be
//! careful when using tasks. Any data that might be mutated by a task must not be accessed from
//! Rust. In general, you must not call any Julia function that schedules and returns a task, but
//! use the trait methods of `CallAsync` to schedule function calls as new tasks instead. Working
//! tasks is only supported when an async runtime is used, it's not supported by the local runtime
//! or when calling Rust from Julia.
//!
//! # Memory-safety
//!
//! All Julia data is owned by its garbage collector. In order to ensure this data is not freed
//! while you're still using it, it must be rooted in the GC frame of a scope. All data that can
//! e reached from these roots won't be freed. Frames form a stack, every time you enter a new
//! scope is created a new frame is pushed to the stack, and it's popped when you leave that
//! scope.
//!
//! Methods that return new data generally take an argument that implements `Scope` or a
//! `PartialScope`, the result is automatically rooted in the frame associated with that scope
//! before it is returned. Rooted Julia data is always returned as a pointer wrapper type like
//! `Value` or `Array`, these types have at least one lifetime which ensures they can't be
//! returned from the scope whose frame roots them.
//!
//! There are many cases where data doesn't have to be rooted. If you call a Julia function that
//! returns a result you don't care about and will never use, you don't need to root the result. A
//! Julia module is a global scope, its contents can be considered globally rooted as long as the
//! module isn't redefined. This means you can use functions and constants defined in modules
//! without rooting them.
//!
//! Methods that return Julia data without rooting it are available, rather than a pointer wrapper
//! they return a `Ref` instead. A `Ref` can be converted to its associated wrapper type by
//! calling either `Ref::wrapper` or `Ref::root`. It's your responsibility to ensure that the
//! `Ref` points to valid data.
//!
//! Some other examples of data that doesn't need to be rooted are singletons like `nothing`,
//! `Bool` and `UInt8` values, concrete `DataType`s (types with no free type parameters that can
//! be instantiated), and `Symbol`s.
//!
//! # `ccall`-specific rules
//!
//! Julia has a powerful interface, `ccall`, that can be used to call arbitrary functions with
//! the C ABI, i.e. `extern "C"` functions. Immutable data is unboxed whenever it's used as an
//! argument of a `ccall`ed function, and boxed if used as a return type. For example,  if you
//! pass a `UInt8` as an argument, the function is called with a `u8`; if it returns one it's
//! automatically converted to a `UInt8`. The signature can't be checked for correctness, it's
//! your responsibility to ensure the types match.
//!
//! The following table lists the most relevant matching types.
//!
//! | Julia Base Type | Rust Type                         |
//! |-----------------|-----------------------------------|
//! | `Bool`          | [`Bool`]                          |
//! | `Char`          | [`Char`]                          |
//! | `UInt8`         | `u8`                              |
//! | `Int8`          | `i8`                              |
//! | `UInt16`        | `u16`                             |
//! | `Int16`         | `i16`                             |
//! | `UInt32`        | `u32`                             |
//! | `Int32`         | `i32`                             |
//! | `UInt64`        | `u64`                             |
//! | `Int64`         | `i64`                             |
//! | `UInt`          | `usize`                           |
//! | `Int`           | `isize`                           |
//! | `Float32`       | `f32`                             |
//! | `Float64`       | `f64`                             |
//! | `Cvoid`         | `()`                              |
//! | `T` (immutable) | `T` (generated by JlrsReflect.jl) |
//! | `Any`           | [`Value`]                         |
//! | `T` (mutable)   | [`Value`]                         |
//! | `Array{T, N}`   | [`Array`] or [`TypedArray<T>`]    |
//!
//! In order to make use of jlrs from a `ccall`ed function you must first create an instance of
//! `CCall`. This is unsafe because you must create only one per ccall'ed function. If you
//! create multiple you can easily corrupt the state of the garbage collector.
//!
//! After creating an instance of `CCall`, you can create a new scope from which you can use most
//! features of jlrs. In general, you should avoid calling unchecked variants of methods. These
//! methods call a function from the Julia C API that can throw an exception which can't be caught
//! in Rust. Exceptions in Julia are implemented as jumps, and jumping over a Rust function back
//! to Julia is undefined behavior.
//!
//! [`Bool`]: crate::data::layout::bool::Bool
//! [`Char`]: crate::data::layout::char::Char
//! [`Value`]: crate::data::managed::value::Value
//! [`Array`]: crate::data::managed::array::Array
//! [`TypedArray<T>`]: crate::data::managed::array::TypedArray
