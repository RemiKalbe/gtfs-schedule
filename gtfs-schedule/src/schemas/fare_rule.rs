//! Provides data structures related to fare rules.
//!
//! The main type is:
//! - [`FareRule`]: Represents a rule that specifies how fares apply to an itinerary.

use serde::{Deserialize, Serialize};

use super::{RouteId, Schema};
use crate::{
    error::{Result, SchemaValidationError},
    schemas::fare_attribute::FareId,
};

/// Represents a rule that specifies how fares apply to an itinerary.
///
/// See [fare_rules.txt](https://gtfs.org/schedule/reference/#fare_rulestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FareRule {
    /// Identifies a fare class.
    pub fare_id: FareId,
    /// Identifies a route associated with the fare class. If several routes with the
    /// same fare attributes exist, create a record in [`FareRule`] for each route.
    pub route_id: Option<RouteId>,
    /// Identifies an origin zone. If a fare class has multiple origin zones,
    /// create a record in [`FareRule`] for each [`FareRule::origin_id`].
    pub origin_id: Option<String>,
    /// Identifies a destination zone. If a fare class has multiple destination
    /// zones, create a record in [`FareRule`] for each [`FareRule::destination_id`].
    pub destination_id: Option<String>,
    /// Identifies the zones that a rider will enter while using a given fare
    /// class. Used in some systems to calculate correct fare class.
    pub contains_id: Option<String>,
}

impl FareRule {
    /// Validates if the FareRule is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate fare_id.
        if self.fare_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "fare_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
