//! Provides data structures related to feed information.
//!
//! The main type is:
//! - [`FeedInfo`]: Represents dataset metadata.

use chrono::NaiveDate;
use email_address::EmailAddress;
use oxilangtag::LanguageTag;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

use super::{deserialize_optional_date, serialize_optional_date, Schema};
use crate::error::{Result, SchemaValidationError};

/// Represents dataset metadata.
///
/// The file contains information about the dataset itself, rather than the services
/// that the dataset describes. In some cases, the publisher of the dataset is a different entity than any of the agencies.
///
/// See [feed_info.txt](https://gtfs.org/schedule/reference/#feed_infotxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct FeedInfo {
    /// Full name of the organization that publishes the dataset. This may be
    /// the same as one of the [`crate::schemas::agency::Agency::agency_name`] values.
    pub feed_publisher_name: String,
    /// URL of the dataset publishing organization's website. This may be the same
    /// as one of the [`crate::schemas::agency::Agency::agency_url`] values.
    pub feed_publisher_url: Url,
    /// Default language used for the text in this dataset. This setting helps GTFS consumers
    /// choose capitalization rules and other language-specific settings for the dataset. The file [`crate::schemas::translation::Translation`] can be used if the text needs to be translated into languages other than the default one.
    ///
    /// The default language may be multilingual for datasets with the original text in
    /// multiple languages. In such cases, the [`FeedInfo::feed_lang`] field should contain
    /// the language code `mul` defined by the norm ISO 639-2, and a translation for each
    /// language used in the dataset should be provided in [`crate::schemas::translation::Translation`].
    /// If all the original text in the dataset is in the same language, then `mul` should not be used.
    ///
    /// Example: Consider a dataset from a multilingual country like Switzerland, with the
    /// original [`crate::schemas::stop::Stop::stop_name`] field populated with stop names
    /// in different languages. Each stop name is written according to the dominant language
    /// in that stop's geographic location, e.g. "Genève" for the French-speaking city of Geneva,
    /// "Zürich" for the German-speaking city of Zurich, and "Biel/Bienne" for the bilingual city
    /// of Biel/Bienne. The dataset [`FeedInfo::feed_lang`] should be `mul` and translations would
    /// be provided in [`crate::schemas::translation::Translation`], in German: "Genf", "Zürich" and
    /// "Biel"; in French: "Genève", "Zurich" and "Bienne"; in Italian: "Ginevra", "Zurigo" and
    /// "Bienna"; and in English: "Geneva", "Zurich" and "Biel/Bienne".
    pub feed_lang: LanguageTag<String>,
    /// Defines the language that should be used when the data consumer doesn't know the
    /// language of the rider. It will often be `en` (English).
    pub default_lang: Option<LanguageTag<String>>,
    /// The dataset provides complete and reliable schedule information for service in the
    /// period from the beginning of the [`FeedInfo::feed_start_date`] day to the end of
    /// the [`FeedInfo::feed_end_date`] day. Both days may be left empty if unavailable.
    /// The [`FeedInfo::feed_end_date`] date must not precede the [`FeedInfo::feed_start_date`]
    /// date if both are given. It is recommended that dataset providers give schedule data
    /// outside this period to advise of likely future service, but dataset consumers should
    /// treat it mindful of its non-authoritative status. If [`FeedInfo::feed_start_date`] or
    /// [`FeedInfo::feed_end_date`] extend beyond the active calendar dates defined in
    /// [`crate::schemas::calendar::Calendar`] and [`crate::schemas::calendar_date::CalendarDate`],
    /// the dataset is making an explicit assertion that there is no service for dates within
    /// the [`FeedInfo::feed_start_date`] or [`FeedInfo::feed_end_date`] range but not included
    /// in the active calendar dates.
    #[serde(
        serialize_with = "serialize_optional_date",
        deserialize_with = "deserialize_optional_date",
        default
    )]
    pub feed_start_date: Option<NaiveDate>,
    /// See the description for [`FeedInfo::feed_start_date`].
    #[serde(
        serialize_with = "serialize_optional_date",
        deserialize_with = "deserialize_optional_date",
        default
    )]
    pub feed_end_date: Option<NaiveDate>,
    /// String that indicates the current version of their GTFS dataset. GTFS-consuming applications
    /// can display this value to help dataset publishers determine whether the latest dataset has been incorporated.
    pub feed_version: Option<String>,
    /// Email address for communication regarding the GTFS dataset and data publishing practices.
    /// [`FeedInfo::feed_contact_email`] is a technical contact for GTFS-consuming applications.
    /// Provide customer service contact information through [`crate::schemas::agency::Agency`].
    /// It's recommended that at least one of [`FeedInfo::feed_contact_email`] or
    /// [`FeedInfo::feed_contact_url`] are provided.
    pub feed_contact_email: Option<EmailAddress>,
    /// URL for contact information, a web-form, support desk, or other tools for communication
    /// regarding the GTFS dataset and data publishing practices. [`FeedInfo::feed_contact_url`]
    /// is a technical contact for GTFS-consuming applications. Provide customer service contact
    /// information through [`crate::schemas::agency::Agency`]. It's recommended that at least
    /// one of [`FeedInfo::feed_contact_url`] or [`FeedInfo::feed_contact_email`] are provided.
    pub feed_contact_url: Option<Url>,
}

impl FeedInfo {
    /// Validates if the FeedInfo is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate feed_publisher_name.
        if self.feed_publisher_name.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "feed_publisher_name".to_string(),
                Some("must be non-empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate feed_start_date and feed_end_date.
        if let (Some(start_date), Some(end_date)) = (self.feed_start_date, self.feed_end_date) {
            if start_date > end_date {
                return Err(SchemaValidationError::new_invalid_value(
                    "feed_start_date or feed_end_date".to_string(),
                    Some("start_date cannot be greater than end_date".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        Ok(())
    }
}
