use std::collections::HashMap;
use std::collections::hash_map::Entry;
use syntax::ast::{Expr, Ident, TokenTree};
use syntax::codemap::{BytePos, CodeMap};
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ToTokens;
use syntax::fold::{self, Folder};
use syntax::parse::{self, token};
use syntax::ptr::P;
use syntax::print::pprust;

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

fn register_pos_rec(map: &mut HashMap<BytePos, i32>,
                    codemap: &CodeMap,
                    tt: &TokenTree,
                    ptt: &TokenTree) {
    let line = codemap.span_to_lines(ptt.get_span()).unwrap();
    register_pos(map, tt.get_span().lo, line.lines[0].start_col.0 as i32);

    assert_eq!(tt.len(), ptt.len());
    for i in 0..tt.len() {
        register_pos_rec(map, codemap, &tt.get_tt(i), &ptt.get_tt(i));
    }
}

fn create_pos_map(cx: &mut ExtCtxt,
                  ppstr: String,
                  original_tokens: &[TokenTree])
                  -> HashMap<BytePos, i32> {
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
    ident: Ident,
    push_count: usize,
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
                    self.push_count += 1;
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
                    self.push_count += 1;
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
                    self.push_count += 1;
                    quote_expr!(self.cx, {
                        let expr = $conv_expr;
                        $ident.push(($col, format!("{:?}", expr)));
                        expr
                    })
                }
                // ExprUnary
                // ExprLit
                // ExprCast
                ExprIf(..) |
                ExprIfLet(..) |
                ExprMatch(..) => {
                    let col = self.pos_map.get(&expr.span.lo).unwrap();
                    let conv_expr = P(fold::noop_fold_expr(expr, self));
                    let ident = self.ident;
                    self.push_count += 1;
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
                ExprField(..) => {
                    self.convert_field(P(expr.clone()))
                }
                ExprTupField(..) => {
                    self.convert_field(P(expr.clone()))
                }
                ExprIndex(_, ref i) => {
                    // TODO: Fix column
                    let col = self.pos_map.get(&i.span.lo).unwrap();
                    let conv_expr = P(fold::noop_fold_expr(expr.clone(), self));
                    let ident = self.ident;
                    self.push_count += 1;
                    quote_expr!(self.cx, {
                        let expr = $conv_expr;
                        $ident.push(($col, format!("{:?}", expr)));
                        expr
                    })
                }
                // ExprRange
                ExprPath(..) => {
                    let col = self.pos_map.get(&expr.span.lo).unwrap();
                    let expr = P(expr);
                    let ident = self.ident;
                    self.push_count += 1;
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
                _ => P(fold::noop_fold_expr(expr, self)),
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
                ExprPath(..) => {
                    ident = cur_expr.clone();
                    let id = self.ident;
                    let col = self.pos_map.get(&expr.span.lo).unwrap();
                    self.push_count += 1;
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
        let exprs = exprs.into_iter()
                         .scan(ident, |st, item| {
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
                                 _ => panic!(),
                             };
                             *st = P(item);
                             Some((st.clone(), col))
                         })
                         .collect::<Vec<_>>();
        let last_expr = exprs[exprs.len() - 1].0.clone();
        let mut stmts = vec![prelude];
        for (e, col) in exprs {
            let ident = self.ident;
            self.push_count += 1;
            stmts.push(quote_stmt!(self.cx, $ident.push(($col, format!("{:?}", $e)));));
        }
        quote_expr!(self.cx, {
            $stmts;
            $last_expr
        })
    }
}

pub fn convert_expr(cx: &mut ExtCtxt,
                    expr: P<Expr>,
                    ident: Ident,
                    tts: &[TokenTree])
                    -> (P<Expr>, bool) {
    // Pretty-printed snippet is used for showing assersion failed message, but
    // its tokens may have different spans from original code snippet's tokens.
    // Power-assert using those spans, so, parse the pretty-printed snippet
    // again to get tokens that have the same spans with pretty-printed snippet.
    let ppexpr_str = pprust::expr_to_string(&expr);
    let pos_map = create_pos_map(cx, ppexpr_str, tts);

    // Convert
    let mut folder = AssertFolder {
        cx: cx,
        pos_map: &pos_map,
        ident: ident,
        push_count: 0,
    };
    (folder.fold_expr(expr), folder.push_count > 0)
}
