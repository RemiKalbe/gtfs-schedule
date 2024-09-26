//! Provides data structures related to the assignment of routes to networks.
//!
//! The main type is:
//! - [`RouteNetwork`]: Assigns routes from [`crate::schemas::route::Route`] to networks.

use serde::{Deserialize, Serialize};

use super::{NetworkId, RouteId, Schema};
use crate::error::{Result, SchemaValidationError};

/// Assigns routes from [`crate::schemas::route::Route`] to networks.
///
/// See [route_networks.txt](https://gtfs.org/schedule/reference/#route_networkstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RouteNetwork {
    /// Identifies a network to which one or multiple [`RouteNetwork::route_id`]
    /// belong. A [`RouteNetwork::route_id`] can only be defined in one [`RouteNetwork::network_id`].
    pub network_id: NetworkId,
    /// Identifies a route.
    pub route_id: RouteId,
}

impl RouteNetwork {
    /// Validates if the RouteNetwork is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate network_id.
        if self.network_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "network_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate route_id.
        if self.route_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "route_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
