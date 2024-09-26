//! Provides data structures and enumerations related to pathways.
//!
//! The main types are:
//! - [`Pathway`]: Represents a pathway linking together locations within stations.
//! - [`PathwayId`]: Identifies a pathway.
//! - [`PathwayMode`]: Type of pathway between the specified (from_stop_id, to_stop_id) pair.

use std::time::Duration;

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use serde_with::{serde_as, DurationSeconds};

use super::{
    common::{deserialize_bool_as_int, serialize_bool_as_int},
    Schema, StopId,
};
use crate::error::{Result, SchemaValidationError};

/// Identifies a pathway. Used by systems as an internal identifier for the record. Must be unique in the dataset.
///
/// Different pathways may have the same values for [`Pathway::from_stop_id`] and [`Pathway::to_stop_id`].
///
/// Example: When two escalators are side-by-side in opposite directions, or when a stair set and elevator go from the same place to the same place, different [`PathwayId`] may have the same [`Pathway::from_stop_id`] and [`Pathway::to_stop_id`] values.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct PathwayId(pub String);

/// Type of pathway between the specified (from_stop_id, to_stop_id) pair.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum PathwayMode {
    /// Walkway.
    Walkway = 1,
    /// Stairs.
    Stairs = 2,
    /// Moving sidewalk/travelator.
    MovingSidewalk = 3,
    /// Escalator.
    Escalator = 4,
    /// Elevator.
    Elevator = 5,
    /// Fare gate (or payment gate): A pathway that crosses into an area of
    /// the station where proof of payment is required to cross. Fare gates may
    /// separate paid areas of the station from unpaid ones, or separate different
    /// payment areas within the same station from each other. This information
    /// can be used to avoid routing passengers through stations using shortcuts
    /// that would require passengers to make unnecessary payments, like directing
    /// a passenger to walk through a subway platform to reach a busway.
    FareGate = 6,
    /// Exit gate: A pathway exiting a paid area into an unpaid area where proof of payment is not required to cross.
    ExitGate = 7,
}

/// Represents a pathway linking together locations within stations.
///
/// See [pathways.txt](https://gtfs.org/schedule/reference/#pathwaystxt) for more details.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pathway {
    /// Identifies a pathway.
    pub pathway_id: PathwayId,
    /// Location at which the pathway begins.
    pub from_stop_id: StopId,
    /// Location at which the pathway ends.
    pub to_stop_id: StopId,
    /// Type of pathway between the specified ([`Pathway::from_stop_id`], [`Pathway::to_stop_id`]) pair.
    pub pathway_mode: PathwayMode,
    /// Indicates the direction that the pathway can be taken:
    /// - `false` - Unidirectional pathway that can only be used from [`Pathway::from_stop_id`] to [`Pathway::to_stop_id`].
    /// - `true` - Bidirectional pathway that can be used in both directions.
    ///
    /// Exit gates ([`PathwayMode::ExitGate`]) must not be bidirectional.
    #[serde(
        serialize_with = "serialize_bool_as_int",
        deserialize_with = "deserialize_bool_as_int"
    )]
    pub is_bidirectional: bool,
    /// Horizontal length in meters of the pathway from the origin location (defined in
    /// [`Pathway::from_stop_id`]) to the destination location (defined in [`Pathway::to_stop_id`]).
    ///
    /// This field is recommended for walkways ([`PathwayMode::Walkway`]), fare gates
    /// ([`PathwayMode::FareGate`]) and exit gates ([`PathwayMode::ExitGate`]).
    pub length: Option<f32>,
    /// Average time in seconds needed to walk through the pathway from the origin location
    /// (defined in [`Pathway::from_stop_id`]) to the destination location (defined in [`Pathway::to_stop_id`]).
    ///
    /// This field is recommended for moving sidewalks ([`PathwayMode::MovingSidewalk`]),
    /// escalators ([`PathwayMode::Escalator`]) and elevators ([`PathwayMode::Elevator`]).
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub traversal_time: Option<Duration>,
    /// Number of stairs of the pathway.
    ///
    /// A positive [`Pathway::stair_count`] implies that the rider walks up from [`Pathway::from_stop_id`]
    /// to [`Pathway::to_stop_id`]. And a negative [`Pathway::stair_count`] implies that the rider walks down
    /// from [`Pathway::from_stop_id`] to [`Pathway::to_stop_id`].
    ///
    /// This field is recommended for stairs ([`PathwayMode::Stairs`]).
    ///
    /// If only an estimated stair count can be provided, it is recommended to approximate 15 stairs for 1 floor.
    pub stair_count: Option<i32>,
    /// Maximum slope ratio of the pathway.
    ///
    /// This field should only be used with walkways ([`PathwayMode::Walkway`]) and moving sidewalks ([`PathwayMode::MovingSidewalk`]).
    ///
    /// Example: In the US, 0.083 (also written 8.3%) is the maximum slope ratio for hand-propelled
    /// wheelchair, which means an increase of 0.083m (so 8.3cm) for each 1m.
    pub max_slope: Option<f32>,
    /// Minimum width of the pathway in meters.
    ///
    /// This field is recommended if the minimum width is less than 1 meter.
    pub min_width: Option<f32>,
    /// Public facing text from physical signage that is visible to riders.
    ///
    /// May be used to provide text directions to riders, such as 'follow signs to '. The text
    /// in [`Pathway::signposted_as`] should appear exactly how it is printed on the signs.
    ///
    /// When the physical signage is multilingual, this field may be populated and translated
    /// following the example of [`crate::schemas::stop::Stop::stop_name`] in the field
    /// definition of [`crate::schemas::feed_info::FeedInfo::feed_lang`].
    pub signposted_as: Option<String>,
    /// Same as [`Pathway::signposted_as`], but when the pathway is used from the [`Pathway::to_stop_id`] to the [`Pathway::from_stop_id`].
    pub reversed_signposted_as: Option<String>,
}

impl Pathway {
    /// Validates if the Pathway is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate pathway_id.
        if self.pathway_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "pathway_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate from_stop_id and to_stop_id.
        if self.from_stop_id.is_empty() || self.to_stop_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "from_stop_id or to_stop_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate is_bidirectional.
        if self.pathway_mode == PathwayMode::ExitGate && self.is_bidirectional {
            return Err(SchemaValidationError::new_invalid_value(
                "is_bidirectional".to_string(),
                Some("must be false when pathway_mode is ExitGate".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate length.
        if let Some(length) = self.length {
            if length < 0.0 {
                return Err(SchemaValidationError::new_invalid_value(
                    "length".to_string(),
                    Some("must be positive".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        // Validate min_width.
        if let Some(width) = self.min_width {
            if width < 0.0 {
                return Err(SchemaValidationError::new_invalid_value(
                    "min_width".to_string(),
                    Some("must be positive".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        Ok(())
    }
}
