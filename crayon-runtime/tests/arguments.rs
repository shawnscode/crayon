extern crate crayon;

use crayon::core::arguments::*;

#[test]
fn arguments() {
    let arguments = Arguments::new("tests/resources/arguments.json").unwrap();
    assert_eq!(arguments.load_as_i32("i32").unwrap(), 32);
    assert_eq!(arguments.load_as_str("str").unwrap(), "mock");
    assert_eq!(arguments.load_as_i32("nested/inner").unwrap(), 32);

    let slice = arguments.load_as_slice("nested").unwrap();
    assert_eq!(slice.load_as_i32("inner").unwrap(), 32);
}