//! This module contains the data structures for the GTFS (General Transit Feed Specification) format.
//! It is based on the GTFS Schedule Reference, which defines the format and structure of the files
//! that comprise a GTFS dataset.
//!
//! The main structures are:
//! - [`Agency`]: Represents a transit agency.
//! - [`Stop`]: Represents a stop where vehicles pick up or drop off riders.
//! - [`Route`]: Represents a transit route.
//! - [`Trip`]: Represents a trip.
//! - [`StopTime`]: Represents a stop time.
//! - [`Calendar`]: Represents a calendar entry.
//! - [`CalendarDate`]: Represents a calendar date.
//! - [`FareAttribute`]: Represents fare information.
//! - [`FareRule`]: Represents a rule that specifies how fares apply to an itinerary.
//! - [`Shape`]: Represents a shape.
//! - [`Frequency`]: Represents a frequency-based service.
//! - [`Transfer`]: Represents additional rules and overrides for selected transfers.
//! - [`Pathway`]: Represents a pathway linking together locations within stations.
//! - [`Level`]: Represents a level within a station.
//! - [`FeedInfo`]: Represents dataset metadata.
//!
//! For more information, see the [GTFS Schedule Reference](https://gtfs.org/schedule/reference).

mod agency;
mod area;
mod attribution;
mod booking_rule;
mod calendar;
mod calendar_date;
mod common;
mod fare_attribute;
mod fare_leg_rule;
mod fare_media;
mod fare_product;
mod fare_rule;
mod fare_transfer_rule;
mod feed_info;
mod frequency;
mod level;
mod location_group;
mod location_group_stop;
mod network;
mod pathway;
mod route;
mod route_network;
mod shape;
mod stop;
mod stop_area;
mod stop_time;
mod timeframe;
mod transfer;
mod translation;
mod trip;

// Reexport all public items from each module
pub use agency::*;
pub use area::*;
pub use attribution::*;
pub use booking_rule::*;
pub use calendar::*;
pub use calendar_date::*;
pub use common::*;
pub use fare_attribute::*;
pub use fare_leg_rule::*;
pub use fare_media::*;
pub use fare_product::*;
pub use fare_rule::*;
pub use fare_transfer_rule::*;
pub use feed_info::*;
pub use frequency::*;
pub use level::*;
pub use location_group::*;
pub use location_group_stop::*;
pub use network::*;
pub use pathway::*;
pub use route::*;
pub use route_network::*;
pub use shape::*;
pub use stop::*;
pub use stop_area::*;
pub use stop_time::*;
pub use timeframe::*;
pub use transfer::*;
pub use translation::*;
pub use trip::*;
