use power_assert::{power_assert, power_assert_eq};

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

    power_assert!(bar.val == bar.foo.val);
    power_assert!(bar.val == bar.foo.val,);
    power_assert!(bar.val == bar.foo.val, "message");
    power_assert!(bar.val == bar.foo.val, "message",);
    power_assert!(bar.val == bar.foo.val, "message with format {}", "foo");
    power_assert!(bar.val == bar.foo.val, "message with format {}", "foo",);

    power_assert_eq!(bar.val, bar.foo.val);
    power_assert_eq!(bar.val, bar.foo.val,);
    power_assert_eq!(bar.val, bar.foo.val, "message");
    power_assert_eq!(bar.val, bar.foo.val, "message",);
    power_assert_eq!(bar.val, bar.foo.val, "message with format {}", "foo");
    power_assert_eq!(bar.val, bar.foo.val, "message with format {}", "foo",);
}
