extern crate crayon;

#[test]
fn basic() {
    let mut shortcuts = crayon::res::shortcut::Shortcut::new();

    shortcuts.add("home:", "file://docs/").unwrap();
    shortcuts.add("a:", "home:").unwrap();
    shortcuts.add("b:", "a:crayon/").unwrap();

    assert!(shortcuts.has("home:"));
    assert!(shortcuts.has("a:"));
    assert!(shortcuts.has("b:"));
    assert!(!shortcuts.has("abc:"));

    assert_eq!(shortcuts.resolve("home:"), Some("file://docs/".into()));
    assert_eq!(shortcuts.resolve("b:"), Some("file://docs/crayon/".into()));

    shortcuts.add("home:", "http://www.rust-lang.org/").unwrap();
    assert_eq!(
        shortcuts.resolve("b:"),
        Some("http://www.rust-lang.org/crayon/".into())
    );
}

#[test]
fn err() {
    let mut shortcuts = crayon::res::shortcut::Shortcut::new();
    // Shortcut MUST ends with a colon (':').
    assert!(shortcuts.add("home", "file://docs/").is_err());
    // Shortcut MUST be at least 2 chars to not be confused with DOS drive letters.
    assert!(shortcuts.add(":", "file://docs/").is_err());
    // Fullname must end in a '/' (dir) or ':' (other shortcut).
    assert!(shortcuts.add("home:", "file://docs").is_err());
}
