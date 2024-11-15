use std::{marker::PhantomData, str::FromStr};

#[cfg(feature = "diesel")]
mod diesel_impls;

#[cfg_attr(feature = "diesel", derive(diesel::FromSqlRow, diesel::AsExpression))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Uuid))]
pub struct Uuid<T> {
    uuid: uuid::Uuid,
    marker: PhantomData<T>,
}

// Derive traits
impl<T> Copy for Uuid<T> {}
impl<T> Clone for Uuid<T> {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> Default for Uuid<T> {
    fn default() -> Self {
        Self {
            uuid: Default::default(),
            marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for Uuid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uuid.fmt(f)
    }
}

impl<T> std::cmp::Eq for Uuid<T> {}
impl<T> std::cmp::PartialEq for Uuid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl<T> std::cmp::Ord for Uuid<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uuid.cmp(&other.uuid)
    }
}

impl<T> std::cmp::PartialOrd for Uuid<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.uuid.cmp(&other.uuid))
    }
}

impl<T> std::hash::Hash for Uuid<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

// Extra formatting
impl<T> std::fmt::Display for Uuid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uuid.fmt(f)
    }
}

impl<T> std::fmt::LowerHex for Uuid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uuid.fmt(f)
    }
}

impl<T> std::fmt::UpperHex for Uuid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.uuid.fmt(f)
    }
}

// Serde
impl<T> serde::Serialize for Uuid<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.uuid.serialize(serializer)
    }
}

impl<'de, T> serde::Deserialize<'de> for Uuid<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        uuid::Uuid::deserialize(deserializer).map(From::from)
    }
}

// Parsing
impl<T> FromStr for Uuid<T> {
    type Err = <uuid::Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::from_str(s).map(From::from)
    }
}

// Conversions
impl<T> From<uuid::Uuid> for Uuid<T> {
    fn from(uuid: uuid::Uuid) -> Self {
        Self {
            uuid,
            marker: PhantomData,
        }
    }
}

impl<T> From<Uuid<T>> for uuid::Uuid {
    fn from(value: Uuid<T>) -> Self {
        value.uuid
    }
}

impl<T> From<Uuid<T>> for String {
    fn from(value: Uuid<T>) -> Self {
        value.uuid.into()
    }
}

impl<T> TryFrom<&str> for Uuid<T> {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        uuid::Uuid::try_parse(value).map(From::from)
    }
}

impl<T> From<Uuid<T>> for Vec<u8> {
    fn from(value: Uuid<T>) -> Self {
        value.uuid.into()
    }
}

impl<T> TryFrom<std::vec::Vec<u8>> for Uuid<T> {
    type Error = uuid::Error;

    fn try_from(value: std::vec::Vec<u8>) -> Result<Self, Self::Error> {
        uuid::Uuid::try_from(value).map(From::from)
    }
}

impl<T> TryFrom<&[u8]> for Uuid<T> {
    type Error = uuid::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        uuid::Uuid::from_slice(value).map(From::from)
    }
}