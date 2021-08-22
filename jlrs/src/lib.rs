//! jlrs provides access to most of the Julia C API, it can be used to embed Julia in Rust
//! applications and to use functionality from the Julia C API when writing `ccall`able functions.
//! Currently this crate is only tested on Linux in combination with Julia 1.7 and is not
//! compatible with earlier versions of Julia.
//!
//! The documentation assumes you have a basic understanding of Julia's type system.
//!
//! # Features
//!
//! An incomplete list of features that are currently supported by jlrs:
//!
//!  - Access arbitrary Julia modules and their contents.
//!  - Call Julia functions, including functions that take keyword arguments.
//!  - Exceptions can be handled or converted to their error message, optionally with color.
//!  - Include and call your own Julia code.
//!  - Use a custom system image.
//!  - Create values that Julia can use, and convert them back to Rust, from Rust.
//!  - Access the type information and fields of values. The contents of inline and bits-union
//!    fields can be accessed directly.
//!  - Create and use n-dimensional arrays. The `jlrs-ndarray` feature can be enabled for
//!    integration with ndarray.
//!  - Support for mapping Julia structs to Rust structs that can be generated by JlrsReflect.jl.
//!  - Structs that can be mapped to Rust include those with type parameters and bits unions.
//!  - An async runtime is available when the `async` feature is enabled, which can be used from
//!    multiple threads and supports scheduling Julia tasks and `await`ing the result without
//!    blocking the runtime.
//!
//!
//! # Generating the bindings
//!
//! This crate depends on jl-sys which contains the raw bindings to the Julia C API, these are
//! generated by bindgen. You can find the requirements for using bindgen in [their User Guide].
//!
//! #### Linux
//!
//! The recommended way to install Julia is to download the binaries from the official website,
//! which is distributed in an archive containing a directory called `julia-x.y.z`. This directory
//! contains several other directories, including a `bin` directory containing the `julia`
//! executable.
//!
//! In order to ensure the `julia.h` header file can be found, either `/usr/include/julia/julia.h`
//! must exist, or you have to set the `JULIA_DIR` environment variable to `/path/to/julia-x.y.z`.
//! This environment variable can be used to override the default. Similarly, in order to load
//! `libjulia.so` you must add `/path/to/julia-x.y.z/lib` to the `LD_LIBRARY_PATH` environment
//! variable.
//!
//! #### Windows
//!
//! Support for Windows was dropped in jlrs 0.10 due to compilation and dependency issues. If you
//! want to use jlrs on Windows you must use WSL. An installation guide to install WSL on Windows
//! can be found [on Microsoft's website]. After installing a Linux distribution, follow the
//! installation instructions for Linux.
//!
//!
//! # Using this crate
//!
//! The first thing you should do is `use` the [`prelude`]-module with an asterisk, this will
//! bring all the structs and traits you're likely to need into scope. When embedding Julia, it
//! must be initialized before it can be used. You can do this by calling [`Julia::init`] which
//! returns an instance of [`Julia`]. Note that this method can only be called once while the
//! application is running; if you drop it you won't be able to create a new instance but have to
//! restart the application. If you want to use a custom system image, you must call
//! [`Julia::init_with_image`] instead of `Julia::init`. If you're calling Rust from Julia
//! everything has already been initialized, you can use `CCall` instead. If you want to use the
//! async runtime, one of the initialization methods of [`AsyncJulia`] must be used.
//!
//!
//! ## Calling Julia from Rust
//!
//! After initialization you have an instance of [`Julia`], [`Julia::include`] can be used to
//! include files with custom Julia code. In order to call Julia functions and create new values
//! that can be used by these functions, [`Julia::scope`] and [`Julia::scope_with_slots`] must be
//! used. These two methods take a closure with two arguments, a [`Global`] and a mutable
//! reference to a [`GcFrame`]. `Global` is a token that is used to access Julia modules, their
//! contents and other global values, while `GcFrame` is used to root local values. Rooting a
//! value in a frame prevents it from being freed by the garbage collector until that frame has
//! been dropped. The frame is created when `Julia::scope(_with_slots)` is called and dropped
//! when that method returns.
//!
//! Because you can use both a `Global` and a mutable reference to a `GcFrame` inside the closure,
//! it's possible to access the contents of modules and create new values that can be used by
//! Julia. The methods of [`Module`] let you access the contents of arbitrary modules, several
//! methods are available to create new values.
//!
//! The simplest is to call [`Value::eval_string`], a method that takes two arguments. The first
//! must implement the [`Scope`] trait, the second is a string which has to contain valid Julia
//! code. The most important thing to know about the [`Scope`] trait for now is that it's used
//! by functions that create new values to ensure the result is rooted. Mutable references to
//! [`GcFrame`]s implement [`Scope`], in this case the [`Value`] that is returned is rooted in
//! that frame, so the result is protected from garbage collection until the frame is dropped when
//! that scope ends.
//!
//! In practice, [`Value::eval_string`] is relatively limited. It can be used to evaluate simple
//! function calls like `sqrt(2.0)`, but can't take any arguments. Its most important use-case is
//! importing installed packages by evaluating an `import` or `using` statement. A more
//! interesting method, [`Value::new`], can be used with data of any type that implements
//! [`IntoJulia`]. This trait is implemented by primitive types like `i8` and `char`. Any type
//! that implements [`IntoJulia`] also implements [`Unbox`] which is used to extract the contents
//! of a Julia value.
//!
//! In addition to evaluating raw commands with `Value::eval_string`, it's possible to call
//! anything that implements [`Call`] as a Julia function, `Value` implements this trait because
//! any Julia value is potentially callable as a function. Functions can be called with any number
//! of positional arguments and be provided with keyword arguments. Both `Value::eval_string` and
//! the trait methods of `Call` are all unsafe. It's trivial to write a function like
//! `boom() = unsafe_load(Ptr{Float64}(C_NULL))`, which causes a segfault when it's called, and
//! call it with these methods.
//!
//! As a simple example, let's convert two numbers to Julia values and add them:
//!
//! ```no_run
//! use jlrs::prelude::*;
//!
//! # fn main() {
//! // Initializing Julia is unsafe because it can race with another crate that does
//! // the same.
//! let mut julia = unsafe { Julia::init().unwrap() };
//! let res = julia.scope(|global, frame| {
//!     // Create the two arguments. Note that the first argument, something that
//!     // implements Scope, is taken by value and mutable references don't implement
//!     // Copy, so it's necessary to mutably reborrow the frame.
//!     let i = Value::new(&mut *frame, 2u64)?;
//!     let j = Value::new(&mut *frame, 1u32)?;
//!
//!     // The `+` function can be found in the base module.
//!     let func = Module::base(global).function(&mut *frame, "+")?;
//!
//!     // Call the function and unbox the result as a `u64`. The result of the function
//!     // call is a nested `Result`; the outer error doesn't contain to any Julia
//!     // data, while the inner error contains the exception if one is thrown. Here the
//!     // exception is converted to the outer error type by calling `into_jlrs_result`, this new
//!     // error contains the error message Julia would have shown. Colors can be enabled by
//!     // calling `Julia::error_color`.
//!     unsafe {
//!         func.call2(&mut *frame, i, j)?
//!             .into_jlrs_result()?
//!             .unbox::<u64>()
//!     }
//! }).unwrap();
//!
//! assert_eq!(res, 3);
//! # }
//! ```
//!
//! Many more features are available, including creating and accessing n-dimensional Julia arrays
//! and nesting scopes. To learn how to use them, please see the documentation for the [`memory`]
//! and [`wrappers`] modules.
//!
//!
//! ## Calling Rust from Julia
//!
//! Julia's `ccall` interface can be used to call `extern "C"` functions defined in Rust, for most
//! use-cases you shouldn't need jlrs. There are two major ways to use `ccall`, with a pointer to
//! the function or a `(:function, "library")` pair.
//!
//! A function can be cast to a void pointer and converted to a [`Value`]:
//!
//! ```no_run
//! # use jlrs::prelude::*;
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
//! let mut julia = unsafe { Julia::init().unwrap() };
//! julia.scope(|global, frame| unsafe {
//!     // Cast the function to a void pointer
//!     let call_me_val = Value::new(&mut *frame, call_me as *mut std::ffi::c_void)?;
//!
//!     // Value::eval_string can be used to create new functions.
//!     let func = Value::eval_string(
//!         &mut *frame,
//!         "myfunc(callme::Ptr{Cvoid})::Int = ccall(callme, Int, (Bool,), true)"
//!     )?.unwrap();
//!
//!     // Call the function and unbox the result.  
//!     let output = func.call1(&mut *frame, call_me_val)?
//!         .into_jlrs_result()?
//!         .unbox::<isize>()?;
//!
//!     assert_eq!(output, 1);
//!     
//!     Ok(())
//! }).unwrap();
//! # }
//! ```
//!
//! You can also use functions defined in `dylib` and `cdylib` libraries. In order to create such
//! a library you need to add
//!
//! ```toml
//! [lib]
//! crate-type = ["dylib"]
//! ```
//!
//! or  
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! respectively to your crate's `Cargo.toml`. Use a `dylib` if you want to use the crate in other
//! Rust crates, but if it's only intended to be called through `ccall` a `cdylib` is the better
//! choice. On Linux, compiling such a crate will be compiled to `lib<crate_name>.so`.
//!
//! The functions you want to use with `ccall` must be both `extern "C"` functions to ensure the C
//! ABI is used, and annotated with `#[no_mangle]` to prevent name mangling. Julia can find
//! libraries in directories that are either on the default library search path or included by
//! setting the `LD_LIBRARY_PATH` environment variable on Linux. If the compiled library is not
//! directly visible to Julia, you can open it with `Libdl.dlopen` and acquire function pointers
//! with `Libdl.dlsym`. These pointers can be called the same way as the pointer in the previous
//! example.
//!
//! If the library is visible to Julia you can access it with the library name. If `call_me` is
//! defined in a crate called `foo`, the following should work if the function is annotated with
//! `#[no_mangle]`:
//!
//! ```julia
//! ccall((:call_me, "libfoo"), Int, (Bool,), false)
//! ```
//!
//! One important aspect of calling Rust from other languages in general is that panicking across
//! an FFI boundary is undefined behaviour. If you're not sure your code will never panic, wrap it
//! with `std::panic::catch_unwind`.
//!
//! Most features provided by jlrs including accessing modules, calling functions, and borrowing
//! array data require a [`Global`] or a frame. You can access these by creating a [`CCall`]
//! first. Another method provided by [`CCall`] is [`CCall::uv_async_send`], this method can be
//! used in combination with `Base.AsyncCondition`. In particular, it lets you write a `ccall`able
//! function that does its actual work on another thread, return early and `wait` on the async
//! condition, which happens when [`CCall::uv_async_send`] is called when that work is finished.
//! The advantage of this is that the long-running function will not block the Julia runtime,
//! There's an example available on GitHub that shows how to do this.
//!
//!
//! ## Async runtime
//!
//! The async runtime runs Julia in a separate thread and allows multiple tasks to run in
//! parallel. This works by offloading functions to a new thread in Julia and waiting for them to
//! complete without blocking the runtime. To use this feature you must enable the `async` feature
//! flag:
//!
//! ```toml
//! [dependencies]
//! jlrs = { version = "0.12", features = ["async"] }
//! ```
//!
//! The struct [`AsyncJulia`] is exported by the prelude and lets you initialize the runtime in
//! two ways, either as a blocking task or as a thread. The first way should be used if you want
//! to integrate the async runtime into a larger project that uses `async_std`.
//!
//! In order to call Julia when using the async runtime you must implement the either the
//! [`AsyncTask`] or [`GeneratorTask`] trait. An `AsyncTask` can be called once, its `run` method
//! replaces the closure that's used in the example above for the sync runtime; it provides you
//! with a `Global` and an [`AsyncGcFrame`] which provides mostly the same functionality as
//! `GcFrame`. The `AsyncGcFrame` is required to call the methods of the [`CallAsync`] trait.
//! These methods schedule the function call on another thread and return a `Future`. While
//! awaiting the result the runtime can handle another task.
//!
//! A `GeneratorTask` can be called multiple times. In addition to `run` it also has an `init`
//! method. This method is called when the `GeneratorTask` is created and can be used to prepare
//! the initial state of the task. The frame provided to `init` is not dropped after this method
//! returns, which means this initial state can contain Julia data. Whenever a `GeneratorTask` is
//! created, a [`GeneratorHandle`] is returned. This handle can be used to call the
//! `GeneratorTask` which calls its `run` method once. A `GeneratorHandle` can be cloned and
//! shared across threads.
//!
//! You can find basic examples that show how to implement these traits in
//! [the examples directory of the GitHub repository].
//!
//!
//! # Testing
//!
//! The restriction that Julia can be initialized once must be taken into account when running
//! tests that use `jlrs`. The recommended approach is to create a thread-local static `RefCell`:
//!
//! ```no_run
//! use jlrs::prelude::*;
//! use std::cell::RefCell;
//! thread_local! {
//!     pub static JULIA: RefCell<Julia> = {
//!         let julia = RefCell::new(unsafe { Julia::init().unwrap() });
//!         julia.borrow_mut().scope(|_global, _frame| {
//!             /* include everything you need to use */
//!             Ok(())
//!         }).unwrap();
//!         julia
//!     };
//! }
//! ```
//!
//! Tests that use this construct can only use one thread for testing, so you must use
//! `cargo test -- --test-threads=1`, otherwise the code above will panic when a test
//! tries to call `Julia::init` a second time from another thread.
//!
//! If these tests also involve the async runtime, the `JULIA_NUM_THREADS` environment
//! variable must be set to a value larger than 2.
//!
//! If you want to run jlrs's tests, both these requirements must be taken into account:
//! `JULIA_NUM_THREADS=3 cargo test -- --test-threads=1`
//!
//!
//! # Custom types
//!
//! In order to map a struct in Rust to one in Julia you can derive [`ValidLayout`], [`Unbox`],
//! and [`Typecheck`]. If the struct in Julia has no type parameters and is a bits type you can
//! also derive [`IntoJulia`], which lets you use the type in combination with [`Value::new`].
//!
//! You should normally not need to implement these structs or traits manually. The JlrsReflect.jl
//! package can generate the correct Rust struct and automatically derive the supported traits for
//! types that have no tuple or union fields with type parameters. The reason for this restriction
//! is that the layout of tuple and union fields can be very different depending on these
//! parameters in a way that can't be expressed in Rust.
//!
//! These custom types can also be used when you call Rust from Julia with `ccall`.
//!
//! [their User Guide]: https://rust-lang.github.io/rust-bindgen/requirements.html
//! [on Microsoft's website]: https://docs.microsoft.com/en-us/windows/wsl/install-win10
//! [the examples directory of the repo]: https://github.com/Taaitaaiger/jlrs/tree/master/examples
//! [`IntoJulia`]: crate::convert::into_julia::IntoJulia
//! [`Typecheck`]: crate::layout::typecheck::Typecheck
//! [`ValidLayout`]: crate::layout::valid_layout::ValidLayout
//! [`Unbox`]: crate::convert::unbox::Unbox
//! [`CallAsync::call_async`]: crate::extensions::multitask::call_async::CallAsync
//! [`AsyncGcFrame`]: crate::extensions::multitask::async_frame::AsyncGcFrame
//! [`Frame`]: crate::memory::frame::Frame
//! [`AsyncTask`]: crate::extensions::multitask::async_task::AsyncTask
//! [`GeneratorTask`]: crate::extensions::multitask::async_task::GeneratorTask
//! [`GeneratorHandle`]: crate::extensions::multitask::async_task::GeneratorHandle
//! [`AsyncJulia`]: crate::extensions::multitask::AsyncJulia
//! [`DataType`]: crate::wrappers::ptr::datatype::DataType
//! [`TypedArray`]: crate::wrappers::ptr::array::TypedArray
//! [`Output`]: crate::memory::output::Output
//! [`OutputScope`]: crate::memory::output::OutputScope
//! [`ScopeExt`]: crate::memory::scope::ScopeExt
//! [`ScopeExt::scope`]: crate::memory::scope::ScopeExt::scope
//! [`Scope`]: crate::memory::scope::Scope
//! [`Scope::value_scope`]: crate::memory::scope::Scope::value_scope
//! [`Scope::result_scope`]: crate::memory::scope::Scope::result_scope

