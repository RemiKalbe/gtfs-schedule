//! Provides data structures and enumerations related to fare products.
//!
//! The main types are:
//! - [`FareProduct`]: Represents a fare product.
//! - [`FareProductId`]: Identifies a fare product or set of fare products.

use gtfs_schedule_macros::StringWrapper;
use iso_currency::Currency;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::Schema;
use crate::{
    error::{Result, SchemaValidationError},
    schemas::fare_media::FareMediaId,
};

/// Identifies a fare product or set of fare products.
///
/// Multiple records in [`FareProduct`] may share the same [`FareProductId`], in
/// which case all records with that ID will be retrieved when referenced from another file.
///
/// Multiple records may share the same [`FareProductId`] but with different [`FareMediaId`],
/// indicating various methods available for employing the fare product, potentially at different prices.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct FareProductId(pub String);

/// Represents a fare product.
///
/// Used to describe the range of fares available for purchase by riders or taken into
/// account when computing the total fare for journeys with multiple legs, such as transfer costs.
///
/// See [fare_products.txt](https://gtfs.org/schedule/reference/#fare_productstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct FareProduct {
    /// Identifies a fare product or set of fare products.
    pub fare_product_id: FareProductId,
    /// The name of the fare product as displayed to riders.
    pub fare_product_name: Option<String>,
    /// Identifies a fare media that can be employed to use the fare product during the
    /// trip. When [`FareProduct::fare_media_id`] is empty, it is considered that the fare media is unknown.
    pub fare_media_id: Option<FareMediaId>,
    /// The cost of the fare product. May be negative to represent transfer discounts.
    /// May be zero to represent a fare product that is free.
    pub amount: f32,
    /// The currency of the cost of the fare product.
    pub currency: Currency,
}

impl FareProduct {
    /// Validates if the FareProduct is valid in regards to the GTFS specification constraints.
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
