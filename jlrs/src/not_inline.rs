use crate::traits::JuliaType;
use std::marker::PhantomData;
use crate::value::Value;

pub struct NotInline<'frame, 'data, T: JuliaType>(
    Value<'frame, 'data>,
    PhantomData<T>,
);

/*
#[derive(JuliaBitsUnion)]
#[jlrs(union_of(u8, u16))]
struct MyBitsUnion;
*/
