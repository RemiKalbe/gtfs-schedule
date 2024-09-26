//! Provides data structures and enumerations related to fare leg rules.
//!
//! The main types are:
//! - [`FareLegRule`]: Represents a fare leg rule.
//! - [`FareLegRuleId`]: Identifies a group of entries in `fare_leg_rules.txt`.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};

use crate::schemas::timeframe::TimeframeGroupId;
use crate::{
    error::{Result, SchemaValidationError},
    schemas::fare_product::FareProductId,
};

use super::{AreaId, NetworkId, Schema};

/// Identifies a group of entries in `fare_leg_rules.txt`.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct FareLegRuleId(pub String);

/// Represents a fare leg rule.
///
/// See [fare_leg_rules.txt](https://gtfs.org/schedule/reference/#fare_leg_rulestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FareLegRule {
    /// Identifies a group of entries in `fare_leg_rules.txt`.
    pub leg_group_id: Option<FareLegRuleId>,
    /// Identifies a route network that applies for the fare leg rule.
    pub network_id: Option<NetworkId>,
    /// Identifies a departure area.
    pub from_area_id: Option<AreaId>,
    /// Identifies an arrival area.
    pub to_area_id: Option<AreaId>,
    /// Defines the timeframe for the fare validation event at the start of the fare leg.
    pub from_timeframe_group_id: Option<TimeframeGroupId>,
    /// Defines the timeframe for the fare validation event at the end of the fare leg.
    pub to_timeframe_group_id: Option<TimeframeGroupId>,
    /// The fare product required to travel the leg.
    pub fare_product_id: FareProductId,
    /// Defines the order of priority in which matching rules are applied to legs.
    pub rule_priority: Option<u32>,
}

impl FareLegRule {
    /// Validates if the FareLegRule is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate fare_product_id.
        if self.fare_product_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "fare_product_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
