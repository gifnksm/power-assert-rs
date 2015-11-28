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
extern crate rustc_plugin;

use syntax::ast::MetaWord;
use rustc_plugin::Registry;

mod expand;
mod convert;

struct Arg {
    override_builtins: bool,
}

impl Default for Arg {
    fn default() -> Arg {
        Arg { override_builtins: false }
    }
}

impl Arg {
    fn parse(reg: &mut Registry) -> Arg {
        let mut result = Arg::default();

        for arg in reg.args() {
            match arg.node {
                MetaWord(ref ns) => {
                    match &**ns {
                        "override_builtins" => {
                            result.override_builtins = true;
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

        result
    }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    let arg = Arg::parse(reg);

    if arg.override_builtins {
        reg.register_macro("assert", expand::expand_assert);
        reg.register_macro("assert_eq", expand::expand_assert_eq);
    }

    reg.register_macro("power_assert", expand::expand_assert);
    reg.register_macro("power_assert_eq", expand::expand_assert_eq);
}
