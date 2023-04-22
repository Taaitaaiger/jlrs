//! jlrs is a crate that provides access to most of the Julia C API, it can be used to embed Julia
//! in Rust applications and to use functionality it provides when writing `ccall`able
//! functions in Rust. Currently this crate is only tested in combination with Julia 1.6 and 1.8,
//! but also supports Julia 1.7 and 1.9. Using the current stable version is highly recommended.
//! The minimum supported Rust version is currently 1.65.
//!
//! The documentation assumes you're already familiar with the Julia and Rust programming
//! languages.
//!
//! An incomplete list of features that are currently supported by jlrs:
//!
//!  - Access arbitrary Julia modules and their content.
//!  - Call Julia functions, including functions that take keyword arguments.
//!  - Handle exceptions or convert them to an error message, optionally with color.
//!  - Include and call your own Julia code.
//!  - Use a custom system image.
//!  - Create values that Julia can use, and convert them back to Rust, from Rust.
//!  - Access the type information and fields of values. The contents of inline and bits-union
//!    fields can be accessed directly.
//!  - Create and use n-dimensional arrays. The `jlrs-ndarray` feature can be enabled for
//!    integration with ndarray.
//!  - Map Julia structs to Rust structs, the Rust implementation can be generated with the
//!    JlrsCore package.
//!  - Structs that can be mapped to Rust include those with type parameters and bits unions.
//!  - Use Julia from multiple threads with an async runtime, these runtimes support scheduling
//!    Julia `Task`s and `await`ing them without blocking the runtime thread.
//!  - Export Rust types, methods and functions to Julia with the `julia_module` macro; libraries
//!    that use this macro can be compiled with BinaryBuilder and distributed as JLLs.
//!
//!
//! NB: Active development happens on the `dev` branch, the `master` branch points to the most
//! recently released version.
//!
//!
//! # Prerequisites
//!
//! Julia must be installed before jlrs can be used, jlrs is compatible with Julia 1.6 up to and
//! including Julia 1.9. The JlrsCore package must also have been installed, if this is not the
//! case it will automatically be added when jlrs is initialized by default. jlrs has not been
//! tested with juliaup yet on Linux and macOS.
//!
//! ## Linux
//!
//! The recommended way to install Julia is to download the binaries from the official website,
//! which is distributed in an archive containing a directory called `julia-x.y.z`. This directory
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
//! environment variable. When the `uv` feature is enabled, `/path/to/julia-x.y.z/lib/julia` must
//! also be added to `LD_LIBRARY_PATH`. The latter path should not be added to the default path
//! because this can break tools installed on your system.
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
//! ## macOS
//!
//! Follow the instructions for Linux, but replace `LD_LIBARY_PATH` with `DYLD_LIBARY_PATH`.
//!
//!
//! # Features
//!
//! Most functionality of jlrs is only available if the proper features are enabled. These
//! features generally belong to one of three categories: versions, runtimes and utilities.
//!
//! ## Versions
//!
//! There are minor incompatibilities between different versions of Julia, to ensure the correct
//! bindings are used for a particular version of Julia you must enable a version features to use
//! jlrs. The following version features currently exist:
//!
//!  - `julia-1-6`
//!  - `julia-1-7`
//!  - `julia-1-8`
//!  - `julia-1-9`
//!
//! Exactly one version feature must be enabled. If no version is enabled, or multiple are, jl-sys
//! will fail to compile.
//!
//! If you want your crate to be compatible with multiple versions of Julia, you should reexport
//! these version features:
//!
//! ```toml
//! [features]
//! julia-1-6 = ["jlrs/julia-1-6"]
//! julia-1-7 = ["jlrs/julia-1-7"]
//! julia-1-8 = ["jlrs/julia-1-8"]
//! julia-1-9 = ["jlrs/julia-1-9"]
//! ```
//!
//! In this case you must provide this feature when you build or run your crate:
//! `cargo (build,run) --feature julia-1-8`.
//!
//! ## Runtimes
//!
//! A runtime lets you embed Julia in a Rust application, the following features enable a runtime:
//!
//! - `sync-rt`
//!
//!   Enables the sync runtime, [`Julia`]. The sync runtime provides single-threaded, blocking
//!   access to the Julia C API.
//!
//! - `async-rt`
//!
//!   Enables the async runtime, [`AsyncJulia`]. The async runtime runs on a separate thread and
//!   can be used from multiple threads. Since Julia 1.9 it's possible to start the async runtime
//!   with multiple worker threads.
//!
//! - `tokio-rt` and `async-std-rt`
//!
//!   These features provide a backing runtime for the async runtime. The first uses tokio, the
//!   second async-std. The `async-rt` feature is automatically enabled when one of these features
//!   is enabled.
//!
//! If you're writing a library, either one that will be called from Julia or one that will be
//! used by a Rust application that embeds Julia, no runtime is required.
//!
//! ## Utilities
//!
//! In addition to these runtimes, the following utility features are available:
//!
//! - `prelude`
//!
//!   Provides a prelude module, [`jlrs::prelude`]. This feature is enabled by default.
//!
//! - `async`
//!
//!   Enable the features of the async runtime which don't depend on the backing runtime. This
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
//! - `ccall`
//!
//!   Julia's `ccall` interface can be used to call functions written in Rust from Julia. No
//!   runtime can be used in this case because Julia has already been initialized, when this
//!   feature is enabled the `CCall` struct is available which offers the same functionality as
//!   the sync runtime without initializing Julia. The [`julia_module`] macro is provided to
//!   easily export functions, types, and data in combination with the macros from the Wrap
//!   module in the JlrsCore package.
//!
//! - `uv`
//!
//!   This feature enables the method `CCall::uv_async_send`, which can be used to wake a Julia
//!   `AsyncCondition` from Rust. The `ccall` feature is automically enabled when this feature
//!   is used.
//!
//! - `pyplot`
//!
//!   This feature lets you plot data using the Pyplot package and Gtk 3 from Rust.
//!
//! - `internal-types`
//!
//!   Provide extra managed types for types that are mostly used internally by Julia.
//!
//! - `extra-fields`
//!
//!   Provide extra field accessor methods for managed types.
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
//!   Don't link Julia, linking can be skipped when writing libraries that will be loaded by
//!   Julia.
//!
//! - `yggdrasil`
//!
//!   Flag that must be enabled when compiling with BinaryBuilder.
//!
//! You can enable all features except `debug`, `i686`, `windows`, `no-link` and `yggdrasil` by
//! enabling the `full` feature.
//!
//!
//! # Using this crate
//!
//! If you want to embed Julia in a Rust application, you must enable a runtime and a version
//! feature:
//!
//! `jlrs = {version = "0.18.0-beta.2", features = ["sync-rt", "julia-1-8"]}`
//!
//! `jlrs = {version = "0.18.0-beta.2", features = ["tokio-rt", "julia-1-8"]}`
//!
//! `jlrs = {version = "0.18.0-beta.2", features = ["async-std-rt", "julia-1-8"]}`
//!
//! When Julia is embedded in an application, it must be initialized before it can be used. The
//! following snippet initializes the sync runtime:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! // Initializing Julia is unsafe because this can load arbitrary
//! // Julia code, and because it can race with other crates unrelated
//! // to jlrs. It returns an error if Julia has already been
//! // initialized.
//! let mut julia = unsafe { RuntimeBuilder::new().start().unwrap() };
//!
//! // A StackFrame must be provided to ensure Julia's GC can be made aware
//! // of references to Julia data that exist in Rust code.
//! let mut frame = StackFrame::new();
//! let _instance = julia.instance(&mut frame);
//! # }
//! ```
//!
//! To use the async runtime you must upgrade the [`RuntimeBuilder`] to an
//! [`AsyncRuntimeBuilder`] by providing a backing runtime. Implementations for tokio
//! and async-std are available if these features have been enabled. When starting the async
//! runtime, you must declare the maximum number of concurrent tasks as a const generic.
//!
//! For example, an async runtime backed by tokio and an unbounded channel, that supports 3
//! concurrent task can be initialized as follows if the `tokio-rt` feature is enabled:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! // Initializing Julia is unsafe for the same reasons as the sync runtime.
//! let (_julia, _task_handle) = unsafe {
//!     RuntimeBuilder::new()
//!         .async_runtime::<Tokio>()
//!         .start::<3>()
//!         .unwrap()
//! };
//! # }
//! ```
//!
//! The async runtime can also be spawned as a blocking task on an existing executor:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initializing Julia is unsafe for the same reasons as the sync runtime.
//!     let (_julia, _task_handle) = unsafe {
//!         RuntimeBuilder::new()
//!             .async_runtime::<Tokio>()
//!             .start_async::<3>()
//!             .unwrap()
//!     };
//! }
//! ```
//!
//! If you're calling Rust from Julia everything has already been initialized. If the `ccall`
//! feature is enabled [`CCall`] is available which provides the same functionality as the sync
//! runtime.
//!
//! ## Calling Julia from Rust
//!
//! This section will focus on some topics that are common between the sync and async runtimes.
//!
//! After initialization you have an instance of [`Julia`] or [`AsyncJulia`], both provide a
//! method called `include` that lets you include files with custom Julia code. In order to
//! create Julia data and call Julia functions, a scope must be created first.
//!
//! When the sync runtime is used this can be done by calling the method [`Julia::scope`]. This
//! method takes a closure with a single argument, a [`GcFrame`] (frame). This frame can be used
//! to access Julia data, and ensure it's not freed by the GC while it's accessible from Rust.
//!
//! The async runtime can't create a new scope directly, `AsyncJulia` is a handle to the async
//! runtime which runs on another thread. Instead, the async runtime deals with tasks, each task
//! runs in its own scope. The simplest kind of task is a blocking task, which can be executed by
//! calling `AsyncJulia::blocking_task`. This method accepts any closure `Julia::scope` can
//! handle with the additional requirement that it must be `Send` and `Sync`. It's called a
//! blocking task because the thread that executes this task is blocked while executing it. The
//! other kinds of tasks that the async runtime can handle will be introduced later.
//!
//! Inside the closure provided to `Julia::scope` or `AsyncJulia::blocking_task` it's possible to
//! interact with Julia. Global Julia data can be accessed through its module system, the methods
//! [`Module::main`], [`Module::base`], and [`Module::core`] can be used to access the `Main`,
//! `Base`, and `Core` modules respectively. The contents of these modules can then be accessed by
//! calling [`Module::function`] which returns a [`Function`], [`Module::global`] which returns a
//! [`Value`], and [`Module::submodule`] which returns another [`Module`]. These types are
//! examples of managed types, handles to data owned by Julia's GC. Most functionality in jlrs
//! is provided through methods implemented by managed types.
//!
//! The most generic managed type is `Value`, all other managed types can always be converted to
//! a `Value`. It provides several methods to allocate new Julia data. The simplest one is
//! [`Value::eval_string`], which evaluates the contents of the string passed to it and returns
//! the result as a `Value`. For example, you can evaluate `2` to convert it to  `Value`. In
//! practice, this method should rarely be used. It can be used to evaluate simple function calls
//! like `sqrt(2)`, but it must be parsed, compiled, and can't take any non-literal arguments. Its
//! most important use-case is importing installed and standard library packages by evaluating an
//! `import` or `using` statement.
//!
//! A more interesting method, [`Value::new`], can be used with data of any type that implements
//! [`IntoJulia`]. This trait is implemented by primitive types like `i8` and `char`. Any type
//! that implements [`IntoJulia`] also implements [`Unbox`] which is used to extract the contents
//! of a `Value`. Managed types like [`Array`] don't implement [`IntoJulia`] or [`Unbox`], if they
//! can be created from Rust they provide methods to do so.
//!
//! As a simple example, let's convert two numbers to Julia values and add them:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! // Initializing Julia is unsafe because it can race with another crate that does
//! // the same.
//! let mut julia = unsafe { RuntimeBuilder::new().start().unwrap() };
//! let mut frame = StackFrame::new();
//! let mut julia = julia.instance(&mut frame);
//!
//! let res = julia.scope(|mut frame| {
//!     // Create the two arguments.
//!     let i = Value::new(&mut frame, 2u64);
//!     let j = Value::new(&mut frame, 1u32);
//!
//!     // The `+` function can be found in the base module.
//!     let func = Module::base(&frame).function(&mut frame, "+")?;
//!
//!     // Call the function and unbox the result as a `u64`. The result of the function
//!     // call is a nested `Result`; the outer error doesn't contain to any Julia
//!     // data, while the inner error contains the exception if one is thrown. Here the
//!     // exception is converted to the outer error type by calling `into_jlrs_result`, this new
//!     // error contains the error message Julia would have shown.
//!     unsafe { func.call2(&mut frame, i, j) }
//!         .into_jlrs_result()?
//!         .unbox::<u64>()
//! }).unwrap();
//!
//! assert_eq!(res, 3);
//! # }
//! ```
//!
//! Evaluating raw code and calling Julia functions is always unsafe. Nothing prevents you from
//! calling a function like `nasaldemons() = unsafe_load(Ptr{Float64}(0x05391A445))`. Similarly,
//! mutating Julia data is unsafe because nothing prevents you from mutating data that shouldn't
//! be mutated, e.g. a `DataType`. A full overview of the rules that you should keep in mind can
//! be found in the [`safety`] module.
//!
//! ### Async and persistent tasks
//!
//! In addition to blocking tasks, the async runtime lets you execute async tasks which implement
//! the [`AsyncTask`] trait, and persistent tasks which implement [`PersistentTask`]. Both of
//! these traits are async traits.
//!
//! An async task is similar to a blocking task, except that you must implement the async `run`
//! method instead of providing a closure. This method takes an [`AsyncGcFrame`]. This new frame
//! type not only provides access to the same features as [`GcFrame`], it can also be used to call
//! async methods provided by the [`CallAsync`] trait. These methods schedule a function call as a
//! new Julia `Task` and can be `await`ed until this task has completed. The async runtime can
//! switch to another task while the result is pending, allowing multiple tasks to run
//! concurrently on a single thread.
//!
//! The previous example can be rewritten as an async task:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! struct AdditionTask {
//!     a: u64,
//!     b: u32,
//! }
//!
//! // Only the runtime thread can call the Julia C API, so the async
//! // trait methods of `AsyncTask` must not return a future that
//! // implements `Send` or `Sync`.
//! #[async_trait(?Send)]
//! impl AsyncTask for AdditionTask {
//!     // The type of the result of this task if it succeeds.
//!     type Output = u64;
//!
//!     // The affinity of the task. Setting it to `DispatchAny` allows the
//!     // task to be dispatched to both the main thread and worker threads
//!     // if they are available.
//!     type Affinity = DispatchAny;
//!
//!     // This async method replaces the closure from the previous examples,
//!     // an `AsyncGcFrame` can be used the same way as a `GcFrame` but also
//!     // can be used in combination with methods from the `CallAsync` trait.
//!     async fn run<'frame>(
//!         &mut self,
//!         mut frame: AsyncGcFrame<'frame>,
//!     ) -> JlrsResult<Self::Output> {
//!         let a = Value::new(&mut frame, self.a);
//!         let b = Value::new(&mut frame, self.b);
//!
//!         let func = Module::base(&frame).function(&mut frame, "+")?;
//!
//!         // CallAsync::call_async schedules the function call on another
//!         // thread and returns a Future that resolves when the scheduled
//!         // function has returned or thrown an error.
//!         unsafe { func.call_async(&mut frame, &mut [a, b]) }
//!             .await
//!             .into_jlrs_result()?
//!             .unbox::<u64>()
//!     }
//! }
//! ```
//!
//! While blocking and async tasks run once and return their result, a persistent task returns a
//! handle to the task. This handle can be shared across threads and used to call its `run`
//! method. In addition to a global and an async frame, this method can use the state and input
//! data provided by the caller.
//!
//! As an example, let's accumulate some number of values in a Julia array and return the sum of
//! its contents:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! struct AccumulatorTask {
//!     n_values: usize,
//! }
//!
//! struct AccumulatorTaskState<'state> {
//!     array: TypedArray<'state, 'static, usize>,
//!     offset: usize,
//! }
//!
//! // Only the runtime thread can call the Julia C API, so the async trait
//! // methods of `PersistentTask` must not return a future that implements
//! // `Send` or `Sync`.
//! #[async_trait(?Send)]
//! impl PersistentTask for AccumulatorTask {
//!     // The type of the result of the task if it succeeds.
//!     type Output = usize;
//!
//!     // The type of the task's internal state.
//!     type State<'state> = AccumulatorTaskState<'state>;
//!
//!     // The type of the additional data that the task must be called with.
//!     type Input = usize;
//!
//!     // The affinity of the task. Setting it to `DispatchAny` allows the
//!     // task to be dispatched to both the main thread and worker threads
//!     // if they are available.
//!     type Affinity = DispatchAny;
//!
//!     // This method is called before the task can be called. Note that the
//!     // frame is not dropped until the task has completed, so the task's
//!     // internal state can contain Julia data rooted in this frame.
//!     async fn init<'frame>(
//!         &mut self,
//!         mut frame: AsyncGcFrame<'frame>,
//!     ) -> JlrsResult<Self::State<'frame>> {
//!         // A `Vec` can be moved from Rust to Julia if the element type
//!         // implements `IntoJulia`.
//!         let data = vec![0usize; self.n_values];
//!         let array = TypedArray::from_vec(frame.as_extended_target(), data, self.n_values)?
//!             .into_jlrs_result()?;
//!
//!         Ok(AccumulatorTaskState { array, offset: 0 })
//!     }
//!
//!     // Whenever the task is called through its handle this method
//!     // is called. Unlike `init`, the frame that this method can use
//!     // is dropped after `run` returns.
//!     async fn run<'frame, 'state: 'frame>(
//!         &mut self,
//!         mut frame: AsyncGcFrame<'frame>,
//!         state: &mut Self::State<'state>,
//!         input: Self::Input,
//!     ) -> JlrsResult<Self::Output> {
//!         {
//!             // Array data can be directly accessed from Rust.
//!             // The data is tracked first to ensure it's not
//!             // already borrowed from Rust.
//!             unsafe {
//!                 let mut tracked = state.array.track_exclusive()?;
//!                 let mut data = tracked.bits_data_mut()?;
//!                 data[state.offset] = input;
//!             };
//!
//!             state.offset += 1;
//!             if (state.offset == self.n_values) {
//!                 state.offset = 0;
//!             }
//!         }
//!
//!         // Return the sum of the contents of `state.array`.
//!         unsafe {
//!             Module::base(&frame)
//!                 .function(&mut frame, "sum")?
//!                 .call1(&mut frame, state.array.as_value())
//!                 .into_jlrs_result()?
//!                 .unbox::<usize>()
//!         }
//!     }
//! }
//! ```
//!
//! ## Calling Rust from Julia
//!
//! Julia's `ccall` interface can be used to call `extern "C"` functions defined in Rust.
//! A function pointer can be cast to a void pointer and converted to a [`Value`]:
//!
//! ```
//! use jlrs::prelude::*;
//!
//! // This function will be provided to Julia as a pointer, so its name can be mangled.
//! unsafe extern "C" fn call_me(arg: bool) -> isize {
//!     if arg {
//!         1
//!     } else {
//!         -1
//!     }
//! }
//!
//! # fn main() {
//! let mut frame = StackFrame::new();
//! let mut julia = unsafe { RuntimeBuilder::new().start().unwrap() };
//! let mut julia = julia.instance(&mut frame);
//!
//! julia
//!     .scope(|mut frame| unsafe {
//!         // Cast the function to a void pointer
//!         let call_me_val = Value::new(&mut frame, call_me as *mut std::ffi::c_void);
//!
//!         // Value::eval_string can be used to create new functions.
//!         let func = Value::eval_string(
//!             &mut frame,
//!             "myfunc(callme::Ptr{Cvoid})::Int = ccall(callme, Int, (Bool,), true)",
//!         )
//!         .into_jlrs_result()?;
//!
//!         // Call the function and unbox the result.
//!         let result = func
//!             .call1(&mut frame, call_me_val)
//!             .into_jlrs_result()?
//!             .unbox::<isize>()?;
//!
//!         assert_eq!(result, 1);
//!
//!         Ok(())
//!     })
//!     .unwrap();
//! # }
//! ```
//!
//! You can also use functions defined in `cdylib` libraries. In order to create such
//! a library you need to add
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! to your crate's `Cargo.toml`. It's also recommended to abort on panic:
//!
//! ```toml
//! [profile.release]
//! panic = "abort"
//! ```
//!
//! The easiest way to export Rust functions like `call_me` from the previous example is by
//! using the [`julia_module`] macro. The content of the macro is converted to an initialization
//! function that can be called from Julia to generate the module.
//!
//! In Rust, the macro can be used like this:
//!
//! ```ignore
//! julia_module! {
//!     become callme_init_fn;
//!     fn call_me(arg: bool) -> isize;
//! }
//! ```
//!
//! while on the Julia side things look like this:
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
//! This macro has many more capabilities than just exporting extern "C" functions, for more
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
//! tests that use `jlrs`. Because tests defined in a single crate are not guaranteed to be run
//! from the same thread you must guarantee that each crate has only one test that initializes
//! Julia. It's recommended you only use jlrs in integration tests because each top-level
//! integration test file is treated as a separate crate.
//!
//! ```
//! use jlrs::prelude::*;
//!
//! fn test_1(julia: &mut Julia) {
//!     // use instance
//! }
//! fn test_2(julia: &mut Julia) {
//!     // use instance
//! }
//!
//! #[test]
//! fn call_tests() {
//!     let mut pending = unsafe { RuntimeBuilder::new().start().unwrap() };
//!     let mut frame = StackFrame::new();
//!     let mut julia = pending.instance(&mut frame);
//!
//!     test_1(&mut julia);
//!     test_2(&mut julia);
//! }
//! ```
//!
//! Because `AsyncJulia` is thread-safe, it is possible to have multiple tests in a single crate
//! when the async runtime is used:
//!
//! ```
//! use std::{num::NonZeroUsize, sync::Arc};
//!
//! use jlrs::prelude::*;
//! use once_cell::sync::OnceCell;
//!
//! fn init() -> Arc<AsyncJulia<Tokio>> {
//!     unsafe {
//!         Arc::new(
//!             RuntimeBuilder::new()
//!                 .async_runtime::<Tokio>()
//!                 .n_threads(4)
//!                 .channel_capacity(NonZeroUsize::new_unchecked(32))
//!                 .start::<4>()
//!                 .expect("Could not init Julia")
//!                 .0,
//!         )
//!     }
//! }
//!
//! pub static JULIA: OnceCell<Arc<AsyncJulia<Tokio>>> = OnceCell::new();
//!
//! #[test]
//! fn test_1() {
//!     let julia = JULIA.get_or_init(init);
//!
//!     // use instance
//! }
//!
//! #[test]
//! fn test_2() {
//!     let julia = JULIA.get_or_init(init);
//!
//!     // use instance
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
//! [`Julia`]: crate::runtime::sync_rt::Julia
//! [`Julia::scope`]: crate::runtime::sync_rt::Julia::scope
//! [`Julia::scope_with_capacity`]: crate::runtime::sync_rt::Julia::scope_with_capacity
//! [`Julia::init`]: crate::runtime::sync_rt::Julia::init
//! [`AsyncJulia::init`]: crate::multitask::runtime::AsyncJulia::init
//! [`AsyncJulia::init_async`]: crate::multitask::runtime::AsyncJulia::init_async
//! [`Julia::init_with_image`]: crate::runtime::sync_rt::Julia::init_with_image
//! [`CCall`]: crate::ccall::CCall
//! [`CCall::uv_async_send`]: crate::ccall::CCall::uv_async_send
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
//! [`RuntimeBuilder`]: crate::runtime::builder::RuntimeBuilder
//! [`AsyncRuntimeBuilder`]: crate::runtime::builder::AsyncRuntimeBuilder
//! [`jlrs::prelude`]: crate::prelude
//! [`julia_module`]: jlrs_macros::julia_module
//! [documentation]: jlrs_macros::julia_module
//! [rustfft_jl]: https://github.com/Taaitaaiger/rustfft-jl
//! [here]: https://github.com/JuliaPackaging/Yggdrasil/tree/master/R/rustfft

