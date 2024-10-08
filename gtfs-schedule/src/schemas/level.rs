//! Provides data structures related to levels within a station.
//!
//! The main types are:
//! - [`Level`]: Represents a level within a station.
//! - [`LevelId`]: Identifies a level in a station.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::Schema;
use crate::error::{Result, SchemaValidationError};

/// Identifies a level in a station.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct LevelId(pub String);

/// Represents a level within a station.
///
/// See [levels.txt](https://gtfs.org/schedule/reference/#levelstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Level {
    /// Identifies a level in a station.
    pub level_id: LevelId,
    /// Numeric index of the level that indicates its relative position.
    /// Ground level should have index 0, with levels above ground indicated by positive indices
    /// and levels below ground by negative indices.
    pub level_index: f32,
    /// Name of the level as seen by the rider inside the building or station.
    ///
    /// Example: "Mezzanine", "Platform" or "-1".
    pub level_name: Option<String>,
}

impl Level {
    /// Validates if the Level is valid in regards to the GTFS specification constraints.
    pub fn validate(&mut self) -> Result<()> {
        // Validate level_id.
        if self.level_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "level_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
