use power_assert::{power_assert, power_assert_eq};

#[derive(Debug, Eq, PartialEq)]
struct Foo {
    val: i32,
}

#[derive(Debug, Eq, PartialEq)]
struct Bar {
    val: i32,
    foo: Foo,
}

fn get_bar(val1: i32, val2: i32) -> Bar {
    Bar {
        val: val1,
        foo: Foo { val: val2 },
    }
}

fn main() {
    power_assert!(get_bar(3 + 5, -(3 + 1)) == get_bar(4 * 2, 3 * 3 / 3));
    power_assert!(get_bar(3 + 5, -(3 + 1)) == get_bar(4 * 2, 3 * 3 / 3),);
    power_assert!(
        get_bar(3 + 5, -(3 + 1)) == get_bar(4 * 2, 3 * 3 / 3),
        "message"
    );
    power_assert!(
        get_bar(3 + 5, -(3 + 1)) == get_bar(4 * 2, 3 * 3 / 3),
        "message with format {}",
        "foo"
    );
    power_assert_eq!(get_bar(3 + 5, -(3 + 1)), get_bar(4 * 2, 3 * 3 / 3));
    power_assert_eq!(get_bar(3 + 5, -(3 + 1)), get_bar(4 * 2, 3 * 3 / 3),);
}
