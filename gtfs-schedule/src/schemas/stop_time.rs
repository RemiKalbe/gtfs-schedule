//! Provides data structures and enumerations related to stop times.
//!
//! The main types are:
//! - [`StopTime`]: Represents a stop time.
//! - [`PickupType`]: Indicates pickup method.
//! - [`DropOffType`]: Indicates drop off method.
//! - [`Timepoint`]: Indicates if arrival and departure times for a stop
//!   are strictly adhered to by the vehicle or if they are approximate and/or interpolated times.

use serde::{Deserialize, Serialize};
use serde_repr::*;

use super::{ContinuousDropOff, ContinuousPickup, NaiveServiceTime, Schema, StopId, TripId};
use crate::error::{Result, SchemaValidationError};

/// Indicates pickup method.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum PickupType {
    /// Regularly scheduled pickup.
    RegularlyScheduled = 0,
    /// No pickup available.
    NoPickupAvailable = 1,
    /// Must phone agency to arrange pickup.
    MustPhoneAgency = 2,
    /// Must coordinate with driver to arrange pickup.
    MustCoordinateWithDriver = 3,
}

/// Indicates drop off method.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum DropOffType {
    /// Regularly scheduled drop off.
    RegularlyScheduled = 0,
    /// No drop off available.
    NoDropOffAvailable = 1,
    /// Must phone agency to arrange drop off.
    MustPhoneAgency = 2,
    /// Must coordinate with driver to arrange drop off.
    MustCoordinateWithDriver = 3,
}

/// Indicates if arrival and departure times for a stop are strictly adhered to by the vehicle or if they are approximate and/or interpolated times.
#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub enum Timepoint {
    /// Times are considered approximate.
    Approximate = 0,
    /// Times are considered exact.
    Exact = 1,
}

/// Custom deserialization is implemented for [`Timepoint`] to handle cases where no value
/// is provided. If the value is missing, it defaults to `Timepoint::Exact`.
impl<'de> Deserialize<'de> for Timepoint {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Option::<u8>::deserialize(deserializer)?;
        match value {
            None | Some(1) => Ok(Timepoint::Exact),
            Some(0) => Ok(Timepoint::Approximate),
            _ => Err(serde::de::Error::custom(
                "exact times must be 1, 0 or omitted",
            )),
        }
    }
}

