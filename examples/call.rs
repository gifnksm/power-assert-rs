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

#[derive(Debug, Eq, PartialEq)]
struct Foo {
    val: i32
}

#[derive(Debug, Eq, PartialEq)]
struct Bar {
    val: i32,
    foo: Foo
}

fn get_bar(val1: i32, val2: i32) -> Bar {
    Bar { val: val1, foo: Foo { val: val2 }}
}

fn main() {
    power_assert!(get_bar(3 + 5, -(3 + 1)) == get_bar(4 * 2, 3 * 3 / 3));
    power_assert_eq!(get_bar(3 + 5, -(3 + 1)), get_bar(4 * 2, 3 * 3 / 3));
}
