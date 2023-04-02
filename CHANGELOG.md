#### v0.18

 - jlrs is compatible with Julia 1.7 again, but this version isn't actively tested or supported. Version features have been added to select a particular version of Julia, picking a specific version is required.

 - The `wrappers` module has been renamed to `data`. Pointer wrappers are now called managed data, and inline wrappers are layouts. Traits and methods dealing with wrappers have been renamed accordingly.

 - The traits in the `layout` module have been moved to the `data` module. Most can be found in `data::layout`, `typecheck` has been moved to `data::types`, and `field_index::FieldIndex` has been moved to `data::managed::value::field_accessor`.

 - Tasks must now declare their affinity with an associated `Affinity` type, the affinity can be either `DispatchAny`, `DispatchMain`, or `DispatchWorker`. The `try_` methods have been removed from `AsyncJulia`, the non-try methods are now sync and return a `Dispatch` which is used to actually dispatch the task to the async runtime. Which dispatching methods are available for the returned instance of `Dispatch` depends on the `Affinity` of the task. There is both an async and a sync variant for each dispatching method, the async variant waits until the task can be dispatched, the sync variant returns the dispatcher if the targeted queue is full to allow retrying to dispatch the task.

 - The `Compatible` trait has been added which allows a type that implements `ValidLayout` to declare that it has exactly the same layout as another type. The `CompatibleCast` trait lets references to and arrays of types that implement `ValidLayout` to be cast to that compatible type without copying the data.

 - `ForeignType` requires `Send + Sync`, whether a `ForeignType` is large or contains pointers to managed data is now expressed with the associated constant `LARGE`, the super type is returned by the trait method `ForeignType::super_type`. Foreign types can be reinitialized with `reinit_foreign_type`, which is available since Julia 1.9.

 - `OpaqueType` has been added, which is similar to `ForeignType` except that it can't contain any references to Julia data.

 - A managed type for `Binding` has been added, this is available since Julia 1.10.

 - The `Jlrs` Julia module is no longer included with jlrs, but published as a separate package.

 - The jlrs-derive crate is now part of jlrs-macros. Two additional macros are available: `julia_version` can be used as an alternative to writing big cfgs to handle differences between Julia versions, and `julia_module` to generate the data that `Jlrs.Wrap` uses to make the exported functions, methods, types, constant and globals available in a Julia module.

 - Several traits have been added to implement the `julia_module` macro: `CCallArg`, `CCallReturn`, and `ConstructType`, the first two provide a mapping from a Rust type to the types Julia should use in generated Julia functions, the latter is used to construct the `DataType` at runtime. The managed type `TypedValue` has been added to make more types constructible, and `RustResult` to enable writing fallible `ccall`able functions. Similarly, `RankedArray` and `TypedRankedArray` have been added. A `CCallRef` type has been added to work with Julia `Ref`s that have been used as arguments of `ccall`ed functions.

 - The `no-link` feature has been added to skip linking Julia.

 - `Dims::n_dimensions` has been renamed to `Dims::rank`.

 - `RankedArray` and `TypedRankedArray` have been added to the `array` module.

 - `Symbol::generate` and `Symbol::generate_tagged` have been added.

 - `LeakedValue` has been removed in favor of making managed data generally leakable by erasing the `'scope` lifetime with `Managed::leak` or `Ref::leak`.

 - `CCall` now has `invoke` methods to avoid having to manually create a stack frame.

 - Global Julia data can be cached in a static container by using `StaticGlobal` and `StaticSymbol`.

 - `TrackArray` has been removed, the trait methods are now implemented directly by `(Typed)Array`. The tracking methods have been renamed to `track_shared` and `track_exclusive`.

 - `DataType` methods that depend on the type having a layout return `None` if the type has no layout.

 - Methods that can catch a thrown exception are now available on Windows in combination with Julia 1.6.


