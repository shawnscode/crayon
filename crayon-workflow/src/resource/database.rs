use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::sync::RwLock;
use std::fs;
use std::ops::Deref;

use crayon;
use uuid;
use seahash;
use walkdir;

use serialization;
use manifest::Manifest;
use platform;
use errors::*;

use super::{ResourceMetadata, METADATA_EXTENSION};

/// An interface for accessing and performing operations on resources. All paths are
/// relative to the workspace folder, which is indicated by the path of `Crayon.toml`.
pub struct ResourceDatabase {
    manifest: Manifest,

    resources: HashMap<uuid::Uuid, ResourceMetadata>,
    paths: HashMap<uuid::Uuid, PathBuf>,

    database: RwLock<ResourceDatabasePersistentData>,
}

impl ResourceDatabase {
    /// Create a new resource database.
    pub fn new(manifest: Manifest) -> Result<ResourceDatabase> {
        let resource_database_path = manifest.workspace().join("resources.database");
        let database = if resource_database_path.exists() {
            serialization::deserialize(&resource_database_path, true)?
        } else {
            ResourceDatabasePersistentData::new()
        };

        Ok(ResourceDatabase {
               manifest: manifest,
               database: RwLock::new(database),
               resources: HashMap::new(),
               paths: HashMap::new(),
           })
    }

    /// Clean database.
    pub fn clean(&mut self) {
        self.database.write().unwrap().clear();
    }

    /// The length of all resources in workspace.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Write persistent data to workspace.
    pub fn save(&self) -> Result<()> {
        let resource_database_path = self.manifest.workspace().join("resources.database");
        let database = self.database.read().unwrap();
        serialization::serialize(Deref::deref(&database), &resource_database_path, true)
    }

    /// Get the uuid of resource at path.
    pub fn uuid<P>(&self, path: P) -> Option<uuid::Uuid>
        where P: AsRef<Path>
    {
        let path = if path.as_ref().is_relative() {
            ::std::env::current_dir().unwrap().join(path)
        } else {
            path.as_ref().to_path_buf()
        };

        for (uuid, resource) in &self.paths {
            if resource == &path {
                return Some(*uuid);
            }
        }

        None
    }

    /// Build resources into serialization data which could be imported by cyon-runtime
    /// directly.
    pub fn build<P>(&self, version: &str, _: platform::BuildTarget, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        fs::create_dir_all(path.as_ref())?;

        let mut manifest = crayon::resource::manifest::ResourceManifest {
            path: PathBuf::new(),
            version: version.to_owned(),
            items: HashMap::new(),
        };

        let mut out = Vec::new();
        for (id, metadata) in &self.resources {
            if let Some(resource_path) = self.paths.get(&id) {
                if resource_path.exists() {
                    // Read source from disk.
                    let mut file = fs::OpenOptions::new().read(true).open(&resource_path)?;

                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)?;

                    out.clear();
                    metadata.build(&self, &resource_path, &bytes, &mut out)?;

                    // Write to specified path.
                    let name = id.simple().to_string();
                    let mut file = fs::OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open(path.as_ref().join(name))?;

                    file.write(&out)?;
                    file.flush()?;

                    // Use relative path as readable identifier of resources.
                    let mut resource_relative_path = resource_path.to_path_buf();
                    for root in &self.manifest.resources {
                        if let Ok(next) = resource_path.strip_prefix(root) {
                            resource_relative_path = next.to_path_buf();
                            break;
                        }
                    }

                    let item = crayon::resource::manifest::ResourceManifestItem {
                        checksum: seahash::hash(&out),
                        path: resource_relative_path,
                        dependencies: Vec::new(),
                        uuid: *id,
                        payload: metadata.file_type().into(),
                    };

                    manifest.items.insert(*id, item);
                }
            }
        }

        //
        serialization::serialize(&manifest, path.as_ref().join("manifest"), false)?;

