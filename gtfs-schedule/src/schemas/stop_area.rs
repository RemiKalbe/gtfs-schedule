//! Provides data structures related to the assignment of stops to areas.
//!
//! The main type is:
//! - [`StopArea`]: Assigns stops from [`crate::schemas::stop::Stop`] to areas.

use serde::{Deserialize, Serialize};

use super::{AreaId, Schema, StopId};
use crate::error::{Result, SchemaValidationError};

/// Assigns stops from [`crate::schemas::stop::Stop`] to areas.
///
/// See [stop_areas.txt](https://gtfs.org/schedule/reference/#stop_areastxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StopArea {
    /// Identifies an area to which one or multiple [`StopArea::stop_id`] belong.
    /// The same [`StopArea::stop_id`] may be defined in many [`AreaId`].
    pub area_id: AreaId,
    /// Identifies a stop. If a station (i.e. a stop with
    /// [`crate::schemas::stop::Stop::location_type`]=`1`) is defined in
    /// this field, it is assumed that all of its platforms (i.e.
    /// all stops with [`crate::schemas::stop::Stop::location_type`]=`0`
    /// that have this station defined as [`crate::schemas::stop::Stop::parent_station`])
    /// are part of the same area. This behavior can be overridden
    /// by assigning platforms to other areas.
    pub stop_id: StopId,
}

impl StopArea {
    /// Validates if the StopArea is valid in regards to the GTFS specification constraints.
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
