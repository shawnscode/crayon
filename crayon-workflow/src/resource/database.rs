use std::collections::HashMap;
use std::path::Path;
use std::io::Read;
use std::sync::RwLock;
use std::fs;

use uuid;
use seahash;

use serialization;
use manifest::Manifest;
use errors::*;

use super::{Resource, ResourceMetadata};

/// An interface for accessing and performing operations on resources. All paths are
/// relative to the workspace folder, which is indicated by the path of `Crayon.toml`.
pub struct ResourceDatabase {
    manifest: Manifest,
    resources: ResourceRawDatabase,
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
               resources: ResourceRawDatabase::new(),
           })
    }

    /// Write persistent data to workspace.
    pub fn save(&self) -> Result<()> {
        let resource_database_path = self.manifest.workspace().join("resources.database");
        serialization::serialize(&self.database, &resource_database_path, true)
    }

    /// Import any changed resources. This will import any resources that have changed
    /// their content modification data or have been added-removed to the workspace folder.
    #[inline]
    pub fn refresh(&mut self) -> Result<()> {
        self.resources.refresh(&self.manifest)
    }

    /// Import resource at path.
    #[inline]
    pub fn import<P>(&self, path: P) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        let path = self.manifest.dir().join(&path);
        let metadata = self.resources.load_metadata(&self.manifest, &path)?;

        // Read file from disk.
        let mut file = fs::OpenOptions::new().create(true).read(true).open(&path)?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // Validate the resource if any changes has been made since last time we
        // import it.
        let checksum = seahash::hash(&bytes);

        {
            let mut database = self.database.write().unwrap();
            let _modified = database.is_modified(metadata.uuid(), checksum);

            // if self.database.is_modified(metadata.uuid(), checksum) {
            //     metadata.validate(bytes);
            // }

            database.set_checksum(metadata.uuid(), checksum);
        }

        Ok(metadata)
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
}

struct ResourceRawDatabase(HashMap<uuid::Uuid, ResourceMetadata>);

impl ResourceRawDatabase {
    pub fn new() -> ResourceRawDatabase {
        ResourceRawDatabase(HashMap::new())
    }

    pub fn refresh(&mut self, manifest: &Manifest) -> Result<()> {
        self.0.clear();

        for path in &manifest.resources {
            self.refresh_recursive(&manifest, &path)?;
        }

        Ok(())
    }

    fn refresh_recursive(&mut self, manifest: &Manifest, path: &Path) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?.path();
            if entry.is_dir() {
                self.refresh_recursive(&manifest, &entry)?;

            } else {
                let metadata = self.load_metadata(&manifest, &entry)?;
                self.0.insert(metadata.uuid(), metadata);
            }
        }

        Ok(())
    }

    pub fn load_metadata<P>(&self, manifest: &Manifest, path: P) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let metadata_path = path.join(".meta");

        if metadata_path.exists() {
            serialization::deserialize(&metadata_path, true)

        } else {
            // Make a reasonable guesss based on manifest.
            if let Some(&v) = path.extension()
                   .and_then(|v| manifest.types.get(v.to_str().unwrap())) {
                let metadata = ResourceMetadata::new_as(v);
                serialization::serialize(&metadata, &metadata_path, true)?;
                Ok(metadata)

            } else {
                bail!("failed to import file into workspace, undefined extension with {:?}",
                      path);
            }
        }
    }

    pub fn load_metadata_as<P>(&self, tt: Resource, path: P) -> Result<ResourceMetadata>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let metadata_path = path.join(".meta");

        if metadata_path.exists() {
            let metadata = serialization::deserialize::<ResourceMetadata, &Path>(&metadata_path,
                                                                                 true)?;
            if metadata.is(tt) {
                return Ok(metadata);
            }
        }

        let metadata = ResourceMetadata::new_as(tt);
        serialization::serialize(&metadata, &metadata_path, true)?;
        Ok(metadata)
    }
}