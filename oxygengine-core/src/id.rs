use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    str::FromStr,
};
use uuid::Uuid;

/// Universal Identifier (uuidv4).
#[derive(Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct ID<T> {
    id: Uuid,
    #[serde(skip_serializing, skip_deserializing)]
    _phantom: PhantomData<T>,
}

impl<T> ID<T> {
    /// Creates new identifier.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates new identifier from raw bytes.
    #[inline]
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            id: Uuid::from_bytes(bytes),
            _phantom: PhantomData,
        }
    }

    /// Gets underlying UUID object.
    #[inline]
    pub fn uuid(&self) -> Uuid {
        self.id
    }
}

impl<T> Default for ID<T> {
    #[inline]
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            _phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for ID<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<T> ToString for ID<T> {
    #[inline]
    fn to_string(&self) -> String {
        format!("{}", self.id)
    }
}

impl<T> FromStr for ID<T> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Uuid::parse_str(s) {
            Ok(uuid) => Ok(Self {
                id: uuid,
                _phantom: PhantomData,
            }),
            Err(_) => Err(s.to_owned()),
        }
    }
}

impl<T> Hash for ID<T> {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state)
    }
}

impl<T> PartialEq for ID<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ID<T> {}
impl<T> Copy for ID<T> where T: Clone {}

impl<T> PartialOrd for ID<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<T> Ord for ID<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
