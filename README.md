# power-assert-rs [![Build Status](https://travis-ci.org/gifnksm/power-assert-rs.svg)](https://travis-ci.org/gifnksm/power-assert-rs)
Power Assert in Rust. Provides better assertion message like this:

```
$ cargo run --example test -v
       Fresh power-assert v0.1.0 (file:///home/nksm/repos/gifnksm/power-assert-rs)
     Running `target/debug/examples/test`
power_assert!(bar.val == bar.foo.val)
              |   |   |  |   |   |
              |   3   |  |   |   2
              |       |  |   Foo { val: 2 }
              |       |  Bar { val: 3, foo: Foo { val: 2 } }
              |       false
              Bar { val: 3, foo: Foo { val: 2 } }
thread '<main>' panicked at 'assertion failed: bar.val == bar.foo.val', examples/test.rs:26
Process didn't exit successfully: `target/debug/examples/test` (exit code: 101)
```

# How to use

Add this to your `Cargo.toml`:

```toml
[dependencies]
power-assert = "*"
```

and add this to your `lib.rs` or `main.rs`:

```rust
#![feature(plugin)]
#![plugin(power_assert)]
```

Now, you can use `power_assert!()` and `power_assert_eq!()`.

If you want to override builtin `assert!()` and `assert_eq!()`, change your `lib.rs` or `main.rs` as follows.

```rust
#![feature(plugin)]
#![plugin(power_assert(override_builtins))]
```
