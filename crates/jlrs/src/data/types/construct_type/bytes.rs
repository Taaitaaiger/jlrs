//! Encode bytes and strings as types.

use std::{marker::PhantomData, string::FromUtf8Error};

use crate::{
    convert::to_symbol::ToSymbol,
    data::{
        managed::symbol::Symbol,
        types::construct_type::{constants::ConstantU8, type_var::TypeVarName},
    },
    memory::target::Target,
};

/// Encode bytes into `ConstantBytes`.
///
/// See [`ConstantBytes`] for more information.
#[macro_export]
macro_rules! bytes {
    ($t1:literal, $R:literal) => {
        $crate::data::types::construct_type::ConstantBytes<
            $crate::data::types::construct_type::ConstantU8<$t1>,
            $crate::data::types::construct_type::ConstantU8<$R>
        >
    };
    ($t1:literal, $R:literal, $($rest:literal),+) => {
        $crate::data::types::construct_type::ConstantBytes<
            $crate::data::types::construct_type::ConstantU8<$t1>,
            $crate::bytes!($R, $($rest),+)
        >
    };
}

/// Trait implemented by types that encode bytes.
pub trait EncodedBytes: 'static {
    type B: AsRef<[u8]>;
    fn encoded_bytes() -> Self::B;
}

/// Trait implemented by types that encode a string.
pub trait EncodedString: 'static {
    type S: AsRef<str>;
    fn encoded_string() -> Self::S;
}

/// Constant string. Not a type constructor, but can be used to construct `TypeVar`s with names
/// longer than one character.
pub trait ConstantStr: 'static {
    /// The string constant.
    const STR: &'static str;
}

impl<S: ConstantStr> EncodedString for S {
    type S = &'static str;
    fn encoded_string() -> Self::S {
        Self::STR
    }
}

/// Constant byte slice.
pub trait ConstantByteSlice: 'static {
    /// The byte slice constant.
    const BYTES: &'static [u8];
}

impl<S: ConstantByteSlice> EncodedBytes for S {
    type B = &'static [u8];
    fn encoded_bytes() -> Self::B {
        Self::BYTES
    }
}

/// Trait implemented by `ConstantU8` and `ConstantBytes` to build a list of constant bytes.
pub trait ConstantBytesFragment: 'static {
    /// The size of this fragment.
    const SIZE: usize;

    #[doc(hidden)]
    fn extend(slice: &mut [u8], offset: usize);
}

/// Constant bytes.
///
/// `ConstantBytes` implements `TypeVarName`. In general you should prefer to implement
/// [`ConstantStr`] over using `ConstantBytes`.
///
/// While it isn't possible to use static string slice as a const generic parameter, it is
/// possible to recursively encode its bytes into a type. For example, the string "Foo" can be
/// represented as follows:
///
/// `type Foo = ConstantBytes<ConstantU8<70>, ConstantBytes<ConstantU8<111>, ConstantU8<111>>>`.
///
/// The [`bytes`] macro is less verbose, but only accepts `u8`'s:
///
/// `type Foo = bytes!(70, 111, 111)`
///
/// The [`encode_as_constant_bytes`] macro converts a string literal to `ConstantBytes`:
///
/// `type Foo = encode_as_constant_bytes!("Foo")`.
///
/// The main advantage of `encode_as_constant_bytes` is that it doesn't represent the string as a
/// list like the `bytes` macro does, but represents it as a tree with the minimal depth:
///
/// `type Foo = ConstantBytes<ConstantBytes<ConstantU8<70>, ConstantU8<111>>, ConstantU8<111>>`
///
/// [`encode_as_constant_bytes`]: jlrs_macros::encode_as_constant_bytes
pub struct ConstantBytes<L: ConstantBytesFragment, R: ConstantBytesFragment>(
    PhantomData<L>,
    PhantomData<R>,
);

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> ConstantBytes<L, R> {
    /// Convert the encoded bytes to `Vec<u8>`.
    pub fn into_vec() -> Vec<u8> {
        let mut v = vec![0; Self::SIZE];
        Self::extend(v.as_mut_slice(), 0);
        v
    }

    /// Try to convert the encoded bytes into a string.
    pub fn into_string() -> Result<String, FromUtf8Error> {
        let v = Self::into_vec();
        String::from_utf8(v)
    }
}

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> EncodedBytes for ConstantBytes<L, R> {
    type B = Vec<u8>;
    fn encoded_bytes() -> Vec<u8> {
        Self::into_vec()
    }
}

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> EncodedString for ConstantBytes<L, R> {
    type S = String;
    fn encoded_string() -> String {
        Self::into_string().expect("Invalid string")
    }
}

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> TypeVarName for ConstantBytes<L, R> {
    fn symbol<'target, Tgt: Target<'target>>(target: &Tgt) -> Symbol<'target> {
        Self::into_string()
            .expect("Invalid string")
            .to_symbol(target)
    }
}

impl<const N: u8> ConstantBytesFragment for ConstantU8<N> {
    const SIZE: usize = 1;

    #[inline]
    fn extend(slice: &mut [u8], offset: usize) {
        slice[offset] = N;
    }
}

impl<L: ConstantBytesFragment, R: ConstantBytesFragment> ConstantBytesFragment
    for ConstantBytes<L, R>
{
    const SIZE: usize = L::SIZE + R::SIZE;

    #[inline]
    fn extend(slice: &mut [u8], offset: usize) {
        L::extend(slice, offset);
        R::extend(slice, offset + L::SIZE);
    }
}
