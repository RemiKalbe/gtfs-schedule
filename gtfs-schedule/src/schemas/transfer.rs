//! Provides data structures and enumerations related to transfers.
//!
//! The main types are:
//! - [`Transfer`]: Represents additional rules and overrides for selected transfers.
//! - [`TransferType`]: Indicates the type of connection for the specified (from_stop_id, to_stop_id) pair.

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{RouteId, Schema, StopId, TripId};
use crate::error::{Result, SchemaValidationError};

/// Indicates the type of connection for the specified (from_stop_id, to_stop_id) pair.
#[derive(Serialize, Debug, Clone)]
#[repr(u8)]
pub enum TransferType {
    /// Recommended transfer point between routes.
    RecommendedTransferPoint = 0,
    /// Timed transfer point between two routes. The departing vehicle is expected
    /// to wait for the arriving one and leave sufficient time for a rider to transfer between routes.
    TimedTransferPoint = 1,
    /// Transfer requires a minimum amount of time between arrival and departure to
    /// ensure a connection. The time required to transfer is specified by [`Transfer::min_transfer_time`].
    MinimumTimeTransferPoint = 2,
    /// Transfers are not possible between routes at the location.
    NoTransferPossible = 3,
    /// Passengers can transfer from one trip to another by staying onboard the same vehicle (an "in-seat transfer").
    InSeatTransfer = 4,
    /// In-seat transfers are not allowed between sequential trips. The passenger must alight from the vehicle and re-board.
    NoInSeatTransfer = 5,
}

/// Custom deserialization is implemented for [`TransferType`] to handle cases where no value
/// is provided. If the value is missing, it defaults to [`TransferType::RecommendedTransferPoint`].
impl<'de> Deserialize<'de> for TransferType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Option::<u8>::deserialize(deserializer)?;
        match value {
            None | Some(0) => Ok(TransferType::RecommendedTransferPoint),
            Some(1) => Ok(TransferType::TimedTransferPoint),
            Some(2) => Ok(TransferType::MinimumTimeTransferPoint),
            Some(3) => Ok(TransferType::NoTransferPossible),
            Some(4) => Ok(TransferType::InSeatTransfer),
            Some(5) => Ok(TransferType::NoInSeatTransfer),
            _ => Err(serde::de::Error::custom(
                "transfer type must be 0, 1, 2, 3, 4, 5 or omitted",
            )),
        }
    }
}

