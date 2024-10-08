//! Provides data structures and enumerations related to stops.
//!
//! The main types are:
//! - [`Stop`]: Represents a stop where vehicles pick up or drop off riders.
//! - [`StopId`]: Identifies a location: stop/platform, station, entrance/exit, generic node or boarding area.
//! - [`WheelchairBoarding`]: Indicates whether wheelchair boardings are possible from the location.

use chrono_tz::Tz;
use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use serde_with::skip_serializing_none;

use crate::error::{Result, SchemaValidationError};

use super::{coord_type, GtfsCoord, LevelId, LocationType, Schema};

/// Identifies a location: stop/platform, station, entrance/exit, generic node or boarding area.
///
/// ID must be unique across all [`Stop::stop_id`], locations.geojson id,
/// and [`crate::schemas::location_group::LocationGroup::location_group_id`] values.
///
/// Multiple routes may use the same [`StopId`].
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct StopId(pub String);

/// Indicates whether wheelchair boardings are possible from the location.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum WheelchairBoarding {
    /// For parentless stops: No accessibility information for the stop.
    /// For child stops: Stop will inherit its [`WheelchairBoarding`] behavior
    /// from the parent station, if specified in the parent.
    /// For station entrances/exits: Station entrance will inherit its [`WheelchairBoarding`]
    /// behavior from the parent station, if specified for the parent.
    NoInformation = 0,
    /// For parentless stops: Some vehicles at this stop can be boarded by a rider in a wheelchair.
    /// For child stops: There exists some accessible path from outside the station to the specific stop/platform.
    /// For station entrances/exits: Station entrance is wheelchair accessible.
    SomeAccessibility = 1,
    /// For parentless stops: Wheelchair boarding is not possible at this stop.
    /// For child stops: There exists no accessible path from outside the station to the specific stop/platform.
    /// For station entrances/exits: No accessible path from station entrance to stops/platforms.
    NoAccessibility = 2,
}

/// Represents a stop where vehicles pick up or drop off riders.
///
/// See [stops.txt](https://gtfs.org/schedule/reference/#stopstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Stop {
    /// Identifies a location: stop/platform, station, entrance/exit, generic node or
    /// boarding area (see [`LocationType`]).
    ///
    /// ID must be unique across all [`Stop::stop_id`], locations.geojson id, and
    /// [`crate::schemas::location_group::LocationGroup::location_group_id`] values.
    ///
    /// Multiple routes may use the same [`StopId`].
    pub stop_id: StopId,
    /// Short text or a number that identifies the location for riders. These codes are often
    /// used in phone-based transit information systems or printed on signage to make it easier
    /// for riders to get information for a particular location. The [`Stop::stop_code`] may
    /// be the same as [`Stop::stop_id`] if it is public facing. This field should be left empty
    /// for locations without a code presented to riders.
    pub stop_code: Option<String>,
    /// Name of the location. The [`Stop::stop_name`] should match the agency's rider-facing name for
    /// the location as printed on a timetable, published online, or represented on signage. For
    /// translations into other languages, use [`crate::schemas::translation::Translation`].
    ///
    /// When the location is a boarding area ([`LocationType::BoardingArea`]), the [`Stop::stop_name`]
    /// should contain the name of the boarding area as displayed by the agency. It could be just
    /// one letter (like on some European intercity railway stations), or text like "Wheelchair boarding area"
    /// (NYC's Subway) or "Head of short trains" (Paris' RER).
    ///
    /// **Conditionally Required:**
    /// - Required for locations which are stops ([`LocationType::StopOrPlatform`]), stations
    /// ([`LocationType::Station`]) or entrances/exits ([`LocationType::EntranceOrExit`]).
    /// - Optional for locations which are generic nodes ([`LocationType::GenericNode`]) or boarding
    /// areas ([`LocationType::BoardingArea`]).
    pub stop_name: Option<String>,
    /// Readable version of the [`Stop::stop_name`]. See "Text-to-speech field" in the Term Definitions for more.
    pub tts_stop_name: Option<String>,
    /// Description of the location that provides useful, quality information. Should not be a duplicate of [`Stop::stop_name`].
    pub stop_desc: Option<String>,
    /// Geographic coordinate of the location.
    ///
    /// For stops/platforms ([`LocationType::StopOrPlatform`]) and boarding area
    /// ([`LocationType::BoardingArea`]), the coordinates must be the ones of the
    /// bus pole — if exists — and otherwise of where the travelers are boarding
    /// the vehicle (on the sidewalk or the platform, and not on the roadway or the
    /// track where the vehicle stops).
    ///
    /// **Conditionally Required:**
    /// - Required for locations which are stops ([`LocationType::StopOrPlatform`]),
    /// stations ([`LocationType::Station`]) or entrances/exits ([`LocationType::EntranceOrExit`]).
    /// - Optional for locations which are generic nodes ([`LocationType::GenericNode`])
    /// or boarding areas ([`LocationType::BoardingArea`]).
    #[serde(flatten)]
    pub stop_coord: Option<GtfsCoord<{ coord_type::STOP }>>,
    /// Identifies the fare zone for a stop. If this record represents a station or
    /// station entrance, the [`Stop::zone_id`] is ignored.
    pub zone_id: Option<String>,
    /// URL of a web page about the location. This should be different from the
    /// [`crate::schemas::agency::Agency::agency_url`] and the
    /// [`crate::schemas::route::Route::route_url`] field values.
    pub stop_url: Option<String>,
    /// Location type.
    pub location_type: Option<LocationType>,
    /// Defines hierarchy between the different locations defined in [`Stop`].
    /// It contains the ID of the parent location, as followed:
    ///
    /// - Stop/platform ([`LocationType::StopOrPlatform`]): the [`Stop::parent_station`]
    /// field contains the ID of a station.
    /// - Station ([`LocationType::Station`]): this field must be empty.
    /// - Entrance/exit ([`LocationType::EntranceOrExit`]) or generic node
    /// ([`LocationType::GenericNode`]): the [`Stop::parent_station`] field contains
    /// the ID of a station ([`LocationType::Station`]).
    /// - Boarding Area ([`LocationType::BoardingArea`]): the [`Stop::parent_station`]
    /// field contains ID of a platform.
    ///
    /// **Conditionally Required:**
    /// - Required for locations which are entrances ([`LocationType::EntranceOrExit`]),
    /// generic nodes ([`LocationType::GenericNode`]) or boarding areas ([`LocationType::BoardingArea`]).
    /// - Optional for stops/platforms ([`LocationType::StopOrPlatform`]).
    /// - Forbidden for stations ([`LocationType::Station`]).
    pub parent_station: Option<StopId>,
    /// Timezone of the location. If the location has a parent station, it inherits the
    /// parent station's timezone instead of applying its own. Stations and parentless
    /// stops with empty [`Stop::stop_timezone`] inherit the timezone specified by
    /// [`crate::schemas::agency::Agency::agency_timezone`]. The times provided in
    /// [`crate::schemas::stop_time::StopTime`] are in the timezone specified by
    /// [`crate::schemas::agency::Agency::agency_timezone`], not [`Stop::stop_timezone`].
    /// This ensures that the time values in a trip always increase over the course of a
    /// trip, regardless of which timezones the trip crosses.
    pub stop_timezone: Option<Tz>,
    /// Indicates whether wheelchair boardings are possible from the location.
    pub wheelchair_boarding: Option<WheelchairBoarding>,
    /// Level of the location. The same level may be used by multiple unlinked stations.
    pub level_id: Option<LevelId>,
    /// Platform identifier for a platform stop (a stop belonging to a station). This should
    /// be just the platform identifier (eg. "G" or "3"). Words like "platform" or "track"
    /// (or the feed's language-specific equivalent) should not be included. This allows
    /// feed consumers to more easily internationalize and localize the platform identifier into other languages.
    pub platform_code: Option<String>,
}