#![forbid(rustdoc::broken_intra_doc_links)]

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "sync-rt")]
use once_cell::sync::OnceCell;

use crate::{
    data::managed::{module::Module, value::Value},
    memory::{
        context::{ledger::init_ledger, stack::Stack},
        stack_frame::PinnedFrame,
        target::unrooted::Unrooted,
    },
};

#[cfg(feature = "pyplot")]
macro_rules! init_fn {
    ($name:ident, $include:ident, $file:expr) => {
        pub(crate) static $include: &'static str = include_str!($file);
        pub(crate) unsafe fn $name<'frame>(
            frame: &mut $crate::memory::target::frame::GcFrame<'frame>,
        ) -> () {
            match $crate::data::managed::value::Value::eval_string(frame, $include) {
                Ok(_) => (),
                Err(e) => {
                    panic!(
                        "{}",
                        $crate::data::managed::Managed::error_string_or(
                            e,
                            $crate::error::CANNOT_DISPLAY_VALUE
                        )
                    )
                }
            }
        }
    };
}

#[cfg(feature = "async")]
pub mod async_util;
pub mod call;
pub(crate) mod catch;
#[cfg(feature = "ccall")]
pub mod ccall;
pub mod convert;
pub mod data;
pub mod error;
pub mod info;
pub mod memory;
#[cfg(feature = "prelude")]
pub mod prelude;
pub(crate) mod private;
#[cfg(feature = "pyplot")]
pub mod pyplot;
#[cfg(any(feature = "sync-rt", feature = "async-rt"))]
pub mod runtime;
pub mod safety;
#[doc(hidden)]
#[cfg(feature = "sync-rt")]
pub mod util;

