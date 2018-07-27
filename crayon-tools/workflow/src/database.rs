use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crayon::res::manifest;
use crayon::utils::uuid::Uuid;
use uuid;

use assets::ResourceCompiler;
use errors::*;

pub struct Database {
    compilers: Vec<Box<ResourceCompiler>>,
    exts: HashMap<String, usize>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            compilers: Vec::new(),
            exts: HashMap::new(),
        }
    }

    pub fn register_extensions<T>(&mut self, compiler: T, exts: &[&str])
    where
        T: ResourceCompiler + 'static,
    {
        let idx = self.compilers.len();
        self.compilers.push(Box::new(compiler));

        for &v in exts {
            self.exts.insert(v.to_owned(), idx);
        }
    }

    pub fn build(self, src: &Path, dst: &Path) -> Result<()> {
        assert!(src.is_dir());

        if dst.exists() {
            fs::remove_dir_all(dst)?;
        }

        fs::create_dir(dst)?;

        let mut manifest = manifest::Manifest::new();

        let mut files = Vec::new();
        Self::collect_files(src, &mut files)?;

        for v in files.drain(..) {
            if let Some(w) = v.extension() {
                if let Some(ext) = w.to_str() {
                    if let Some(&idx) = self.exts.get(ext) {
                        let filename = v.strip_prefix(src).unwrap();

                        let uuid = uuid::Uuid::new_v4();
                        let item = manifest::ManifestItem {
                            location: filename.into(),
                            uuid: Uuid::from_bytes(*uuid.as_bytes()),
                        };

                        let p = dst.join(format!("{}", item.uuid));
                        let mut i = fs::OpenOptions::new().read(true).open(&v)?;
                        let mut o = fs::OpenOptions::new().write(true).create(true).open(&p)?;
                        self.compilers[idx].compile(&mut i, &mut o)?;
                        manifest.items.push(item);

                        println!("\tCOMPILE {:?}", v);
                    }
                }
            }
        }

        let p = dst.join(manifest::NAME);
        let mut o = fs::OpenOptions::new().write(true).create(true).open(&p)?;
        ::assets::manifest_compiler::compile(&manifest, &mut o)?;
        Ok(())
    }

    fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_files(&path, files)?;
            } else {
                files.push(path);
            }
        }

        Ok(())
    }
}
