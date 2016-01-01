use convert;

use syntax::ast::{Expr, Ident, TokenTree, Stmt};
use syntax::codemap::Span;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager};
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::token;
use syntax::ptr::P;
use syntax::print::pprust;

// Copy from libsyntax
//
// A variant of 'try!' that panics on Err(FatalError). This is used as a
// crutch on the way towards a non-panic!-prone parser. It should be used
// for fatal parsing errors; eventually we plan to convert all code using
// panictry to just use normal try
macro_rules! panictry {
    ($e:expr) => ({
        use std::result::Result::{Ok, Err};
        use syntax::errors::FatalError;
        match $e {
            Ok(e) => e,
            Err(mut e) => {
                e.emit();
                panic!(FatalError);
            }
        }
    })
}

fn filter_tts_by_span(span: Span, tts: &[TokenTree]) -> Vec<TokenTree> {
    tts.iter()
       .filter(|tt| span.lo <= tt.get_span().lo && tt.get_span().hi <= span.hi)
       .cloned()
       .collect()
}

fn expr_prelude(cx: &ExtCtxt) -> Vec<P<Stmt>> {
    let use_stmt = quote_stmt!(cx, use std::io::Write;);
    let fn_stmt = quote_stmt!(
        cx,
        fn inspect<T: Write>(writer: &mut T, mut vals: Vec<(i32, String)>, offset: i32, repr: &str) {
            fn width(s: &str) -> i32 { s.len() as i32 }
            fn align<T: Write>(writer: &mut T, cur: &mut i32, col: i32, s: &str) {
                while *cur < col {
                    let _ = write!(writer, " ");
                    *cur += 1;
                }
                let _ = write!(writer, "{}", s);
                *cur += width(s);
            }

            vals.sort();

            let _ = writeln!(writer, "{}", repr);
            {
                let mut cur = 0;
                for &(c, _) in &vals {
                    align(writer, &mut cur, offset + c, "|");
                }
                let _ = writeln!(writer, "");
            }
            while !vals.is_empty() {
                let mut cur = 0;
                let mut i = 0;
                while i < vals.len() {
                    if i == vals.len() - 1 ||
                        vals[i].0 + width(&vals[i].1) < vals[i + 1].0 {
                            align(writer, &mut cur, offset + vals[i].0, &vals[i].1);
                            let _ = vals.remove(i);
                        } else {
                            align(writer, &mut cur, offset + vals[i].0, "|");
                            i += 1;
                        }
                }
                let _ = writeln!(writer, "");
            }
        });
    let write_stmt = quote_stmt!(cx, let mut write_buf = vec![];);

    vec![use_stmt.unwrap(), fn_stmt.unwrap(), write_stmt.unwrap()]
}

struct ExprGen {
    ident: Ident,
    tts: Vec<TokenTree>,
    expr: P<Expr>,
    ppstr: String,
}

impl ExprGen {
    fn new(ident: &str, expr: P<Expr>, tts: &[TokenTree]) -> ExprGen {
        let tts = filter_tts_by_span(expr.span, tts);
        let ppstr = pprust::expr_to_string(&expr);

        ExprGen {
            ident: token::gensym_ident(ident),
            tts: tts,
            expr: expr,
            ppstr: ppstr,
        }
    }

    fn ppstr(&self) -> &str {
        &self.ppstr
    }

    fn init_stmt(&self, cx: &mut ExtCtxt, pushed: bool) -> P<Stmt> {
        let ident = self.ident;
        if pushed {
            quote_stmt!(cx, let mut $ident = vec![];).unwrap()
        } else {
            quote_stmt!(cx, let $ident = vec![];).unwrap()
        }
    }

    fn converted_expr(&self, cx: &mut ExtCtxt) -> (P<Expr>, bool) {
        convert::convert_expr(cx, self.expr.clone(), self.ident, &self.tts)
    }

    fn inspect_expr(&self, cx: &mut ExtCtxt, left: &str, right: &str) -> P<Expr> {
        let offset = left.len() as i32;
        let repr = format!("{}{}{}", left, self.ppstr, right);
        let ident = self.ident;
        quote_expr!(cx, inspect(&mut write_buf, $ident, $offset, $repr))
    }
}


pub fn expand_assert(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    let cond_expr = panictry!(parser.parse_expr());

    let msg_tts = if panictry!(parser.eat(&token::Token::Comma)) {
        let mut span = sp.clone();
        span.lo = parser.span.lo;
        Some(filter_tts_by_span(span, args))
    } else {
        None
    };

    let prelude = expr_prelude(cx);

    let gen = ExprGen::new("vals", cond_expr, args);
    let inspect = gen.inspect_expr(cx, "power_assert!(", ")");
    let (converted, pushed) = gen.converted_expr(cx);
    let init = gen.init_stmt(cx, pushed);

    let panic_msg = if let Some(tts) = msg_tts {
        quote_expr!(cx, format!($tts))
    } else {
        let msg = format!("assertion failed: {}", gen.ppstr());
        quote_expr!(cx, $msg)
    };

    let expr = quote_expr!(cx, {
        $init;
        let cond = $converted;
        if !cond {
            $prelude;
            let _ = writeln!(&mut write_buf, $panic_msg);
            $inspect;
            panic!(String::from_utf8(write_buf).unwrap())
        }
    });

    // println!("{:?}", expr);

    MacEager::expr(expr)
}

pub fn expand_assert_eq(cx: &mut ExtCtxt,
                        _sp: Span,
                        args: &[TokenTree])
                        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    let lhs = panictry!(parser.parse_expr());
    panictry!(parser.expect(&token::Token::Comma));
    let rhs = panictry!(parser.parse_expr());

    let lhs_gen = ExprGen::new("lhs_vals", lhs, args);
    let rhs_gen = ExprGen::new("rhs_vals", rhs, args);

    let prelude = expr_prelude(cx);

    let lhs_inspect = lhs_gen.inspect_expr(cx, "left: ", "");
    let rhs_inspect = rhs_gen.inspect_expr(cx, "right: ", "");
    let (lhs_converted, lhs_pushed) = lhs_gen.converted_expr(cx);
    let (rhs_converted, rhs_pushed) = rhs_gen.converted_expr(cx);
    let lhs_init = lhs_gen.init_stmt(cx, lhs_pushed);
    let rhs_init = rhs_gen.init_stmt(cx, rhs_pushed);
    let assert_msg = format!("power_assert_eq!({}, {})", lhs_gen.ppstr(), rhs_gen.ppstr());

    let expr = quote_expr!(cx, {
        $lhs_init;
        $rhs_init;
        match (&$lhs_converted, &$rhs_converted) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    $prelude;
                    let _ = writeln!(&mut write_buf, "assertion failed: `(left == right)` \
                            (left: `{:?}`, right: `{:?}`)", *left_val, *right_val);
                    let _ = writeln!(&mut write_buf, $assert_msg);
                    $lhs_inspect;
                    $rhs_inspect;
                    panic!(String::from_utf8(write_buf).unwrap());
                }
            }
        }
    });

    // println!("{:?}", expr);

    MacEager::expr(expr)
}