impl Stop {
    /// Validates if the Stop is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate stop_name based on location_type.
        match self.location_type {
            Some(LocationType::StopOrPlatform)
            | Some(LocationType::Station)
            | Some(LocationType::EntranceOrExit) => {
                if self.stop_name.is_none() {
                    return Err(SchemaValidationError::new_missing_value(
                        "stop_name".to_string(),
                        Some(
                            "LocationType is StopOrPlatform, Station or EntranceOrExit".to_string(),
                        ),
                        Schema::from(self.clone()),
                    )
                    .into());
                }
            }
            _ => {}
        }

        // Validate stop_lat, stop_lon based on location_type.
        match self.location_type {
            Some(LocationType::StopOrPlatform)
            | Some(LocationType::Station)
            | Some(LocationType::EntranceOrExit) => {
                if self.stop_coord.is_none() {
                    return Err(SchemaValidationError::new_missing_value(
                        "stop_coord".to_string(),
                        Some(
                            "LocationType is StopOrPlatform, Station or EntranceOrExit".to_string(),
                        ),
                        Schema::from(self.clone()),
                    )
                    .into());
                }
            }
            _ => {}
        }

        // Validate parent_station based on location_type.
        match self.location_type {
            Some(LocationType::EntranceOrExit)
            | Some(LocationType::GenericNode)
            | Some(LocationType::BoardingArea) => {
                if self.parent_station.is_none() {
                    return Err(SchemaValidationError::new_missing_value(
                        "parent_station".to_string(),
                        Some(
                            "LocationType is EntranceOrExit, GenericNode or BoardingArea"
                                .to_string(),
                        ),
                        Schema::from(self.clone()),
                    )
                    .into());
                }
            }
            Some(LocationType::Station) => {
                if self.parent_station.is_some() {
                    return Err(SchemaValidationError::new_forbidden_value(
                        "parent_station".to_string(),
                        Some("LocationType is Station".to_string()),
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
