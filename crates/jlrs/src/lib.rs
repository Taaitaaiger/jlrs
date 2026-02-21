//! jlrs is a crate that provides access to the Julia C API. It can be used to embed Julia in Rust
//! applications and to write interop libraries to Rust crates that can be used by Julia.
//!
//! Julia versions 1.10, 1.11 and 1.12 are currently supported. In general jlrs aims to support all
//! versions starting at the current LTS version, but only the LTS and stable versions are
//! actively tested. Using the current stable version of Julia is highly recommended. The minimum
//! supported Rust version is currently 1.85.
//!
//! A tutorial is available [here](https://taaitaaiger.github.io/jlrs-tutorial/).
//!
//! # Overview
//!
//! An incomplete list of features that are currently supported by jlrs:
//!
//!  - Access arbitrary Julia modules and their content.
//!  - Call Julia functions, including functions that take keyword arguments.
//!  - Handle exceptions or convert them to an error message.
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
//! To use jlrs, supported versions of Rust and Julia must have been installed. Currently, Julia
//! 1.10, 1.11 and 1.12 are supported, the minimum supported Rust version is 1.85. Some features may
//! require a more recent version of Rust. jlrs uses the JlrsCore package for Julia, if this
//! package has not been installed, the latest version will be installed automatically by default.
//!
//! ## With juliaup
//!
//! It is possible to use jlrs in combination with juliaup, but the default approach jlrs uses to
//! detect the installed version of Julia, its header files, and the libjulia itself will not
//! work. Instead, the jlrs-launcher application can be installed. This is an application that
//! uses the juliaup crate itself to determine this information and launches an application with
//! an updated environment.
//!
//! ## Without juliaup
//!
//! The recommended way to install Julia is to download the binaries from the official website,
//! which is distributed as an archive containing a directory called `julia-x.y.z`. This directory
//! contains several other directories, including a `bin` directory containing the `julia`
//! executable.
//!
//! ### Linux
//!
//! During compilation, the paths to the header and library are normally detected automatically by
//! executing the command `which julia`. The path to `julia.h` must be
//! `$(which julia)/../include/julia/julia.h` and the path to the library
//! `$(which julia)/../lib/libjulia.so`. If you want to override this default behavior or Julia
//! is not available on the path, the `JLRS_JULIA_DIR` environment variable must be set to it to
//! the appropriate `julia.x-y-z` directory, in this case `$JLRS_JULIA_DIR/include/julia/julia.h`
//! and`$JLRS_JULIA_DIR/lib/libjulia.so` are used instead.
//!
//! In order to be able to load `libjulia.so` this file must be on the library search path. If
//! this is not the case you must add `/path/to/julia-x.y.z/lib` to the `LD_LIBRARY_PATH`
//! environment variable.
//!
//! ### macOS
//!
//! Follow the instructions for Linux, but replace `LD_LIBRARY_PATH` with `DYLD_LIBRARY_PATH`.
//!
//! ### Windows
//!
//! Julia can be installed with the installer or portable installation downloaded from the
//! official website. In the first case, Julia has been likely installed in
//! `%USERPROFILE%\.julia\juliaup\julia-x.y.z+0~x64`, using the installer or extracting allows you
//! to pick the destination. After installation or extraction a folder called `Julia-x.y.z`
//! exists, which contains several folders including a `bin` folder containing `julia.exe`. The
//! path to the `bin` folder must be added to the `Path` environment variable.
//!
//! Julia is automatically detected by executing the command `where julia`. If this returns
//! multiple locations the first one is used. The default can be overridden by setting the
//! `JLRS_JULIA_DIR` environment variable.
//!
//!
//! # Features
//!
//! Most functionality of jlrs is only available if the proper features are enabled. These
//! features generally belong to one of two categories: runtimes and utilities.
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
//!    thread pools.
//!
//!
//! <div class="warning"><strong>WARNING</strong>: Runtime features must only be enabled by applications that embed Julia.
//! Libraries must never enable a runtime feature.</div>
//!
//! <div class="warning"><strong>WARNING</strong>: When building an application that embeds Julia, set
//! <code>RUSTFLAGS="-Clink-arg=-rdynamic"</code> if you want fast code.</div>
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
//!   This feature should be used in combination with the code generation provided by the
//!   `Reflect` module in the JlrsCore package. This module lets you generate Rust implementations
//!   of Julia structs, this generated code uses custom derive macros made available with this
//!   feature to enable the safe conversion of data from Julia to Rust, and from Rust to Julia in
//!   some cases.
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
//!   Julia's `ccall` interface can be used to call functions written in Rust from Julia. The
//!   `julia_module` macro is provided to easily export functions, types, and data in
//!   combination with the macros from the Wrap module in the JlrsCore package.
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
//!   `RUSTFLAGS="-Clinker-plugin-lto -Clinker=clang-XX -Clink-arg=-fuse-ld=lld -Clink-arg=-rdynamic"`.
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
//! You can enable all features except `debug`, `i686`, `windows`, and `lto` by enabling the
//! `full` feature. If you don't want to enable any runtimes either, you can use `full-no-rt`.
//!
//!
//! ## Environment variables
//!
//! It's possible to override certain defaults of jlrs and Julia by setting environment variables.
//! Many of the environment variables mentioned
//! [in the Julia documentation] should apply to applications that use jlrs as well, but this is
//! mostly untested.
//!
//! Several additional environment variables can be set which only affect applications that use
//! jlrs.
//!
//! - `JLRS_CORE_VERSION=major.minor.patch`
//! Installs the set version of JlrsCore before loading it.
//!
//! - `JLRS_CORE_REVISION=rev`
//! Installs the set revision of JlrsCore before loading it.
//!
//! - `JLRS_CORE_REPO=repo-url`
//! Can be used with `JLRS_CORE_REVISION` to set the repository JlrsCore will be downloaded from.
//!
//! - `JLRS_CORE_NO_INSTALL=...`
//! Don't install JlrsCore, its value is ignored.
//!
//! `JLRS_CORE_NO_INSTALL` takes priority over `JLRS_CORE_REVISION`, which takes priority over
//! `JLRS_CORE_VERSION`.
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
//! `jlrs = {version = "0.22", features = ["local-rt"]}`
//!
//! `jlrs = {version = "0.22", features = ["tokio-rt"]}`
//!
//! `jlrs = {version = "0.22", features = ["multi-rt"]}`
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
//!     // panic unless we changed `local_scope::<_, 1>` to `local_scope::<_, 2>`.
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
//!             func.call(&mut frame, [v1, v2])? // 4
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
//!                 .call(&mut frame, [v1, v2, v3])? // 5
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
//! Julia from arbitrary threads.
//!
//! To start this runtime you need to create a `Builder` and call [`Builder::start_mt`]. It has
//! its own handle type, [`MtHandle`], which can be used to spawn new threads that can call into
//! Julia. Unlike the local runtime's `LocalHandle`, it can't be used directly, you must call
//! [`MtHandle::with`] first to ensure the thread is in a state where it can call into Julia.
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
//! # {
//! // When the multithreaded runtime is started the current thread initializes Julia.
//! Builder::new().start_mt(|mt_handle| {
//!     let t1 = mt_handle.spawn(move |mut mt_handle| {
//!         // By calling `MtHandle::with` we enable the thread to call into Julia. The handle you can
//!         // use in that closure provides the same functionality as the local runtime's
//!         // `LocalHandle`.
//!         mt_handle.with(|handle| {
//!             handle.local_scope::<_, 1>(|mut frame| unsafe {
//!                 let _v = Value::new(&mut frame, 1);
//!             })
//!         })
//!     });
//!
//!     let t2 = mt_handle.spawn(move |mut mt_handle| {
//!         mt_handle.with(|handle| {
//!             handle.local_scope::<_, 1>(|mut frame| unsafe {
//!                 let _v = Value::new(&mut frame, 2);
//!             })
//!         })
//!     });
//!
//!     t1.join().expect("thread 1 panicked");
//!     t2.join().expect("thread 2 panicked");
//! }).unwrap();
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
//! impl AsyncTask for AdditionTask {
//!     // The type of the result of this task.
//!     type Output = JlrsResult<u64>;
//!
//!     // This async method replaces the closure from the previous examples,
//!     // an `AsyncGcFrame` can be used the same way as other frame types.
//!     async fn run<'frame>(self, mut frame: AsyncGcFrame<'frame>) -> Self::Output {
//!         let a = Value::new(&mut frame, self.a);
//!         let b = Value::new(&mut frame, self.b);
//!
//!         let func = Module::base(&frame).global(&mut frame, "+")?;
//!
//!         // CallAsync::call_async schedules the function call on another thread.
//!         // The runtime can switch to other tasks while awaiting the result.
//!         // Safety: adding two numbers is safe.
//!         unsafe { func.call_async(&mut frame, [a, b]) }
//!             .await?
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
//! Async closures implement `AsyncTask`:
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
//! let a = 1u64;
//! let b = 2u64;
//!
//! // It's necessary to provide frame's type
//! let async_task = julia
//!     .task(async move |mut frame: AsyncGcFrame| -> JlrsResult<u64> {
//!         let a = Value::new(&mut frame, a);
//!         let b = Value::new(&mut frame, b);
//!
//!         let func: Value = Module::base(&frame).global(&mut frame, "+")?;
//!         unsafe { func.call_async(&mut frame, [a, b]) }
//!             .await?
//!             .unbox::<u64>()
//!     })
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
//!         let array = TypedArray::from_vec(&mut frame, data, self.n_values)??;
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
//!                 .global(&mut frame, "sum")?
//!                 .call(&mut frame, [state.array.as_value()])?
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
//! # {
//! Builder::new()
//!     .async_runtime(Tokio::<3>::new(false))
//!     .start_mt(|mt_handle, _async_handle| {
//!         let pool_handle = mt_handle
//!             .pool_builder(Tokio::<1>::new(false))
//!             .n_workers(2.try_into().unwrap())
//!             .spawn();
//!     })
//!     .unwrap();
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
//!     if arg.as_bool() { 1 } else { -1 }
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
//!             )?;
//!
//!             // Call the function and unbox the result.
//!             let result = func.call(&mut frame, [call_me_val])?.unbox::<isize>()?;
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
//! [`Unrooted`]: crate::memory::target::unrooted::Unrooted
//! [`GcFrame`]: crate::memory::target::frame::GcFrame
//! [`Module`]: crate::data::managed::module::Module
//! [`Value`]: crate::data::managed::value::Value
//! [`Call`]: crate::call::Call
//! [`Value::eval_string`]: crate::data::managed::value::Value::eval_string
//! [`Value::new`]: crate::data::managed::value::Value::new
//! [`Array`]: crate::data::managed::array::Array
//! [`JuliaString`]: crate::data::managed::string::JuliaString
//! [`Module::main`]: crate::data::managed::module::Module::main
//! [`Module::base`]: crate::data::managed::module::Module::base
//! [`Module::core`]: crate::data::managed::module::Module::core
//! [`Module::global`]: crate::data::managed::module::Module::global
//! [`Module::submodule`]: crate::data::managed::module::Module::submodule
//! [`IntoJulia`]: crate::convert::into_julia::IntoJulia
//! [`Typecheck`]: crate::data::types::typecheck::Typecheck
//! [`ValidLayout`]: crate::data::layout::valid_layout::ValidLayout
//! [`ValidField`]: crate::data::layout::valid_layout::ValidField
//! [`Unbox`]: crate::convert::unbox::Unbox
//! [`AsyncGcFrame`]: crate::memory::target::frame::AsyncGcFrame
//! [`AsyncTask`]: crate::async_util::task::AsyncTask
//! [`PersistentTask`]: crate::async_util::task::PersistentTask
//! [`CallAsync`]: crate::call::CallAsync
//! [`DataType`]: crate::data::managed::datatype::DataType
//! [`TypedArray`]: crate::data::managed::array::TypedArray
//! [`Builder`]: crate::runtime::builder::Builder
//! [`Builder::start_mt`]: crate::runtime::builder::Builder::start_mt
//! [`jlrs::prelude`]: crate::prelude
//! [`julia_module`]: jlrs_macros::julia_module
//! [documentation]: jlrs_macros::julia_module
//! [rustfft_jl]: https://github.com/Taaitaaiger/rustfft-jl
//! [here]: https://github.com/JuliaPackaging/Yggdrasil/tree/master/R/rustfft
//! [in the Julia documentation]: https://docs.julialang.org/en/v1/manual/environment-variables/

