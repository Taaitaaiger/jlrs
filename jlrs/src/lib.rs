//! jlrs is a crate that provides access to the Julia C API. It can be used to embed Julia in Rust
//! applications and to write interop libraries to Rust crates that can be used by Julia.
//!
//! Julia versions 1.6 up to and including 1.11 are supported, but only the LTS and stable versions
//! are actively tested. Using the current stable version of Julia is highly recommended. The
//! minimum supported Rust version is currently 1.77.
//!
//! A tutorial is available [here](https://taaitaaiger.github.io/jlrs-tutorial/).
//!
//! # Overview
//!
//! An incomplete list of features that are currently supported by jlrs:
//!
//!  - Access arbitrary Julia modules and their content.
//!  - Call Julia functions, including functions that take keyword arguments.
//!  - Handle exceptions or convert them to an error message, optionally with color.
//!  - Include and call your own Julia code.
//!  - Use custom system images.
//!  - Create values that Julia can use, and convert them back to Rust, from Rust.
//!  - Access the type information and fields of such values. Inline and bits-union fields can be
//!    accessed directly.
//!  - Create and use n-dimensional arrays. The `jlrs-ndarray` feature can be enabled for
//!    integration with ndarray.
//!  - Map Julia structs to Rust structs, the Rust implementation can be generated with the
//!    JlrsCore package.
//!  - Structs that can be mapped to Rust include those with type parameters and bits unions.
//!  - Use Julia from multiple threads either directly or via Julia-aware thread pools.
//!  - Export Rust types, methods and functions to Julia with the `julia_module` macro.
//!  - Libraries that use `julia_module` can be compiled with BinaryBuilder and distributed as
//!    JLLs.
//!
//!
//! # Prerequisites
//!
//! Julia must be installed before jlrs can be used, jlrs is compatible with Julia 1.6 up to and
//! including Julia 1.11. If the JlrsCore package has not been installed, it will automatically be
//! installed when jlrs is initialized by default. jlrs has not been tested with juliaup yet on
//! Linux and macOS.
//!
//! ## Linux
//!
//! The recommended way to install Julia is to download the binaries from the official website,
//! which is distributed as an archive containing a directory called `julia-x.y.z`. This directory
//! contains several other directories, including a `bin` directory containing the `julia`
//! executable.
//!
//! During compilation, the paths to the header and library are normally detected automatically by
//! executing the command `which julia`. The path to `julia.h` must be
//! `$(which julia)/../include/julia/julia.h` and the path to the library
//! `$(which julia)/../lib/libjulia.so`. If you want to override this default behaviour the
//! `JULIA_DIR` environment variable must be set to the path to the appropriate `julia.x-y-z`
//! directory, in this case `$JULIA_DIR/include/julia/julia.h` and
//! `$JULIA_DIR/lib/libjulia.so` are used instead.
//!
//! In order to be able to load `libjulia.so` this file must be on the library search path. If
//! this is not the case you must add `/path/to/julia-x.y.z/lib` to the `LD_LIBRARY_PATH`
//! environment variable.
//!
//! ## macOS
//!
//! Follow the instructions for Linux, but replace `LD_LIBRARY_PATH` with `DYLD_LIBRARY_PATH`.
//!
//! ## Windows
//!
//! Julia can be installed using juliaup, or with the installer or portable installation
//! downloaded from the official website. In the first case, Julia has been likely installed in
//! `%USERPROFILE%\.julia\juliaup\julia-x.y.z+0~x64`, while using the installer or extracting
//! allows you to pick the destination. After installation or extraction a folder called
//! `Julia-x.y.z` exists, which contains several folders including a `bin` folder containing
//! `julia.exe`. The path to the `bin` folder must be added to the `Path` environment variable.
//!
//! Julia is automatically detected by executing the command `where julia`. If this returns
//! multiple locations the first one is used. The default can be overridden by setting the
//! `JULIA_DIR` environment variable. This doesn't work correctly with juliaup, in this case
//! the environment variable must be set.
//!
//!
//! # Features
//!
//! Most functionality of jlrs is only available if the proper features are enabled. These
//! features generally belong to one of three categories: versions, runtimes and utilities.
//!
//! ## Versions
//!
//! The Julia C API is unstable and there are minor incompatibilities between different versions
//! of Julia. To ensure the correct bindings are used for a particular version of Julia you must
//! enable a version feature. The following version features currently exist:
//!
//!  - `julia-1-6`
//!  - `julia-1-7`
//!  - `julia-1-8`
//!  - `julia-1-9`
//!  - `julia-1-10`
//!  - `julia-1-11`
//!
//! Exactly one version feature must be enabled. Otherwise, jlrs will fail to compile.
//!
//! If you want your crate to be compatible with multiple versions of Julia, you should "reexport"
//! these version features as follows:
//!
//! ```toml
//! [features]
//! julia-1-6 = ["jlrs/julia-1-6"]
//! julia-1-7 = ["jlrs/julia-1-7"]
//! julia-1-8 = ["jlrs/julia-1-8"]
//! julia-1-9 = ["jlrs/julia-1-9"]
//! julia-1-10 = ["jlrs/julia-1-10"]
//! julia-1-11 = ["jlrs/julia-1-11"]
//! ```
//!
//! ## Runtimes
//!
//! A runtime lets initialize Julia from Rust application, the following features enable a
//! runtime:
//!
//!  - `local-rt`
//!
//!    Enables the local runtime. The local runtime provides single-threaded, blocking access to
//!    Julia.
//!
//!  - `async-rt`
//!
//!    Enables the async runtime. The async runtime runs on a separate thread and can be used from
//!    multiple threads.
//!
//!  - `tokio-rt`
//!
//!    The async runtime requires an executor. This feature provides a tokio-based executor.
//!
//!  - `multi-rt`
//!
//!    Enables the multithreaded runtime. The multithreaded runtime lets you call Julia from
//!    arbitrary threads. It can be combined with the `async-rt` feature to create Julia-aware
//!    thread pools. This feature requires Julia 1.9 or higher.
//!
//!
//! <div class="warning"><strong>WARNING</strong>: Runtime features must only be enabled by applications that embed Julia.
//! Libraries must never enable a runtime feature.</div>
//!
//! <div class="warning"><strong>WARNING</strong>: When a runtime feature is enabled on Linux, set
//! <code>RUSTFLAGS="-Clink-args=-rdynamic"</code> if you want fast code.</div>
//!
//! ## Utilities
//!
//! All other features are called utility features. The following are available:
//!
//! - `async`
//!
//!   Enable the features of the async runtime which don't depend on the executor. This
//!   can be used in libraries which provide implementations of tasks that the async runtime can
//!   handle.
//!
//! - `jlrs-derive`
//!
//!   This feature should be used in combination with the code generation provided by the Reflect
//!   module in the JlrsCore package. This module lets you generate Rust implementations of Julia
//!   structs, this generated code uses custom derive macros made available with this feature to
//!   enable the safe conversion of data from Julia to Rust, and from Rust to Julia in some cases.
//!
//! - `jlrs-ndarray`
//!
//!   Access the content of a Julia array as an `ArrayView` or `ArrayViewMut` from ndarray.
//!
//! - `f16`
//!
//!   Adds support for working with Julia's `Float16` type from Rust using half's `f16` type.
//!
//! - `complex`
//!
//!   Adds support for working with Julia's `Complex` type from Rust using num's `Complex` type.
//!
//! - `ccall`
//!
//!   Julia's `ccall` interface can be used to call functions written in Rust from Julia. No
//!   runtime can be used in this case because Julia has already been initialized, when this
//!   feature is enabled the `CCall` struct is available which offers the same functionality as
//!   the local runtime without initializing Julia. The `julia_module` macro is provided to
//!   easily export functions, types, and data in combination with the macros from the Wrap
//!   module in the JlrsCore package.
//!
//! - `lto`
//!
//!   jlrs depends on a support library written in C, if this feature is enabled this support
//!   library is built with support for cross-language LTO which can provide a significant
//!   performance boost.
//!
//!   This feature has only been tested on Linux and requires building the support library using a
//!   version of `clang` with the same major version as `rustc`'s LLVM version; e.g. rust 1.78.0
//!   uses LLVM 18, so it requires `clang-18`. You can check what version you need by executing
//!   `rustc -vV`.
//!
//!   You must set the `RUSTFLAGS` environment variable if this feature is enabled, and possibly the
//!   `CC` environment variable. Setting `RUSTFLAGS` overrides the default flags that jlrs sets, so
//!   you must set at least the following flags:
//!   `RUSTFLAGS="-Clinker-plugin-lto -Clinker=clang-XX -Clink-arg=-fuse-ld=lld -Clink-args=-rdynamic"`.
//!
//! - `diagnostics`
//!
//!   Enable custom diagnostics for several traits because the default lint is unhelpful. This feature
//!   requires Rust 1.78.
//!
//! - `i686`
//!
//!   Link with a 32-bit build of Julia on Linux, only used for cross-compilation.
//!
//! - `windows`
//!
//!   Flag that must be enabled when cross-compiling for Windows from Linux.
//!
//! - `debug`
//!
//!   Link with a debug build of Julia on Linux.
//!
//! - `no-link`
//!
//!   Don't link Julia.
//!
//! - `yggdrasil`
//!
//!   Flag that must be enabled when compiling with BinaryBuilder.
//!
//! You can enable all features except `debug`, `i686`, `windows`, `no-link`, `lto` and
//! `yggdrasil` by enabling the `full` feature. If you don't want to enable any runtimes either,
//! you can use `full-no-rt`.
//!
//!
//! # Using jlrs
//!
//! How you should use this crate depends on whether you're embedding Julia in a Rust application,
//! or writing a library you want to call from Julia. We're going to focus on embedding first.
//! Some topics covered in the section about the local runtime section are relevant for users of
//! the other runtimes, and library authors who want to call into Rust from Julia and into Julia
//! again from Rust.
//!
//!
//! ## Calling Julia from Rust
//!
//! If you want to embed Julia in a Rust application, you must enable a runtime and a version
//! feature:
//!
//! `jlrs = {version = "0.20.0", features = ["local-rt", "julia-1-11"]}`
//!
//! `jlrs = {version = "0.20.0", features = ["tokio-rt", "julia-1-11"]}`
//!
//! `jlrs = {version = "0.20.0", features = ["multi-rt", "julia-1-11"]}`
//!
//! When Julia is embedded in an application, it must be initialized before it can be used. A
//! [`Builder`] is available to configure the runtime before starting it. This lets you set
//! options like the number of threads Julia can start or instruct Julia to use a custom system
//! image.
//!
//! There are three runtimes: the local, async and multithreaded runtime. Let's take a look at them
//! in that same order.
//!
//!
//! ### Local runtime
//!
//! The local runtime initializes Julia on the current thread and lets you call into Julia from
//! that one thread.
//!
//! Starting this runtime is quite straightforward, you only need to create a `Builder` and call
//! [`Builder::start_local`]. This initializes Julia on the current thread and returns a
//! [`LocalHandle`] that lets you call into Julia. The runtime shuts down when this handle is
//! dropped.
//!
//! The handle by itself doesn't let you do much directly. In order to create Julia data and call
//! Julia functions, a scope must be created first. These scopes ensure Julia data can only be
//! used while it's guaranteed to be safe from being freed by Julia's garbage collector. jlrs has
//! dynamically-sized scopes and statically-sized local scopes. The easiest way to familiarize
//! ourselves with these scopes is with a simple example where we allocate some Julia data.
//!
//! Dynamically-sized scope:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! // To create to dynamically-sized scope we need to create a stack first.
//! //
//! // NB: This is a relatively expensive operation, if you need to create a stack you should do
//! // so early and reuse it as much as possible.
//! julia.with_stack(|mut stack| {
//!     stack.scope(|mut frame| {
//!         // We use `frame` every time we create Julia data. This roots the data in the
//!         // frame, which means the garbage collector is guaranteed to leave this data alone
//!         // at least until we leave this scope. Even if the frame is dropped, the data is
//!         // guaranteed to be protected until the scope ends.
//!         //
//!         // This value inherits `frame`'s lifetime, which prevents it from being returned
//!         // from this closure.
//!         let _v = Value::new(&mut frame, 1usize);
//!     })
//! })
//! # }
//! ```
//!
//! Statically-sized local scope:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! // Local scopes can be created without creating a stack, but you need to provide the exact
//! // number of slots you need.
//! julia.local_scope::<_, 1>(|mut frame| {
//!     // We root one value in this frame, so the required capacity of this local scope is 1.
//!     let _v = Value::new(&mut frame, 1usize);
//!
//!     // Because there is only one slot available, uncommenting the next line would cause a
//!     // panic unless we changed `local_scope::<1>` to `local_scope::<2>`.
//!     // let _v2 = Value::new(&mut frame, 2usize);
//! })
//! # }
//! ```
//!
//! In general you should prefer using local scopes over dynamic scopes. For more information
//! about scopes, frames, and other important topics involving memory management, see the
//! [`memory`] module.
//!
//! In the previous two examples we saw the function [`Value::new`], which converts Rust to Julia
//! data. In particular, calling `Value::new(&mut frame, 1usize)` returned a Julia `UInt` with the
//! value 1. Any type that implements [`IntoJulia`] can be converted to Julia data with this
//! method. Similarly, any type that implements [`Unbox`] can be converted from Julia to Rust.
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! julia.local_scope::<_, 1>(|mut frame| {
//!     // We root one value in this frame, so the required capacity of this local scope is 1.
//!     let v = Value::new(&mut frame, 1.0f32);
//!
//!     // `Value::unbox` checks if the conversion is valid before unboxing the value.
//!     let unboxed = v.unbox::<f32>().expect("not a Float32");
//!     assert_eq!(unboxed, 1.0f32);
//! })
//! # }
//! ```
//!
//! We don't just want to unbox the exact same data we've just allocated, obviously. We want to
//! call functions written in Julia with that data. This boils down to accessing the function in
//! the right module and calling it.
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! // This scope contains a fallible operation. Whenever the return type is a `Result` and the
//! // `?` operator is used, the closure typically has to be annotated with its return type.
//! julia
//!     .local_scope::<_, 4>(|mut frame| -> JlrsResult<()> {
//!         let v1 = Value::new(&mut frame, 1.0f32); // 1
//!         let v2 = Value::new(&mut frame, 2.0f32); // 2
//!
//!         // The Base module is globally rooted, so we can access it with `&frame` instead of
//!         // `&mut frame`. Only uses of mutable references count towards the necessary capacity
//!         // of the local scope.
//!         let base = Module::base(&frame);
//!
//!         // The Base module contains the `+` function.
//!         let func = base.global(&mut frame, "+")?; // 3
//!
//!         // `Value` implements the `Call` trait which lets us call it as a function. Any
//!         // callable object can be called this way. Functions can throw exceptions, if it does
//!         // it's caught and returned as the `Err` branch of a `Result`. Converting the result
//!         // to a `JlrsResult` converts it to its error message and lets it be returned with the
//!         // `?` operator.
//!         //
//!         // Calling Julia functions is unsafe. Some functions are inherently unsafe to call,
//!         // their names typically start with `unsafe`. Other functions might involve
//!         // multithreading and affect how you must access certain global variables. Adding two
//!         // numbers is not an issue.
//!         let v3 = unsafe {
//!             func.call(&mut frame, [v1, v2]) // 4
//!                 .into_jlrs_result()?
//!         };
//!
//!         let unboxed = v3.unbox::<f32>().expect("not a Float32");
//!         assert_eq!(unboxed, 3.0f32);
//!
//!         Ok(())
//!     })
//!     .unwrap()
//! # }
//! ```
//!
//! Julia functions are highly generic, calling functions with the `Call` trait calls the most
//! appropriate function given the arguments. The `+` function for example accepts any number of
//! arguments and returns their sum, so we can easily adjust the previous example to add more
//! numbers together.
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! julia
//!     .local_scope::<_, 5>(|mut frame| -> JlrsResult<()> {
//!         let v1 = Value::new(&mut frame, 1.0f32); // 1
//!         let v2 = Value::new(&mut frame, 2.0f32); // 2
//!         let v3 = Value::new(&mut frame, 3.0f32); // 3
//!
//!         let v3 = unsafe {
//!             Module::base(&frame)
//!                 .global(&mut frame, "+")? // 4
//!                 .call(&mut frame, [v1, v2, v3]) // 5
//!                 .into_jlrs_result()?
//!         };
//!
//!         let unboxed = v3.unbox::<f32>()?;
//!         assert_eq!(unboxed, 6.0f32);
//!
//!         Ok(())
//!     })
//!     .unwrap()
//! # }
//! ```
//!
//! By default you can only access the `Main`, `Base` and `Core` module. If you want to use
//! functions defined in standard libraries or installed packages, you must load them first.
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! let mut julia = Builder::new().start_local().unwrap();
//!
//! unsafe {
//!     julia
//!         .using("LinearAlgebra")
//!         .expect("LinearAlgebra package does not exist");
//! }
//!
//! julia.local_scope::<_, 1>(|mut frame| {
//!     let lin_alg = Module::package_root_module(&frame, "LinearAlgebra");
//!     assert!(lin_alg.is_some());
//!
//!     let mul_mut_func = lin_alg.unwrap().global(&mut frame, "mul!");
//!     assert!(mul_mut_func.is_ok());
//! })
//! # }
//! ```
//!
//!
//! ### Multithreaded runtime
//!
//! The multithreaded runtime initializes Julia on some background thread, and allows calling into
//! Julia from arbitrary threads. This runtime is available since Julia 1.9.
//!
//! To start this runtime you need to create a `Builder` and call [`Builder::spawn_mt`]. It has
//! its own handle type, [`MtHandle`], which can be cloned and sent to other threads. Unlike the
//! local runtime's `LocalHandle`, it can't be used directly, you must call [`MtHandle::with`]
//! first to ensure the thread is in a state where it can call into Julia.
//!
//! Let's call into Julia from two separate threads to see it in action:
//!
//! ```
//! use std::thread;
//!
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! # #[cfg(feature = "multi-rt")]
//! # #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
//! # {
//! // When the multithreaded runtime is spawned, a new thread is spawned that initializes Julia.
//! // This thread simply waits for shutdown to be requested when the final `mt_handle` is
//! // dropped. An `MtHandle` and a `JoinHandle` to that runtime thread are returned.
//! let (mut mt_handle, th_handle) = Builder::new().spawn_mt().unwrap();
//!
//! // We can send different instances of `MtHandle` to different threads. `MtHandle` is `Send`,
//! // not `Sync` so we need to clone it in advance.
//! let mut mt_handle2 = mt_handle.clone();
//!
//! let t1 = thread::spawn(move || {
//!     // By calling `MtHandle::with` we enable the thread to call into Julia. The handle you can
//!     // use in that closure provides the same functionality as the local runtime's
//!     // `LocalHandle`.
//!     mt_handle.with(|handle| {
//!         handle.local_scope::<_, 1>(|mut frame| unsafe {
//!             let _v = Value::new(&mut frame, 1);
//!         })
//!     })
//! });
//!
//! let t2 = thread::spawn(move || {
//!     mt_handle2.with(|handle| {
//!         handle.local_scope::<_, 1>(|mut frame| unsafe {
//!             let _v = Value::new(&mut frame, 2);
//!         })
//!     })
//! });
//!
//! t1.join().expect("thread 1 panicked");
//! t2.join().expect("thread 2 panicked");
//!
//! // No more handles exist, so the runtime thread has shut down.
//! th_handle.join().unwrap();
//! # }
//! # }
//! ```
//!
//! It's important that you avoid blocking operations unrelated to Julia in a call to
//! `MtHandle::with`. The reason is that this can prevent the garbage collector from running.
//! Roughly speaking, whenever Julia data is allocated the garbage collector can signal it has to
//! run. This blocks the thread that tried to allocate data, and every other thread will similarly
//! block when they try to allocate data, until every thread is blocked. When all threads are
//! blocked, the garbage collector collects garbage and unblocks the threads when it's done.
//!
//! The implication is that long-running operations which don't allocate Julia data can block the
//! garbage collector, which can grind Julia to a halt. Outside calls to `MtHandle::with`, the
//! thread is guaranteed to be in a state where it won't block the garbage collector from running.
//!
//!
//! ### Async runtime
//!
//! While the sync and multithreaded runtimes let you call into Julia directly from one or more
//! threads, the async runtime runs on a background thread and uses an executor to allow
//! running multiple tasks on that thread concurrently. Its handle type, `AsyncHandle`, can be
//! shared across threads like the `MtHandle`, and lets you send tasks to the runtime thread.
//!
//! The async runtime supports three kinds of tasks: blocking, async, and persistent tasks.
//! Blocking tasks run as a single unit and prevent other tasks from running until they've
//! completed. Async tasks run as a separate task on the executor, they can use async operations
//! and long-running Julia functions can be dispatched to a background thread. Persistent tasks
//! are similar to async tasks, they run as separate tasks but additionally have internal state
//! and can be called multiple times.
//!
//! Blocking task:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! let (julia, thread_handle) = Builder::new()
//!     .async_runtime(Tokio::<3>::new(false))
//!     .spawn()
//!     .unwrap();
//!
//! // When a task cannot be dispatched to the runtime because the
//! // channel is full, the dispatcher is returned in the `Err` branch.
//! // `blocking_task` is the receiving end of a tokio oneshot channel.
//! let blocking_task = julia
//!     .blocking_task(|mut frame| -> JlrsResult<f32> {
//!         Value::new(&mut frame, 1.0f32).unbox::<f32>()
//!     })
//!     .try_dispatch()
//!     .expect("unable to dispatch task");
//!
//! let res = blocking_task
//!     .blocking_recv()
//!     .expect("unable to receive result")
//!     .expect("blocking task failed.");
//!
//! assert_eq!(res, 1.0);
//!
//! // The runtime thread exits when the last instance of `julia` is dropped.
//! std::mem::drop(julia);
//! thread_handle.join().unwrap();
//! # }
//! ```
//!
//! Async task:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! struct AdditionTask {
//!     a: u64,
//!     b: u32,
//! }
//!
//! // Async tasks must implement the `AsyncTask` trait. Only the runtime thread can call the
//! // Julia C API, so the `run` method must not return a future that implements `Send` or `Sync`.
//! #[async_trait(?Send)]
//! impl AsyncTask for AdditionTask {
//!     // The type of the result of this task.
//!     type Output = JlrsResult<u64>;
//!
//!     // This async method replaces the closure from the previous examples,
//!     // an `AsyncGcFrame` can be used the same way as other frame types.
//!     async fn run<'frame>(&mut self, mut frame: AsyncGcFrame<'frame>) -> Self::Output {
//!         let a = Value::new(&mut frame, self.a);
//!         let b = Value::new(&mut frame, self.b);
//!
//!         let func = Module::base(&frame).global(&mut frame, "+")?;
//!
//!         // CallAsync::call_async schedules the function call on another thread.
//!         // The runtime can switch to other tasks while awaiting the result.
//!         // Safety: adding two numbers is safe.
//!         unsafe { func.call_async(&mut frame, [a, b]) }
//!             .await
//!             .into_jlrs_result()?
//!             .unbox::<u64>()
//!     }
//! }
//!
//! # fn main() {
//! let (julia, thread_handle) = Builder::new()
//!     .async_runtime(Tokio::<3>::new(false))
//!     .spawn()
//!     .unwrap();
//!
//! // When a task cannot be dispatched to the runtime because the
//! // channel is full, the dispatcher is returned in the `Err` branch.
//! // `async_task` is the receiving end of a tokio oneshot channel.
//! let async_task = julia
//!     .task(AdditionTask { a: 1, b: 2 })
//!     .try_dispatch()
//!     .expect("unable to dispatch task");
//!
//! let res = async_task
//!     .blocking_recv()
//!     .expect("unable to receive result")
//!     .expect("AdditionTask failed");
//!
//! assert_eq!(res, 3);
//!
//! // The runtime thread exits when the last instance of `julia` is dropped.
//! std::mem::drop(julia);
//! thread_handle.join().unwrap();
//! # }
//! ```
//!
//! Persistent task:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! struct AccumulatorTask {
//!     n_values: usize,
//! }
//!
//! // The internal state of a persistent task can contain Julia data.
//! struct AccumulatorTaskState<'state> {
//!     array: TypedArray<'state, 'static, usize>,
//!     offset: usize,
//! }
//!
//! // The same is true for implementations of `PersistentTask`.
//! #[async_trait(?Send)]
//! impl PersistentTask for AccumulatorTask {
//!     type Output = JlrsResult<usize>;
//!
//!     // The type of the task's internal state.
//!     type State<'state> = AccumulatorTaskState<'state>;
//!
//!     // The type of the additional data that the task must be called with.
//!     type Input = usize;
//!
//!     // This method is called before the task can be called.
//!     async fn init<'frame>(
//!         &mut self,
//!         mut frame: AsyncGcFrame<'frame>,
//!     ) -> JlrsResult<Self::State<'frame>> {
//!         // A `Vec` can be moved from Rust to Julia if the element type
//!         // implements `IntoJulia`.
//!         let data = vec![0usize; self.n_values];
//!         let array =
//!             TypedArray::from_vec(&mut frame, data, self.n_values)?.into_jlrs_result()?;
//!
//!         Ok(AccumulatorTaskState { array, offset: 0 })
//!     }
//!
//!     // Whenever the task is called, it's called with its state and the provided input.
//!     async fn run<'frame, 'state: 'frame>(
//!         &mut self,
//!         mut frame: AsyncGcFrame<'frame>,
//!         state: &mut Self::State<'state>,
//!         input: Self::Input,
//!     ) -> Self::Output {
//!         unsafe {
//!             let mut data = state.array.bits_data_mut();
//!             data[state.offset] = input;
//!         };
//!
//!         state.offset += 1;
//!         if (state.offset == self.n_values) {
//!             state.offset = 0;
//!         }
//!
//!         unsafe {
//!             Module::base(&frame)
//!                 .function(&mut frame, "sum")?
//!                 .call1(&mut frame, state.array.as_value())
//!                 .into_jlrs_result()?
//!                 .unbox::<usize>()
//!         }
//!     }
//! }
//!
//! # fn main() {
//! let (julia, thread_handle) = Builder::new()
//!     .async_runtime(Tokio::<3>::new(false))
//!     .spawn()
//!     .unwrap();
//!
//! let persistent_task = julia
//!     .persistent(AccumulatorTask { n_values: 2 })
//!     .try_dispatch()
//!     .expect("unable to dispatch task")
//!     .blocking_recv()
//!     .expect("unable to receive handle")
//!     .expect("init failed");
//!
//! // A persistent task can be called with its input, the same dispatch mechanism
//! // is used as above.
//! let res = persistent_task
//!     .call(1)
//!     .try_dispatch()
//!     .expect("unable to dispatch call")
//!     .blocking_recv()
//!     .expect("unable to receive handle")
//!     .expect("call failed");
//!
//! assert_eq!(res, 1);
//!
//! let res = persistent_task
//!     .call(2)
//!     .try_dispatch()
//!     .expect("unable to dispatch call")
//!     .blocking_recv()
//!     .expect("unable to receive handle")
//!     .expect("call failed");
//!
//! assert_eq!(res, 3);
//!
//! // If the `AsyncHandle` is dropped before the task is, the runtime continues
//! // running until the task has been dropped.
//! std::mem::drop(julia);
//! std::mem::drop(persistent_task);
//! thread_handle.join().unwrap();
//! # }
//! ```
//!
//!
//! ### Async, multithreaded runtime
//!
//! There are two non-exclusive ways the async runtime can be combined with the multithreaded
//! runtime. You can start the runtime thread with an async executor, which grants you both an
//! `AsyncHandle` to that thread and a `MtHandle`. This can be useful if you have code that must
//! run on the main thread.
//!
//! The second option is thread pools. When both runtimes are enabled, `MtHandle` lets you
//! construct pools of async worker threads that share a single task queue. Each pool can have an
//! arbitrary number of workers, which are automatically restarted if they die. Like the async
//! runtime, you interact with a pool through its `AsyncHandle`. The pool shuts down when the last
//! handle is dropped.
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! # #[cfg(feature = "multi-rt")]
//! # #[cfg(not(any(feature = "julia-1-6", feature = "julia-1-7", feature = "julia-1-8")))]
//! # {
//! let (mt_handle, async_handle, thread_handle) = Builder::new()
//!     .async_runtime(Tokio::<3>::new(false))
//!     .spawn_mt()
//!     .unwrap();
//!
//! let pool_handle = mt_handle
//!     .pool_builder(Tokio::<1>::new(false))
//!     .n_workers(2.try_into().unwrap())
//!     .spawn();
//!
//! // All handles must be dropped .
//! std::mem::drop(mt_handle);
//! std::mem::drop(pool_handle);
//! std::mem::drop(async_handle);
//! thread_handle.join().unwrap();
//! # }
//! # }
//! ```
//!
//!
//! ## Calling Rust from Julia
//!
//! Julia can call functions written in Rust thanks to its `ccall` interface, which lets you call
//! arbitrary functions which use the C ABI. These functions can be defined in dynamic libraries or
//! provided directly to Julia by converting a function pointer to a `Value`.
//!
//! ```
//! use jlrs::prelude::*;
//!
//! // This function will be provided to Julia as a pointer, so its name can be mangled.
//! unsafe extern "C" fn call_me(arg: Bool) -> isize {
//!     if arg.as_bool() {
//!         1
//!     } else {
//!         -1
//!     }
//! }
//!
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//!
//! julia
//!     .local_scope::<_, 3>(|mut frame| -> JlrsResult<_> {
//!         unsafe {
//!             // Cast the function to a void pointer
//!             let call_me_val = Value::new(&mut frame, call_me as *mut std::ffi::c_void);
//!
//!             // Value::eval_string can be used to create new functions.
//!             let func = Value::eval_string(
//!                 &mut frame,
//!                 "myfunc(callme::Ptr{Cvoid})::Int = ccall(callme, Int, (Bool,), true)",
//!             )
//!             .into_jlrs_result()?;
//!
//!             // Call the function and unbox the result.
//!             let result = func
//!                 .call1(&mut frame, call_me_val)
//!                 .into_jlrs_result()?
//!                 .unbox::<isize>()?;
//!
//!             assert_eq!(result, 1);
//!             Ok(())
//!         }
//!     })
//!     .unwrap();
//! # }
//! ```
//!
//! To create a library that Julia can use, you must compile your crate as a `cdylib`. To achieve
//! this you need to add
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! to your crate's `Cargo.toml`. You must also abort on panic:
//!
//! ```toml
//! [profile.release]
//! panic = "abort"
//! ```
//!
//! You must not enable any of jlrs's runtimes.
//!
//! The most versatile way to export Rust functions like `call_me` from the previous example is by
//! using the [`julia_module`] macro. This macro lets you export custom types and functions in a
//! way that is friendly to precompilation.
//!
//! In Rust, this macro is used as follows:
//!
//! ```ignore
//! use jlrs::prelude::*;
//!
//! fn call_me(arg: Bool) -> isize {
//!     if arg.as_bool() {
//!         1
//!     } else {
//!         -1
//!     }
//! }
//!
//! julia_module! {
//!     become callme_init_fn;
//!     fn call_me(arg: Bool) -> isize;
//! }
//! ```
//!
//! While on the Julia side things look like this:
//!
//! ```julia
//! module CallMe
//! using JlrsCore.Wrap
//!
//! @wrapmodule("./path/to/libcallme.so", :callme_init_fn)
//!
//! function __init__()
//!     @initjlrs
//! end
//! end
//! ```
//!
//! All Julia functions are automatically generated and have the same name as the exported
//! function:
//!
//! ```julia
//! @assert CallMe.call_me(true) == 1
//! @assert CallMe.call_me(false) == -1
//! ```
//!
//! This macro has many more capabilities than just exporting functions written in Rust. For more
//! information see the [documentation]. A practical example that uses this macro is the
//! [rustfft-jl] crate, which uses this macro to expose RustFFT to Julia. The recipe for
//! BinaryBuilder can be found [here].
//!
//! While `call_me` doesn't call back into Julia, it is possible to call arbitrary functions from
//! jlrs from a `ccall`ed function. This will often require a `Target`, to create a target you
//! must create an instance of `CCall` first.
//!
//!
//! # Testing
//!
//! The restriction that Julia can be initialized once must be taken into account when running
//! tests that use jlrs. Because tests defined in a single crate are not guaranteed to be run
//! from the same thread you must guarantee that each crate has only one test that initializes
//! Julia. It's recommended you only use jlrs in integration tests because each top-level
//! integration test file is treated as a separate crate.
//!
//! ```
//! use jlrs::{prelude::*, runtime::handle::local_handle::LocalHandle};
//!
//! fn test_1(julia: &mut LocalHandle) {
//!     // use handle
//! }
//! fn test_2(julia: &mut LocalHandle) {
//!     // use handle
//! }
//!
//! #[test]
//! fn call_tests() {
//!     let mut julia = unsafe { Builder::new().start_local().unwrap() };
//!     test_1(&mut julia);
//!     test_2(&mut julia);
//! }
//! ```
//!
//!
//! # Custom types
//!
//! In order to map a struct in Rust to one in Julia you can derive several traits. You normally
//! shouldn't need to implement these structs or traits manually. The `reflect` function defined
//! in the `JlrsCore.Reflect` module can generate Rust structs whose layouts match their counterparts
//! in Julia and automatically derive the supported traits.
//!
//! The main restriction is that structs with atomic fields, and tuple or union fields with type
//! parameters are not supported. The reason for this restriction is that the layout of such
//! fields can be very different depending on the parameters in a way that can't be easily
//! represented in Rust.
//!
//! These custom types can also be used when you call Rust from Julia with `ccall`.
//!
//! [`LocalHandle`]: crate::runtime::handle::local_handle::LocalHandle
//! [`MtHandle`]: crate::runtime::handle::mt_handle::MtHandle
//! [`MtHandle::with`]: crate::runtime::handle::mt_handle::MtHandle::with
//! [`Builder::start_local`]: crate::runtime::builder::Builder::start_local
//! [`Julia`]: crate::runtime::sync_rt::Julia
//! [`Julia::scope`]: crate::runtime::sync_rt::Julia::scope
//! [`Julia::scope_with_capacity`]: crate::runtime::sync_rt::Julia::scope_with_capacity
//! [`Julia::init`]: crate::runtime::sync_rt::Julia::init
//! [`AsyncJulia::init`]: crate::multitask::runtime::AsyncJulia::init
//! [`AsyncJulia::init_async`]: crate::multitask::runtime::AsyncJulia::init_async
//! [`Julia::init_with_image`]: crate::runtime::sync_rt::Julia::init_with_image
//! [`CCall`]: crate::ccall::runtime::handle::CCall
//! [`Unrooted`]: crate::memory::target::unrooted::Unrooted
//! [`GcFrame`]: crate::memory::target::frame::GcFrame
//! [`Module`]: crate::data::managed::module::Module
//! [`Function`]: crate::data::managed::function::Function
//! [`Value`]: crate::data::managed::value::Value
//! [`Call`]: crate::call::Call
//! [`Value::eval_string`]: crate::data::managed::value::Value::eval_string
//! [`Value::new`]: crate::data::managed::value::Value::new
//! [`Array`]: crate::data::managed::array::Array
//! [`JuliaString`]: crate::data::managed::string::JuliaString
//! [`Module::main`]: crate::data::managed::module::Module::main
//! [`Module::base`]: crate::data::managed::module::Module::base
//! [`Module::core`]: crate::data::managed::module::Module::core
//! [`Module::function`]: crate::data::managed::module::Module::function
//! [`Module::global`]: crate::data::managed::module::Module::global
//! [`Module::submodule`]: crate::data::managed::module::Module::submodule
//! [`AsyncJulia::init_with_image`]: crate::multitask::runtime::AsyncJulia::init_with_image
//! [`AsyncJulia::init_with_image_async`]: crate::multitask::runtime::AsyncJulia::init_with_image_async
//! [`IntoJulia`]: crate::convert::into_julia::IntoJulia
//! [`Typecheck`]: crate::data::types::typecheck::Typecheck
//! [`ValidLayout`]: crate::data::layout::valid_layout::ValidLayout
//! [`ValidField`]: crate::data::layout::valid_layout::ValidField
//! [`Unbox`]: crate::convert::unbox::Unbox
//! [`CallAsync::call_async`]: crate::multitask::call_async::CallAsync
//! [`AsyncGcFrame`]: crate::memory::target::frame::AsyncGcFrame
//! [`Frame`]: crate::memory::frame::Frame
//! [`AsyncTask`]: crate::async_util::task::AsyncTask
//! [`PersistentTask`]: crate::async_util::task::PersistentTask
//! [`PersistentHandle`]: crate::runtime::async_rt::PersistentHandle
//! [`AsyncJulia`]: crate::runtime::async_rt::AsyncJulia
//! [`CallAsync`]: crate::call::CallAsync
//! [`DataType`]: crate::data::managed::datatype::DataType
//! [`TypedArray`]: crate::data::managed::array::TypedArray
//! [`Builder`]: crate::runtime::builder::Builder
//! [`Builder::spawn_mt`]: crate::runtime::builder::Builder::spawn_mt
//! [`jlrs::prelude`]: crate::prelude
//! [`julia_module`]: jlrs_macros::julia_module
//! [documentation]: jlrs_macros::julia_module
//! [rustfft_jl]: https://github.com/Taaitaaiger/rustfft-jl
//! [here]: https://github.com/JuliaPackaging/Yggdrasil/tree/master/R/rustfft

#![forbid(rustdoc::broken_intra_doc_links)]

use std::sync::atomic::{AtomicBool, Ordering};

use jl_sys::jlrs_init_missing_functions;
#[cfg(feature = "local-rt")]
use once_cell::sync::OnceCell;
use prelude::Managed;

use crate::{
    data::{
        managed::{
            module::{init_global_cache, JlrsCore},
            symbol::init_symbol_cache,
            value::Value,
        },
        types::{
            construct_type::init_constructed_type_cache, foreign_type::init_foreign_type_registry,
        },
    },
    memory::{
        context::{ledger::init_ledger, stack::Stack},
        target::unrooted::Unrooted,
    },
};

pub mod args;
#[cfg(feature = "async")]
pub mod async_util;
pub mod call;
pub mod catch;
pub mod convert;
pub mod data;
pub mod error;
pub mod gc_safe;
pub mod info;
pub mod memory;
pub mod prelude;
pub(crate) mod private;
pub mod runtime;
pub mod safety;
pub mod util;

/// The version of the jlrs API this version of jlrs is compatible with.
///
/// If this version number doesn't match `JLRS_API_VERSION` in JlrsCore.jl, initialization fails.
pub const JLRS_API_VERSION: isize = 3;

/// Installation method for the JlrsCore package. If JlrsCore is already installed the installed version
/// is used.
#[derive(Clone)]
pub enum InstallJlrsCore {
    /// Install the most recent version of JlrsCore
    Default,
    /// Don't install JlrsCore
    No,
    /// Install the given version
    Version {
        /// Major version
        major: usize,
        /// Minor version
        minor: usize,
        /// Patch version
        patch: usize,
    },
    /// Install a revision of some git repository
    Git {
        /// URL of the repository
        repo: String,
        /// Revision to be installed
        revision: String,
    },
}

impl InstallJlrsCore {
    #[cfg_attr(
        not(any(
            feature = "local-rt",
            feature = "async-rt",
            feature = "multi-rt",
            feature = "ccall"
        )),
        allow(unused)
    )]
    pub(crate) unsafe fn use_or_install(&self) {
        let unrooted = Unrooted::new();
        let res = match self {
            InstallJlrsCore::Default => {
                Value::eval_string(
                    unrooted,
                    "if !haskey(Base.loaded_modules, Base.PkgId(Base.UUID(\"29be08bc-e5fd-4da2-bbc1-72011c6ea2c9\"), \"JlrsCore\"))
                         try
                             using JlrsCore
                         catch e
                             import Pkg; Pkg.add(\"JlrsCore\")
                             using JlrsCore
                         end
                     end",
                )
            },
            InstallJlrsCore::Git { repo, revision } => {
                Value::eval_string(
                    unrooted,
                    format!(
                        "if !haskey(Base.loaded_modules, Base.PkgId(Base.UUID(\"29be08bc-e5fd-4da2-bbc1-72011c6ea2c9\"), \"JlrsCore\"))
                             try
                                 using JlrsCore
                             catch e
                                 import Pkg; Pkg.add(url=\"{repo}#{revision}\")
                                 using JlrsCore
                             end
                         end"
                    ),
                )
            },
            InstallJlrsCore::Version { major, minor, patch } => {
                Value::eval_string(
                    unrooted,
                    format!(
                        "if !haskey(Base.loaded_modules, Base.PkgId(Base.UUID(\"29be08bc-e5fd-4da2-bbc1-72011c6ea2c9\"), \"JlrsCore\"))
                             try
                                 using JlrsCore
                             catch e
                                 import Pkg; Pkg.add(name=\"JlrsCore\", version=\"{major}.{minor}.{patch}\")
                                 using JlrsCore
                             end
                         end"
                    ),
                )
            },
            InstallJlrsCore::No => {
                Value::eval_string(
                    unrooted,
                    "if !haskey(Base.loaded_modules, Base.PkgId(Base.UUID(\"29be08bc-e5fd-4da2-bbc1-72011c6ea2c9\"), \"JlrsCore\"))
                         using JlrsCore
                     end",
                )
            },
        };

        if let Err(err) = res {
            eprintln!("Failed to load or install JlrsCore package");
            // JlrsCore failed to load, print the error message to stderr without using
            // `Managed::error_string_or`.
            err.as_value().print_error();
            panic!();
        }
    }
}

// The chosen install method is stored in a OnceCell when the local runtime is used to
// avoid having to store it in `PendingJulia`.
#[cfg(feature = "local-rt")]
pub(crate) static INSTALL_METHOD: OnceCell<InstallJlrsCore> = OnceCell::new();

#[cfg_attr(
    not(any(
        feature = "local-rt",
        feature = "async-rt",
        feature = "multi-rt",
        feature = "ccall"
    )),
    allow(unused)
)]
pub(crate) unsafe fn init_jlrs(install_jlrs_core: &InstallJlrsCore) {
    static IS_INIT: AtomicBool = AtomicBool::new(false);

    if IS_INIT.swap(true, Ordering::Relaxed) {
        return;
    }

    jlrs_init_missing_functions();
    init_foreign_type_registry();
    init_constructed_type_cache();
    init_symbol_cache();
    init_global_cache();

    install_jlrs_core.use_or_install();
    let unrooted = Unrooted::new();
    let api_version = JlrsCore::api_version(&unrooted);
    if api_version != JLRS_API_VERSION {
        panic!("Incompatible version of JlrsCore detected. Expected API version{JLRS_API_VERSION}, found {api_version}");
    }

    init_ledger();
    Stack::init();
}