/// Represents a stop time.
///
/// Times that a vehicle arrives at and departs from stops for each trip.
///
/// See [stop_times.txt](https://gtfs.org/schedule/reference/#stop_timestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StopTime {
    /// Identifies a trip.
    pub trip_id: TripId,
    /// Arrival time at the stop (defined by [`StopTime::stop_id`]) for a
    /// specific trip (defined by [`StopTime::trip_id`]) in the time zone specified
    /// by [`crate::schemas::agency::Agency::agency_timezone`], not [`crate::schemas::stop::Stop::stop_timezone`].
    ///
    /// If there are not separate times for arrival and departure at a stop,
    /// [`StopTime::arrival_time`] and [`StopTime::departure_time`] should be the same.
    ///
    /// For times occurring after midnight on the service day, enter the time as a value greater than `24:00:00` in `HH:MM:SS`.
    ///
    /// If exact arrival and departure times ([`Timepoint::Exact`]) are not available,
    /// estimated or interpolated arrival and departure times ([`Timepoint::Approximate`]) should be provided.
    ///
    /// **Conditionally Required:**
    /// - Required for the first and last stop in a trip (defined by [`StopTime::stop_sequence`]).
    /// - Required for [`Timepoint::Exact`].
    /// - Forbidden when [`StopTime::start_pickup_drop_off_window`] or [`StopTime::end_pickup_drop_off_window`] are defined.
    /// - Optional otherwise.
    pub arrival_time: Option<NaiveServiceTime>,
    /// Departure time from the stop (defined by [`StopTime::stop_id`]) for a specific
    /// trip (defined by [`StopTime::trip_id`]) in the time zone specified by
    /// [`crate::schemas::agency::Agency::agency_timezone`], not [`crate::schemas::stop::Stop::stop_timezone`].
    ///
    /// If there are not separate times for arrival and departure at a stop,
    /// [`StopTime::arrival_time`] and [`StopTime::departure_time`] should be the same.
    ///
    /// For times occurring after midnight on the service day, enter the time as
    /// a value greater than `24:00:00` in `HH:MM:SS`.
    ///
    /// If exact arrival and departure times ([`Timepoint::Exact`]) are not available,
    /// estimated or interpolated arrival and departure times ([`Timepoint::Approximate`]) should be provided.
    ///
    /// **Conditionally Required:**
    /// - Required for [`Timepoint::Exact`].
    /// - Forbidden when [`StopTime::start_pickup_drop_off_window`] or [`StopTime::end_pickup_drop_off_window`] are defined.
    /// - Optional otherwise.
    pub departure_time: Option<NaiveServiceTime>,
    /// Identifies the serviced stop. All stops serviced during a trip must have a
    /// record in [`StopTime`]. Referenced locations must be stops/platforms, i.e.
    /// their [`crate::schemas::stop::Stop::location_type`] value must be `0` or
    /// empty. A stop may be serviced multiple times in the same trip, and multiple
    /// trips and routes may service the same stop.
    ///
    /// On-demand service using stops should be referenced in the sequence in
    /// which service is available at those stops. A data consumer should assume
    /// that travel is possible from one stop or location to any stop or location
    /// later in the trip, provided that the [`StopTime::pickup_type`] of each
    /// [`StopTime`] and the time constraints of each [`StopTime::start_pickup_drop_off_window`]/[`StopTime::end_pickup_drop_off_window`] do not forbid it.
    ///
    /// **Conditionally Required:**
    /// - Required if [`StopTime::location_group_id`] AND [`StopTime::location_id`] are NOT defined.
    /// - Forbidden if [`StopTime::location_group_id`] or [`StopTime::location_id`] are defined.
    pub stop_id: Option<StopId>,
    /// Identifies the serviced location group that indicates groups of stops where
    /// riders may request pickup or drop off. All location groups serviced during a
    /// trip must have a record in [`StopTime`]. Multiple trips and routes may service
    /// the same location group.
    ///
    /// On-demand service using location groups should be referenced in the sequence
    /// in which service is available at those location groups. A data consumer should
    /// assume that travel is possible from one stop or location to any stop or
    /// location later in the trip, provided that the [`StopTime::pickup_type`] of
    /// each [`StopTime`] and the time constraints of each [`StopTime::start_pickup_drop_off_window`]/[`StopTime::end_pickup_drop_off_window`] do not forbid it.
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden if [`StopTime::stop_id`] or [`StopTime::location_id`] are defined.
    pub location_group_id: Option<String>,
    /// Identifies the GeoJSON location that corresponds to serviced zone where riders
    /// may request pickup or drop off. All GeoJSON locations serviced during a trip
    /// must have a record in [`StopTime`]. Multiple trips and routes may service the same GeoJSON location.
    ///
    /// On-demand service within locations should be referenced in the sequence
    /// in which service is available in those locations. A data consumer should
    /// assume that travel is possible from one stop or location to any stop or
    /// location later in the trip, provided that the [`StopTime::pickup_type`]
    /// of each [`StopTime`] and the time constraints of each [`StopTime::start_pickup_drop_off_window`]/[`StopTime::end_pickup_drop_off_window`] do not forbid it.
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden if [`StopTime::stop_id`] or [`StopTime::location_group_id`] are defined.
    pub location_id: Option<String>,
    /// Order of stops, location groups, or GeoJSON locations for a particular trip.
    /// The values must increase along the trip but do not need to be consecutive.
    ///
    /// Example: The first location on the trip could have a [`StopTime::stop_sequence`]=`1`,
    /// the second location on the trip could have a [`StopTime::stop_sequence`]=`23`,
    /// the third location could have a [`StopTime::stop_sequence`]=`40`, and so on.
    ///
    /// Travel within the same location group or GeoJSON location requires two records
    /// in [`StopTime`] with the same [`StopTime::location_group_id`] or [`StopTime::location_id`].
    pub stop_sequence: u32,
    /// Text that appears on signage identifying the trip's destination to riders.
    /// This field overrides the default [`crate::schemas::trip::Trip::trip_headsign`]
    /// when the headsign changes between stops. If the headsign is displayed for
    /// an entire trip, [`crate::schemas::trip::Trip::trip_headsign`] should be used instead.
    ///
    /// A [`StopTime::stop_headsign`] value specified for one [`StopTime`] does
    /// not apply to subsequent [`StopTime`] in the same trip. If you want to
    /// override the [`crate::schemas::trip::Trip::trip_headsign`] for multiple
    /// [`StopTime`] in the same trip, the [`StopTime::stop_headsign`] value must
    /// be repeated in each [`StopTime`] row.
    pub stop_headsign: Option<String>,
    /// Time that on-demand service becomes available in a GeoJSON location, location group, or stop.
    ///
    /// **Conditionally Required:**
    /// - Required if [`StopTime::location_group_id`] or [`StopTime::location_id`] is defined.
    /// - Required if [`StopTime::end_pickup_drop_off_window`] is defined.
    /// - Forbidden if [`StopTime::arrival_time`] or [`StopTime::departure_time`] is defined.
    /// - Optional otherwise.
    pub start_pickup_drop_off_window: Option<String>,
    /// Time that on-demand service ends in a GeoJSON location, location group, or stop.
    ///
    /// **Conditionally Required:**
    /// - Required if [`StopTime::location_group_id`] or [`StopTime::location_id`] is defined.
    /// - Required if [`StopTime::start_pickup_drop_off_window`] is defined.
    /// - Forbidden if [`StopTime::arrival_time`] or [`StopTime::departure_time`] is defined.
    /// - Optional otherwise.
    pub end_pickup_drop_off_window: Option<String>,
    /// Indicates pickup method.
    ///
    /// **Conditionally Forbidden:**
    /// - [`PickupType::RegularlyScheduled`] forbidden if [`StopTime::start_pickup_drop_off_window`]
    ///   or [`StopTime::end_pickup_drop_off_window`] are defined.
    /// - [`PickupType::MustCoordinateWithDriver`] forbidden if [`StopTime::start_pickup_drop_off_window`]
    ///   or [`StopTime::end_pickup_drop_off_window`] are defined.
    /// - Optional otherwise.
    pub pickup_type: Option<PickupType>,
    /// Indicates drop off method.
    ///
    /// **Conditionally Forbidden:**
    /// - [`DropOffType::RegularlyScheduled`] forbidden if [`StopTime::start_pickup_drop_off_window`]
    ///   or [`StopTime::end_pickup_drop_off_window`] are defined.
    /// - Optional otherwise.
    pub drop_off_type: Option<DropOffType>,
    /// Indicates that the rider can board the transit vehicle at any point along the vehicle's
    /// travel path as described by [`crate::schemas::shape::Shape`], from this [`StopTime`] to
    /// the next [`StopTime`] in the trip's [`StopTime::stop_sequence`].
    ///
    /// If this field is populated, it overrides any continuous pickup behavior defined in
    /// [`crate::schemas::route::Route`]. If this field is empty, the [`StopTime`] inherits
    /// any continuous pickup behavior defined in [`crate::schemas::route::Route`].
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden if [`StopTime::start_pickup_drop_off_window`] or [`StopTime::end_pickup_drop_off_window`] are defined.
    /// - Optional otherwise.
    pub continuous_pickup: Option<ContinuousPickup>,
    /// Indicates that the rider can alight from the transit vehicle at any point along the
    /// vehicle's travel path as described by [`crate::schemas::shape::Shape`], from this
    /// [`StopTime`] to the next [`StopTime`] in the trip's [`StopTime::stop_sequence`].
    ///
    /// If this field is populated, it overrides any continuous drop-off behavior defined in
    /// [`crate::schemas::route::Route`]. If this field is empty, the [`StopTime`] inherits
    /// any continuous drop-off behavior defined in [`crate::schemas::route::Route`].
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden if [`StopTime::start_pickup_drop_off_window`] or [`StopTime::end_pickup_drop_off_window`] are defined.
    /// - Optional otherwise.
    pub continuous_drop_off: Option<ContinuousDropOff>,
    /// Actual distance traveled along the associated [`crate::schemas::shape::Shape`], from
    /// the first stop to the stop specified in this record. This field specifies how much of
    /// the [`crate::schemas::shape::Shape`] to draw between any two stops during a trip.
    /// Must be in the same units used in [`crate::schemas::shape::Shape`]. Values used for
    /// [`StopTime::shape_dist_traveled`] must increase along with [`StopTime::stop_sequence`];
    /// they must not be used to show reverse travel along a route.
    ///
    /// Recommended for routes that have looping or inlining (the vehicle crosses or travels
    /// over the same portion of alignment in one trip).
    ///
    /// See [`crate::schemas::shape::Shape::shape_dist_traveled`].
    ///
    /// Example: If a bus travels a distance of 5.25 kilometers from the start of the shape
    /// to the stop, [`StopTime::shape_dist_traveled`]=`5.25`.
    pub shape_dist_traveled: Option<f32>,
    /// Indicates if arrival and departure times for a stop are strictly adhered to by the
    /// vehicle or if they are instead approximate and/or interpolated times. This field
    /// allows a GTFS producer to provide interpolated stop-times, while indicating that
    /// the times are approximate. Valid options are:
    ///
    /// - [`Timepoint::Approximate`] - Times are considered approximate.
    /// - [`Timepoint::Exact`] - Times are considered exact.
    pub timepoint: Option<Timepoint>,
    /// Identifies the boarding booking rule at this stop time.
    ///
    /// Recommended when [`StopTime::pickup_type`]=[`PickupType::MustPhoneAgency`].
    pub pickup_booking_rule_id: Option<String>,
    /// Identifies the alighting booking rule at this stop time.
    ///
    /// Recommended when [`StopTime::drop_off_type`]=[`DropOffType::MustPhoneAgency`].
    pub drop_off_booking_rule_id: Option<String>,
}

