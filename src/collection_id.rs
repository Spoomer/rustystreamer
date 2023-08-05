use serde::{
    de::{self, Deserialize, Deserializer, Visitor},
    Serialize,
};
use std::fmt;

/// Wrapper for u32 as VideoId
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct CollectionId(pub u32);

impl<'de> Deserialize<'de> for CollectionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = CollectionId;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("video ID as a number or string")
            }

            fn visit_u32<E>(self, id: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(CollectionId(id))
            }

            fn visit_str<E>(self, id: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                id.parse().map(CollectionId).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_any(IdVisitor)
    }
}

impl Serialize for CollectionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}
