#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum GroupAccessType {
    Public,
    Plus,
    Members,
    #[default]
    Other,
}

impl From<&str> for GroupAccessType {
    fn from(value: &str) -> Self {
        let value = value.to_lowercase();
        Self::from(value)
    }
}

impl From<String> for GroupAccessType {
    fn from(value: String) -> Self {
        let value = value.to_lowercase();
        match value.as_str() {
            "public" => GroupAccessType::Public,
            "plus" => GroupAccessType::Plus,
            "members" => GroupAccessType::Members,
            _ => panic!("Unknown group access type: {}", value),
        }
    }
}
