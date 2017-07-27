/// Target build platform.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum BuildTarget {
    MacOS,
}