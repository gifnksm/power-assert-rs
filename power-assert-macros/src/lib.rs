use crate::fold::ExprFolder;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{fold::Fold as _, parse_macro_input, parse_quote, punctuated::Punctuated, Expr, Token};

mod fold;
mod parse;

struct PowerAssert {
    expr: Expr,
    message: Punctuated<Expr, Token![,]>,
}

#[proc_macro]
pub fn power_assert(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let PowerAssert { expr, message } = parse_macro_input!(input as PowerAssert);
    let expr_src = format!("{}", expr.to_token_stream());

    let panic_msg: Expr = if message.is_empty() {
        let message = format!("assertion failed: {}", expr_src);
        parse_quote! { #message }
    } else {
        parse_quote! { ::std::format_args!(#message) }
    };

    let formatter = Ident::new("__power_assert_formatter", Span::call_site());
    let mut folder = ExprFolder::new(formatter.clone());
    let expr = folder.fold_expr(syn::parse_str(&expr_src).unwrap());

    let output: Expr = parse_quote! {
        {
            let mut #formatter = ::power_assert::Formatter::new();
            let cond = #expr;
            if !cond {
                panic!("{}\n{}", #panic_msg, #formatter.format());
            }
        }
    };

    //println!("{}", output);
    output.to_token_stream().into()
}

struct PowerAssertEq {
    lhs: Expr,
    rhs: Expr,
    message: Punctuated<Expr, Token![,]>,
}

#[proc_macro]
pub fn power_assert_eq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let PowerAssertEq { lhs, rhs, message } = parse_macro_input!(input as PowerAssertEq);
    let lhs_src = format!("{}", lhs.to_token_stream());
    let rhs_src = format!("{}", rhs.to_token_stream());

    let panic_msg = if message.is_empty() {
        quote! {
            ::std::format_args!(
                "assertion failed: `(left == right)` (left: `{:?}`, right: `{:?}`)",
                *left_val, *right_val,
            )
        }
    } else {
        quote! { ::std::format_args!(#message) }
    };

    let formatter = Ident::new("__power_assert_formatter", Span::call_site());
    let mut folder = ExprFolder::new(formatter.clone());
    let lhs = folder.fold_expr(syn::parse_str(&lhs_src).unwrap());
    let rhs = folder.fold_expr(syn::parse_str(&rhs_src).unwrap());

    let output = quote! {
        {
            let mut #formatter = ::power_assert::Formatter::new();
            match (&(#lhs), &(#rhs)) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        panic!("{}\n{}", #panic_msg, #formatter.format());
                    }
                }
            }
        }
    };

    //println!("{}", output);
    output.into()
}
