use power_assert::power_assert;

#[test]
fn issue4() {
    fn check(_: &str) -> Result<(), ()> {
        Ok(())
    }

    let s = "hello".to_owned();

    assert!(check(&s) == Ok(()));
    power_assert!(check(&s) == Ok(()));
}
