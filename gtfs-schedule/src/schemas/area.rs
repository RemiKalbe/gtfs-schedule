//! Provides data structures and enumerations related to areas.
//!
//! The main types are:
//! - [`Area`]: Defines area identifiers.
//! - [`AreaId`]: Identifies an area.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::Schema;
use crate::error::{Result, SchemaValidationError};

/// Identifies an area.
///
/// Must be unique in [areas.txt](https://gtfs.org/schedule/reference/#areastxt).
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct AreaId(pub String);

/// Defines area identifiers.
///
/// See [areas.txt](https://gtfs.org/schedule/reference/#areastxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Area {
    /// Identifies an area. Must be unique in [areas.txt](https://gtfs.org/schedule/reference/#areastxt).
    pub area_id: AreaId,
    /// The name of the area as displayed to the rider.
    pub area_name: Option<String>,
}

impl Area {
    /// Validates if the Area is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate area_id.
        if self.area_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "area_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
