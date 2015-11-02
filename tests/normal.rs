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

use std::thread;

#[derive(Debug)]
struct Foo {
    val: u32
}

#[derive(Debug)]
struct Bar {
    val: u32,
    foo: Foo
}

fn run_panic_test<F>(f: F) -> String
    where F: FnOnce(), F: Send + 'static
{
    thread::spawn(f).join().unwrap_err().downcast_ref::<String>().unwrap().clone()
}

#[test]
fn test_member_access() {
    assert_eq!(run_panic_test(|| {
        let bar = Bar { val: 3, foo: Foo { val: 2 }};
        power_assert!(bar.val == bar.foo.val);
    }), "assertion failed: bar.val == bar.foo.val
power_assert!(bar.val == bar.foo.val)
              |   |   |  |   |   |
              |   3   |  |   |   2
              |       |  |   Foo { val: 2 }
              |       |  Bar { val: 3, foo: Foo { val: 2 } }
              |       false
              Bar { val: 3, foo: Foo { val: 2 } }
");

    assert_eq!(run_panic_test(|| {
        let bar = Bar { val: 3, foo: Foo { val: 2 }};
        power_assert_eq!(bar.val, bar.foo.val);
    }), "assertion failed: `(left == right)` (left: `3`, right: `2`)
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
");
}

#[test]
fn issue4() {
    fn check(_: &str) -> Result<(), ()> {
        Ok(())
    }

    let s = "hello".to_owned();

    assert!(check(&s) == Ok(()));
    // power_assert!(check(&s) == Ok(())); // FIXME(#4)
}

#[test]
fn issue5() {
    assert_eq!(2*2, 4);
    power_assert_eq!(2*2, 4);
}
