//! Provides data structures and enumerations related to fare attributes.
//!
//! The main types are:
//! - [`FareAttribute`]: Represents fare information.
//! - [`FareId`]: Identifies a fare class.
//! - [`FarePaymentMethod`]: Indicates when the fare must be paid.
//! - [`FareTransfers`]: Indicates the number of transfers permitted on this fare.

use std::time::Duration;

use gtfs_schedule_macros::StringWrapper;
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use serde_repr::*;

use crate::{
    error::{Result, SchemaValidationError},
    schemas::{AgencyId, Schema},
};

/// Identifies a fare class.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct FareId(pub String);

/// Indicates when the fare must be paid.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum FarePaymentMethod {
    /// Fare is paid on board.
    OnBoard = 0,
    /// Fare must be paid before boarding.
    BeforeBoarding = 1,
}

/// Indicates the number of transfers permitted on this fare.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum FareTransfers {
    /// No transfers permitted on this fare.
    NoTransfers = 0,
    /// Riders may transfer once.
    OneTransfer = 1,
    /// Riders may transfer twice.
    TwoTransfers = 2,
    /// Unlimited transfers are permitted.
    UnlimitedTransfers,
}

/// Represents fare information.
///
/// See [fare_attributes.txt](https://gtfs.org/schedule/reference/#fare_attributestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FareAttribute {
    /// Identifies a fare class.
    pub fare_id: FareId,
    /// Fare price, in the unit specified by [`FareAttribute::currency_type`].
    pub price: f32,
    /// Currency used to pay the fare.
    pub currency_type: Currency,
    /// Indicates when the fare must be paid.
    pub payment_method: FarePaymentMethod,
    /// Indicates the number of transfers permitted on this fare.
    pub transfers: FareTransfers,
    /// Identifies the relevant agency for a fare.
    ///
    /// **Conditionally Required:**
    /// - Required if multiple agencies are defined in [`crate::schemas::agency::Agency`].
    /// - Recommended otherwise.
    pub agency_id: Option<AgencyId>,
    /// Length of time in seconds before a transfer expires.
    pub transfer_duration: Option<Duration>,
}

impl FareAttribute {
    /// Validates if the FareAttribute is valid in regards to the GTFS specification constraints.
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

        // Validate price.
        if self.price < 0.0 {
            return Err(SchemaValidationError::new_invalid_value(
                "price".to_string(),
                Some("cannot be negative".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
