use power_assert::{power_assert, power_assert_eq};
use std::panic::UnwindSafe;

#[derive(Debug)]
struct Foo {
    val: u32,
}

#[derive(Debug)]
struct Bar {
    val: u32,
    foo: Foo,
}

fn run_panic_test<F>(f: F) -> String
where
    F: FnOnce() + UnwindSafe,
    F: Send + 'static,
{
    std::panic::catch_unwind(f)
        .unwrap_err()
        .downcast_ref::<String>()
        .unwrap()
        .clone()
}

#[test]
fn test_member_access() {
    assert_eq!(
        run_panic_test(|| {
            let bar = Bar {
                val: 3,
                foo: Foo { val: 2 },
            };
            power_assert!(bar.val == bar.foo.val);
        }),
        "\
assertion failed: bar.val == bar.foo.val
power_assert!(bar.val == bar.foo.val)
              |   |   |  |   |   |
              |   3   |  |   |   2
              |       |  |   Foo { val: 2 }
              |       |  Bar { val: 3, foo: Foo { val: 2 } }
              |       false
              Bar { val: 3, foo: Foo { val: 2 } }
"
    );

    assert_eq!(
        run_panic_test(|| {
            let bar = Bar {
                val: 3,
                foo: Foo { val: 2 },
            };
            power_assert_eq!(bar.val, bar.foo.val);
        }),
        "\
assertion failed: `(left == right)` (left: `3`, right: `2`)
power_assert_eq!(bar.val, bar.foo.val)
left: bar.val
      |   |
      |   3
      Bar { val: 3, foo: Foo { val: 2 } }
right: bar.foo.val
       |   |   |
       |   |   2
       |   Foo { val: 2 }
       Bar { val: 3, foo: Foo { val: 2 } }
"
    );
}
