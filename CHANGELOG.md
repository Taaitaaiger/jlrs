#### v0.14
 - `TemporarySymbol` has been renamed to `ToSymbol`. The method `ToSymbol::to_symbol` has been added to this trait.

 - The wrappers for `CodeInstance`, `Expr`, `MethodInstance`, `MethodMatch`, `MethodTable`, `Method`, `OpaqueClosure`, `SSAValue`, `TypemapEntry`, `TypemapLevel` and `WeakRef` are considered internal types, they are only available when the `internal-types` feature is enabled.

 - `Array::copy_inline_data` and `TypedArray::copy_inline_data` require a reference to a `Frame`.

 - `CopiedArray::splat` returns a boxed slice instead of a `Vec`.

 - `IntoJulia::into_julia` is a safe method.

 - `Align`, `BitsUnionContainer`, and `Flag` are sealed traits.

 - All methods of the `Gc` trait are safe.

 - Mutating Julia data is considered unsafe, as a result `Module::set_global` and related methods are unsafe. So are all methods that provide mutable access to array data.

 - Unchecked methods are unsafe because not catching Julia exceptions is unsound when calling Rust from Julia.

 - `Array::as_typed_array` has been renamed to `Array::try_as_typed`.

 - Outputs reserve a slot in a frame and immediately set this slot when they're used. Multiple outputs can exists for the same frame simultaneously.

 - Methods like `Scope::value_scope` and `AsyncFrame::async_value_scope` have been removed because `Frame::scope` and `AsyncFrame::scope` can return rooted data that outlives the frame.

 - The `Scope` trait no longer has a `'data` lifetime and most of its methods have been moved to the `Frame` trait. The `ScopeExt` trait has been removed completely, `ScopeExt::scope_with_slots` has been renamed to `Frame::scope_with_capacity`.

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

 - The different runtimes have been moved to the `runtime` crate. Both the sync and async runtime can no longer be initialized directly, but require using a `RuntimeBuilder` or `AsyncRuntimeBuilder`. To create an async runtime an implementation of `AsyncRuntime` and a backing channel that implements `Channel` must be provided.

 - All methods that send a new task to the async runtime take a `OneshotSender` and return an error rather than panicking if they fail.

 - Like the async runtime, creating new `PersistentTask`s requires providing a backing channel.

 - `Array::reshape` is available for arrays that have data originating from Rust. Unchecked versions of `Array::reshape`, `Array::grow_begin`, `Array::grow_end`, `Array::del_begin`, and `Array::del_end` are available.

 - `Array::from_vec` and `Array::from_slice` can return an error, unchecked versions are also available.

 - A `TypedArray` can be created directly, and generally offers the same API as `Array` does.

 - Methods that take multiple values as `AsMut` now only need `AsRef` because the data is never mutated from C.

 - `SimpleVector` no longer has a type to indicate the type of its contents because this property can't be checked with `ValidLayout` or `Typecheck`. Instead, a type can now be provided when accessing the contents which is checked for compatibility at runtime.

 - The `ValidLayout` trait has an associated constant `IS_REF` to indicate whether the implementor is an inline or pointer wrapper type.
