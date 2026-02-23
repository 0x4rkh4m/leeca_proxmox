//! Serde helpers for custom serialization.

use serde::{Deserialize, Deserializer, Serializer};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Serialization and deserialization for `SystemTime` as seconds since UNIX epoch.
pub mod system_time {
    use super::*;

    /// Serialize a `SystemTime` as a u64 representing seconds since UNIX epoch.
    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| serde::ser::Error::custom("SystemTime before UNIX epoch"))?;
        serializer.serialize_u64(duration.as_secs())
    }

    /// Deserialize a u64 representing seconds since UNIX epoch into a `SystemTime`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}
