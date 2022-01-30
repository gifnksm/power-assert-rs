use power_assert::power_assert_eq;

#[test]
fn issue5() {
    assert_eq!(2 * 2, 4);
    power_assert_eq!(2 * 2, 4);
}
