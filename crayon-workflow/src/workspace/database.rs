use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::fs;
use std::sync::RwLock;
use std::ops::Deref;

use uuid::Uuid;
use walkdir;
use seahash;
use crayon;

use resource::{ResourceMetadata, ResourceType};

use errors::*;
use utils::{yaml, bincode};
use platform::BuildTarget;
use super::WorkspaceSettings;

const METADATA_EXTENSION: &'static str = "meta";
const DATABASE_FILENAME: &'static str = ".database";

/// An interface for accessing and performing operations on resources.
pub struct Database {
    resources: HashMap<Uuid, ResourceMetadata>,
    paths: HashMap<Uuid, PathBuf>,
    database: RwLock<DatabasePersistentData>,
}

impl Database {
    /// Load database at path.
    pub fn load_from<P>(path: P) -> Result<Database>
        where P: AsRef<Path>
    {
        let database_path = path.as_ref().join(DATABASE_FILENAME);
        let database = if database_path.exists() {
            yaml::deserialize(&database_path)?
        } else {
            DatabasePersistentData::new()
        };

        Ok(Database {
               resources: HashMap::new(),
               paths: HashMap::new(),
               database: RwLock::new(database),
           })
    }

    /// Get the length of all resources in workspace.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Write persistent data to workspace.
    pub fn save<P>(&self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        let database_path = path.as_ref().join(DATABASE_FILENAME);
        let database = self.database.read().unwrap();
        yaml::serialize(Deref::deref(&database), &database_path)
    }

    /// Get the uuid of resource at path.
    pub fn uuid<P>(&self, path: P) -> Option<Uuid>
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

