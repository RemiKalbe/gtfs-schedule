//! Provides data structures and enumerations related to fare media.
//!
//! The main types are:
//! - [`FareMedia`]: Represents a fare media.
//! - [`FareMediaId`]: Identifies a fare media.
//! - [`FareMediaType`]: The type of fare media.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_repr::*;

use super::Schema;
use crate::error::{Result, SchemaValidationError};

/// Identifies a fare media.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct FareMediaId(pub String);

/// The type of fare media.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum FareMediaType {
    /// Used when there is no fare media involved in purchasing or validating,
    /// such as paying cash to a driver or conductor with no physical ticket provided.
    None = 0,
    /// Physical paper ticket that allows a passenger to take either a certain number
    /// of pre-purchased trips or unlimited trips within a fixed period of time.
    Paper = 1,
    /// Physical transit card that has stored tickets, passes or monetary value.
    TransitCard = 2,
    /// cEMV (contactless Europay, Mastercard and Visa) as an open-loop token
    /// container for account-based ticketing.
    PaymentCard = 3,
    /// Mobile app that have stored virtual transit cards, tickets, passes, or monetary value.
    MobileApp = 4,
}

/// Represents a fare media.
///
/// To describe the different fare media that can be employed to use fare products.
/// Fare media are physical or virtual holders used for the representation and/or
/// validation of a fare product.
///
/// See [fare_media.txt](https://gtfs.org/schedule/reference/#fare_mediatxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FareMedia {
    /// Identifies a fare media.
    pub fare_media_id: FareMediaId,
    /// Name of the fare media.
    ///
    /// For fare media which are transit cards ([`FareMediaType::TransitCard`]) or
    /// mobile apps ([`FareMediaType::MobileApp`]), the [`FareMedia::fare_media_name`]
    /// should be included and should match the rider-facing name used by the organizations delivering them.
    pub fare_media_name: Option<String>,
    /// The type of fare media.
    pub fare_media_type: FareMediaType,
}

impl FareMedia {
    /// Validates if the FareMedia is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate fare_media_id.
        if self.fare_media_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "fare_media_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
