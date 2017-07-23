extern crate crayon;

use crayon::resource::*;
use std::fs;

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

    let fs = FilesystemArchive::new("tests/resources").unwrap();
    assert!(fs.exists("mock.prefab".as_ref()));

    let mut prefab = String::new();
    let len = fs.open("mock.prefab".as_ref())
        .unwrap()
        .read_to_string(&mut prefab)
        .unwrap();
    assert_eq!(len, "mock".to_string().len());
    assert_eq!(prefab, "mock");
}

#[test]
fn zip() {
    let zip_file = fs::File::open("tests/resources/mock.zip").unwrap();
    let zip = ZipArchive::new(zip_file).unwrap();
    assert!(zip.exists("foo/mock.prefab".as_ref()));

    let mut prefab = String::new();
    let len = zip.open("foo/mock.prefab".as_ref())
        .unwrap()
        .read_to_string(&mut prefab)
        .unwrap();
    assert_eq!(len, "mock".to_string().len());
    assert_eq!(prefab, "mock");
}

struct Text {
    pub value: String,
}

impl resource::Resource for Text {
    fn size(&self) -> usize {
        self.value.len()
    }
}

#[derive(Debug)]
struct TextLoader {}

impl resource::ResourceLoader for TextLoader {
    type Item = Text;

    fn create(file: &mut archive::File) -> errors::Result<Self::Item> {
        let mut prefab = String::new();
        file.read_to_string(&mut prefab)?;
        Ok(Text { value: prefab })
    }
}

#[test]
fn clean() {
    let mut collection = ArchiveCollection::new();
    collection.register(FilesystemArchive::new("tests/resources").unwrap());

    let mut rs = ResourceSystem::new();

    {
        let t1 = rs.load::<TextLoader, &str>(&collection, "mock.prefab")
            .unwrap();
        assert_eq!(t1.read().unwrap().value, "mock");
    }

    assert_eq!(rs.size(), 4);
    rs.unload_unused();
    assert_eq!(rs.size(), 0);
}