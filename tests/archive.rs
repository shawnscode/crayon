extern crate crayon;

use crayon::resource::*;

#[test]
fn collection() {
    let mut collection = ArchiveCollection::new();
    assert!(!collection.exists("mock.prefab"));
    collection.register(FilesystemArchive::new("tests/resources").unwrap());
    assert!(collection.exists("mock.prefab"));

    let mut buf = vec![];
    let len = collection.read("mock.prefab", &mut buf).unwrap();
    assert_eq!(len, "mock".to_string().len());
    assert_eq!(buf, "mock".as_bytes());

    let mut sbuf = String::new();
    let len = collection.read_to_string("mock.prefab", &mut sbuf).unwrap();
    assert_eq!(len, "mock".to_string().len());
    assert_eq!(sbuf, "mock");
}

#[test]
fn filesystem() {
    assert!(FilesystemArchive::new("tests/_invalid_path_").is_err());

    let mut fs = FilesystemArchive::new("tests/resources").unwrap();
    assert!(fs.exists("mock.prefab"));

    let mut prefab = String::new();
    let len = fs.open("mock.prefab").unwrap().read_to_string(&mut prefab).unwrap();
    assert_eq!(len, "mock".to_string().len());
    assert_eq!(prefab, "mock");
}

#[test]
fn zip() {
    let mut fs = FilesystemArchive::new("tests/resources").unwrap();
    assert!(fs.exists("mock.zip"));

    let mut zip = ZipArchive::new(fs.open("mock.zip").unwrap()).unwrap();
    assert!(zip.exists("foo/mock.prefab"));

    let mut prefab = String::new();
    let len = zip.open("foo/mock.prefab").unwrap().read_to_string(&mut prefab).unwrap();
    assert_eq!(len, "mock".to_string().len());
    assert_eq!(prefab, "mock");
}