//! Provides data structures and enumerations related to routes.
//!
//! The main types are:
//! - [`Route`]: Represents a transit route.
//! - [`RouteId`]: Identifies a route.
//! - [`RouteType`]: Indicates the type of transportation used on a route.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_repr::*;

use super::{AgencyId, ContinuousDropOff, ContinuousPickup, NetworkId, Schema};
use crate::error::{Result, SchemaValidationError};

/// Identifies a route.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct RouteId(pub String);

/// Indicates the type of transportation used on a route.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum RouteType {
    /// Tram, Streetcar, Light rail. Any light rail or street level system within a metropolitan area.
    LightRail = 0,
    /// Subway, Metro. Any underground rail system within a metropolitan area.
    Subway = 1,
    /// Rail. Used for intercity or long-distance travel.
    Rail = 2,
    /// Bus. Used for short- and long-distance bus routes.
    Bus = 3,
    /// Ferry. Used for short- and long-distance boat service.
    Ferry = 4,
    /// Cable tram. Used for street-level rail cars where the cable runs beneath the vehicle (e.g., cable car in San Francisco).
    CableTram = 5,
    /// Aerial lift, suspended cable car (e.g., gondola lift, aerial tramway).
    /// Cable transport where cabins, cars, gondolas or open chairs are suspended by means of one or more cables.
    AerialLift = 6,
    /// Funicular. Any rail system designed for steep inclines.
    Funicular = 7,
    /// Trolleybus. Electric buses that draw power from overhead wires using poles.
    Trolleybus = 11,
    /// Monorail. Railway in which the track consists of a single rail or a beam.
    Monorail = 12,
}

/// Represents a transit route.
///
/// See [routes.txt](https://gtfs.org/schedule/reference/#routestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Route {
    /// Identifies a route.
    pub route_id: RouteId,
    /// Agency for the specified route.
    ///
    /// **Conditionally Required:**
    /// - Required if multiple agencies are defined in [`crate::schemas::agency::Agency`].
    /// - Recommended otherwise.
    pub agency_id: Option<AgencyId>,
    /// Short name of a route.
    ///
    /// **Conditionally Required:**
    /// - Required if [`Route::route_long_name`] is empty.
    /// - Recommended if there is a brief service designation. This should be the commonly-known
    /// passenger name of the service, and should be no longer than 12 characters.
    pub route_short_name: Option<String>,
    /// Full name of a route.
    ///
    /// **Conditionally Required:**
    /// - Required if [`Route::route_short_name`] is empty.
    /// - Optional otherwise.
    pub route_long_name: Option<String>,
    /// Description of a route that provides useful, quality information.
    ///
    /// Should not be a duplicate of [`Route::route_short_name`] or [`Route::route_long_name`].
    ///
    /// Example: "A" trains operate between Inwood-207 St, Manhattan and Far Rockaway-Mott Avenue,
    /// Queens at all times. Also from about 6AM until about midnight, additional "A" trains operate
    /// between Inwood-207 St and Lefferts Boulevard (trains typically alternate between Lefferts Blvd and Far Rockaway).
    pub route_desc: Option<String>,
    /// Indicates the type of transportation used on a route.
    pub route_type: RouteType,
    /// URL of a web page about the particular route. Should be different from the [`crate::schemas::agency::Agency::agency_url`] value.
    pub route_url: Option<String>,
    /// Route color designation that matches public facing material. Defaults to white (`FFFFFF`)
    /// when omitted or left empty. The color difference between [`Route::route_color`] and
    /// [`Route::route_text_color`] should provide sufficient contrast when viewed on a black and white screen.
    pub route_color: Option<String>,
    /// Legible color to use for text drawn against a background of [`Route::route_color`].
    /// Defaults to black (`000000`) when omitted or left empty. The color difference between
    /// [`Route::route_color`] and [`Route::route_text_color`] should provide sufficient contrast
    /// when viewed on a black and white screen.
    pub route_text_color: Option<String>,
    /// Orders the routes in a way which is ideal for presentation to customers. Routes with
    /// smaller [`Route::route_sort_order`] values should be displayed first.
    pub route_sort_order: Option<u32>,
    /// Indicates that the rider can board the transit vehicle at any point along the vehicle's
    /// travel path as described by [`crate::schemas::shape::Shape`], on every trip of the route.
    ///
    /// Values for [`Route::continuous_pickup`] may be overridden by defining values in
    /// [`crate::schemas::stop_time::StopTime::continuous_pickup`] for specific stop_times along the route.
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden if [`crate::schemas::stop_time::StopTime::start_pickup_drop_off_window`] or
    /// [`crate::schemas::stop_time::StopTime::end_pickup_drop_off_window`] are defined for any trip of this route.
    /// - Optional otherwise.
    pub continuous_pickup: Option<ContinuousPickup>,
    /// Indicates that the rider can alight from the transit vehicle at any point along the
    /// vehicle's travel path as described by [`crate::schemas::shape::Shape`], on every trip of the route.
    ///
    /// Values for [`Route::continuous_drop_off`] may be overridden by defining values in
    /// [`crate::schemas::stop_time::StopTime::continuous_drop_off`] for specific stop_times along the route.
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden if [`crate::schemas::stop_time::StopTime::start_pickup_drop_off_window`] or
    /// [`crate::schemas::stop_time::StopTime::end_pickup_drop_off_window`] are defined for any trip of this route.
    /// - Optional otherwise.
    pub continuous_drop_off: Option<ContinuousDropOff>,
    /// Identifies a group of routes. Multiple rows in [`Route`] may have the same [`Route::network_id`].
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden if [`crate::schemas::route_network::RouteNetwork`] exists.
    /// - Optional otherwise.
    pub network_id: Option<NetworkId>,
}

impl Route {
    /// Validates if the Route is valid in regards to the GTFS specification constraints.
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

        // Validate route_short_name and route_long_name.
        if self.route_short_name.is_none() && self.route_long_name.is_none() {
            return Err(SchemaValidationError::new_missing_value(
                "route_short_name or route_long_name".to_string(),
                Some("at least one of them must be non-empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
