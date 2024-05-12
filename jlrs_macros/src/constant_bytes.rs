use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, Type};

enum Pair<T> {
    Two(T, T),
    One(T),
}

struct Pairs<I>(I);

impl<I, T> Iterator for Pairs<I>
where
    I: Iterator<Item = T>,
{
    type Item = Pair<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.0.next()?;
        match self.0.next() {
            Some(second) => Some(Pair::Two(first, second)),
            None => Some(Pair::One(first)),
        }
    }
}

fn wrap_bytes(pair: Pair<u8>) -> Type {
    match pair {
        Pair::Two(a, b) => parse_quote! {
            ::jlrs::data::types::construct_type::ConstantBytes<
                ::jlrs::data::types::construct_type::ConstantU8<#a>,
                ::jlrs::data::types::construct_type::ConstantU8<#b>,
            >
        },
        Pair::One(a) => parse_quote! {
            ::jlrs::data::types::construct_type::ConstantU8<#a>
        },
    }
}

fn wrap_tokens(pair: Pair<&Type>) -> Type {
    match pair {
        Pair::Two(a, b) => parse_quote! {
            ::jlrs::data::types::construct_type::ConstantBytes<
                #a,
                #b,
            >
        },
        Pair::One(a) => a.clone(),
    }
}

pub(crate) fn convert_to_constant_bytes(s: String) -> TokenStream {
    let bytes = s.as_bytes();
    let n_bytes = bytes.len();
    if n_bytes == 0 {
        panic!("Must be at least 1 byte long");
    }

    let mut types = Pairs(bytes.into_iter().copied())
        .map(wrap_bytes)
        .collect::<Vec<_>>();
    let mut buffer = Vec::with_capacity(types.len());

    while types.len() != 1 {
        for ty in Pairs(types.iter()).map(wrap_tokens) {
            buffer.push(ty);
        }
        std::mem::swap(&mut types, &mut buffer);
        buffer.clear();
    }

    let ty = &types[0];
    quote! {
        #ty
    }
    .into()
}
