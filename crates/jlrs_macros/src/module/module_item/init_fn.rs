use syn::{
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

pub struct InitFn {
    pub _become_token: Token![become],
    pub init_fn: Ident,
}

impl Parse for InitFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let init_fn_token = input.parse()?;
        let init_fn = input.parse()?;

        Ok(InitFn {
            _become_token: init_fn_token,
            init_fn,
        })
    }
}
