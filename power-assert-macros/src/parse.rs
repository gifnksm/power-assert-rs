use crate::{PowerAssert, PowerAssertEq};
use syn::{
    parse::{Parse, ParseStream},
    Expr, Token,
};

impl Parse for PowerAssert {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr = input.parse()?;
        let message = if input.parse::<Option<Token![,]>>()?.is_some() {
            input.parse_terminated(Expr::parse)?
        } else {
            Default::default()
        };
        Ok(Self { expr, message })
    }
}

impl Parse for PowerAssertEq {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lhs = input.parse()?;
        input.parse::<Token!(,)>()?;
        let rhs = input.parse()?;
        let message = if input.parse::<Option<Token![,]>>()?.is_some() {
            input.parse_terminated(Expr::parse)?
        } else {
            Default::default()
        };
        Ok(Self { lhs, rhs, message })
    }
}
