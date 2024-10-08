//! Provides data structures and enumerations related to trips.
//!
//! The main types are:
//! - [`Trip`]: Represents a trip.
//! - [`TripId`]: Identifies a trip.
//! - [`DirectionId`]: Indicates the direction of travel for a trip.
//! - [`WheelchairAccessible`]: Indicates wheelchair accessibility.
//! - [`BikesAllowed`]: Indicates whether bikes are allowed.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use serde_with::skip_serializing_none;

use super::{RouteId, Schema};
use crate::error::{Result, SchemaValidationError};

use super::CalendarServiceId;

/// Identifies a trip.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct TripId(pub String);

/// Indicates the direction of travel for a trip. This field should not be
/// used in routing; it provides a way to separate trips by direction when publishing time tables.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum DirectionId {
    /// Travel in one direction (e.g. outbound travel).
    OneDirection = 0,
    /// Travel in the opposite direction (e.g. inbound travel).
    OppositeDirection = 1,
}

/// Indicates wheelchair accessibility.
#[derive(Serialize, Debug, Clone)]
#[repr(u8)]
pub enum WheelchairAccessible {
    /// No accessibility information for the trip.
    NoInformation = 0,
    /// Vehicle being used on this particular trip can accommodate at least one rider in a wheelchair.
    SomeAccessibility = 1,
    /// No riders in wheelchairs can be accommodated on this trip.
    NoAccessibility = 2,
}

/// Custom deserialization is implemented for [`WheelchairAccessible`] to handle cases where no value
/// is provided. If the value is missing, it defaults to [`WheelchairAccessible::NoInformation`].
impl<'de> Deserialize<'de> for WheelchairAccessible {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Option::<u8>::deserialize(deserializer)?;
        match value {
            None | Some(0) => Ok(WheelchairAccessible::NoInformation),
            Some(1) => Ok(WheelchairAccessible::SomeAccessibility),
            Some(2) => Ok(WheelchairAccessible::NoAccessibility),
            _ => Err(serde::de::Error::custom(
                "wheelchair accessible must be 0, 1, 2 or omitted",
            )),
        }
    }
}

/// Indicates whether bikes are allowed.
#[derive(Serialize, Debug, Clone)]
pub enum BikesAllowed {
    /// No bike information for the trip.
    NoInformation = 0,
    /// Vehicle being used on this particular trip can accommodate at least one bicycle.
    SomeBikesAllowed = 1,
    /// No bicycles are allowed on this trip.
    NoBikesAllowed = 2,
}

/// Custom deserialization is implemented for [`BikesAllowed`] to handle cases where no value
/// is provided. If the value is missing, it defaults to [`BikesAllowed::NoInformation`].
impl<'de> Deserialize<'de> for BikesAllowed {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Option::<u8>::deserialize(deserializer)?;
        match value {
            None | Some(0) => Ok(BikesAllowed::NoInformation),
            Some(1) => Ok(BikesAllowed::SomeBikesAllowed),
            Some(2) => Ok(BikesAllowed::NoBikesAllowed),
            _ => Err(serde::de::Error::custom(
                "bikes allowed must be 0, 1, 2 or omitted",
            )),
        }
    }
}

/// Represents a trip.
///
/// Trips for each route. A trip is a sequence of two or more stops that occur during a specific time period.
///
/// See [trips.txt](https://gtfs.org/schedule/reference/#tripstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Trip {
    /// Identifies a route.
    pub route_id: RouteId,
    /// Identifies a set of dates when service is available for one or more routes.
    pub service_id: CalendarServiceId,
    /// Identifies a trip.
    pub trip_id: TripId,
    /// Text that appears on signage identifying the trip's destination to riders.
    /// Should be used to distinguish between different patterns of service on the same route.
    ///
    /// If the headsign changes during a trip, values for [`Trip::trip_headsign`] may be
    /// overridden by defining values in [`crate::schemas::stop_time::StopTime::stop_headsign`]
    /// for specific stop_times along the trip.
    pub trip_headsign: Option<String>,
    /// Public facing text used to identify the trip to riders, for instance, to identify
    /// train numbers for commuter rail trips. If riders do not commonly rely on trip names,
    /// [`Trip::trip_short_name`] should be empty. A [`Trip::trip_short_name`] value,
    /// if provided, should uniquely identify a trip within a service day; it should
    /// not be used for destination names or limited/express designations.
    pub trip_short_name: Option<String>,
    /// Indicates the direction of travel for a trip. This field should not be used
    /// in routing; it provides a way to separate trips by direction when publishing time tables.
    pub direction_id: Option<DirectionId>,
    /// Identifies the block to which the trip belongs. A block consists of a single
    /// trip or many sequential trips made using the same vehicle, defined by shared
    /// service days and [`Trip::block_id`]. A [`Trip::block_id`] may have trips with
    /// different service days, making distinct blocks. See the example below. To provide
    /// in-seat transfers information, transfers of [`crate::schemas::transfer::TransferType::InSeatTransfer`] should be provided instead.
    pub block_id: Option<String>,
    /// Identifies a geospatial shape describing the vehicle travel path for a trip.
    ///
    /// **Conditionally Required:**
    /// - Required if the trip has a continuous pickup or drop-off behavior defined
    ///   either in [`crate::schemas::route::Route`] or in [`crate::schemas::stop_time::StopTime`].
    /// - Optional otherwise.
    pub shape_id: Option<String>,
    /// Indicates wheelchair accessibility.
    pub wheelchair_accessible: Option<WheelchairAccessible>,
    /// Indicates whether bikes are allowed.
    pub bikes_allowed: Option<BikesAllowed>,
}

impl Trip {
    /// Validates if the Trip is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate route_id.
        if self.route_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "route_id".to_string(),
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
        // Validate trip_id.
        if self.trip_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "trip_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