/// Represents additional rules and overrides for selected transfers.
///
/// When calculating an itinerary, GTFS-consuming applications interpolate
/// transfers based on allowable time and stop proximity. [`Transfer`]
/// specifies additional rules and overrides for selected transfers.
///
/// See [transfers.txt](https://gtfs.org/schedule/reference/#transferstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Transfer {
    /// Identifies a stop or station where a connection between routes begins.
    ///
    /// **Conditionally Required:**
    /// - Required for [`TransferType::NoTransferPossible`] and [`TransferType::InSeatTransfer`].
    /// - Forbidden for [`TransferType::NoInSeatTransfer`].
    pub from_stop_id: Option<StopId>,
    /// Identifies a stop or station where a connection between routes ends.
    ///
    /// **Conditionally Required:**
    /// - Required for [`TransferType::NoTransferPossible`] and [`TransferType::InSeatTransfer`].
    /// - Forbidden for [`TransferType::NoInSeatTransfer`].
    pub to_stop_id: Option<StopId>,
    /// Identifies a route where a connection begins.
    ///
    /// If [`Transfer::from_route_id`] is defined, the transfer will apply
    /// to the arriving trip on the route for the given [`Transfer::from_stop_id`].
    ///
    /// If both [`Transfer::from_trip_id`] and [`Transfer::from_route_id`]
    /// are defined, the [`Transfer::from_trip_id`] must belong to the
    /// [`Transfer::from_route_id`], and [`Transfer::from_trip_id`] will take precedence.
    pub from_route_id: Option<RouteId>,
    /// Identifies a route where a connection ends.
    ///
    /// If [`Transfer::to_route_id`] is defined, the transfer will apply to
    /// the departing trip on the route for the given [`Transfer::to_stop_id`].
    ///
    /// If both [`Transfer::to_trip_id`] and [`Transfer::to_route_id`] are
    /// defined, the [`Transfer::to_trip_id`] must belong to the
    /// [`Transfer::to_route_id`], and [`Transfer::to_trip_id`] will take precedence.
    pub to_route_id: Option<RouteId>,
    /// Identifies a trip where a connection between routes begins.
    ///
    /// If [`Transfer::from_trip_id`] is defined, the transfer will apply to
    /// the arriving trip for the given [`Transfer::from_stop_id`].
    ///
    /// If both [`Transfer::from_trip_id`] and [`Transfer::from_route_id`] are
    /// defined, the [`Transfer::from_trip_id`] must belong to the
    /// [`Transfer::from_route_id`], and [`Transfer::from_trip_id`] will take precedence.
    ///
    /// **Conditionally Required:**
    /// - Required if [`TransferType`] is [`TransferType::InSeatTransfer`] or [`TransferType::NoInSeatTransfer`].
    pub from_trip_id: Option<TripId>,
    /// Identifies a trip where a connection between routes ends.
    ///
    /// If [`Transfer::to_trip_id`] is defined, the transfer will apply to
    /// the departing trip for the given [`Transfer::to_stop_id`].
    ///
    /// If both [`Transfer::to_trip_id`] and [`Transfer::to_route_id`] are
    /// defined, the [`Transfer::to_trip_id`] must belong to the
    /// [`Transfer::to_route_id`], and [`Transfer::to_trip_id`] will take precedence.
    ///
    /// **Conditionally Required:**
    /// - Required if [`TransferType`] is [`TransferType::InSeatTransfer`] or [`TransferType::NoInSeatTransfer`].
    pub to_trip_id: Option<TripId>,
    /// Indicates the type of connection for the specified ([`Transfer::from_stop_id`], [`Transfer::to_stop_id`]) pair.
    pub transfer_type: TransferType,
    /// Amount of time, in seconds, that must be available to permit a transfer
    /// between routes at the specified stops. The [`Transfer::min_transfer_time`]
    /// should be sufficient to permit a typical rider to move between the two stops,
    /// including buffer time to allow for schedule variance on each route.
    pub min_transfer_time: Option<u32>,
}

impl Transfer {
    /// Validates if the Transfer is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        let from_stop_id_is_none_or_empty =
            self.from_stop_id.is_none() || self.from_stop_id.as_ref().unwrap().is_empty();
        let to_stop_id_is_none_or_empty =
            self.to_stop_id.is_none() || self.to_stop_id.as_ref().unwrap().is_empty();

        // Validate from_stop_id and to_stop_id.
        match self.transfer_type {
            TransferType::InSeatTransfer | TransferType::NoTransferPossible => {
                if from_stop_id_is_none_or_empty || to_stop_id_is_none_or_empty {
                    return Err(SchemaValidationError::new_missing_value(
                        "from_stop_id or to_stop_id".to_string(),
                        Some(
                            "required when transfer_type is InSeatTransfer or NoTransferPossible"
                                .to_string(),
                        ),
                        Schema::from(self.clone()),
                    )
                    .into());
                }
            }
            TransferType::NoInSeatTransfer => {
                if !from_stop_id_is_none_or_empty || !to_stop_id_is_none_or_empty {
                    return Err(SchemaValidationError::new_forbidden_value(
                        "from_stop_id or to_stop_id".to_string(),
                        Some("forbidden when transfer_type is NoInSeatTransfer".to_string()),
                        Schema::from(self.clone()),
                    )
                    .into());
                }
            }
            _ => {}
        }

        let from_trip_id_is_none_or_empty =
            self.from_trip_id.is_none() || self.from_trip_id.as_ref().unwrap().is_empty();
        let to_trip_id_is_none_or_empty =
            self.to_trip_id.is_none() || self.to_trip_id.as_ref().unwrap().is_empty();

        // Validate from_trip_id and to_trip_id.
        match self.transfer_type {
            TransferType::InSeatTransfer | TransferType::NoInSeatTransfer => {
                if from_trip_id_is_none_or_empty || to_trip_id_is_none_or_empty {
                    return Err(SchemaValidationError::new_missing_value(
                        "from_trip_id or to_trip_id".to_string(),
                        Some(
                            "required when transfer_type is InSeatTransfer or NoInSeatTransfer"
                                .to_string(),
                        ),
                        Schema::from(self.clone()),
                    )
                    .into());
                }
            }
            _ => {}
        }

        Ok(())
    }
}