#![forbid(rustdoc::broken_intra_doc_links)]

pub mod convert;
pub mod error;
pub mod extensions;
pub mod info;
pub mod layout;
pub mod memory;
pub mod prelude;
pub(crate) mod private;
#[doc(hidden)]
pub mod util;
pub mod wrappers;

use convert::into_jlrs_result::IntoJlrsResult;
use error::{JlrsError, JlrsResult, CANNOT_DISPLAY_VALUE};
use info::Info;
#[cfg(not(feature = "coverage"))]
use jl_sys::uv_async_send;
use jl_sys::{
    jl_array_dims_ptr, jl_array_ndims, jl_atexit_hook, jl_init, jl_init_with_image,
    jl_is_initialized,
};
use memory::frame::{GcFrame, NullFrame};
use memory::global::Global;
use memory::mode::Sync;
use memory::stack_page::StackPage;
use prelude::Wrapper;
use private::Private;
use std::ffi::{c_void, CString};
use std::io::{Error as IOError, ErrorKind};
use std::mem::{self, MaybeUninit};
use std::path::Path;
use std::ptr::null_mut;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use wrappers::ptr::module::Module;
use wrappers::ptr::string::JuliaString;
use wrappers::ptr::value::Value;
use wrappers::ptr::{array::Array, call::Call, private::Wrapper as _};

