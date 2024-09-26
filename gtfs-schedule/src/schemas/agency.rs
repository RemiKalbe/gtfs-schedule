//! Provides data structures and enumerations related to transit agencies.
//!
//! The main types are:
//! - [`Agency`]: Represents a transit agency.
//! - [`AgencyId`]: Identifies a transit brand which is often synonymous with a transit agency.

use chrono_tz::Tz;
use email_address::EmailAddress;
use gtfs_schedule_macros::StringWrapper;
use oxilangtag::LanguageTag;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use super::Schema;
use crate::error::{Result, SchemaValidationError};

/// Identifies a transit brand which is often synonymous with a transit agency.
/// Note that in some cases, such as when a single agency operates multiple
/// separate services, agencies and brands are distinct.
///
/// See [agency.txt](https://gtfs.org/schedule/reference/#agencytxt) for more details.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct AgencyId(pub String);

/// Represents a transit agency.
///
/// See [agency.txt](https://gtfs.org/schedule/reference/#agencytxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Agency {
    /// Identifies a transit brand which is often synonymous with a transit agency.
    /// Note that in some cases, such as when a single agency operates multiple
    /// separate services, agencies and brands are distinct. This document uses
    /// the term "agency" in place of "brand". A dataset may contain data from
    /// multiple agencies.
    ///
    /// **Conditionally Required:**
    /// - Required when the dataset contains data for multiple transit agencies.
    /// - Recommended otherwise.
    pub agency_id: Option<AgencyId>,
    /// Full name of the transit agency.
    pub agency_name: String,
    /// URL of the transit agency.
    pub agency_url: Url,
    /// Timezone where the transit agency is located. If multiple agencies are
    /// specified in the dataset, each must have the same [`Agency::agency_timezone`].
    pub agency_timezone: Tz,
    /// Primary language used by this transit agency. Should be provided to help
    /// GTFS consumers choose capitalization rules and other language-specific
    /// settings for the dataset.
    pub agency_lang: Option<LanguageTag<String>>,
    /// A voice telephone number for the specified agency. This field is a string
    /// value that presents the telephone number as typical for the agency's service
    /// area. It may contain punctuation marks to group the digits of the number.
    /// Dialable text (for example, TriMet's "503-238-RIDE") is permitted, but the
    /// field must not contain any other descriptive text.
    pub agency_phone: Option<String>,
    /// URL of a web page that allows a rider to purchase tickets or other fare
    /// instruments for that agency online.
    pub agency_fare_url: Option<Url>,
    /// Email address actively monitored by the agency's customer service department.
    /// This email address should be a direct contact point where transit riders can
    /// reach a customer service representative at the agency.
    pub agency_email: Option<EmailAddress>,
}

impl Agency {
    /// Validates if the Agency is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate agency_id
        if let Some(agency_id) = &self.agency_id {
            if agency_id.is_empty() {
                return Err(SchemaValidationError::new_missing_value(
                    "agency_id".to_string(),
                    Some("agency_id is required".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        Ok(())
    }
}
