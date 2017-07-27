/// Target build platform.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum BuildTarget {
    MacOS,
}

impl BuildTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            &BuildTarget::MacOS => "macOS",
        }
    }
}