//! Provides common data structures and enumerations used across multiple modules.
//!
//! The main types are:
//! - [`LocationType`]: Indicates the type of the location.
//! - [`ContinuousPickup`]: Indicates that the rider can board the
//!   transit vehicle at any point along the vehicle's travel path as described
//!   by [`crate::schemas::shape::Shape`], on every trip of the route.
//! - [`ContinuousDropOff`]: Indicates that the rider can alight from
//!   the transit vehicle at any point along the vehicle's travel path as
//!   described by [`crate::schemas::shape::Shape`], on every trip of the route.

use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, Deref, DerefMut},
    time::Duration,
};

use chrono::{NaiveDate, NaiveTime, Timelike};
use geo::Coord;
use serde::de::{self, Error as DeError, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::*;

use crate::error::{Error, ParseError};

use super::{
    Agency, Area, Attribution, BookingRule, Calendar, CalendarDate, FareAttribute, FareLegRule,
    FareMedia, FareProduct, FareRule, FareTransferRule, FeedInfo, Frequency, Level, LocationGroup,
    LocationGroupStop, Network, Pathway, Route, RouteNetwork, Shape, Stop, StopArea, StopTime,
    Timeframe, Transfer, Translation, Trip,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Schema {
    Agency(Agency),
    Area(Area),
    Attribution(Attribution),
    BookingRule(BookingRule),
    Calendar(Calendar),
    CalendarDate(CalendarDate),
    FareAttribute(FareAttribute),
    FareLegRule(FareLegRule),
    FareMedia(FareMedia),
    FareProduct(FareProduct),
    FareRule(FareRule),
    FareTransferRule(FareTransferRule),
    FeedInfo(FeedInfo),
    Frequency(Frequency),
    Level(Level),
    LocationGroup(LocationGroup),
    LocationGroupStop(LocationGroupStop),
    Network(Network),
    Pathway(Pathway),
    Route(Route),
    RouteNetwork(RouteNetwork),
    Shape(Shape),
    Stop(Stop),
    StopArea(StopArea),
    StopTime(StopTime),
    Timeframe(Timeframe),
    Transfer(Transfer),
    Translation(Translation),
    Trip(Trip),
}

impl From<Agency> for Schema {
    fn from(agency: Agency) -> Self {
        Schema::Agency(agency)
    }
}

impl From<Area> for Schema {
    fn from(area: Area) -> Self {
        Schema::Area(area)
    }
}

impl From<Attribution> for Schema {
    fn from(attribution: Attribution) -> Self {
        Schema::Attribution(attribution)
    }
}

impl From<BookingRule> for Schema {
    fn from(booking_rule: BookingRule) -> Self {
        Schema::BookingRule(booking_rule)
    }
}

impl From<Calendar> for Schema {
    fn from(calendar: Calendar) -> Self {
        Schema::Calendar(calendar)
    }
}

impl From<CalendarDate> for Schema {
    fn from(calendar_date: CalendarDate) -> Self {
        Schema::CalendarDate(calendar_date)
    }
}

impl From<FareAttribute> for Schema {
    fn from(fare_attribute: FareAttribute) -> Self {
        Schema::FareAttribute(fare_attribute)
    }
}

impl From<FareLegRule> for Schema {
    fn from(fare_leg_rule: FareLegRule) -> Self {
        Schema::FareLegRule(fare_leg_rule)
    }
}

impl From<FareMedia> for Schema {
    fn from(fare_media: FareMedia) -> Self {
        Schema::FareMedia(fare_media)
    }
}

impl From<FareProduct> for Schema {
    fn from(fare_product: FareProduct) -> Self {
        Schema::FareProduct(fare_product)
    }
}

impl From<FareRule> for Schema {
    fn from(fare_rule: FareRule) -> Self {
        Schema::FareRule(fare_rule)
    }
}

impl From<FareTransferRule> for Schema {
    fn from(fare_transfer_rule: FareTransferRule) -> Self {
        Schema::FareTransferRule(fare_transfer_rule)
    }
}

impl From<FeedInfo> for Schema {
    fn from(feed_info: FeedInfo) -> Self {
        Schema::FeedInfo(feed_info)
    }
}

impl From<Frequency> for Schema {
    fn from(frequency: Frequency) -> Self {
        Schema::Frequency(frequency)
    }
}

impl From<Level> for Schema {
    fn from(level: Level) -> Self {
        Schema::Level(level)
    }
}

impl From<LocationGroup> for Schema {
    fn from(location_group: LocationGroup) -> Self {
        Schema::LocationGroup(location_group)
    }
}

impl From<LocationGroupStop> for Schema {
    fn from(location_group_stop: LocationGroupStop) -> Self {
        Schema::LocationGroupStop(location_group_stop)
    }
}

impl From<Network> for Schema {
    fn from(network: Network) -> Self {
        Schema::Network(network)
    }
}

impl From<Pathway> for Schema {
    fn from(pathway: Pathway) -> Self {
        Schema::Pathway(pathway)
    }
}

impl From<Route> for Schema {
    fn from(route: Route) -> Self {
        Schema::Route(route)
    }
}

impl From<RouteNetwork> for Schema {
    fn from(route_network: RouteNetwork) -> Self {
        Schema::RouteNetwork(route_network)
    }
}

impl From<Shape> for Schema {
    fn from(shape: Shape) -> Self {
        Schema::Shape(shape)
    }
}

impl From<Stop> for Schema {
    fn from(stop: Stop) -> Self {
        Schema::Stop(stop)
    }
}

impl From<StopArea> for Schema {
    fn from(stop_area: StopArea) -> Self {
        Schema::StopArea(stop_area)
    }
}

impl From<StopTime> for Schema {
    fn from(stop_time: StopTime) -> Self {
        Schema::StopTime(stop_time)
    }
}

impl From<Timeframe> for Schema {
    fn from(timeframe: Timeframe) -> Self {
        Schema::Timeframe(timeframe)
    }
}

impl From<Transfer> for Schema {
    fn from(transfer: Transfer) -> Self {
        Schema::Transfer(transfer)
    }
}

impl From<Translation> for Schema {
    fn from(translation: Translation) -> Self {
        Schema::Translation(translation)
    }
}

impl From<Trip> for Schema {
    fn from(trip: Trip) -> Self {
        Schema::Trip(trip)
    }
}

/// Indicates the type of the location.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum LocationType {
    /// A location where passengers board or disembark from a transit vehicle.
    /// Is called a platform when defined within a [`LocationType::Station`].
    StopOrPlatform = 0,
    /// A physical structure or area that contains one or more [`LocationType::StopOrPlatform`].
    Station = 1,
    /// A location where passengers can enter or exit a station from the street.
    /// If an entrance/exit belongs to multiple stations, it may be linked by
    /// pathways to both, but the data provider must pick one of them as parent.
    EntranceOrExit = 2,
    /// A location within a station, not matching any other [`LocationType`],
    /// that may be used to link together pathways defined in [`crate::schemas::pathway::Pathway`].
    GenericNode = 3,
    /// A specific location on a platform, where passengers can board and/or alight vehicles.
    BoardingArea = 4,
}

/// Indicates that the rider can board the transit vehicle at any point along
/// the vehicle's travel path as described by [`crate::schemas::shape::Shape`], on every trip of the route.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum ContinuousPickup {
    /// Continuous stopping pickup.
    ContinuousStopping = 0,
    /// No continuous stopping pickup.
    NoContinuousStopping = 1,
    /// Must phone agency to arrange continuous stopping pickup.
    PhoneAgencyToArrange = 2,
    /// Must coordinate with driver to arrange continuous stopping pickup.
    CoordinateWithDriver = 3,
}

