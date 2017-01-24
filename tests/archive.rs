extern crate lemon3d;

use lemon3d::resource::*;

#[test]
fn register() {
    let mut collection = ArchiveCollection::new();
    assert!(!collection.exists("mock.prefab"));
    collection.register(FilesystemArchive::new("tests/resources"));
    assert!(collection.exists("mock.prefab"));
}

#[test]
fn filesystem() {
    let fs = FilesystemArchive::new("tests/resources");
    assert!(fs.exists("mock.prefab"));

    let mut prefab = String::new();
    let len = fs.open("mock.prefab").unwrap().read_to_string(&mut prefab).unwrap();
    assert_eq!(len, "mock".to_string().len());
    assert_eq!(prefab, "mock");
}