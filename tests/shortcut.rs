extern crate crayon;

use crayon::res::shortcut::ShortcutResolver;

#[test]
fn basic() {
    let mut resolver = ShortcutResolver::new();

    resolver.add("home:", "file://docs/").unwrap();
    resolver.add("a:", "home:").unwrap();
    resolver.add("b:", "a:crayon/").unwrap();

    assert!(resolver.has("home:"));
    assert!(resolver.has("a:"));
    assert!(resolver.has("b:"));
    assert!(!resolver.has("abc:"));

    assert_eq!(resolver.resolve("home:"), Some("file://docs/".into()));
    assert_eq!(resolver.resolve("b:"), Some("file://docs/crayon/".into()));

    resolver.add("home:", "http://www.rust-lang.org/").unwrap();
    assert_eq!(
        resolver.resolve("b:"),
        Some("http://www.rust-lang.org/crayon/".into())
    );
}

#[test]
fn err() {
    let mut resolver = ShortcutResolver::new();
    // Shortcut MUST ends with a colon (':').
    assert!(resolver.add("home", "file://docs/").is_err());
    // Shortcut MUST be at least 2 chars to not be confused with DOS drive letters.
    assert!(resolver.add(":", "file://docs/").is_err());
    // Fullname must end in a '/' (dir) or ':' (other shortcut).
    assert!(resolver.add("home:", "file://docs").is_err());
}