impl StopTime {
    /// Validates if the StopTime is valid in regards to the GTFS specification constraints.
    pub fn validate(&mut self) -> Result<()> {
        // Validate trip_id.
        if self.trip_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "trip_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        if self.stop_id.is_some()
            && (self.location_group_id.is_some() || self.location_id.is_some())
        {
            return Err(SchemaValidationError::new_forbidden_value(
                "location_group_id and/or location_id".to_string(),
                Some("are not allowed when stop_id is defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.location_group_id.is_some() && self.location_id.is_some() {
            return Err(SchemaValidationError::new_forbidden_value(
                "location_group_id or location_id".to_string(),
                Some("cannot both be defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.stop_id.is_none() && self.location_group_id.is_none() && self.location_id.is_none()
        {
            return Err(SchemaValidationError::new_missing_value(
                "stop_id, location_group_id, or location_id".to_string(),
                Some("exactly one of them must be defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate shape_dist_traveled.
        if self.shape_dist_traveled.is_some_and(|x| x < 0.0) {
            return Err(SchemaValidationError::new_invalid_value(
                "shape_dist_traveled".to_string(),
                Some("must be positive".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate arrival_time and departure_time based on timepoint and presence of start_pickup_drop_off_window and end_pickup_drop_off_window.
        if self.start_pickup_drop_off_window.is_some() || self.end_pickup_drop_off_window.is_some()
        {
            if self.arrival_time.is_some() || self.departure_time.is_some() {
                return Err(SchemaValidationError::new_forbidden_value(
                    "arrival_time or departure_time".to_string(),
                    Some("are not allowed when start_pickup_drop_off_window or end_pickup_drop_off_window is defined".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        } else if self.timepoint == Some(Timepoint::Exact) {
            if self.arrival_time.is_none() || self.departure_time.is_none() {
                return Err(SchemaValidationError::new_missing_value(
                    "arrival_time or departure_time".to_string(),
                    Some("because Timepoint is Exact".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        // Validate pickup_type and drop_off_type based on the presence of start_pickup_drop_off_window and end_pickup_drop_off_window.
        if self.start_pickup_drop_off_window.is_some() || self.end_pickup_drop_off_window.is_some()
        {
            if matches!(
                self.pickup_type,
                Some(PickupType::RegularlyScheduled) | Some(PickupType::MustCoordinateWithDriver)
            ) {
                self.pickup_type = None;
            }
            if matches!(self.drop_off_type, Some(DropOffType::RegularlyScheduled)) {
                self.drop_off_type = None;
            }
        } else {
            // Set default values for pickup_type and drop_off_type if they are not provided.
            self.pickup_type = self
                .pickup_type
                .clone()
                .or(Some(PickupType::RegularlyScheduled));
            self.drop_off_type = self
                .drop_off_type
                .clone()
                .or(Some(DropOffType::RegularlyScheduled));
        }

        // Validate continuous_pickup and continuous_drop_off based on the presence of start_pickup_drop_off_window and end_pickup_drop_off_window.
        if self.start_pickup_drop_off_window.is_some() || self.end_pickup_drop_off_window.is_some()
        {
            if self.continuous_pickup.is_some() || self.continuous_drop_off.is_some() {
                return Err(SchemaValidationError::new_forbidden_value(
                    "continuous_pickup or continuous_drop_off".to_string(),
                    Some("are not allowed when start_pickup_drop_off_window or end_pickup_drop_off_window is defined".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        Ok(())
    }
}
