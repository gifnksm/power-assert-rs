#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(quote)]

extern crate syntax;
extern crate rustc;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use syntax::ast::{Expr, Ident, TokenTree, Stmt, MetaWord};
use syntax::codemap::{BytePos, CodeMap, Span};
use syntax::ext::base::{ExtCtxt, MacResult, MacEager};
use syntax::ext::quote::rt::ToTokens;
use syntax::fold::{self, Folder};
use syntax::parse::{self, token};
use syntax::ptr::P;
use syntax::print::pprust;
use rustc::plugin::Registry;

// Copy from libsyntax
//
// A variant of 'try!' that panics on Err(FatalError). This is used as a
// crutch on the way towards a non-panic!-prone parser. It should be used
// for fatal parsing errors; eventually we plan to convert all code using
// panictry to just use normal try
macro_rules! panictry {
    ($e:expr) => ({
        use std::result::Result::{Ok, Err};
        use syntax::diagnostic::FatalError;
        match $e {
            Ok(e) => e,
            Err(FatalError) => panic!(FatalError)
        }
    })
}

fn register_pos(map: &mut HashMap<BytePos, i32>, org_pos: BytePos, pp_pos: i32) {
    match map.entry(org_pos) {
        Entry::Occupied(e) => {
            assert_eq!(e.get(), &pp_pos);
        }
        Entry::Vacant(e) => {
            let _ = e.insert(pp_pos);
        }
    }
}

fn register_pos_rec(map: &mut HashMap<BytePos, i32>, codemap: &CodeMap, tt: &TokenTree, ptt: &TokenTree) {
    let line = codemap.span_to_lines(ptt.get_span()).unwrap();
    register_pos(map, tt.get_span().lo, line.lines[0].start_col.0 as i32);
    register_pos(map, tt.get_span().hi, line.lines[0].end_col.0 as i32);
    assert_eq!(tt.len(), ptt.len());
    for i in 0..tt.len() {
        register_pos_rec(map, codemap, &tt.get_tt(i), &ptt.get_tt(i));
    }
}

fn create_pos_map(
    cx: &mut ExtCtxt, ppstr: String, original_tokens: &[TokenTree])
    -> HashMap<BytePos, i32>
{
    let codemap = cx.parse_sess().codemap();
    let filemap = codemap.new_filemap("".to_string(), ppstr);
    let ptokens = parse::filemap_to_tts(cx.parse_sess(), filemap);

    let mut map = HashMap::new();
    for (tt, ptt) in original_tokens.iter().zip(ptokens) {
        register_pos_rec(&mut map, &codemap, &tt, &ptt);
    }
    map
}

struct AssertFolder<'cx> {
    cx: &'cx ExtCtxt<'cx>,
    pos_map: &'cx HashMap<BytePos, i32>,
    ident: Ident
}

impl<'cx> Folder for AssertFolder<'cx> {
    fn fold_expr(&mut self, expr: P<Expr>) -> P<Expr> {
        expr.and_then(|expr| {
            use syntax::ast::*;
            match expr.node {
                // ExprBox
                // ExprVec
                ExprCall(ref i, ref args) => {
                    let col = self.pos_map.get(&i.span.lo).unwrap();
                    let args = args.iter()
                        .map(|expr| self.fold_expr(expr.clone()))
                        .collect::<Vec<_>>();
                    let ident = self.ident;
                    let mut call_expr = expr.clone();
                    call_expr.node = ExprCall(i.clone(), args);

                    let call_expr = P(call_expr);
                    quote_expr!(self.cx, {
                        let expr = $call_expr;
                        $ident.push(($col, format!("{:?}", expr)));
                        expr
                    })
                }
                ExprMethodCall(i, _, _) => {
                    // TODO: fix column
                    let col = self.pos_map.get(&i.span.lo).unwrap();
                    let conv_expr = P(fold::noop_fold_expr(expr, self));
                    let ident = self.ident;
                    quote_expr!(self.cx, {
                        let expr = $conv_expr;
                        $ident.push(($col, format!("{:?}", expr)));
                        expr
                    })
                }
                // ExprTup
                ExprBinary(op, _, _) => {
                    let col = self.pos_map.get(&op.span.lo).unwrap();
                    let conv_expr = P(fold::noop_fold_expr(expr, self));
                    let ident = self.ident;
                    quote_expr!(self.cx, {
                        let expr = $conv_expr;
                        $ident.push(($col, format!("{:?}", expr)));
                        expr
                    })
                }
                // ExprUnary
                // ExprLit
                // ExprCast
                ExprIf(_, _, _) |
                ExprIfLet(_, _, _, _) |
                ExprMatch(_, _, _) => {
                    let col = self.pos_map.get(&expr.span.lo).unwrap();
                    let conv_expr = P(fold::noop_fold_expr(expr, self));
                    let ident = self.ident;
                    quote_expr!(self.cx, {
                        let expr = $conv_expr;
                        $ident.push(($col, format!("{:?}", expr)));
                        expr
                    })
                }
                // ExprWhile
                // ExprWhileLet
                // ExprForLoop
                // ExprLoop
                // ExprMatch: See above
                // ExprClosure
                // ExprBlock
                // ExprAssign
                // ExprAssignOp
                ExprField(_, _) => {
                    self.convert_field(P(expr.clone()))
                }
                ExprTupField(_, _) => {
                    self.convert_field(P(expr.clone()))
                }
                ExprIndex(_, ref i) => {
                    // TODO: Fix column
                    let col = self.pos_map.get(&i.span.lo).unwrap();
                    let conv_expr = P(fold::noop_fold_expr(expr.clone(), self));
                    let ident = self.ident;
                    quote_expr!(self.cx, {
                        let expr = $conv_expr;
                        $ident.push(($col, format!("{:?}", expr)));
                        expr
                    })
                }
                // ExprRange
                ExprPath(_, _) => {
                    let col = self.pos_map.get(&expr.span.lo).unwrap();
                    let expr = P(expr);
                    let ident = self.ident;
                    quote_expr!(self.cx, {
                        $ident.push(($col, format!("{:?}", $expr)));
                        $expr
                    })
                }
                // ExprAddrOf
                // ExprBreak
                // ExprAgain
                // ExprRet
                // ExprInlineAsm
                // ExprMac
                // ExprStruct
                // ExprRepeat
                // ExprParen
                _ => P(fold::noop_fold_expr(expr, self))
            }
        })
    }
}

