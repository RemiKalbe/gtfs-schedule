//! Provides data structures and enumerations related to attributions.
//!
//! The main types are:
//! - [`Attribution`]: Defines the attributions applied to the dataset.
//! - [`AttributionId`]: Identifies an attribution for the dataset or a subset of it.

use email_address::EmailAddress;
use gtfs_schedule_macros::StringWrapper;
use phonenumber::PhoneNumber;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use crate::error::{Result, SchemaValidationError};

use super::{AgencyId, RouteId, Schema, TripId};

/// Identifies an attribution for the dataset or a subset of it.
/// This is mostly useful for translations.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct AttributionId(pub String);

/// Defines the attributions applied to the dataset.
///
/// See [attributions.txt](https://gtfs.org/schedule/reference/#attributionstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Attribution {
    /// Identifies an attribution for the dataset or a subset of it. This is mostly useful for translations.
    pub attribution_id: Option<AttributionId>,
    /// Agency to which the attribution applies.
    ///
    /// If one [`Attribution::agency_id`], [`Attribution::route_id`], or [`Attribution::trip_id`] attribution is
    /// defined, the other ones must be empty. If none of them is specified, the attribution will apply to the whole dataset.
    pub agency_id: Option<AgencyId>,
    /// Functions in the same way as [`Attribution::agency_id`] except the attribution applies to a route. Multiple attributions may apply to the same route.
    pub route_id: Option<RouteId>,
    /// Functions in the same way as [`Attribution::agency_id`] except the attribution applies to a trip. Multiple attributions may apply to the same trip.
    pub trip_id: Option<TripId>,
    /// Name of the organization that the dataset is attributed to.
    pub organization_name: String,
    /// The role of the organization is producer.
    ///
    /// At least one of the fields [`Attribution::is_producer`], [`Attribution::is_operator`], or [`Attribution::is_authority`] should be set at `true`.
    pub is_producer: Option<bool>,
    /// Functions in the same way as [`Attribution::is_producer`] except the role of the organization is operator.
    pub is_operator: Option<bool>,
    /// Functions in the same way as [`Attribution::is_producer`] except the role of the organization is authority.
    pub is_authority: Option<bool>,
    /// URL of the organization.
    pub attribution_url: Option<Url>,
    /// Email of the organization.
    pub attribution_email: Option<EmailAddress>,
    /// Phone number of the organization.
    pub attribution_phone: Option<PhoneNumber>,
}

impl Attribution {
    /// Validates if the Attribution is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate organization_name.
        if self.organization_name.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "organization_name".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