        // Readable manifest.
        serialization::serialize(&manifest, path.as_ref().join("readable_manifest"), true)?;
        Ok(())
    }

    /// Import any changed resources. This will import any resources that have changed
    /// their content modification data or have been added-removed to the workspace folder.
    #[inline]
    pub fn refresh(&mut self) -> Result<()> {
        self.resources.clear();

        let dirs = self.manifest.resources.clone();
        for path in dirs {
            let files: Vec<_> = walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|v| v.ok())
                .collect();

            let mut resources = HashMap::new();
            for v in files {
                if v.file_type().is_file() {
                    let is_metafile = v.path()
                        .extension()
                        .map(|v| v == METADATA_EXTENSION)
                        .unwrap_or(false);

                    if is_metafile {
                        let parent_file = v.path().with_extension("");

                        // Remove deprecated meta files if its asset not exists any more.
                        if !parent_file.exists() {
                            fs::remove_file(&parent_file)?;
                            println!("A meta data file (.meta) exists but its asset '{:?}' can't be found. \
                            When moving or deleting files, \
                            please ensure that the corresponding .meta file is moved or deleted along with it.",
                                     parent_file);
                        } else {
                            resources.insert(parent_file, v.path().to_path_buf());
                        }
                    } else {
                        let meta_file = ResourceDatabase::metadata_path(&v.path());
                        resources.insert(v.path().to_path_buf(), meta_file);
                    }
                }
            }

            for (file, _) in resources {
                let rsp = self.load_metadata(&file);
                if let Ok(metadata) = rsp {

                    self.paths.insert(metadata.uuid(), file);
                    self.resources.insert(metadata.uuid(), metadata);
                } else {
                    // println!("{:?}", rsp);
                }
            }
        }

        Ok(())
    }

    /// Load meta file from disk. This method will treat the resource as specified type.
    pub fn load_metadata_as<P>(&self, path: P, tt: super::Resource) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        let metadata_path = ResourceDatabase::metadata_path(&path);

        let metadata = if metadata_path.exists() {
            let metadata: ResourceMetadata = serialization::deserialize(&metadata_path, true)?;
            if !metadata.is(tt) {
                let metadata = ResourceMetadata::new_as(tt);
                serialization::serialize(&metadata, &metadata_path, true)?;
                metadata
            } else {
                metadata
            }
        } else {
            let metadata = ResourceMetadata::new_as(tt);
            serialization::serialize(&metadata, &metadata_path, true)?;
            metadata
        };

        self.validate_metadata(path, metadata_path, metadata)
    }

    /// Load corresponding meta file from disk, and deserialize it into `ResourceMetadata`. A new
    /// and empty meta file will be generated if it is not exits.
    pub fn load_metadata<P>(&self, path: P) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        if !path.exists() {
            bail!("Failed to find resource file located at {:?}.", path);
        }

        let metadata_path = ResourceDatabase::metadata_path(&path);
        let metadata = if metadata_path.exists() {
            serialization::deserialize(&metadata_path, true)?

        } else {
            // Make a reasonable guesss based on manifest.
            if let Some(&v) = path.extension()
                   .and_then(|v| self.manifest.types.get(v.to_str().unwrap())) {
                let metadata = ResourceMetadata::new_as(v);
                serialization::serialize(&metadata, &metadata_path, true)?;
                metadata

            } else {
                bail!("Failed to import file into workspace, undefined extension with {:?}",
                      path);
            }
        };

        self.validate_metadata(path, metadata_path, metadata)
    }

    fn validate_metadata<P, P2>(&self,
                                path: P,
                                metadata_path: P2,
                                metadata: ResourceMetadata)
                                -> Result<ResourceMetadata>
        where P: AsRef<Path>,
              P2: AsRef<Path>
    {
        // Read file from disk.
        let mut file = fs::OpenOptions::new().read(true).open(&path)?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // Validate the resource if any changes has been made since last time we
        // import it.
        let checksum = seahash::hash(&bytes);

        {
            let mut database = self.database.write().unwrap();

            if database.is_modified(metadata.uuid(), checksum) {
                let validation = metadata.validate(&bytes);
                if validation.is_err() {
                    fs::remove_file(&metadata_path)?;
                    bail!("Failed to import file into workspace, can not validate it as {:?}. \n\t{:?}.",
                          metadata.file_type(),
                          validation);
                }
            }

            database.set_checksum(metadata.uuid(), checksum);
        }

        Ok(metadata)
    }

    fn metadata_path<P>(path: P) -> PathBuf
        where P: AsRef<Path>
    {
        let path = path.as_ref();

        if let Some(exts) = path.extension() {
            path.with_extension(exts.to_string_lossy().into_owned() + "." + METADATA_EXTENSION)
        } else {
            path.with_extension(METADATA_EXTENSION)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceDatabasePersistentData {
    checksums: HashMap<uuid::Uuid, u64>,
}

impl ResourceDatabasePersistentData {
    pub fn new() -> ResourceDatabasePersistentData {
        ResourceDatabasePersistentData { checksums: HashMap::new() }
    }

    pub fn clear(&mut self) {
        self.checksums.clear();
    }

    pub fn is_modified(&self, uuid: uuid::Uuid, checksum: u64) -> bool {
        if let Some(&crc) = self.checksums.get(&uuid) {
            crc != checksum
        } else {
            true
        }
    }

    pub fn set_checksum(&mut self, uuid: uuid::Uuid, checksum: u64) {
        self.checksums.insert(uuid, checksum);
    }

    pub fn len(&self) -> usize {
        self.checksums.len()
    }
}