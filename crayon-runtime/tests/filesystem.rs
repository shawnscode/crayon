extern crate crayon;

use crayon::prelude::*;

#[test]
fn driver() {
    let mut driver = FilesystemDriver::new();

    {
        assert!(!driver.exists("resources", "foo/mock.prefab"));

        let fs = filesystem::ZipFS::new("tests/support/resources/mock.zip").unwrap();
        driver.mount("resources", fs).unwrap();
        assert!(driver.exists("resources", "foo/mock.prefab"));

        driver.unmount("resources");
        assert!(!driver.exists("resources", "foo/mock.prefab"));
    }

    let fs = filesystem::DirectoryFS::new("tests/support/resources").unwrap();
    driver.mount("resources", fs).unwrap();

    let buf = driver
        .load("resources", "mock.txt")
        .unwrap()
        .wait()
        .unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), "Hello, World!");
}

#[test]
fn dir() {
    assert!(filesystem::DirectoryFS::new("tests/_invalid_path_").is_err());

    let fs = filesystem::DirectoryFS::new("tests/support/resources").unwrap();
    assert!(fs.exists("mock.zip".as_ref()));
    assert!(fs.exists("mock.txt".as_ref()));

    let mut buf = Vec::new();
    fs.load_into("mock.txt".as_ref(), &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "Hello, World!");
}

#[test]
fn zip() {
    assert!(filesystem::ZipFS::new("tests/_invalid_path_").is_err());

    let fs = filesystem::ZipFS::new("tests/support/resources/mock.zip").unwrap();
    assert!(fs.exists("foo/mock.prefab".as_ref()));

    let mut buf = Vec::new();
    fs.load_into("foo/mock.prefab".as_ref(), &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "mock");
}