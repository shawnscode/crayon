extern crate crayon;

use crayon::res::url::Url;

#[test]
fn basic() {
    let url = Url::new("https://www.rust-lang.org/en-US/index.html").unwrap();
    assert_eq!(url.schema(), "https");
    assert_eq!(url.username(), None);
    assert_eq!(url.password(), None);
    assert_eq!(url.host(), "www.rust-lang.org");
    assert_eq!(url.port(), None);
    assert_eq!(url.path(), "/en-US/index.html");
    assert_eq!(url.fragment(), None);
    assert_eq!(url.queries(), None);

    let url = Url::new("https://shawn@www.rust-lang.org/en-US/index.html").unwrap();
    assert_eq!(url.schema(), "https");
    assert_eq!(url.username(), Some("shawn"));
    assert_eq!(url.password(), None);
    assert_eq!(url.host(), "www.rust-lang.org");
    assert_eq!(url.port(), None);
    assert_eq!(url.path(), "/en-US/index.html");
    assert_eq!(url.fragment(), None);
    assert_eq!(url.queries(), None);

    let url = Url::new("https://shawn:123456@www.rust-lang.org/en-US/index.html").unwrap();
    assert_eq!(url.schema(), "https");
    assert_eq!(url.username(), Some("shawn"));
    assert_eq!(url.password(), Some("123456"));
    assert_eq!(url.host(), "www.rust-lang.org");
    assert_eq!(url.port(), None);
    assert_eq!(url.path(), "/en-US/index.html");
    assert_eq!(url.fragment(), None);
    assert_eq!(url.queries(), None);
}

#[test]
fn basic_2() {
    let url = Url::new("https://shawn:123456@www.rust-lang.org:8080/en-US/index.html").unwrap();
    assert_eq!(url.schema(), "https");
    assert_eq!(url.username(), Some("shawn"));
    assert_eq!(url.password(), Some("123456"));
    assert_eq!(url.host(), "www.rust-lang.org");
    assert_eq!(url.port(), Some("8080"));
    assert_eq!(url.path(), "/en-US/index.html");
    assert_eq!(url.fragment(), None);
    assert_eq!(url.queries(), None);

    let url = Url::new("https://shawn:123456@www.rust-lang.org:8080/en-US/index.html#abc").unwrap();
    assert_eq!(url.schema(), "https");
    assert_eq!(url.username(), Some("shawn"));
    assert_eq!(url.password(), Some("123456"));
    assert_eq!(url.host(), "www.rust-lang.org");
    assert_eq!(url.port(), Some("8080"));
    assert_eq!(url.path(), "/en-US/index.html");
    assert_eq!(url.fragment(), Some("abc"));
    assert_eq!(url.queries(), None);

    let url = Url::new("file:///en-US/index.html#abc").unwrap();
    assert_eq!(url.schema(), "file");
    assert_eq!(url.username(), None);
    assert_eq!(url.password(), None);
    assert_eq!(url.host(), "");
    assert_eq!(url.port(), None);
    assert_eq!(url.path(), "/en-US/index.html");
    assert_eq!(url.fragment(), Some("abc"));
    assert_eq!(url.queries(), None);
}

#[test]
fn queries() {
    let url = Url::new("https://www.rust-lang.org/index.html?key0=value0&key1=value1").unwrap();
    let queries = url.queries().unwrap();
    let mut iter = queries.iter();
    assert_eq!(iter.next(), Some((&"key0".into(), &Some("value0".into()))));
    assert_eq!(iter.next(), Some((&"key1".into(), &Some("value1".into()))));
    assert_eq!(iter.next(), None);

    let url = Url::new("https://www.rust-lang.org/index.html?key0=value0&key1=value1#abc").unwrap();
    let queries = url.queries().unwrap();
    let mut iter = queries.iter();
    assert_eq!(iter.next(), Some((&"key0".into(), &Some("value0".into()))));
    assert_eq!(iter.next(), Some((&"key1".into(), &Some("value1".into()))));
    assert_eq!(iter.next(), None);
    assert_eq!(url.fragment(), Some("abc"));

    let url = Url::new("https://www.rust-lang.org/index.html?key0=value0&key1#abc").unwrap();
    let queries = url.queries().unwrap();
    let mut iter = queries.iter();
    assert_eq!(iter.next(), Some((&"key0".into(), &Some("value0".into()))));
    assert_eq!(iter.next(), Some((&"key1".into(), &None)));
    assert_eq!(iter.next(), None);
    assert_eq!(url.fragment(), Some("abc"));
}

#[test]
fn err() {
    // URL must have a schema.
    assert!(Url::new("www.rust-lang.org/index.html").is_err());
    assert!(Url::new(":www.rust-lang.org/index.html").is_err());
    // URL must have a hostname.
    assert!(Url::new("http://index.html").is_err());
    assert!(Url::new("file://index.html").is_err());
    assert!(Url::new("file:///index.html").is_ok());
    // URL must have a filename.
    assert!(Url::new("http://www.rust-lang.org").is_err());
}
