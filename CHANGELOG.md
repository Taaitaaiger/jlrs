#### v0.14
 - `TemporarySymbol` has been renamed to `ToSymbol`. The method `ToSymbol::to_symbol` has been added to this trait.

 - The wrappers for `CodeInstance`, `Expr`, `MethodInstance`, `MethodMatch`, `MethodTable`, `Method`, `OpaqueClosure`, `SSAValue`, `TypemapEntry`, `TypemapLevel` and `WeakRef` are considered internal types, they are only available when the `internal-types` feature is enabled.

 - `Array::copy_inline_data` and `TypedArray::copy_inline_data` require a reference to a `Frame`.

 - `CopiedArray::splat` returns a boxed slice instead of a `Vec`.

 - `IntoJulia::into_julia` is now a safe method.

 - `Align`, `BitsUnionContainer`, and `Flag` are now sealed traits.

 - All methods of the `Gc` trait are now safe.

 - Mutating Julia data is considered unsafe, as a result `Module::set_global` and related methods are now unsafe. So are all methods that provide mutable access to array data. 