pub(crate) static INIT: AtomicBool = AtomicBool::new(false);

pub(crate) static JLRS_JL: &'static str = include_str!("jlrs.jl");

/// A Julia instance. You must create it with [`Julia::init`] or [`Julia::init_with_image`]
/// before you can do anything related to Julia. While this struct exists Julia is active,
/// dropping it causes the shutdown code to be called but this doesn't leave Julia in a state from which it can be reinitialized.
pub struct Julia {
    page: StackPage,
}

impl Julia {
    /// Initialize Julia, this method can only be called once. If it's called a second time it
    /// will return an error. If this struct is dropped, you will need to restart your program to
    /// be able to call Julia code again.
    ///
    /// This method is unsafe because it can race with another crate initializing Julia.
    pub unsafe fn init() -> JlrsResult<Self> {
        if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
            return Err(JlrsError::AlreadyInitialized.into());
        }

        jl_init();
        let mut jl = Julia {
            page: StackPage::default(),
        };

        jl.scope_with_slots(1, |_, frame| {
            Value::eval_string(&mut *frame, JLRS_JL)?.into_jlrs_result()?;
            Ok(())
        })
        .expect("Could not load Jlrs module");

        Ok(jl)
    }

    /// This method is similar to [`Julia::init`] except that it loads a custom system image. A
    /// custom image can be generated with the [`PackageCompiler`] package for Julia. The main
    /// advantage of using a custom image over the default one is that it allows you to avoid much
    /// of the compilation overhead often associated with Julia.
    ///
    /// Two arguments are required to call this method compared to [`Julia::init`];
    /// `julia_bindir` and `image_relative_path`. The first must be the absolute path to a
    /// directory that contains a compatible Julia binary (eg `${JULIA_DIR}/bin`), the second must
    /// be either an absolute or a relative path to a system image.
    ///
    /// This method will return an error if either of the two paths doesn't  exist or if Julia
    /// has already been initialized. It is unsafe because it can race with another crate
    /// initializing Julia.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub unsafe fn init_with_image<P: AsRef<Path>, Q: AsRef<Path>>(
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<Self> {
        if INIT.swap(true, Ordering::SeqCst) {
            Err(JlrsError::AlreadyInitialized)?;
        }

        let julia_bindir_str = julia_bindir.as_ref().to_string_lossy().to_string();
        let image_path_str = image_path.as_ref().to_string_lossy().to_string();

        if !julia_bindir.as_ref().exists() {
            let io_err = IOError::new(ErrorKind::NotFound, julia_bindir_str);
            return Err(JlrsError::other(io_err))?;
        }

        if !image_path.as_ref().exists() {
            let io_err = IOError::new(ErrorKind::NotFound, image_path_str);
            return Err(JlrsError::other(io_err))?;
        }

        let bindir = CString::new(julia_bindir_str).unwrap();
        let im_rel_path = CString::new(image_path_str).unwrap();

        jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr());

        let mut jl = Julia {
            page: StackPage::default(),
        };

        jl.scope_with_slots(1, |_, frame| {
            Value::eval_string(&mut *frame, JLRS_JL)?.into_jlrs_result()?;
            Ok(())
        })
        .expect("Could not load Jlrs module");

        Ok(jl)
    }

    /// Enable or disable colored error messages originating from Julia. If this is enabled the
    /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
    /// disabled by default.
    pub fn error_color(&mut self, enable: bool) -> JlrsResult<()> {
        self.scope(|global, _frame| unsafe {
            let enable = if enable {
                Value::true_v(global)
            } else {
                Value::false_v(global)
            };
            Module::main(global)
                .submodule_ref("Jlrs")?
                .wrapper_unchecked()
                .global_ref("color")?
                .value_unchecked()
                .set_field_unchecked("x", enable)?;
            Ok(())
        })?;

        Ok(())
    }

    /// Calls `include` in the `Main` module in Julia, which executes the file's contents in that
    /// module. This has the same effect as calling `include` in the Julia REPL.
    ///
    /// Example:
    ///
    /// ```no_run
    /// # use jlrs::prelude::*;
    /// # fn main() {
    /// # let mut julia = unsafe { Julia::init().unwrap() };
    /// julia.include("Path/To/MyJuliaCode.jl").unwrap();
    /// # }
    /// ```
    pub fn include<P: AsRef<Path>>(&mut self, path: P) -> JlrsResult<()> {
        if path.as_ref().exists() {
            return self.scope_with_slots(2, |global, frame| unsafe {
                let path_jl_str = JuliaString::new(&mut *frame, path.as_ref().to_string_lossy())?;
                let include_func = Module::main(global)
                    .function_ref("include")?
                    .wrapper_unchecked();

                let res = include_func.call1(frame, path_jl_str)?;

                return match res {
                    Ok(_) => Ok(()),
                    Err(e) => Err(JlrsError::IncludeError {
                        path: path.as_ref().to_string_lossy().into(),
                        msg: e.display_string_or(CANNOT_DISPLAY_VALUE),
                    })?,
                };
            });
        }

        Err(JlrsError::IncludeNotFound {
            path: path.as_ref().to_string_lossy().into(),
        })?
    }

    /// This method is a main entrypoint to interact with Julia. It takes a closure with two
    /// arguments, a `Global` and a mutable reference to a `GcFrame`, and can return arbitrary
    /// results.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope(|_global, frame| {
    ///       let _i = Value::new(&mut *frame, 1u64)?;
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let global = Global::new();
            let mut frame = GcFrame::new(self.page.as_mut(), 0, Sync);
            func(global, &mut frame)
        }
    }

    /// This method is a main entrypoint to interact with Julia. It takes a closure with two
    /// arguments, a `Global` and a mutable reference to a `GcFrame`, and can return arbitrary
    /// results. The frame will preallocate `slots` slots.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.scope_with_slots(1, |_global, frame| {
    ///       // Uses the preallocated slot
    ///       let _i = Value::new(&mut *frame, 1u64)?;
    ///       // Allocates a new slot, because only a single slot was preallocated
    ///       let _j = Value::new(&mut *frame, 1u64)?;
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    pub fn scope_with_slots<T, F>(&mut self, slots: usize, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let global = Global::new();
            if slots + 2 > self.page.size() {
                self.page = StackPage::new(slots + 2);
            }
            let mut frame = GcFrame::new(self.page.as_mut(), slots, Sync);
            func(global, &mut frame)
        }
    }

    /// Provides access to global information.
    pub fn info(&self) -> Info {
        Info::new()
    }
}

