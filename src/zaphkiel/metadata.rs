use std::cmp::Ordering;
use std::fmt::Formatter;

use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserializer, Serializer};

#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    pub count: u32,
    pub max: u32,
    pub total: u32,
    pub percentage: f64,
    pub percentile: f64,
}

impl PartialEq for Metadata {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let &Self {
            count,
            total,
            max,
            percentage: _,
            percentile: _,
        } = self;

        count == other.count && total == other.total && max == other.max
    }
}

impl serde::Serialize for Metadata {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Self {
            count,
            max,
            total,
            percentage,
            percentile,
        } = self;

        let mut s = serializer.serialize_tuple(5)?;
        s.serialize_element(count)?;
        s.serialize_element(max)?;
        s.serialize_element(total)?;
        s.serialize_element(percentage)?;
        s.serialize_element(percentile)?;
        s.end()
    }
}

struct MetadataVisitor;

impl<'de> Visitor<'de> for MetadataVisitor {
    type Value = Metadata;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a tuple of len 5")
    }

    #[inline]
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // $(
        //         let $name = match tri!(seq.next_element()) {
        //         Some(value) => value,
        //         None => return Err(Error::invalid_length($n, &self)),
        //     };
        // )+
        //
        // Ok(($($name,)+))

        todo!()
    }
}

impl<'de> serde::Deserialize<'de> for Metadata {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(5, MetadataVisitor)
    }
}

impl Eq for Metadata {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Metadata {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let &Self {
            count,
            total,
            max,
            percentage: _,
            percentile: _,
        } = self;
        let count = count.cmp(&other.count);
        let total = total.cmp(&other.total);
        let max = max.cmp(&other.max);

        match (total, max) {
            (Ordering::Equal, Ordering::Equal) => Some(count),
            _ => None,
        }
    }
}

impl Ord for Metadata {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
