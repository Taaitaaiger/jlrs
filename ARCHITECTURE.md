# Architecture

This document aims to provide a bottom-up description of the internal architecture of jlrs. It starts with some information about the Julia C API, this is mostly a summary of the chapter [Embedding Julia](https://docs.julialang.org/en/v1/manual/embedding/) from the Julia manual. This chapter and the chapter [Calling C and Fortran Code](https://docs.julialang.org/en/v1/manual/calling-c-and-fortran-code/) are recommended reading. The Developer Documentation in the Julia manual contains more detailed information about topics as the [layout of Julia values](https://docs.julialang.org/en/v1/devdocs/reflection/) and [how Julia functions work](https://docs.julialang.org/en/v1/devdocs/functions/).


## Julia C API

The Julia C API defines many types, functions and macros that can be used when embedding Julia in another application. There are functions to initialize and stop Julia, create and access new values, access modules and their contents, evaluate Julia code directly, and so on. Some type checking is implemented as a macro, and so is managing the garbage collector. How this functionality is made accessible to Rust with jl-sys is explained later.

Before Julia can be used it must first be initialized. Two functions are available for this purpose, `jl_init` and `jl_init_with_image`. The first uses the default system image, while the latter can be used to load a custom system image. Before your program exits, it should call `jl_atexit_hook` to ensure pending write requests are cleaned up and all finalizers will be executed. This does _not_ leave Julia in a state from which it can be restarted, `jl_init` can only be called once during the lifetime of a program. The C API must only be used from the thread that initialized Julia.

The easiest way to call Julia with the C API is provide some Julia expression as a string to `jl_eval_string`. This function returns a Julia value as a pointer to the heap-allocated object: `jl_value_t*`. This is an opaque pointer, the type of the data pointed to is not explicitly specified. The data pointed to is guaranteed to be preceded by a header, which contains a pointer to the value's type information. 


### Types and their layout

This type information is itself a Julia value, it's an instance of `Core.DataType`. A `Core.DataType` is backed by a type defined in the C API, `jl_datatype_t`. Other types in Julia that are defined in the C API include `Core.Array` (`jl_array_t`), `Core.Module` (`jl_module_t`), and `Core.Symbol` (`jl_symbol_t`). When a value has such a type, the pointer to the heap-allocated object is a pointer to the backing type. This means that `jl_value_t *ptr` can be cast to `jl_module_t *ptr` if its type is `Core.Module`. The same holds true for primitive types, if the type of a value is a primitive type like `Core.UInt16`, `jl_value_t *ptr` can be cast to `uint16_t *ptr`. 

The layout of user-defined types is more complex. In the simplest case, none of the fields contain a pointer or a union. These types are known as bits-types. Builtin types like `Core.Array` and `Core.Module` are examples of pointer types, `Union{Float64,Int64}` is an example of a union. Bits-types have the same layout as if they were defined in C. Pointer fields are also relatively straightforward; they can be represented by `jl_value_t*`, or a pointer to the corresponding builtin type if it's known. Unions are more complex, if at least one of the variants is a pointer it is represented as a pointer itself. If all variants are bits-types, the bits-union optimization applies: the data is stored with the same alignment as the variant with the largest aligment, has the same size as the variant with the same size, and is followed by a single byte that serves as a flag to indicate which variant is stored. While types with bits-unions are not bits-types, the bits-union optimization does apply to them.


### Creating new values

The easiest kind of data to convert to Julia is data with a primitive type like `uint8_t` of `float`. Functions with the name `jl_box_<type>` are available that convert a value of type `<type>` to a value managed by Julia. A few functions are available to create new instances of some pointer types, such as Julia strings and `jl_symbol_t*`. New arrays can also be created, but this requires constructing the appropriate array type first. Constructing the appropriate type is also required for other functions that create new values of that type.


### Modules, globals, and functions

Modules provide separate global scopes in Julia. They contain things like custom types, functions, submodules, and other global values. By default Julia provides the `Core` and `Base` modules, your own code is available relative to the `Main` module. The module tree can be traversed through the relevant child modules, these child modules are acquired the same way as other globals in a module: with `jl_get_global`. 


### Calling Julia functions

One interesting aspect of Julia functions is that there is no specific function type, `jl_function_t*` is just an alias for `jl_value_t*`. All Julia values can potentially be called as functions. Several functions with names like `jl_call` are available which call the Julia function with some number of arguments. One nice feature of these functions is that they use the `JL_TRY`/`JL_CATCH` macros internally which makes it possible to acquire the exception if one was thrown.

Julia functions can have keyword arguments. Calling these functions from the C API involves some extra steps. Let's say there's a function `f` which takes one or more keyword arguments. First of all, a `Core.NamedTuple` has to be created which contains the overridden keyword arguments. Next, `f` must be transformed so it can take these arguments; the function `jl_get_kwsorter` is used to do this. It takes the `Core.DataType` of `f` and returns a new Julia function which takes the following arguments: the `Core.NamedTuple` with the overriden keyword arguments, the original function `f`, and then all positional arguments. 


### Garbage collector

Julia's heap is managed by a garbage collector. The garbage collector is not aware of any values returned by the C API, but several macros are available that help ensure values are not freed by the garbage collector while they're in use. The garbage collector uses a set of roots to determine what values are in use. When a value is a root, it and any other value it points to can be safely used; all of these values are rooted. The garbage collector can run whenever a safepoint is reached, one example of a safepoint is allocating a new value. At a safepoint, all values that are in use must be rooted. Structurally, the root set is a stack of variably-sized arrays that contain pointers (a stack of frames) to values which is managed in lockstep with stack frames in C: one or more slots for a root are created with a `JL_GC_PUSH` macro, which are used to root temporary values. After acquiring the final result, which is not rooted in that frame, the roots are popped from the stack with the `JL_GC_POP` macro. The result is returned and rooted in that function's frame. Each frame has two slots of overhead; the first is used for the number slots, the second has a pointer to the previous frame.


## jl-sys

In order to use the Julia C API from Rust bindings are needed, which are provided by jl-sys. With the exception of macros and functions marked as `STATIC_INLINE` these bindings can be generated with `bindgen`. Macros and `STATIC_INLINE` functions are manually reimplemented. In most cases this is a very straightforward process, but some cases are problematic. In particular, the `GC_PUSH` macros make use of `alloca` to allocate a dynamically-sized array on the stack, and the `JL_TRY`/`JL_CATCH` macros depend on `longjmp`. Due to these limitations, jl-sys provides no functions to manage the garbage collector stack, and errors thrown by Julia can only be handled when a Julia function is called because they make use of the `JL_TRY`/`JL_CATCH` macros internally. This situation should improve when the `"C-unwind"` ABI can be used.


## jlrs

The bindings provided by jl-sys are very much a C API. Everything is unsafe, it's full of raw pointers which are non-trivial to use correctly, and from a Rust programmer's perspective things have odd names. In order to improve this situation jlrs has several design goals:

 - Users should only be able to use the Julia C API after it has been initialized.
 - Users should never work with raw pointers from the Julia C API, but rather wrapper types that have Rustified names.
 - These wrapper types should be rooted automatically, their validity should be expressed through lifetimes.
 - Functions from the C API should be implemented as methods of their corresponding wrapper type, with similarly Rustified names.
 - Methods that are used to convert data between Julia and Rust should be extensible.


### Garbage collector

As stated previously, jl-sys provides no functions to push frames to and pop them from the garbage collector stack. jlrs has to provide its own implementation, the structs and traits involved are defined in the `memory` module.

Memory used to store frames is allocated one `StackPage` at a time, a `StackPage` is a boxed slice of void pointers. It's not a `Vec` because its contents must not move. Multiple frames can be stored on the same page, if the page has insufficient space left for a new frame a new page is allocated. Julia has a pointer to the frame at the top of the stack, when a new frame is pushed to or one is popped from it, this pointer is updated. How this should be done exactly depends on the runtime mode which will be discussed later. The runtime modes implement the `Mode` trait to handle the differences. 

The raw frame is slice of the stack page, it is managed through a `GcFrame`. A `GcFrame` has a number of slots, a number of rooted values, and a capacity. Unlike a frame in the Julia C API, a `GcFrame` in jlrs can grow its number of slots dynamically up to its capacity. It cannot span multiple `StackPage`s. Most of a `GcFrame`s functionality can be found in the `Frame` trait. Neither `GcFrame` nor `Frame` offer a public method the create a new `GcFrame`, rather methods like `frame`, `value_frame`, and `call_frame` are available which take a closure. These methods create and push a new `GcFrame` before calling the closure with a mutable reference to that new frame, when the frame is dropped it's popped from the stack. If possible, the raw frame backing this new frame is acquired from the current frame's remaining capacity. Schematically, contents of the stack page looks like

    [n0, p0, s00, s01, ..., s0m, n1, p1, s10, s11, ..., s1n, ...]
    
where `n<i>` is the two times the number of slots of the ith frame, `p<i>` is a pointer to `n<i-1>`, and `s<i,j>` is the jth slot of the ith frame. 

The return types of these closures are constrained in different ways depending on what function was used. The closure used with `frame` can return pretty much anything except a Julia value created in that closure. In order to return one of those, `value_frame` or `call_frame` can be used. In addition to the mutable reference to the frame, these methods provide the closure with an `Output`. The frame can be used to create temporary values, before calling function whose result should be returned from the closure the `Output` must be converted to an `OutputScope`. `OutputScope` and mutable references to frames both implement the `Scope` trait. Functions that create new Julia values take something that implements `Scope` by value. If this scope is a mutable reference to a frame the result is rooted in the current frame, while `OutputScope` leaves it unrooted until returning to the frame that the `Output` targets. After creating an unrooted value the frame can no longer be used, this prevents methods that can allocate from being called in order to guarantee the garbage collector won't run until the value has been rooted.


### Wrappers for builtin types

Wrappers for the builtin pointer types, like `jl_value_t*` and `jl_array_t*`, are all defined in the `value` module. The wrappers generally have the same name as the types do in Julia, so `jl_array_t*` is wrapped by `Array` and `jl_module_t*` by `Module`. There's one exception: `jl_value_t*` is wrapped by `Value` rather than `Any`. These wrapper types contain the appropriate pointer, and one or two `PhantomData` fields to give them that number of lifetimes. The wrapper is marked as `#[repr(transparent)]`, which ensures the layout and ABI of the struct is guaranteed to match that of the pointer. This means these wrapper types can be used as the arguments of Rust functions called from Julia with `ccall`.

The lifetimes serve two purposes. The first relates to the frame the value has been rooted in, it's normally named `'frame`. A `GcFrame` has a lifetime because it mutably borrows its raw frame from the stack page. This lifetime is inherited by the wrapper which ensures it can't be returned from the closure that provides access to the frame, i.e. the wrapper can only be used while it's rooted. The second is relevant when arrays are involved, it's possible to create a Julia array that is backed by data mutably borrowed from Rust. In this case, a second lifetime is needed to prevent the Julia value from being used after the borrow ends. Whenever a function is called that creates a new value and depends on one or more values, the second lifetime is inherited to ensure the lifetime of a borrow isn't accidentally extended.

Most functionality from the Julia C API is available through `Value`. Other wrapper types that are often used are `Array`, ` DataType`, `JuliaString`, `Module`, `Symbol` and the different `Tuple` types. Less often used are `SimpleVector`, `TypeName`, `TypeVar`, `Union` and `UnionAll`. The other builtin types are available for completeness sake, but using them should generally be avoided.


### Arrays

Julia arrays, `jl_array_t*` in the C API, are generic, multidimensional arrays that store their contents in column-major order, which is also known as Fortran order. The raw pointer is wrapped by `Array` and `TypedArray` in jlrs; the first does not refer to the type of its elements while the latter does. These types don't provide access to their contents directly from Rust, to do so a method like `Array::inline_data` must be called first. The methods that provide access to the array's contents require a `Frame` to ensure no mutable aliasing occurs. 

New arrays can be created from Rust by calling one of the following methods: `Value::new_array`, `Value::move_array`, or `Value::borrow_array`. The first lets Julia allocate the array, the second moves an array from Rust to Julia, and the last borrows its data from Rust. These methods are currently only compatible with types that implement `IntoJulia`. Accessing the data of arrays whose elements are bits-unions is also not supported.


### Converting data between languages and typechecks

In order to use Julia and Rust together, it's often necessary to (safely) convert data between both languages. Traits that deal with this are defined in the `convert` and `layout` modules, but their functionality is mostly used indirectly through methods of `Value`.

The most important of these traits are `IntoJulia` and `Cast`. The first of these is used in combination with `Value::new` and converts data from Rust to Julia. This trait is implemented for all primitive types and strings. The second is used in combination with `Value::cast(_unchecked)` and can convert a `Value` to a more useful type, in the case of casting to a wrapper for a builtin pointer type this amounts to a pointer cast. In most other cases it dereferences the pointer. Before actually casting the value, `Value::cast` checks if the value can be cast to its target. To do so, it calls `Value::is`, which performs a typecheck. Any type that implements `JuliaTypecheck` can be used as a typecheck, if a type also implements `Cast` the typecheck ensures that the value can be successfully cast to that type. Other kinds of checks are also available as typechecks, such as `Concrete` which checks if a `DataType` is concrete, and `Mutable` which checks if it's mutable. These typechecks are how the functionality of the `jl_is_<x>` macros from the C API is made available.

Additional methods are provided by `Value` to create values with types that are not compatible with `Value::new`, such as arrays and named tuples. With `Value::apply_type` arbitrary `DataType`s can be created; concrete types that are not arrays can be instantiated with `Value::instantiate` or `DataType::instantiate`. The methods expose the functions `jl_apply_type` and `jl_new_structv` from the C API.


### Custom types

Several other traits are available in the `layout` module, in particular, `ValidLayout` is used to determine if the layout of the type that implements the trait is compatible with the layout of a value in Julia with some `DataType`. If a struct derives the `JuliaStruct` trait, `ValidLayout`, `Cast`, and `JuliaTypecheck` are implemented for that struct. These custom types can be automatically generated with `JlrsReflect.jl`. `ValidLayout` is implemented by recursively calling itself for each field of the struct, where care is taken to handle bits union fields correctly. Unlike other fields, bits union fields require three separate fields to deal with their size and alignment requirements: unlike "normal" fields, the size of a bits union field is not necessarily a multiple of its alignment. This depends on the traits defined in the `bits_union` submodule of `layout`. `JuliaTypecheck` is implemented by calling `ValidLayout`.

If a type is guaranteed to be a bits-type, `IntoJulia` can also be derived. This happens automatically when `JlrsReflect.jl` is used.


### ccall

Julia's `ccall` interface is very powerful because it can call arbitrary functions with the C ABI, i.e. functions defined as `extern "C"` in Rust. It's not necessary to use jlrs to write functions in Rust that can be called from Julia with `ccall`, but 


### Async runtime

