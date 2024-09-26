//! Provides data structures related to the assignment of stops to location groups.
//!
//! The main type is:
//! - [`LocationGroupStop`]: Assigns stops from [`crate::schemas::stop::Stop`] to location groups.

use serde::{Deserialize, Serialize};

use super::{LocationGroupId, Schema, StopId};
use crate::error::{Result, SchemaValidationError};

/// Assigns stops from [`crate::schemas::Stop`] to location groups.
///
/// See [location_group_stops.txt](https://gtfs.org/schedule/reference/#location_group_stopstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationGroupStop {
    /// Identifies a location group to which one or multiple [`LocationGroupStop::stop_id`] belong.
    /// The same [`LocationGroupStop::stop_id`] may be defined in many [`LocationGroupStop::location_group_id`].
    pub location_group_id: LocationGroupId,
    /// Identifies a stop belonging to the location group.
    pub stop_id: StopId,
}

impl LocationGroupStop {
    /// Validates if the LocationGroupStop is valid in regards to the GTFS specification constraints.
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

        // Validate stop_id.
        if self.stop_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "stop_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
