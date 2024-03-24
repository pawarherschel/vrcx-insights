use std::sync::{Arc, LazyLock, OnceLock};

use serde::{Deserialize, Serialize};

pub static KAT_ID: LazyLock<Id> =
    LazyLock::new(|| Id("usr_c2a23c47-1622-4b7a-90a4-b824fcaacc69".into()));
pub static KAT_DISPLAY_NAME: OnceLock<Name> = OnceLock::new();
pub static KAT_EXISTS: LazyLock<bool> = LazyLock::new(|| std::fs::metadata(".kat").is_ok());

pub trait IsKat {
    fn is_kat(&self) -> bool;
}

impl IsKat for Arc<str> {
    #[inline]
    fn is_kat(&self) -> bool {
        self.clone() == KAT_ID.0 || self.clone() == KAT_DISPLAY_NAME.get().unwrap().0
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Id(Arc<str>);

impl Clone for Id {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl ToString for Id {
    #[inline]
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<String> for Id {
    #[inline]
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<Arc<str>> for Id {
    #[inline]
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

impl From<&Arc<str>> for Id {
    #[inline]
    fn from(value: &Arc<str>) -> Self {
        Self(value.clone())
    }
}

impl From<&str> for Id {
    #[inline]
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<Id> for Arc<str> {
    #[inline]
    fn from(value: Id) -> Self {
        value.0
    }
}

impl IsKat for Id {
    #[inline]
    fn is_kat(&self) -> bool {
        *self == *KAT_ID
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Name(pub Arc<str>);

impl Clone for Name {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl ToString for Name {
    #[inline]
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<String> for Name {
    #[inline]
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<Arc<str>> for Name {
    #[inline]
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

impl From<Name> for Arc<str> {
    #[inline]
    fn from(value: Name) -> Self {
        value.0
    }
}

impl From<&Name> for Arc<str> {
    #[inline]
    fn from(value: &Name) -> Self {
        value.into()
    }
}

impl AsRef<str> for Name {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl IsKat for Name {
    #[inline]
    fn is_kat(&self) -> bool {
        self == KAT_DISPLAY_NAME.get().unwrap()
    }
}
