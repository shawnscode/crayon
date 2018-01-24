extern crate crayon;

use crayon::resource::filesystem::*;

#[test]
fn driver() {
    let mut driver = FilesystemDriver::new();

    {
        assert!(!driver.exists("/res/foo/mock.prefab"));
        assert!(!driver.exists("/res//foo/mock.prefab"));
        assert!(!driver.exists("/res/./foo/mock.prefab"));

        let fs = ZipFS::new("tests/assets/mock.zip").unwrap();
        driver.mount("res", fs).unwrap();

        // canonicalized
        assert!(driver.exists("/res/foo/mock.prefab"));
        assert!(driver.exists("/res//foo/mock.prefab"));
        assert!(driver.exists("/res/./foo/mock.prefab"));

        driver.unmount("res");
        assert!(!driver.exists("/res/foo/mock.prefab"));
    }

    let fs = DirectoryFS::new("tests/assets").unwrap();
    driver.mount("res", fs).unwrap();

    let mut buf = Vec::new();
    driver.load_into("/res/mock.txt", &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf.to_owned()).unwrap(), "Hello, World!");
}

#[test]
fn dir() {
    assert!(DirectoryFS::new("tests/_invalid_path_").is_err());

    let fs = DirectoryFS::new("tests/assets").unwrap();
    assert!(fs.exists("mock.zip".as_ref()));
    assert!(fs.exists("mock.txt".as_ref()));

    let mut buf = Vec::new();
    fs.load_into("mock.txt".as_ref(), &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "Hello, World!");
}

#[test]
fn zip() {
    assert!(ZipFS::new("tests/_invalid_path_").is_err());

    let fs = ZipFS::new("tests/assets/mock.zip").unwrap();
    assert!(fs.exists("foo/mock.prefab".as_ref()));

    let mut buf = Vec::new();
    fs.load_into("foo/mock.prefab".as_ref(), &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "mock");
}