impl Drop for Julia {
    fn drop(&mut self) {
        unsafe {
            jl_atexit_hook(0);
        }
    }
}

/// When you call Rust from Julia through `ccall`, Julia has already been initialized and trying to
/// initialize it again would cause a crash. In order to still be able to call Julia from Rust
/// and to borrow arrays (if you pass them as `Array` rather than `Ptr{Array}`), you'll need to
/// create a frame first. You can use this struct to do so. It must never be used outside
/// functions called through `ccall`, and only once for each `ccall`ed function.
///
/// If you only need to use a frame to borrow array data, you can use [`CCall::null_scope`].
/// Unlike [`Julia`], `CCall` postpones the allocation of the stack that is used for managing the
/// GC until a `GcFrame` is created. In the case of a null scope, this stack isn't allocated at
/// all.
pub struct CCall {
    page: Option<StackPage>,
}

impl CCall {
    /// Create a new `CCall`. This function must never be called outside a function called through
    /// `ccall` from Julia and must only be called once during that call. The stack is not
    /// allocated until a [`GcFrame`] is created.
    pub unsafe fn new() -> Self {
        CCall { page: None }
    }

    /// Wake the task associated with `handle`. The handle must be the `handle` field of a
    /// `Base.AsyncCondition` in Julia. This can be used to call a long-running Rust function from
    /// Julia with ccall in another thread and wait for it to complete in Julia without blocking,
    /// there's an example available in the repository: ccall_with_threads.
    #[cfg(not(feature = "coverage"))]
    pub unsafe fn uv_async_send(handle: *mut c_void) -> bool {
        uv_async_send(handle.cast()) == 0
    }

