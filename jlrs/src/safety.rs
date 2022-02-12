//! One thing you'll notice when using this crate is that initializing Julia, including custom 
//! Julia code, calling Julia functions, and accessing and mutating Julia data are all unsafe.
//! There are several reasons behind this, which is explained in more detail in the rest of this
//! document.
//! 
//! The first bit of unsafety you'll encounter is initializing Julia. This unsafety is mostly a
//! technicality: Julia can be initialized once, while jlrs prevents Julia from being initialized 
//! multiple times it is possible to use another crate, unrelated to jlrs, to initialize Julia 
//! from multiple threads simultaneously. 
//! 
//! There are two major ways to include custom code: a file containing Julia code can be included
//! and Julia code can be evaluated directly. The correctness of this code can't be checked and
//! arbitrary Julia functions can be called, so this is generally unsafe.
//! 
//! Calling Julia functions is unsafe for a very simple reason. There exist functions with 
//! illustrative names like `unsafe_load` which are obviously unsafe to call. However, there is no
//! `unsafe` keyword in Julia to prevent you from calling such functions as easily as a function 
//! like `+`.
//! 
//! There's nothing that prevents a function, or included or evaluated code, from scheduling a new
//! task. Like the lack of an `unsafe` keyword, there's no `Send` or `Sync` trait in Julia either.
//! If these tasks mutate shared data, it's your responsibility that this is free of data races. 
//! It's only safe to call a Julia function if it doesn't cause any data races.
//! 
//! If a task is running on another thread, the main thread is free to continue using the Julia C 
//! API. For example, let's say you call a function that takes a `Vector{UInt8}` and spawns a new 
//! task that mutates the vector's contents for some time before completing. While this task is 
//! running, control of the main thread returns to Rust. At this point, it's possible to access 
//! the vector's contents from Rust, which is unsound. As a result, it's unsafe to access the 
//! contents of a Julia array from Rust.
//! 
//! The same applies to other mutable data. It's not possible to safely access their contents 
//! unless you can guarantee it's not being mutated by any task. There is one exception, fields
//! that are constant are generally safe to access. Accessing the fields of immutable data is 
//! generally safe.
//!
//! As you might have noticed, all unsafety related to accessing data is essentially due to the
//! existence of tasks. 