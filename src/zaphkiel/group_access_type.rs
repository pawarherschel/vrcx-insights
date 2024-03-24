#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum GroupAccessType {
    Public,
    Plus,
    Members,
    #[default]
    Other,
}

impl From<&str> for GroupAccessType {
    #[inline]
    fn from(value: &str) -> Self {
        let value = value.to_lowercase();
        Self::from(value)
    }
}

#[allow(clippy::fallible_impl_from)] // we WANT it to fail if it's wrong
impl From<String> for GroupAccessType {
    #[inline]
    fn from(value: String) -> Self {
        let value = value.to_lowercase();
        match value.as_str() {
            "public" => Self::Public,
            "plus" => Self::Plus,
            "members" => Self::Members,
            _ => panic!("Unknown group access type: {value}"),
        }
    }
}