impl<'cx> AssertFolder<'cx> {
    fn convert_field(&mut self, expr: P<Expr>) -> P<Expr> {
        use syntax::ast::*;
        let mut exprs = vec![];
        let mut cur_expr = expr.clone();
        let prelude;
        let ident;
        loop {
            cur_expr = match cur_expr.node {
                ExprField(ref e, _) | ExprTupField(ref e, _) => {
                    exprs.push(cur_expr.clone());
                    e.clone()
                }
                ExprPath(_, _) => {
                    ident = cur_expr.clone();
                    let id = self.ident;
                    let col = self.pos_map.get(&expr.span.lo).unwrap();
                    prelude = quote_stmt!(self.cx, $id.push(($col, format!("{:?}", $ident))));
                    break;
                }
                _ => {
                    let id = token::gensym_ident("a");
                    ident = quote_expr!(self.cx, $id);
                    let conv_expr = P(fold::noop_fold_expr((*cur_expr).clone(), self));
                    prelude = quote_stmt!(self.cx, let $id = $conv_expr;);
                    break;
                }
            }
        }
        exprs.reverse();
        let exprs = exprs.into_iter().scan(ident, |st, item| {
            let mut item = (*item).clone();
            let col = match item.node {
                ExprField(_, i) => {
                    item.node = ExprField(st.clone(), i);
                    self.pos_map.get(&i.span.lo).unwrap()
                }
                ExprTupField(_, i) => {
                    item.node = ExprTupField(st.clone(), i);
                    self.pos_map.get(&i.span.lo).unwrap()
                }
                _ => panic!()
            };
            *st = P(item);
            Some((st.clone(), col))
        }).collect::<Vec<_>>();
        let last_expr = exprs[exprs.len() - 1].0.clone();
        let mut stmts = vec![prelude];
        for (e, col) in exprs {
            let ident = self.ident;
            stmts.push(quote_stmt!(self.cx, $ident.push(($col, format!("{:?}", $e)));));
        }
        quote_expr!(self.cx, {
            $stmts;
            $last_expr
        })
    }
}

fn expr_prelude(cx: &ExtCtxt) -> Vec<P<Stmt>> {
    vec![quote_stmt!(cx, use std::io::{self, Write};).unwrap(),
         quote_stmt!(cx, fn width(s: &str) -> i32 { s.len() as i32 }).unwrap(),
         quote_stmt!(cx, fn align<T: Write>(writer: &mut T, cur: &mut i32, col: i32, s: &str) {
             while *cur < col {
                 let _ = write!(writer, " ");
                 *cur += 1;
             }
             let _ = write!(writer, "{}", s);
             *cur += width(s);
         }).unwrap(),
         quote_stmt!(cx, fn inspect(mut vals: Vec<(i32, String)>, offset: i32, repr: &str) {
             vals.sort();

             let mut err = io::stderr();
             let _ = writeln!(err, "{}", repr);
             {
                 let mut cur = 0;
                 for &(c, _) in &vals {
                     align(&mut err, &mut cur, offset + c, "|");
                 }
                 let _ = writeln!(err, "");
             }
             while !vals.is_empty() {
                 let mut cur = 0;
                 let mut i = 0;
                 while i < vals.len() {
                     if i == vals.len() - 1 ||
                         vals[i].0 + width(&vals[i].1) < vals[i + 1].0 {
                             align(&mut err, &mut cur, offset + vals[i].0, &vals[i].1);
                             let _ = vals.remove(i);
                         } else {
                             align(&mut err, &mut cur, offset + vals[i].0, "|");
                             i += 1;
                         }
                 }
                 let _ = writeln!(err, "");
             }
         }).unwrap()]
}