    /// Creates a [`GcFrame`], calls the given closure, and returns its result.
    pub fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let page = self.get_init_page();
            let global = Global::new();
            let mut frame = GcFrame::new(page.as_mut(), 0, Sync);
            func(global, &mut frame)
        }
    }

    /// Creates a [`GcFrame`] with `slots` slots, calls the given closure, and returns its result.
    pub fn scope_with_slots<T, F>(&mut self, slots: usize, func: F) -> JlrsResult<T>
    where
        for<'base> F: FnOnce(Global<'base>, &mut GcFrame<'base, Sync>) -> JlrsResult<T>,
    {
        unsafe {
            let page = self.get_init_page();
            let global = Global::new();
            if slots + 2 > page.size() {
                *page = StackPage::new(slots + 2);
            }
            let mut frame = GcFrame::new(page.as_mut(), slots, Sync);
            func(global, &mut frame)
        }
    }

    /// Create a [`NullFrame`] and call the given closure. A [`NullFrame`] cannot be nested and
    /// can only be used to (mutably) borrow array data. Unlike other scope-methods, no `Global`
    /// is provided to the closure.
    pub fn null_scope<'base, 'julia: 'base, T, F>(&'julia mut self, func: F) -> JlrsResult<T>
    where
        F: FnOnce(&mut NullFrame<'base>) -> JlrsResult<T>,
    {
        unsafe {
            let mut frame = NullFrame::new(self);
            func(&mut frame)
        }
    }

    #[inline(always)]
    fn get_init_page(&mut self) -> &mut StackPage {
        if self.page.is_none() {
            self.page = Some(StackPage::default());
        }

        self.page.as_mut().unwrap()
    }
}

unsafe extern "C" fn droparray(a: Array) {
    // The data of a moved array is allocated by Rust, this function is called by
    // a finalizer in order to ensure it's also freed by Rust.
    let mut arr_nn_ptr = a.unwrap_non_null(Private);
    let arr_ref = arr_nn_ptr.as_mut();

    if arr_ref.flags.how() != 2 {
        return;
    }

    // Set data to null pointer
    let data_ptr = arr_ref.data.cast::<MaybeUninit<u8>>();
    arr_ref.data = null_mut();

    // Set all dims to 0
    let arr_ptr = arr_nn_ptr.as_ptr();
    let dims_ptr = jl_array_dims_ptr(arr_ptr);
    let n_dims = jl_array_ndims(arr_ptr);
    let mut_dims_slice = slice::from_raw_parts_mut(dims_ptr, n_dims as _);
    for dim in mut_dims_slice {
        *dim = 0;
    }

    // Drop the data
    let n_els = arr_ref.elsize as usize * arr_ref.length;
    let data = Vec::from_raw_parts(data_ptr, n_els, n_els);
    mem::drop(data);
}
