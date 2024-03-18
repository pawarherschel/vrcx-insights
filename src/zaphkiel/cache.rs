#![feature(impl_trait_in_assoc_type)]

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::sync::Arc;

use petgraph::visit::Walker;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Serialize, Serializer};

pub type ArcStrSet<T> = ArcStrMap<T, ()>;

#[derive(Debug, Default)]
pub struct ArcStrMap<K, V> {
    map: Arc<HashMap<K, V>>,
}

impl<K, V> Hash for ArcStrMap<K, V>
where
    HashMap<K, V>: Hash,
    K: Hash,
    V: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        let map = &*self.map.clone();
        map.hash(state);
    }
}

impl<K, V> Clone for ArcStrMap<K, V>
where
    K: Hash + PartialEq + Eq,
{
    fn clone(&self) -> Self {
        let map = self.map.clone();
        Self { map }
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for ArcStrMap<K, V>
where
    K: Eq + Hash + Into<Arc<str>>,
{
    fn from(value: [(K, V); N]) -> Self {
        let map = HashMap::from(value).into();
        Self { map }
    }
}

impl<K, V> FromIterator<(K, V)> for ArcStrMap<K, V>
where
    K: Eq + Hash + Into<Arc<str>>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let map = HashMap::from_iter(iter).into();
        Self { map }
    }
}

impl<K, V> Index<&K> for ArcStrMap<K, V>
where
    K: Eq + Hash + Into<Arc<str>>,
    Arc<str>: std::borrow::Borrow<K>,
{
    type Output = V;

    fn index(&self, index: &K) -> &Self::Output {
        &self.map[index]
    }
}

impl<K, V> ArcStrMap<K, V> {
    #[must_use]
    pub fn new_empty_arc_str() -> Self {
        let map = HashMap::new().into();
        Self { map }
    }

    #[must_use]
    pub fn get_map(&self) -> &HashMap<K, V> {
        &self.map
    }
}

impl<K, V> ArcStrMap<K, V>
where
    K: Eq + Hash + Into<Arc<str>> + Clone,
    V: ToOwned<Owned = V> + Clone,
    HashMap<K, V>: FromIterator<(<K as ToOwned>::Owned, <V as ToOwned>::Owned)>,
{
    #[inline]
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(
        self,
    ) -> impl IntoIterator<Item = (<K as ToOwned>::Owned, <V as ToOwned>::Owned)> {
        let it = self
            .map
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<HashMap<K, V>>();
        it.into_iter()
    }
}

impl<K, V> ArcStrMap<K, V> {
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }
}

impl<K> ArcStrSet<K>
where
    K: Eq + PartialEq + Hash,
{
    #[inline]
    pub fn insert_set(&mut self, query: impl Into<K>) -> bool {
        Arc::get_mut(&mut self.map).is_some_and(|it| it.insert(query.into(), ()).is_some())
    }

    #[inline]
    #[must_use]
    pub fn keys_set(&self) -> std::collections::hash_map::Keys<'_, K, ()> {
        self.map.keys()
    }
}

impl<K, V> ArcStrMap<K, V>
where
    K: Eq + PartialEq + Hash,
{
    #[inline]
    #[must_use]
    pub fn get(&self, query: &K) -> Option<&V> {
        self.map.get(query)
    }

    #[inline]
    pub fn insert(&mut self, query: impl Into<K>, value: V) -> Option<V> {
        Arc::get_mut(&mut self.map)?.insert(query.into(), value)
    }

    #[inline]
    #[must_use]
    pub fn values(&self) -> std::collections::hash_map::Values<'_, K, V> {
        self.map.values()
    }

    #[inline]
    #[must_use]
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, K, V> {
        self.map.keys()
    }
}

impl<K, V> Serialize for ArcStrMap<K, V>
where
    K: Serialize + AsRef<str>,
    V: Serialize,
    Arc<str>: for<'a> From<&'a K>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = self.iter().map(|(k, v)| (k.as_ref(), v));

        let mut s = serializer.serialize_map(Some(self.len()))?;

        for (k, v) in map {
            s.serialize_entry(k, v)?;
        }
        s.end()
    }
}

impl<K, V> ArcStrMap<K, V> {
    #[must_use]
    pub fn new() -> Self {
        let map = Arc::new(HashMap::default());
        Self { map }
    }

    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
