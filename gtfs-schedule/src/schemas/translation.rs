//! Provides data structures related to translations.
//!
//! The main type is:
//! - [`Translation`]: Represents a translation.

use oxilangtag::LanguageTag;
use serde::{Deserialize, Serialize};

use super::Schema;
use crate::error::{Result, SchemaValidationError};

/// Defines the table that contains the field to be translated.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableName {
    Agency,
    Stops,
    Routes,
    Trips,
    StopTimes,
    Pathways,
    Levels,
    FeedInfo,
    Attributions,
    // Tables below shouldn't be used in translations
    // but are included here as "producers sometimes add extra fields
    // that are outside the official specification and these
    // unofficial fields may be translated" (c.f. https://gtfs.org/schedule/reference/#translationstxt)
    Calendar,
    CalendarDates,
    FareAttributes,
    FareRules,
    Shapes,
    Frequencies,
    Transfers,
}

/// Represents a translation.
///
/// In regions that have multiple official languages, transit agencies/operators
/// typically have language-specific names and web pages. In order to best serve
/// riders in those regions, it is useful for the dataset to include these language-dependent values.
///
/// If both referencing methods ([`Translation::record_id`], [`Translation::record_sub_id`])
/// and [`Translation::field_value`] are used to translate the same value in 2 different rows,
/// the translation provided with ([`Translation::record_id`], [`Translation::record_sub_id`]) takes precedence.
///
/// See [translations.txt](https://gtfs.org/schedule/reference/#translationstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Translation {
    /// Defines the table that contains the field to be translated.
    ///
    /// Any file added to GTFS will have a [`Translation::table_name`] value equivalent to the
    /// file name, as listed below (i.e., not including the .txt file extension).
    pub table_name: TableName,
    /// Name of the field to be translated. Fields with type Text may be translated, fields
    /// with type URL, Email and Phone number may also be "translated" to provide resources
    /// in the correct language. Fields with other types should not be translated.
    pub field_name: String,
    /// Language of translation.
    ///
    /// If the language is the same as in [`crate::schemas::feed_info::FeedInfo::feed_lang`],
    /// the original value of the field will be assumed to be the default value to use in
    /// languages without specific translations (if [`Translation::default_lang`] doesn't specify otherwise).
    ///
    /// Example: In Switzerland, a city in an officially bilingual canton is officially called
    /// "Biel/Bienne", but would simply be called "Bienne" in French and "Biel" in German.
    pub language: LanguageTag<String>,
    /// Translated value.
    pub translation: String,
    /// Defines the record that corresponds to the field to be translated. The value in
    /// [`Translation::record_id`] must be the first or only field of a table's primary key,
    /// as defined in the primary key attribute for each table and below:
    ///
    /// - [`crate::schemas::agency::AgencyId`] for [`crate::schemas::agency::Agency`]
    /// - [`crate::schemas::stop::StopId`] for [`crate::schemas::stop::Stop`]
    /// - [`crate::schemas::route::RouteId`] for [`crate::schemas::route::Route`]
    /// - [`crate::schemas::trip::TripId`] for [`crate::schemas::trip::Trip`]
    /// - [`crate::schemas::trip::TripId`] for [`crate::schemas::stop_time::StopTime`]
    /// - [`crate::schemas::pathway::PathwayId`] for [`crate::schemas::pathway::Pathway`]
    /// - [`crate::schemas::level::LevelId`] for [`crate::schemas::level::Level`]
    /// - [`crate::schemas::attribution::AttributionId`] for [`crate::schemas::attribution::Attribution`]
    ///
    /// Fields in tables not defined above should not be translated. However, producers
    /// sometimes add extra fields that are outside the official specification, and these
    /// unofficial fields may be translated. Below is the recommended way to use [`Translation::record_id`] for those tables:
    ///
    /// - [`crate::schemas::calendar::CalendarServiceId`] for [`crate::schemas::calendar::Calendar`]
    /// - [`crate::schemas::calendar::CalendarServiceId`] for [`crate::schemas::calendar_date::CalendarDate`]
    /// - [`crate::schemas::fare_attribute::FareId`] for [`crate::schemas::fare_attribute::FareAttribute`]
    /// - [`crate::schemas::fare_attribute::FareId`] for [`crate::schemas::fare_rule::FareRule`]
    /// - [`crate::schemas::shape::ShapeId`] for [`crate::schemas::shape::Shape`]
    /// - [`crate::schemas::trip::TripId`] for [`crate::schemas::frequency::Frequency`]
    /// - [`Transfer::from_stop_id`] for [`crate::schemas::transfer::Transfer`]
    ///
    /// **Conditionally Required:**
    /// - Forbidden if [`Translation::table_name`] is [`crate::schemas::feed_info::FeedInfo`].
    /// - Forbidden if [`Translation::field_value`] is defined.
    /// - Required if [`Translation::field_value`] is empty.
    pub record_id: Option<String>,
    /// Helps the record that contains the field to be translated when the table doesn't
    /// have a unique ID. Therefore, the value in [`Translation::record_sub_id`] is the
    /// secondary ID of the table, as defined by the table below:
    ///
    /// - None for [`crate::schemas::agency::Agency`]
    /// - None for [`crate::schemas::stop::Stop`]
    /// - None for [`crate::schemas::route::Route`]
    /// - None for [`crate::schemas::trip::Trip`]
    /// - [`crate::schemas::stop_time::StopTime::stop_sequence`] for [`crate::schemas::stop_time::StopTime`]
    /// - None for [`crate::schemas::pathway::Pathway`]
    /// - None for [`crate::schemas::level::Level`]
    /// - None for [`crate::schemas::attribution::Attribution`]
    ///
    /// Fields in tables not defined above should not be translated. However, producers
    /// sometimes add extra fields that are outside the official specification, and these
    /// unofficial fields may be translated. Below is the recommended way to use [`Translation::record_sub_id`] for those tables:
    ///
    /// - None for [`crate::schemas::calendar::Calendar`]
    /// - [`crate::schemas::calendar_date::CalendarDate::date`] for [`crate::schemas::calendar_date::CalendarDate`]
    /// - None for [`crate::schemas::fare_attribute::FareAttribute`]
    /// - [`crate::schemas::route::RouteId`] for [`crate::schemas::fare_rule::FareRule`]
    /// - None for [`crate::schemas::shape::Shape`]
    /// - [`crate::schemas::frequency::Frequency::start_time`] for [`crate::schemas::frequency::Frequency`]
    /// - [`Transfer::to_stop_id`] for [`crate::schemas::transfer::Transfer`]
    ///
    /// **Conditionally Required:**
    /// - Forbidden if [`Translation::table_name`]=`"feed_info"`.
    /// - Forbidden if [`Translation::field_value`] is defined.
    /// - Required if [`Translation::table_name`]=`"stop_times"` and [`Translation::record_id`] is defined.
    pub record_sub_id: Option<String>,
    /// Instead of defining which record should be translated by using [`Translation::record_id`]
    /// and [`Translation::record_sub_id`], this field can be used to define the value
    /// which should be translated. When used, the translation will be applied when the
    /// fields identified by [`Translation::table_name`] and [`Translation::field_name`]
    /// contain the exact same value defined in [`Translation::field_value`].
    ///
    /// The field must have exactly the value defined in [`Translation::field_value`]. If only
    /// a subset of the value matches [`Translation::field_value`], the translation won't be applied.
    ///
    /// If two translation rules match the same record (one with [`Translation::field_value`],
    /// and the other one with [`Translation::record_id`]), the rule with [`Translation::record_id`] takes precedence.
    ///
    /// **Conditionally Required:**
    /// - Forbidden if [`Translation::table_name`] is [`crate::schemas::feed_info::FeedInfo`].
    /// - Forbidden if [`Translation::record_id`] is defined.
    /// - Required if [`Translation::record_id`] is empty.
    pub field_value: Option<String>,
}