/// Indicates that the rider can alight from the transit vehicle at any point along
/// the vehicle's travel path as described by [`crate::schemas::shape::Shape`], on every trip of the route.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum ContinuousDropOff {
    /// Continuous stopping drop off.
    ContinuousStopping = 0,
    /// No continuous stopping drop off.
    NoContinuousStopping = 1,
    /// Must phone agency to arrange continuous stopping drop off.
    PhoneAgencyToArrange = 2,
    /// Must coordinate with driver to arrange continuous stopping drop off.
    CoordinateWithDriver = 3,
}

/// Represents a time value in the GTFS format, allowing for times beyond 24 hours.
///
/// In GTFS, time values can exceed 24 hours to represent service days that go beyond midnight.
/// For example, "25:35:00" represents 1:35 AM on the next service day.
///
/// The `NaiveServiceTime` struct contains a `NaiveTime` value representing the actual time,
/// and an `overflow` flag indicating whether the time exceeds 24 hours.
/// For example, if the `time` field is 1:35 AM and the `overflow` flag is `true`,
/// the actual time (in the GTFS format) would be "25:35:00".
#[derive(Debug, Clone, Copy, Hash)]
pub struct NaiveServiceTime {
    pub time: NaiveTime,
    pub overflow: bool,
}

impl TryFrom<&str> for NaiveServiceTime {
    type Error = Error;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let parts: Vec<_> = s.split(':').collect();
        let hours: u32 = parts[0].parse().map_err(ParseError::from)?;
        let minutes: u32 = parts[1].parse().map_err(ParseError::from)?;
        let seconds: u32 = parts[2].parse().map_err(ParseError::from)?;