    /// Build all resources into serialization data which could be used at runtime.
    pub fn build<P>(&self,
                    version: &str,
                    _: BuildTarget,
                    path: P,
                    settings: &WorkspaceSettings)
                    -> Result<()>
        where P: AsRef<Path>
    {
        let mut manifest = crayon::resource::workflow::ResourceManifest {
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
                    for root in &settings.resource_folders {
                        if let Ok(next) = resource_path.strip_prefix(root) {
                            resource_relative_path = next.to_path_buf();
                            break;
                        }
                    }

                    let item = crayon::resource::workflow::ResourceManifestItem {
                        checksum: seahash::hash(&out),
                        path: resource_relative_path,
                        dependencies: Vec::new(),
                        uuid: *id,
                        payload: metadata.payload(),
                    };

                    manifest.items.insert(*id, item);
                }
            }
        }

        //
        bincode::serialize(&manifest, path.as_ref().join("manifest"))?;
        yaml::serialize(&manifest, path.as_ref().join("readable_manifest"))?;

        Ok(())
    }

    /// Refresh `Database` by scanning all the resource folders.
    ///
    /// For resources that have been added to workspace, this will try to guess the internal
    /// representations and generate a empty `.meta` for it. And re-import any resources that
    /// have changed their content.
    pub fn refresh(&mut self, settings: &WorkspaceSettings) -> Result<()> {
        // Collect all the tracking resources.
        let mut files = Vec::new();
        for dir in &settings.resource_folders {
            for file in walkdir::WalkDir::new(&dir).into_iter() {
                if let Ok(file) = file {
                    if file.file_type().is_file() {
                        files.push(file);
                    }
                }
            }
        }

        let mut resources = HashMap::new();
        for file in files {
            let is_metafile = file.path()
                .extension()
                .map(|v| v == METADATA_EXTENSION)
                .unwrap_or(false);

            if is_metafile {
                let resource_file = resource_file_path(&file.path());

                // Remove deprecated meta files if its asset not exists any more.
                if !resource_file.exists() {
                    fs::remove_file(&file.path())?;
                    println!("A meta data file (.meta) exists but its asset '{:?}' can't be found. \
                            When moving or deleting files, \
                            please ensure that the corresponding .meta file is moved or deleted along with it.",
                             resource_file);
                } else {
                    resources.insert(resource_file, file.path().to_path_buf());
                }
            } else {
                let meta_file = metadata_file_path(&file.path());
                resources.insert(file.path().to_path_buf(), meta_file);
            }
        }

        // Load all the metafile from disks, and varify the validation of resources if neccessary.
        self.resources.clear();
        self.paths.clear();

        for (file, _) in resources {
            match self.import(&file, &settings) {
                Ok(metadata) => {
                    self.paths.insert(metadata.uuid(), file);
                    self.resources.insert(metadata.uuid(), metadata);
                }
                Err(err) => {
                    println!("Failed to import resource from {:?}. \n{:?}", file, err);
                }
            }
        }

        Ok(())
    }

    /// Import resource at path into database.
    pub fn import<P>(&self, path: P, settings: &WorkspaceSettings) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        if !path.exists() {
            bail!(ErrorKind::FileNotFound);
        }

        let metadata_path = metadata_file_path(&path);
        if metadata_path.exists() {
            let metadata = yaml::deserialize(&metadata_path)?;
            self.validate(path, &metadata)?;
            Ok(metadata)

        } else {
            // Make a reasonable guesss based on settings.
            let tt = path.extension()
                .and_then(|v| settings.resource_exts.get(v.to_str().unwrap()))
                .and_then(|v| Some(*v))
                .unwrap_or(ResourceType::Bytes);

            let metadata = {
                let metadata = ResourceMetadata::new_as(tt);
                if self.validate(path, &metadata).is_err() {
                    ResourceMetadata::new_as(ResourceType::Bytes)
                } else {
                    metadata
                }
            };

            yaml::serialize(&metadata, &metadata_path)?;
            Ok(metadata)
        }
    }

    /// Re-import resource as specified type. This will remove original meta file if its ok
    /// to treat the resource as type `tt`.
    pub fn reimport<P>(&self, path: P, tt: ResourceType) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        if !path.exists() {
            bail!(ErrorKind::FileNotFound);
        }

        let metadata_path = metadata_file_path(&path);
        if metadata_path.exists() {
            let metadata: ResourceMetadata = yaml::deserialize(&metadata_path)?;
            if metadata.is(tt) {
                return Ok(metadata);
            }
        }

        let metadata = ResourceMetadata::new_as(tt);
        self.validate(path, &metadata)?;
        yaml::serialize(&metadata, &metadata_path)?;
        Ok(metadata)
    }

    fn validate<P>(&self, path: P, metadata: &ResourceMetadata) -> Result<()>
        where P: AsRef<Path>
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
                metadata.validate(&bytes)?;
            }

            database.set_checksum(metadata.uuid(), checksum);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabasePersistentData {
    checksums: HashMap<Uuid, u64>,
}

impl DatabasePersistentData {
    pub fn new() -> DatabasePersistentData {
        DatabasePersistentData { checksums: HashMap::new() }
    }

    pub fn clear(&mut self) {
        self.checksums.clear();
    }

    pub fn is_modified(&self, uuid: Uuid, checksum: u64) -> bool {
        if let Some(&crc) = self.checksums.get(&uuid) {
            crc != checksum
        } else {
            true
        }
    }

    pub fn set_checksum(&mut self, uuid: Uuid, checksum: u64) {
        self.checksums.insert(uuid, checksum);
    }

    pub fn len(&self) -> usize {
        self.checksums.len()
    }
}

fn metadata_file_path<P>(path: P) -> PathBuf
    where P: AsRef<Path>
{
    let path = path.as_ref();

    if let Some(exts) = path.extension() {
        path.with_extension(exts.to_string_lossy().into_owned() + "." + METADATA_EXTENSION)
    } else {
        path.with_extension(METADATA_EXTENSION)
    }
}

fn resource_file_path<P>(path: P) -> PathBuf
    where P: AsRef<Path>
{
    path.as_ref().with_extension("")
}