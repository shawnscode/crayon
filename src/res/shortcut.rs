//! Central registry for shortcut definitions. Shortcuts are path aliases that
//! could be resolved into full path.

use crate::utils::hash::FastHashMap;

/// Central registry for shortcut definitions. Shortcuts are path aliases that
/// could be resolved into full path.
#[derive(Debug, Default, Clone)]
pub struct ShortcutResolver {
    registry: FastHashMap<String, String>,
}

impl ShortcutResolver {
    /// Creates a new shortcut registry.
    pub fn new() -> Self {
        ShortcutResolver {
            registry: FastHashMap::default(),
        }
    }

    /// Add or replace a shortcut definition.
    pub fn add<T1, T2>(&mut self, shortcut: T1, fullname: T2) -> Result<(), failure::Error>
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        let shortcut = shortcut.into();
        let fullname = fullname.into();

        if !shortcut.ends_with(':') {
            bail!("Shortcut MUST ends with a colon (':').");
        }

        if shortcut.len() < 2 {
            bail!("Shortcut MUST be at least 2 chars to not be confused with DOS drive letters.");
        }

        if !fullname.ends_with(':') && !fullname.ends_with('/') {
            bail!("Fullname must end in a '/' (dir) or ':' (other shortcut).");
        }

        self.registry.insert(shortcut, fullname);
        Ok(())
    }

    /// Checks if a shortcut exists.
    #[inline]
    pub fn has<T: AsRef<str>>(&self, shortcut: T) -> bool {
        self.registry.contains_key(shortcut.as_ref())
    }

    /// Resolve shortcuts in the provided string recursively and return None if not exists.
    pub fn resolve<T: AsRef<str>>(&self, src: T) -> Option<String> {
        unsafe {
            let mut dst = src.as_ref().to_string();
            loop {
                // find schema letters.
                if dst.find("://").is_some() {
                    break;
                }

                if let Some(index) = dst.find(':') {
                    // ignore DOS drive letters.
                    if let Some(fullname) = self.registry.get(dst.get_unchecked(0..=index)) {
                        dst.replace_range(0..=index, fullname);
                    } else {
                        return None;
                    }
                } else {
                    break;
                }
            }

            Some(dst)
        }
    }
}