        let overflow = hours >= 24;
        let time = NaiveTime::from_hms_opt(hours % 24, minutes, seconds)
            .ok_or_else(|| ParseError::InvalidValue(format!("Invalid time: {}", s)))?;

        Ok(NaiveServiceTime { time, overflow })
    }
}

impl From<NaiveServiceTime> for String {
    fn from(service_time: NaiveServiceTime) -> String {
        let overflow_time = if service_time.overflow {
            service_time.time
        } else {
            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
        };
        let (hours_overflow, minutes_overflow, seconds_overflow) = (
            overflow_time.hour(),
            overflow_time.minute(),
            overflow_time.second(),
        );
        let (hours, minutes, seconds) = (
            service_time.time.hour(),
            service_time.time.minute(),
            service_time.time.second(),
        );
        let hours = hours + hours_overflow;
        let minutes = minutes + minutes_overflow;
        let seconds = seconds + seconds_overflow;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl PartialEq for NaiveServiceTime {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.overflow == other.overflow
    }
}

impl Eq for NaiveServiceTime {}

impl PartialOrd for NaiveServiceTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NaiveServiceTime {
    fn cmp(&self, other: &Self) -> Ordering {
        let (self_h, self_m, self_s) = (self.time.hour(), self.time.minute(), self.time.second());
        let (other_h, other_m, other_s) =
            (other.time.hour(), other.time.minute(), other.time.second());

        if self_h < other_h {
            return Ordering::Less;
        } else if self_h > other_h {
            return Ordering::Greater;
        }

        if self_m < other_m {
            return Ordering::Less;
        } else if self_m > other_m {
            return Ordering::Greater;
        }

        if self_s < other_s {
            return Ordering::Less;
        } else if self_s > other_s {
            return Ordering::Greater;
        }

        Ordering::Equal
    }
}

impl Add<Duration> for NaiveServiceTime {
    type Output = NaiveServiceTime;

    fn add(self, rhs: Duration) -> Self::Output {
        let (self_h, self_m, self_s) = (self.time.hour(), self.time.minute(), self.time.second());
        let self_as_secs = self_h * 3600 + self_m * 60 + self_s;
        let add = u32::try_from(rhs.as_secs()).unwrap() + self_as_secs;
        let (hours, minutes, seconds) = (add / 3600, add / 60 % 60, add % 60);
        let overflow = hours >= 24;
        let time = NaiveTime::from_hms_opt(hours % 24, minutes, seconds)
            .expect(format!("Could not generate a valid chrono::NaiveTime from the addition of NaiveServiceTime and Duration: {:?} + {:?}; this may be a bug in the library", self, rhs).as_str());

        NaiveServiceTime { time, overflow }
    }
}

impl<'de> Deserialize<'de> for NaiveServiceTime {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveServiceTime::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl Serialize for NaiveServiceTime {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: String = (*self).into();
        serializer.serialize_str(s.as_str())
    }
}

/// Custom serialization function for NaiveDate
pub fn serialize_date<S>(date: &NaiveDate, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date_string = date.format("%Y%m%d").to_string();
    serializer.serialize_str(&date_string)
}

/// Custom deserialization function for NaiveDate
pub fn deserialize_date<'de, D>(deserializer: D) -> std::result::Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let date_string = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&date_string, "%Y%m%d")
        .map_err(|err| DeError::custom(format!("Invalid date format: {}", err)))
}

/// Custom serialization function for Option<NaiveDate>
pub fn serialize_optional_date<S>(
    date: &Option<NaiveDate>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(d) => {
            let date_string = d.format("%Y%m%d").to_string();
            serializer.serialize_some(&date_string)
        }
        None => serializer.serialize_none(),
    }
}

/// Custom deserialization function for Option<NaiveDate>
pub fn deserialize_optional_date<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(date_string) => NaiveDate::parse_from_str(&date_string, "%Y%m%d")
            .map(Some)
            .map_err(|err| DeError::custom(format!("Invalid date format: {}", err))),
        None => Ok(None),
    }
}

/// Custom deserialization function for 0/1 to bool
pub fn deserialize_bool_as_int<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}

/// Custom serialization function for bool to 0/1
pub fn serialize_bool_as_int<S>(value: &bool, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(if *value { 1 } else { 0 })
}

