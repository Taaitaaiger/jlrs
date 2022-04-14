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

 - All pointer wrapper types now have a `root` method that can be used to safely extend their lifetime using an `Output`.

 - The `AsUnrooted` trait has been removed.
 
 - Most of the extensions defined in the extensions module have moved: `jlrs::extensions::f16` to `jlrs::wrappers::inline::f16`. `jlrs::extensions::ndarray` to `jlrs::ndarray`, `jlrs::extensions::multitask` to `jlrs::multitask`, and `jlrs::extensions::pyplot` to `jlrs::pyplot`.

 - Pointer wrapper types don't implement `ValidLayout`, only `Ref` and inline wrappers do.
  