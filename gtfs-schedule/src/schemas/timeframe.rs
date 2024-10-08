//! Provides data structures related to timeframes for fares.
//!
//! The main types are:
//! - [`Timeframe`]: Represents a timeframe for a fare.
//! - [`TimeframeGroupId`]: Identifies a timeframe or set of timeframes.

use chrono::NaiveTime;
use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{CalendarServiceId, Schema};
use crate::error::{Result, SchemaValidationError};

/// Identifies a timeframe or set of timeframes.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct TimeframeGroupId(pub String);

/// Represents a timeframe for a fare.
///
/// Used to describe fares that can vary based on the time of day, the day of the week, or a particular day in the year.
///
/// There must not be overlapping time intervals for the same [`Timeframe::timeframe_group_id`] and [`Timeframe::service_id`] values.
///
/// See [timeframes.txt](https://gtfs.org/schedule/reference/#timeframestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Timeframe {
    /// Identifies a timeframe or set of timeframes.
    pub timeframe_group_id: TimeframeGroupId,
    /// Defines the beginning of a timeframe. The interval includes the start time.
    ///
    /// Values greater than `24:00:00` are forbidden. An empty value in [`Timeframe::start_time`] is considered `00:00:00`.
    ///
    /// **Conditionally Required:**
    /// - Required if [`Timeframe::end_time`] is defined.
    /// - Forbidden otherwise.
    pub start_time: Option<NaiveTime>,
    /// Defines the end of a timeframe. The interval does not include the end time.
    ///
    /// Values greater than `24:00:00` are forbidden. An empty value in [`Timeframe::end_time`] is considered `24:00:00`.
    ///
    /// **Conditionally Required:**
    /// - Required if [`Timeframe::start_time`] is defined.
    /// - Forbidden otherwise.
    pub end_time: Option<NaiveTime>,
    /// Identifies a set of dates that a timeframe is in effect.
    pub service_id: CalendarServiceId,
}

impl Timeframe {
    /// Validates if the Timeframe is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate timeframe_group_id.
        if self.timeframe_group_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "timeframe_group_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate service_id.
        if self.service_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "service_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate start_time and end_time.
        if self.start_time.is_some() && self.end_time.is_none() {
            return Err(SchemaValidationError::new_missing_value(
                "end_time".to_string(),
                Some("start_time is defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.end_time.is_some() && self.start_time.is_none() {
            return Err(SchemaValidationError::new_missing_value(
                "start_time".to_string(),
                Some("end_time is defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.start_time.is_some() && self.end_time.is_some() && self.start_time > self.end_time {
            return Err(SchemaValidationError::new_invalid_value(
                "start_time or end_time".to_string(),
                Some("start_time cannot be greater than end_time".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