/// Module acting as an Enum. Workaround for the lack
/// of support for associated enum constants in Rust.
pub mod coord_type {
    pub type T = u8;
    pub const STOP: T = 0;
    pub const SHAPE: T = 1;
}

/// Represents a coordinate. This is a wrapper around [`geo::Coord`] that implements
/// serialization and deserialization for the GTFS format.
#[derive(Debug, Clone, PartialEq)]
pub struct GtfsCoord<const COORD_TYPE: coord_type::T>(Coord);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Represents a GTFS coordinate for stops.
pub struct GtfsStopCoordFlatten {
    pub stop_lat: f64,
    pub stop_lon: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Represents a GTFS coordinate for shapes.
pub struct GtfsShapeCoordFlatten {
    pub shape_pt_lat: f64,
    pub shape_pt_lon: f64,
}

impl<const COORD_TYPE: coord_type::T> Serialize for GtfsCoord<COORD_TYPE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let (lat_field_name, lon_field_name, struct_name) = match COORD_TYPE {
            coord_type::STOP => ("stop_lat", "stop_lon", "GtfsStopCoordFlatten"),
            coord_type::SHAPE => ("shape_pt_lat", "shape_pt_lon", "GtfsShapeCoordFlatten"),
            _ => unreachable!(),
        };

        let mut state = serializer.serialize_struct(struct_name, 2)?;
        state.serialize_field(lat_field_name, &self.y)?;
        state.serialize_field(lon_field_name, &self.x)?;
        state.end()
    }
}

impl<'de, const COORD_TYPE: coord_type::T> Deserialize<'de> for GtfsCoord<COORD_TYPE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Custom visitor for deserializing GtfsCoord.
        ///
        /// This visitor is responsible for the flexible parsing of coordinate values.
        struct GtfsCoordVisitor<const COORD_TYPE: coord_type::T>;

        impl<'de, const COORD_TYPE: coord_type::T> Visitor<'de> for GtfsCoordVisitor<COORD_TYPE> {
            type Value = GtfsCoord<{ COORD_TYPE }>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a GtfsCoord")
            }

            fn visit_map<V>(self, mut map: V) -> Result<GtfsCoord<COORD_TYPE>, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut lat = None;
                let mut lon = None;

                // Determine the field names based on the coordinate type
                let (lat_field_name, lon_field_name, _) = match COORD_TYPE {
                    coord_type::STOP => ("stop_lat", "stop_lon", "GtfsStopCoordFlatten"),
                    coord_type::SHAPE => ("shape_pt_lat", "shape_pt_lon", "GtfsShapeCoordFlatten"),
                    _ => unreachable!(),
                };

                // Iterate through the map fields
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        ref field_name if field_name == &lat_field_name => {
                            let value: serde_json::Value = map.next_value()?;
                            lat = Some(parse_float(&value).map_err(de::Error::custom)?);
                        }
                        ref field_name if field_name == &lon_field_name => {
                            let value: serde_json::Value = map.next_value()?;
                            lon = Some(parse_float(&value).map_err(de::Error::custom)?);
                        }
                        _ => {
                            // Ignore unknown fields
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                // Ensure both latitude and longitude are present
                let lat = lat.ok_or_else(|| de::Error::missing_field(lat_field_name))?;
                let lon = lon.ok_or_else(|| de::Error::missing_field(lon_field_name))?;

                Ok(GtfsCoord(Coord { x: lon, y: lat }))
            }
        }

        // Use the custom visitor for deserialization
        deserializer.deserialize_map(GtfsCoordVisitor)
    }
}

/// Helper function to parse float values from various representations.
///
/// This function handles:
/// - Standard JSON number formats
/// - String representations of floats, with optional leading/trailing whitespace
///
/// # Arguments
///
/// * `value` - A `serde_json::Value` that may contain a float in various formats
///
/// # Returns
///
/// * `Ok(f64)` if parsing is successful
/// * `Err(String)` with an error message if parsing fails
fn parse_float(value: &serde_json::Value) -> Result<f64, String> {
    match value {
        serde_json::Value::Number(num) => num.as_f64().ok_or_else(|| "Invalid float".to_string()),
        serde_json::Value::String(s) => s.trim().parse::<f64>().map_err(|e| e.to_string()),
        _ => Err("Expected number or string".to_string()),
    }
}

// Implement Deref and DerefMut to make GtfsCoord behave like Coord
impl<const COORD_TYPE: coord_type::T> Deref for GtfsCoord<COORD_TYPE> {
    type Target = Coord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const COORD_TYPE: coord_type::T> DerefMut for GtfsCoord<COORD_TYPE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
