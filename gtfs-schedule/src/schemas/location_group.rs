//! Provides data structures related to location groups.
//!
//! The main types are:
//! - [`LocationGroup`]: Defines location groups, which are groups of stops where a rider may request pickup or drop off.
//! - [`LocationGroupId`]: Identifies a location group.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::Schema;
use crate::error::{Result, SchemaValidationError};

/// Identifies a location group. ID must be unique across all [`crate::schemas::stop::Stop::stop_id`],
/// [`crate::schemas::location_group::LocationGroup::location_group_id`], and locations.geojson id values.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct LocationGroupId(pub String);

/// Defines location groups, which are groups of stops where a rider may request pickup or drop off.
///
/// See [location_groups.txt](https://gtfs.org/schedule/reference/#location_groupstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct LocationGroup {
    /// Identifies a location group.
    pub location_group_id: LocationGroupId,
    /// The name of the location group as displayed to the rider.
    pub location_group_name: Option<String>,
}

impl LocationGroup {
    /// Validates if the LocationGroup is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate location_group_id.
        if self.location_group_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "location_group_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
