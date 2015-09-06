#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

#![feature(plugin)]
#![plugin(power_assert)]

#[derive(Debug)]
struct Foo {
    val: u32
}

#[derive(Debug)]
struct Bar {
    val: u32,
    foo: Foo
}

fn main() {
    let bar = Bar { val: 3, foo: Foo { val: 2 }};
    power_assert!(bar.val == bar.foo.val);
    power_assert_eq!(bar.val, bar.foo.val);
}