#### v0.17

 - Atomic struct fields are now atomic in the generated bindings.

 - `Value` implements `PartialEq` for all wrapper types, allowing a value to be compared with any other wrapper type directly.

 - `JuliaString::to_bytes` has been renamed to `JuliaString::as_bytes`, and returns all data.

 - If a `JuliaString` is unboxed and contains non-utf8 data, all data is returned as a `Vec<u8>` rather than stopping at the first null character.

 - The GC stack is now implemented as a foreign type and can be resized. Methods that could previously only fail due to allocation errors are now infallible. `AllocError` has been removed.

 - When the sync runtime or `CCall` is used, a reference to a `StackFrame` must be provided.

 - `Frame`, `Mode`, `Sync` and `Async` have been removed.

 - `AsyncGcFrame` implements `Deref<Target = GcFrame>` and `DerefMut`. Several methods that previously took a mutable reference to a frame now take a mutable reference to a `GcFrame` specifically.

 - Some fields of `Task` and `TypeName` can only be accessed if the `extra-fields` feature is enabled.

 - Methods that return Julia take a `Target`, `ExceptionTarget`, or one of their extended variants. Both rooting and non-rooting targets exists, specific methods that returned unrooted data have been removed because methods that take a target can return rooted or unrooted data depending on the used target. `Scope` and `PartialScope` have been removed completely.

 - Mutable references to `Output` implement `Target`, if used as a `Target` the returned data is rooted until the borrow ends.

 - Methods of the `Gc` trait take `self` by reference.

 - A ledger is used to track borrowed Julia data, instances of `Array`s and `Value`s can be tracked.

 - `CCall::null_scope` and `NullFrame` have been replaced with `CCall::stackless_scope`.

 - `Ref::leak` and `Ref::data_ptr` have been added.

 - `async_util::task::sleep` has been added.

 - When calling Julia functions, it can now be checked that none of the arguments are borrowed from Rust.

 - It's no longer possible to provide a backing channel for an async runtime.

 - A `nightly` feature is available to test the latest nightly Julia features.

 - When the `nightly` feature is enabled, the async runtime can be started with additional worker threads.

 - When the `nightly` feature is enabled, tasks are scheduled on one of the two available thread pools depending on the method.

 - The `ForeignType` trait has been added which can be used to create new foreign types with custom mark functions.

 - `AsyncJulia::post_blocking_task` has been added, which can be used to schedule a blocking task on an arbitrary thread owned by Julia.

 - `PersistentTask::State` is a GAT, which gets rid of the lifetime-hack that allows the state to contain Julia data.

 - `Ref` wraps a `NonNull` pointer to Julia data, like wrappers do. Undefinedness is expressed through `Option`.

 - Uses of `ValidLayout` have been mostly replaced with `ValidField`. `ValidField` is not implemented for mutable structs because they're not stored inline when used as a field of some other type.

 - `CCall::throw_exception` has been added and can be used to throw an exception from a `ccall`ed function.

 - Add `Affinity` so tasks can be explicitly scheduled on the main runtime thread.

 - `Global` has been renamed to `Unrooted`.


#### v0.16

 - Support for Julia 1.7 has been dropped, by default Julia 1.8 is targeted.


#### v0.15

 - jlrs can be used with 32-bits versions of Julia on Linux by enabling the `i686` feature.

 - Methods that can catch exceptions thrown by Julia, eg `Module::set_const`, return a `JlrsResult<JuliaResult<T>>`.

 - The `Global` provided to `PersistentTask::run` now has the `'static` lifetime.

 - The methods `AsyncFrame::relaxed_async_scope_(with_capacity)` have been added to work around the limitation that `AsyncFrame::relaxed_async_scope` can't return return data that lives shorter than the frame that created it.

 - Elided lifetimes have been added to methods that create arrays with data borrowed from Rust, eg `Array::from_slice`. Such arrays can now be returned from async scopes when `AsyncFrame::relaxed_async_scope_(with_capacity)` is used.

 - The number of threads can be set with `AsyncRuntimeBuilder::n_threads` when the `lts` feature is enabled.