fn expr_inspect(cx: &ExtCtxt, vals: Ident, left: &str, repr: &str, right: &str)
                -> P<Expr>
{
    let offset = left.len() as i32;
    let repr = format!("{}{}{}", left, repr, right);
    quote_expr!(cx, inspect($vals, $offset, $repr))
}

fn expr_convert(cx: &mut ExtCtxt, expr: P<Expr>, ident: Ident, tts: &[TokenTree]) -> P<Expr> {
    // Pretty-printed snippet is used for showing assersion failed message, but
    // its tokens may have different spans from original code snippet's tokens.
    // Power-assert using those spans, so, parse the pretty-printed snippet
    // again to get tokens that have the same spans with pretty-printed snippet.
    let ppexpr_str = pprust::expr_to_string(&expr);
    let pos_map = create_pos_map(cx, ppexpr_str, tts);

    // Convert
    let mut folder = AssertFolder { cx: cx, pos_map: &pos_map, ident: ident };
    folder.fold_expr(expr)
}

fn expand_assert(cx: &mut ExtCtxt, _sp: Span, args: &[TokenTree])
                 -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);

    let cond_expr = parser.parse_expr();
    let msg_tts = if panictry!(parser.eat(&token::Token::Comma)) {
        Some(args[parser.tokens_consumed..].to_vec())
    } else {
        None
    };
    let ppexpr_str = pprust::expr_to_string(&cond_expr);

    let prelude = expr_prelude(cx);
    let ident = token::gensym_ident("vals");
    let converted_expr = expr_convert(cx, cond_expr, ident, args);
    let inspect = expr_inspect(cx, ident, "power_assert!(", &ppexpr_str, ")");
    let panic_expr = if let Some(tts) = msg_tts {
        quote_expr!(cx, panic!($tts))
    } else {
        quote_expr!(cx, panic!(concat!("assertion failed: ", $ppexpr_str)))
    };

    let expr = quote_expr!(cx, {
        let mut $ident = vec![];
        let cond = $converted_expr;
        if !cond {
            $prelude;
            $inspect;
            $panic_expr;
        }
    });

    // println!("{:?}", expr);

    MacEager::expr(expr)
}

fn expand_assert_eq(cx: &mut ExtCtxt, _sp: Span, args: &[TokenTree])
                    -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    let lhs = parser.parse_expr();
    let lhs_tts = &args[..parser.tokens_consumed];
    panictry!(parser.expect(&token::Token::Comma));

    let idx = parser.tokens_consumed;
    let rhs = parser.parse_expr();
    let rhs_tts = &args[idx..];

    let pplhs_str = pprust::expr_to_string(&lhs);
    let pprhs_str = pprust::expr_to_string(&rhs);

    let prelude = expr_prelude(cx);
    let lhs_ident = token::gensym_ident("lhs_vals");
    let converted_lhs = expr_convert(cx, lhs, lhs_ident, lhs_tts);
    let rhs_ident = token::gensym_ident("rhs_vals");
    let converted_rhs = expr_convert(cx, rhs, rhs_ident, rhs_tts);
    let lhs_inspect = expr_inspect(cx, lhs_ident, "left: ", &pplhs_str, "");
    let rhs_inspect = expr_inspect(cx, rhs_ident, "right: ", &pprhs_str, "");

    let expr = quote_expr!(cx, {
        let mut $lhs_ident = vec![];
        let mut $rhs_ident = vec![];
        match (&$converted_lhs, &$converted_rhs) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    $prelude;
                    let _ = writeln!(io::stderr(), "power_assert_eq!({}, {})", $pplhs_str, $pprhs_str);
                    $lhs_inspect;
                    $rhs_inspect;
                    panic!("assertion failed: `(left == right)` \
                            (left: `{:?}`, right: `{:?}`)", *left_val, *right_val);
                }
            }
        }
    });

    // println!("{:?}", expr);

    MacEager::expr(expr)
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    let mut override_builtins = false;

    for arg in reg.args() {
        match arg.node {
            MetaWord(ref ns) => {
                match &**ns {
                    "override_builtins" => {
                        override_builtins = true;
                    }
                    _ => {
                        reg.sess.span_err(arg.span, "invalid argument");
                    }
                }
            }
            _ => {
                reg.sess.span_err(arg.span, "invalid argument");
            }
        }
    }

    if override_builtins {
        reg.register_macro("assert", expand_assert);
        reg.register_macro("assert_eq", expand_assert_eq);
    }

    reg.register_macro("power_assert", expand_assert);
    reg.register_macro("power_assert_eq", expand_assert_eq);
}
