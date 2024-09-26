//! Provides data structures and enumerations related to fare transfer rules.
//!
//! The main types are:
//! - [`FareTransferRule`]: Represents a fare transfer rule.
//! - [`FareTransferRuleDurationLimit`]: Defines the duration limit of the transfer.
//! - [`DurationLimitType`]: Defines the relative start and end of `FareTransferRule::duration_limit`.
//! - [`FareTransferType`]: Indicates the cost processing method of transferring between legs in a journey.

use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};
use serde_repr::*;

use super::{FareLegRuleId, FareProductId, Schema};
use crate::error::{Result, SchemaValidationError};

/// Defines the duration limit of the transfer.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum FareTransferRuleDurationLimit {
    /// The transfer has a limit of the specified number of seconds.
    WithLimit(NonZeroUsize),
    /// The transfer has no time limit.
    Unlimited,
}

/// Defines the relative start and end of [`FareTransferRule::duration_limit`].
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum DurationLimitType {
    /// Between the departure fare validation of the current leg and the arrival fare validation of the next leg.
    BetweenDepartureAndArrival = 0,
    /// Between the departure fare validation of the current leg and the departure fare validation of the next leg.
    BetweenDepartureAndDeparture = 1,
    /// Between the arrival fare validation of the current leg and the departure fare validation of the next leg.
    BetweenArrivalAndDeparture = 2,
    /// Between the arrival fare validation of the current leg and the arrival fare validation of the next leg.
    BetweenArrivalAndArrival = 3,
}

/// Indicates the cost processing method of transferring between legs in a journey.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum FareTransferType {
    /// From-leg [`crate::schemas::fare_leg_rule::FareLegRule::fare_product_id`]
    /// plus [`FareTransferRule::fare_product_id`]; A + AB.
    FromLegPlusTransfer = 0,
    /// From-leg [`crate::schemas::fare_leg_rule::FareLegRule::fare_product_id`]
    /// plus [`FareTransferRule::fare_product_id`] plus to-leg
    /// [`crate::schemas::fare_leg_rule::FareLegRule::fare_product_id`]; A + AB + B.
    FromLegPlusTransferPlusToLeg = 1,
    /// [`FareTransferRule::fare_product_id`]; AB.
    TransferOnly = 2,
}

/// Represents a fare transfer rule.
///
/// Fare rules for transfers between legs of travel defined in [`crate::schemas::fare_leg_rule::FareLegRule`].
///
/// See [fare_transfer_rules.txt](https://gtfs.org/schedule/reference/#fare_transfer_rulestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FareTransferRule {
    /// Identifies a group of pre-transfer fare leg rules.
    pub from_leg_group_id: Option<FareLegRuleId>,
    /// Identifies a group of post-transfer fare leg rules.
    pub to_leg_group_id: Option<FareLegRuleId>,
    /// Defines how many consecutive transfers the transfer rule may be applied to.
    pub transfer_count: Option<i32>,
    /// Defines the duration limit of the transfer.
    pub duration_limit: Option<FareTransferRuleDurationLimit>,
    /// Defines the relative start and end of [`FareTransferRule::duration_limit`].
    pub duration_limit_type: Option<DurationLimitType>,
    /// Indicates the cost processing method of transferring between legs in a journey.
    pub fare_transfer_type: FareTransferType,
    /// The fare product required to transfer between two fare legs. If empty, the cost of the transfer rule is 0.
    pub fare_product_id: Option<FareProductId>,
}

impl FareTransferRule {
    /// Validates if the FareTransferRule is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate transfer_count.
        if self.transfer_count.is_some() && self.from_leg_group_id != self.to_leg_group_id {
            return Err(SchemaValidationError::new_invalid_value(
                "transfer_count".to_string(),
                Some("from_leg_group_id and to_leg_group_id are not equal".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.transfer_count.is_none() && self.from_leg_group_id == self.to_leg_group_id {
            return Err(SchemaValidationError::new_forbidden_value(
                "transfer_count".to_string(),
                Some("from_leg_group_id and to_leg_group_id are equal".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate duration_limit_type.
        if self.duration_limit_type.is_some() && self.duration_limit.is_none() {
            return Err(SchemaValidationError::new_missing_value(
                "duration_limit".to_string(),
                Some("duration_limit_type is defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate duration_limit.
        if self.duration_limit.is_some() && self.duration_limit_type.is_none() {
            return Err(SchemaValidationError::new_missing_value(
                "duration_limit_type".to_string(),
                Some("duration_limit is defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
