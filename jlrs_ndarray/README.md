Borrow data from Julia arrays as `ndarray`'s `ArrayView` and `ArrayViewMut`.

This crate defines a single trait, `NdArray`, that provides methods that return an immutable
or a mutable view of the array data and is implemented by `Array` and `TypedArray` from jlrs.

Example:

```rust
use jlrs::prelude::*;
use jlrs_ndarray::NdArray;

fn main() {
    let mut julia = unsafe { Julia::init().unwrap() };
    julia.dynamic_frame(|_global, frame| {
        let mut data = vec![1usize, 2, 3, 4, 5, 6];
        let slice = &mut data.as_mut_slice();
        let borrowed = Value::borrow_array(frame, slice, (3, 2))?;

        let _array = borrowed.cast::<TypedArray<usize>>()?.array_view(frame)?;

        Ok(())
    }).unwrap();
}
```
