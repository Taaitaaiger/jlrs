#### v0.14
 - `TemporarySymbol` has been renamed to `ToSymbol`. The method `ToSymbol::to_symbol` has been added to this trait.

 - The wrappers for `CodeInstance`, `Expr`, `MethodInstance`, `MethodMatch`, `MethodTable`, `Method`, `OpaqueClosure`, `TypemapEntry`, `TypemapLevel` and `WeakRef` are now considered internal types, they are only available when the `internal-types` feature is enabled.

 - `Array::copy_inline_data` and `TypedArray::copy_inline_data` are now unsafe and require a reference to a `Frame`. `Array::dimensions`, `Array::inline_data`, `Array::inline_data_mut`, `Array::value_data`, `Array::value_data_mut`, `Array::wrapper_data`, and `Array::wrapper_data_mut` are now unsafe, the same holds true for `TypedArray`. The reason these methods are now unsafe is that it can't be guaranteed that no `Task` running in Julia is currently mutating this data.