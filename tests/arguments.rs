extern crate lemon3d;

use lemon3d::core::arguments::*;

#[test]
fn arguments() {
    let arguments = Arguments::new("tests/resources/arguments.json").unwrap();
    assert_eq!(arguments.load_as_i32("i32").unwrap(), 32);
    assert_eq!(arguments.load_as_str("str").unwrap(), "mock");
    assert_eq!(arguments.load_as_i32("nested/inner").unwrap(), 32);
}