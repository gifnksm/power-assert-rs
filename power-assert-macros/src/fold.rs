use proc_macro2::Ident;
use syn::{
    fold::{self, Fold},
    parse_quote,
    spanned::Spanned,
    Expr,
};

pub(crate) struct ExprFolder {
    formatter: Ident,
}

impl ExprFolder {
    pub(crate) fn new(formatter: Ident) -> Self {
        Self { formatter }
    }
}

impl Fold for ExprFolder {
    fn fold_expr(&mut self, i: syn::Expr) -> syn::Expr {
        let formatter = self.formatter.clone();
        let line = i.span().start().line;
        let column = i.span().start().column;
        match i {
            Expr::Call(i) => {
                let i = self.fold_expr_call(i);
                parse_quote! {{
                    let expr = #i;
                    #formatter.push(#line, #column, &expr);
                    expr
                }}
            }
            Expr::MethodCall(i) => {
                let i = self.fold_expr_method_call(i);
                parse_quote! {{
                    let expr = #i;
                    #formatter.push(#line, #column, &expr);
                    expr
                }}
            }
            Expr::Binary(i) => {
                let i = self.fold_expr_binary(i);
                parse_quote! {{
                    let expr = #i;
                    #formatter.push(#line, #column, &expr);
                    expr
                }}
            }
            _ => fold::fold_expr(self, i),
        }
    }
}