#![forbid(rustdoc::broken_intra_doc_links)]

use std::{
    env,
    ffi::c_int,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use data::{
    managed::module::mark_global_cache, static_data::mark_static_data_cache,
    types::construct_type::mark_constructed_type_cache,
};
use jl_sys::jl_gc_set_cb_root_scanner;
use jlrs_sys::jlrs_init_missing_functions;
use memory::get_tls;
use prelude::Managed;
use semver::Version;

use crate::{
    data::managed::{module::JlrsCore, value::Value},
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
pub const JLRS_API_VERSION: isize = 4;

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
    unsafe fn use_or_install(&self, environment: Option<&Path>) {
        let activate_env_if_any = match environment {
            None => "".to_string(),
            Some(path) => {
                let str = path.to_str().expect("Environment path is UTF-8.");
                format!("Pkg.activate({});", str)
            }
        };

        let import_pkg_and_activate_env = format!("import Pkg; {}", activate_env_if_any);

        unsafe {
            let unrooted = Unrooted::new();
            let cmd = match self {
                InstallJlrsCore::Default => {
                    format!(
                        "{}; try; using JlrsCore; catch; Pkg.add(\"JlrsCore\"); using JlrsCore; end",
                        import_pkg_and_activate_env
                    )
                }
                InstallJlrsCore::Git { repo, revision } => {
                    format!(
                        "{}; Pkg.add(url=\"{repo}\", rev=\"{revision}\"); using JlrsCore",
                        import_pkg_and_activate_env
                    )
                }
                InstallJlrsCore::Version {
                    major,
                    minor,
                    patch,
                } => {
                    format!(
                        "{}; Pkg.add(name=\"JlrsCore\", version=\"{major}.{minor}.{patch}\"); using JlrsCore",
                        import_pkg_and_activate_env
                    )
                }
                InstallJlrsCore::No => "using JlrsCore".to_string(),
            };

            let cmd = format!(
                "if !haskey(Base.loaded_modules, Base.PkgId(Base.UUID(\"29be08bc-e5fd-4da2-bbc1-72011c6ea2c9\"), \"JlrsCore\")); {cmd}; end"
            );

            if let Err(err) = Value::eval_string(unrooted, cmd) {
                eprintln!("Failed to load or install JlrsCore package");
                // JlrsCore failed to load, print the error message to stderr without using
                // `Managed::error_string_or`.
                err.as_value().print_error();
                panic!();
            }
        }
    }
}

fn preferred_jlrs_core_version() -> Option<InstallJlrsCore> {
    if let Some(_) = env::var("JLRS_CORE_NO_INSTALL").ok() {
        return Some(InstallJlrsCore::No);
    }

    if let Some(version) = env::var("JLRS_CORE_VERSION").ok() {
        if let Ok(version) = Version::parse(version.as_str()) {
            return Some(InstallJlrsCore::Version {
                major: version.major as _,
                minor: version.minor as _,
                patch: version.patch as _,
            });
        }
    }

    if let Some(revision) = env::var("JLRS_CORE_REVISION").ok() {
        let repo = env::var("JLRS_CORE_REPO")
            .unwrap_or("https://github.com/Taaitaaiger/JlrsCore.jl".into());
        return Some(InstallJlrsCore::Git { repo, revision });
    }

    None
}

#[cfg_attr(
    not(any(
        feature = "local-rt",
        feature = "async-rt",
        feature = "multi-rt",
        feature = "ccall"
    )),
    allow(unused)
)]
pub(crate) unsafe fn init_jlrs(
    install_jlrs_core: &InstallJlrsCore,
    environment: Option<&Path>,
    allow_override: bool,
) {
    unsafe {
        static IS_INIT: AtomicBool = AtomicBool::new(false);

        if IS_INIT.swap(true, Ordering::Relaxed) {
            return;
        }

        jlrs_init_missing_functions();

        jl_gc_set_cb_root_scanner(root_scanner, 1);

        if let Some(preferred_version) = preferred_jlrs_core_version() {
            if allow_override {
                preferred_version.use_or_install(environment);
            } else {
                install_jlrs_core.use_or_install(environment);
            }
        } else {
            install_jlrs_core.use_or_install(environment);
        }

        let unrooted = Unrooted::new();
        let api_version = JlrsCore::api_version(&unrooted);
        if api_version != JLRS_API_VERSION {
            panic!(
                "Incompatible version of JlrsCore detected. Expected API version {JLRS_API_VERSION}, found {api_version}"
            );
        }

        init_ledger();
        Stack::init(&unrooted);
    }
}

#[cfg_attr(
    not(any(
        feature = "local-rt",
        feature = "async-rt",
        feature = "multi-rt",
        feature = "ccall"
    )),
    allow(unused)
)]
unsafe extern "C" fn root_scanner(full: c_int) {
    unsafe {
        let ptls = get_tls();
        debug_assert!(!ptls.is_null());

        let full = full != 0;
        mark_constructed_type_cache(ptls, full);
        mark_global_cache(ptls, full);
        mark_static_data_cache(ptls, full);
    }
}
