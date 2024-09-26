//! Provides data structures and enumerations related to calendar dates.
//!
//! The main types are:
//! - [`CalendarDate`]: Exceptions for the services defined in the [`Calendar`].
//! - [`ExceptionType`]: Indicates whether service is available on the date specified in the date field.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_repr::*;

use super::{deserialize_date, serialize_date, Schema};
use crate::{
    error::{Result, SchemaValidationError},
    schemas::calendar::CalendarServiceId,
};

/// Indicates whether service is available on the date specified in the date field.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum ExceptionType {
    /// Service has been added for the specified date.
    Added = 1,
    /// Service has been removed for the specified date.
    Removed = 2,
}

/// Exceptions for the services defined in the [`Calendar`].
///
/// See [calendar_dates.txt](https://gtfs.org/schedule/reference/#calendar_datestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CalendarDate {
    /// Identifies a set of dates when a service exception occurs for one or more routes.
    /// Each ([`CalendarDate::service_id`], [`CalendarDate::date`]) pair may only appear
    /// once in [`CalendarDate`] if using [`crate::schemas::calendar::Calendar`] and [`CalendarDate`] in conjunction.
    /// If a [`CalendarDate::service_id`] value appears in both [`crate::schemas::calendar::Calendar`] and [`CalendarDate`],
    /// the information in [`CalendarDate`] modifies the service information specified in [`crate::schemas::calendar::Calendar`].
    pub service_id: CalendarServiceId,
    /// Date when service exception occurs.
    #[serde(
        serialize_with = "serialize_date",
        deserialize_with = "deserialize_date"
    )]
    pub date: NaiveDate,
    /// Indicates whether service is available on the date specified in the [`CalendarDate::date`] field.
    pub exception_type: ExceptionType,
}

impl CalendarDate {
    /// Validates if the CalendarDate is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate service_id.
        if self.service_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "service_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
