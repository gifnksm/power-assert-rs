use power_assert::{power_assert as assert, power_assert_eq as assert_eq};

#[derive(Debug)]
struct Foo {
    val: u32,
}

#[derive(Debug)]
struct Bar {
    val: u32,
    foo: Foo,
}

fn main() {
    let bar = Bar {
        val: 3,
        foo: Foo { val: 2 },
    };
    assert!(bar.val == bar.foo.val);
    assert_eq!(bar.val, bar.foo.val);
}
