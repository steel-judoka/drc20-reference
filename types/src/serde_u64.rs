//! Serde helpers for `u64` fields.
//!
//! Why this exists:
//! - Many JS/JSON environments cannot safely represent full 64-bit integers.
//! - Dusk's data-driver typically encodes/decodes `u64` values as strings.
//!
//! This module lets us accept `u64` values in JSON as either a number
//! (`123`) or a string (`"123"`), while serializing back to a string.

#[cfg(feature = "serde")]
use alloc::string::{String, ToString};

#[cfg(feature = "serde")]
use core::fmt;

#[cfg(feature = "serde")]
use serde::{de, Deserializer, Serializer};

/// Serialize a `u64` as a JSON string.
#[cfg(feature = "serde")]
pub fn serialize<S>(v: &u64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&v.to_string())
}

/// Deserialize a `u64` from either a JSON number or a JSON string.
#[cfg(feature = "serde")]
pub fn deserialize<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct U64Visitor;

    impl<'de> de::Visitor<'de> for U64Visitor {
        type Value = u64;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a u64 as a JSON number or string")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
where
            E: de::Error,
        {
            Ok(v)
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
where
            E: de::Error,
        {
            if v < 0 {
                return Err(E::custom("u64 cannot be negative"));
            }
            Ok(v as u64)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
where
            E: de::Error,
        {
            v.parse::<u64>().map_err(E::custom)
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
where
            E: de::Error,
        {
            v.parse::<u64>().map_err(E::custom)
        }
    }

    d.deserialize_any(U64Visitor)
}

// If you hit a compile error about `alloc` not being available, ensure the
// `serde` feature is enabled for this crate.