#### v0.14

 - `TemporarySymbol` has been renamed to `ToSymbol`. The method `ToSymbol::to_symbol` has been added to this trait.

 - The wrappers for `CodeInstance`, `Expr`, `MethodInstance`, `MethodMatch`, `MethodTable`, `Method`, `OpaqueClosure`, `SSAValue`, `TypemapEntry`, `TypemapLevel` and `WeakRef` are considered internal types, they are only available when the `internal-types` feature is enabled.

 - `Array::copy_inline_data` and `TypedArray::copy_inline_data` require a reference to a `Frame`.

 - `CopiedArray::splat` returns a boxed slice instead of a `Vec`.

 - `Align`, `BitsUnionContainer`, and `Flag` are sealed traits.

 - All methods of the `Gc` trait are safe.

 - Mutating Julia data is considered unsafe, as a result `Module::set_global` and related methods are unsafe. So are all methods that provide mutable access to array data. Unchecked methods are unsafe because not catching Julia exceptions is unsound when calling Rust from Julia.

 - `Array::as_typed_array` has been renamed to `Array::try_as_typed`.

 - Outputs reserve a slot in a frame and immediately set this slot when they're used. Multiple outputs can exists for the same frame simultaneously.

 - Methods like `Scope::value_scope` and `AsyncFrame::async_value_scope` have been removed because `Frame::scope` and `AsyncFrame::async_scope` can return rooted data that outlives the frame.

 - The `Scope` trait no longer has a `'data` lifetime and most of its methods have been moved to the `Frame` trait. The `ScopeExt` trait has been removed completely, `ScopeExt::scope_with_slots` has been renamed to `Frame::scope_with_capacity`. The `PartialScope` trait has been added which allows calling methods that only need to root a single value with an `Output`.

 - Methods that return rooted data return the appropriate type, eg `Array::new` returns an `Array`.

 - All pointer wrapper types provide a `root` method that can be used to safely extend their lifetime using an `Output`.

 - The `AsUnrooted` trait has been removed.

 - Most of the extensions defined in the extensions module have moved: `jlrs::extensions::f16` to `jlrs::wrappers::inline::f16`. `jlrs::extensions::ndarray` to `jlrs::convert::ndarray`, `jlrs::extensions::multitask` to `jlrs::multitask`, and `jlrs::extensions::pyplot` to `jlrs::pyplot`.

 - Pointer wrapper types don't implement `ValidLayout`, only `Ref` and inline wrappers do.

 - Raw fields can be accessed with a `FieldAccessor`, the `raw_field` methods have been removed.

 - Add a `FieldIndex` trait which is used in combination with a `FieldAccessor` to access arbitrary fields.

 - A wrapper for `Nothing`/`nothing` has been added.

 - `Dims::index_of` takes `dim_index` by reference.

 - `Array::element_size` and `TypedArray::element_size` have been added.

 - `DataType::field_type_unchecked` and `DataType::field_index` have been added.

 - `GC::enable_logging` has been added.

 - The `call` module has been moved from `jlrs::wrappers::ptr::call` to `jlrs::call`.

 - The `prelude` module can be disabled by opting out of the default features.

 - The different runtimes have been moved to the `runtime` crate. Both the sync and async runtime can no longer be initialized directly, but require using a `RuntimeBuilder` or `AsyncRuntimeBuilder`. To create an async runtime an implementation of `AsyncRuntime` and a backing channel that implements `Channel` must be provided. `AsyncRuntimeBuilder::start_async` is a sync function.

 - All methods that send a new task to the async runtime take a `OneshotSender` and return an error rather than panicking if they fail.

 - Like the async runtime, creating new `PersistentTask`s requires providing a backing channel.

 - `Array::reshape` is available for arrays that have data originating from Rust. Unchecked versions of `Array::reshape`, `Array::grow_begin`, `Array::grow_end`, `Array::del_begin`, and `Array::del_end` are available.

 - `Array::from_vec` and `Array::from_slice` can return an error, unchecked versions are also available.

 - A `TypedArray` can be created directly, and generally offers the same API as `Array` does.

 - Methods that take multiple values as `AsMut` now only need `AsRef` because the data is never mutated from C.

 - `SimpleVector` no longer has a type to indicate the type of its contents because this property can't be checked with `ValidLayout` or `Typecheck`. Instead, a type can now be provided when accessing the contents which is checked for compatibility at runtime.

 - The `ValidLayout` trait has an associated constant `IS_REF` to indicate whether the implementor is an inline or pointer wrapper type.

 - Add `all-features-override` feature that disables the `lts` and `debug` features when `all-features` is `true`.

 - The contents `JlrsPyPlot.jl` are no longer evaluated automatically when the `pyplot` feature has been enabled, `PyPlot::init` must be called.

 - The `Dims` trait and `Frame` types are no longer included in the prelude.

 - There's a single type that provides possibly mutable access to the contents of an array, `ArrayAccessor`, which replaces the large number of types that previously provided this access. A distinction is made between element types that are stored inline which have no and those that have some pointer fields.

 - `NdArray` has been split into `NdArrayView` and `NdArrayViewMut`. Rather than a typed array, their trait methods take a (mutable) reference to an `ArrayAccessor` or `CopiedArray` instead.

 - `JlrsError` has been split into several error types and unused variants have been removed.

 - `JuliaString::as_slice` has been renamed to `JuliaString::to_bytes`, `Symbol::as_slice` has been renamed to `Symbol::as_bytes`.

 - `Symbol::new_bytes_unchecked` has been added.

 - Closures and async trait methods that are called in a scope take a frame by value instead of by mutable reference. This gets rid of the need to reborrow the frame.

 - `CallExt` has been renamed to `ProvideKeywords`, `CallExt::with_keywords` to `ProvideKeywords::provide_keywords`.

 - Type aliases for `Ref` have been moved to the module that also defines their associated pointer wrapper.