/// Installation method for the JlrsCore package. If JlrsCore is already installed the installed version
/// is used.
#[derive(Clone)]
pub enum InstallJlrsCore {
    /// Install the most recent version of JlrsCore
    Default,
    /// Don't install the JlrsCore
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
    pub(crate) unsafe fn use_or_install(&self, unrooted: Unrooted) {
        match self {
            InstallJlrsCore::Default => {
                Value::eval_string(
                    unrooted,
                    "if !isdefined(Main, :JlrsCore)
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
                        "if !isdefined(Main, :JlrsCore)
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
                        "if !isdefined(Main, :JlrsCore)
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
                    "if !isdefined(Main, :JlrsCore)
                         using JlrsCore
                     end",
                )
            },
        }
        .expect("Failed to load or install JlrsCore package");
    }
}

// The chosen install method is stored in a OnceCell when the sync runtime is used to
// avoid having to store it in `PendingJulia`.
#[cfg(feature = "sync-rt")]
pub(crate) static INSTALL_METHOD: OnceCell<InstallJlrsCore> = OnceCell::new();

pub(crate) unsafe fn init_jlrs<const N: usize>(
    frame: &mut PinnedFrame<N>,
    install_jlrs_core: &InstallJlrsCore,
) {
    static IS_INIT: AtomicBool = AtomicBool::new(false);

    if IS_INIT.swap(true, Ordering::Relaxed) {
        return;
    }

    let unrooted = Unrooted::new();
    install_jlrs_core.use_or_install(unrooted);

    let jlrs_module = Module::main(&unrooted)
        .submodule(unrooted, "JlrsCore")
        .unwrap()
        .as_managed();

    init_ledger();

    // Init foreign Stack type
    Stack::init(frame, jlrs_module);
}