impl Translation {
    /// Validates if the Translation is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate field_name.
        if self.field_name.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "field_name".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate translation.
        if self.translation.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "translation".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate record_id, record_sub_id, and field_value.
        if self.table_name == TableName::FeedInfo {
            if self.record_id.is_some()
                || self.record_sub_id.is_some()
                || self.field_value.is_some()
            {
                return Err(SchemaValidationError::new_forbidden_value(
                    "record_id, record_sub_id, or field_value".to_string(),
                    Some("forbidden when table_name is FeedInfo".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        } else {
            if self.record_id.is_none() && self.field_value.is_none() {
                return Err(SchemaValidationError::new_missing_value(
                    "record_id and/or field_value".to_string(),
                    Some("both required when table_name is FeedInfo".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
            if self.record_id.is_some() && self.field_value.is_some() {
                return Err(SchemaValidationError::new_forbidden_value(
                    "record_id and field_value".to_string(),
                    Some("forbidden when table_name is not FeedInfo".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
            if self.record_id.is_some()
                && self.table_name == TableName::StopTimes
                && self.record_sub_id.is_none()
            {
                return Err(SchemaValidationError::new_missing_value(
                    "record_sub_id".to_string(),
                    Some("when table_name is StopTimes and record_id is defined, record_sub_id is required".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        Ok(())
    }
}
