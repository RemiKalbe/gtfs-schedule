//! Provides data structures and enumerations related to frequency-based services.
//!
//! The main types are:
//! - [`Frequency`]: Represents a frequency-based service.
//! - [`ExactTimes`]: Indicates the type of service for a trip.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

use crate::error::{Result, SchemaValidationError};

use super::{NaiveServiceTime, Schema, TripId};

/// Indicates the type of service for a trip.
#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub enum ExactTimes {
    /// Frequency-based trips.
    Approximate = 0,
    /// Schedule-based trips with the exact same headway throughout the day.
    /// In this case the [`Frequency::end_time`] value must be greater than the last desired
    /// trip [`Frequency::start_time`] but less than the last desired trip
    /// `start_time + Frequency::headway_secs`.
    Exact = 1,
}

/// Custom deserialization is implemented for [`ExactTimes`] to handle cases where no value
/// is provided. If the value is missing it defaults to [`ExactTimes::Approximate`].
impl<'de> Deserialize<'de> for ExactTimes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Option::<u8>::deserialize(deserializer)?;
        match value {
            Some(1) => Ok(ExactTimes::Exact),
            None | Some(0) => Ok(ExactTimes::Approximate),
            _ => Err(serde::de::Error::custom(
                "exact times must be 1, 0 or omitted",
            )),
        }
    }
}

/// Represents a frequency-based service.
///
/// [`Frequency`] represents trips that operate on regular headways (time between trips).
/// This file may be used to represent two different types of service:
///
/// 1. Frequency-based service (exact_times = [`ExactTimes::Approximate`]) in which
/// service does not follow a fixed schedule throughout the day.
///    Instead, operators attempt to strictly maintain predetermined headways for trips.
///
/// 2. A compressed representation of schedule-based service (exact_times = [`ExactTimes::Exact`])
/// that has the exact same headway for trips over specified time period(s).
///    In schedule-based service operators try to strictly adhere to a schedule.
///
/// See [frequencies.txt](https://gtfs.org/schedule/reference/#frequenciestxt) for more details.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frequency {
    /// Identifies a trip to which the specified headway of service applies.
    pub trip_id: TripId,
    /// Time at which the first vehicle departs from the first stop of the trip with the specified headway.
    pub start_time: NaiveServiceTime,
    /// Time at which service changes to a different headway (or ceases) at the first stop in the trip.
    pub end_time: NaiveServiceTime,
    /// Time, in seconds, between departures from the same stop for the trip.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub headway_secs: Duration,
    /// Indicates the type of service for a trip.
    pub exact_times: Option<ExactTimes>,
}

impl Frequency {
    /// Validates if the Frequency is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        if self.trip_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "trip_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate start_time and end_time.
        if self.start_time > self.end_time {
            return Err(SchemaValidationError::new_invalid_value(
                "start_time or end_time".to_string(),
                Some("start_time cannot be greater than end_time".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate exact_times.
        if self.exact_times == Some(ExactTimes::Exact) {
            let last_start_time = self.start_time + self.headway_secs;
            if last_start_time >= self.end_time {
                return Err(SchemaValidationError::new_invalid_value(
                    "end_time".to_string(),
                    Some("the last start_time as computed from start_time + headway_secs must be less than end_time".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        Ok(())
    }
}
