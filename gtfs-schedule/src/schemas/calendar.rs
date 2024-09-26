//! Provides data structures and enumerations related to calendar dates.
//!
//! The main types are:
//! - [`Calendar`]: Service dates specified using a weekly schedule with start and end dates.
//! - [`CalendarServiceId`]: Identifies a set of dates when service is available for one or more routes.
//! - [`CalendarDayService`]: Indicates whether service is available on a given day of the week.

use chrono::NaiveDate;
use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_repr::*;

use super::{deserialize_date, serialize_date, Schema};
use crate::error::{Result, SchemaValidationError};

/// Identifies a set of dates when service is available for one or more routes.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct CalendarServiceId(pub String);

/// Indicates whether service is available on a given day of the week.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum CalendarDayService {
    /// Service is available for this day of the week during the entire date range.
    Available = 1,
    /// Service is not available for this day of the week during the entire date range.
    NotAvailable = 0,
}

/// Service dates specified using a weekly schedule with start and end dates.
///
/// See [calendar.txt](https://gtfs.org/schedule/reference/#calendartxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Calendar {
    /// Identifies a set of dates when service is available for one or more routes.
    pub service_id: CalendarServiceId,
    /// Indicates whether the service operates on all Mondays in the date range. Note
    /// that exceptions for particular dates may be listed in [`crate::schemas::calendar_date::CalendarDate`].
    pub monday: CalendarDayService,
    /// Functions in the same way as [`Calendar::monday`] except applies to Tuesdays.
    pub tuesday: CalendarDayService,
    /// Functions in the same way as [`Calendar::monday`] except applies to Wednesdays.
    pub wednesday: CalendarDayService,
    /// Functions in the same way as [`Calendar::monday`] except applies to Thursdays.
    pub thursday: CalendarDayService,
    /// Functions in the same way as [`Calendar::monday`] except applies to Fridays.
    pub friday: CalendarDayService,
    /// Functions in the same way as [`Calendar::monday`] except applies to Saturdays.
    pub saturday: CalendarDayService,
    /// Functions in the same way as [`Calendar::monday`] except applies to Sundays.
    pub sunday: CalendarDayService,
    /// Start service day for the service interval.
    #[serde(
        serialize_with = "serialize_date",
        deserialize_with = "deserialize_date"
    )]
    pub start_date: NaiveDate,
    /// End service day for the service interval. This service day is included in the interval.
    #[serde(
        serialize_with = "serialize_date",
        deserialize_with = "deserialize_date"
    )]
    pub end_date: NaiveDate,
}

impl Calendar {
    /// Validates if the Calendar is valid in regards to the GTFS specification constraints.
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

        // Validate start_date and end_date.
        if self.start_date > self.end_date {
            return Err(SchemaValidationError::new_invalid_value(
                "start_date".to_string(),
                Some("must be before end_date".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
