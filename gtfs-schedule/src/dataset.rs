use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::{cell::OnceCell, env};

use chrono::NaiveDate;
use dashmap::DashMap;
use oxilangtag::LanguageTag;

use crate::error::{DatasetValidationError, ErrorContext, ParseError, ParseErrorKind, Result};
use crate::schemas::*;

pub static CSV_FILES: &[&str] = &[
    "agency.txt",
    "stops.txt",
    "routes.txt",
    "trips.txt",
    "stop_times.txt",
    "calendar.txt",
    "calendar_dates.txt",
    "fare_attributes.txt",
    "fare_rules.txt",
    "timeframes.txt",
    "fare_media.txt",
    "fare_products.txt",
    "fare_leg_rules.txt",
    "fare_transfers.txt",
    "areas.txt",
    "stops_areas.txt",
    "networks.txt",
    "routes_networks.txt",
    "shapes.txt",
    "frequencies.txt",
    "transfers.txt",
    "pathways.txt",
    "levels.txt",
    "location_groups.txt",
    "location_groups_stops.txt",
    "booking_rules.txt",
    "translations.txt",
    "feed_info.txt",
    "attributions.txt",
];

pub struct Dataset {
    /// Transit agencies with service represented in this dataset.
    ///
    /// This field is required.
    ///
    /// Primary key ([`Agency::agency_id`])
    pub agencies: Vec<Agency>, // Vec, because the primary key is nearly all the fields.
    /// Stops where vehicles pick up or drop off riders. Also defines stations and station entrances.
    ///
    /// This field is required.
    ///
    /// Primary key ([`Stop::stop_id`])
    pub stops: Arc<DashMap<StopId, Stop>>,
    /// Transit routes. A route is a group of trips that are displayed to riders as a single service.
    ///
    /// This field is required.
    ///
    /// Primary key ([`Route::route_id`])
    pub routes: Arc<DashMap<RouteId, Route>>,
    /// Trips for each route. A trip is a sequence of two or more stops that occur during a specific time period.
    ///
    /// This field is required.
    ///
    /// Primary key ([`Trip::route_id`])
    pub trips: Arc<DashMap<TripId, Trip>>,
    /// Times that a vehicle arrives at and departs from stops for each trip.
    ///
    /// This field is required.
    ///
    /// Primary key ([`StopTime::trip_id`], [`StopTime::stop_sequence`])
    pub stop_times: Arc<DashMap<(TripId, u32), StopTime>>,
    /// Service dates specified using a weekly schedule with start and end dates.
    ///
    /// This field is conditionally required:
    /// - Required unless all dates of service are defined in calendar_dates.txt.
    /// - Optional otherwise.
    ///
    /// Primary key ([`Calendar::service_id`])
    pub calendar: Arc<DashMap<CalendarServiceId, Calendar>>,
    /// Exceptions for the services defined in the calendar.txt.
    ///
    /// Conditionally Required:
    /// - Required if calendar.txt is omitted. In which case calendar_dates.txt must contain all dates of service.
    /// - Optional otherwise.
    ///
    /// Primary key ([`CalendarDate::service_id`], [`CalendarDate::date`])
    pub calendar_dates: Arc<DashMap<(CalendarServiceId, NaiveDate), CalendarDate>>,
    /// Fare information for a transit agency's routes.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`FareAttribute::fare_id`])
    pub fare_attributes: Arc<DashMap<FareId, FareAttribute>>,
    /// Rules to apply fares for itineraries.
    /// This field is optional.
    pub fare_rules: Vec<FareRule>, // Vec, because there is no primary key.
    /// Date and time periods to use in fare rules for fares that depend on date and time factors.
    ///
    /// This field is optional.
    pub timeframes: Vec<Timeframe>, // Vec, because there is no primary key.
    /// To describe the fare media that can be employed to use fare products.
    ///
    /// File fare_media.txt describes concepts that are not represented in fare_attributes.txt and fare_rules.txt. As such, the use of fare_media.txt is entirely separate from files fare_attributes.txt and fare_rules.txt.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`FareMedia::fare_media_id`])
    pub fare_medias: Arc<DashMap<FareMediaId, FareMedia>>,
    /// To describe the different types of tickets or fares that can be purchased by riders.
    ///
    /// File fare_products.txt describes fare products that are not represented in fare_attributes.txt and fare_rules.txt. As such, the use of fare_products.txt is entirely separate from files fare_attributes.txt and fare_rules.txt.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`FareProduct::fare_product_id`], [`FareProduct::fare_media_id`])
    pub fare_products: Arc<DashMap<(FareProductId, Option<FareMediaId>), FareProduct>>,
    /// Fare rules for individual legs of travel.
    ///
    /// File fare_leg_rules.txt provides a more detailed method for modeling fare structures. As such, the use of fare_leg_rules.txt is entirely separate from files fare_attributes.txt and fare_rules.txt.
    ///
    /// This field is optional.
    pub fare_leg_rules: Vec<FareLegRule>, // Vec, because the primary key is literally ALL the fields.
    /// Fare rules for transfers between legs of travel defined in `fare_leg_rules.txt`.
    ///
    /// This field is optional.
    pub fare_transfers: Vec<FareTransferRule>, // Vec, because the primary key is nearly all the fields.
    /// Area grouping of locations.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`Area::area_id`])
    pub areas: Arc<DashMap<AreaId, Area>>,
    /// Rules to assign stops to areas.
    ///
    /// This field is optional.
    pub stops_areas: Vec<StopArea>, // Vec, because there is no primary key.
    /// Network grouping of routes.
    ///
    /// Conditionally Forbidden:
    /// - Forbidden if network_id exists in routes.txt.
    /// - Optional otherwise.
    ///
    /// Primary key ([`Network::network_id`])
    pub networks: Arc<DashMap<NetworkId, Network>>,
    /// Rules to assign routes to networks.
    ///
    /// Conditionally Forbidden:
    /// - Forbidden if network_id exists in routes.txt.
    /// - Optional otherwise.
    ///
    /// Primary key ([`RouteNetwork::route_id`])
    pub routes_networks: Arc<DashMap<RouteId, RouteNetwork>>,
    /// Rules for mapping vehicle travel paths, sometimes referred to as route alignments.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`Shape::shape_id`], [`Shape::shape_pt_sequence`])
    pub shapes: Arc<DashMap<(ShapeId, u32), Shape>>,
    /// Headway (time between trips) for headway-based service or a compressed representation of fixed-schedule service.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`Frequency::trip_id`], [`Frequency::start_time`])
    pub frequencies: Arc<DashMap<(TripId, NaiveServiceTime), Frequency>>,
    /// Rules for making connections at transfer points between routes.
    ///
    /// This field is optional.
    pub transfers: Vec<Transfer>, // Vec, because the primary key is nearly all the fields.
    /// Pathways linking together locations within stations.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`Pathway::pathway_id`])
    pub pathways: Arc<DashMap<PathwayId, Pathway>>,
    /// Levels within stations.
    ///
    /// Conditionally Required:
    /// - Required when describing pathways with elevators (pathway_mode=5).
    /// - Optional otherwise.
    ///
    /// Primary key ([`Level::level_id`])
    pub levels: Arc<DashMap<LevelId, Level>>,
    /// A group of stops that together indicate locations where a rider may request pickup or drop off.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`LocationGroup::location_group_id`])
    pub location_groups: Arc<DashMap<LocationGroupId, LocationGroup>>,
    /// Rules to assign stops to location groups.
    /// This field is optional.
    pub location_groups_stops: Vec<LocationGroupStop>, // Vec, because there is no primary key.
    /// Booking information for rider-requested services.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`BookingRule::booking_rule_id`])
    pub booking_rules: Arc<DashMap<BookingRuleId, BookingRule>>,
    /// Translations of customer-facing dataset values.
    ///
    /// This field is optional.
    pub translations: Vec<Translation>, // Vec, because the primary key is nearly all the fields.
    /// Dataset metadata, including publisher, version, and expiration information.
    ///
    /// This field is optional.
    pub feed_info: Option<FeedInfo>,
    /// Dataset attributions.
    ///
    /// This field is optional.
    ///
    /// Primary key ([`Attribution::attribution_id`])
    pub attributions: Vec<Attribution>,
}

impl Dataset {
    pub fn default() -> Self {
        Self {
            agencies: vec![],
            stops: Arc::new(DashMap::new()),
            routes: Arc::new(DashMap::new()),
            trips: Arc::new(DashMap::new()),
            stop_times: Arc::new(DashMap::new()),
            calendar: Arc::new(DashMap::new()),
            calendar_dates: Arc::new(DashMap::new()),
            fare_attributes: Arc::new(DashMap::new()),
            fare_rules: vec![],
            timeframes: vec![],
            fare_medias: Arc::new(DashMap::new()),
            fare_products: Arc::new(DashMap::new()),
            fare_leg_rules: vec![],
            fare_transfers: vec![],
            areas: Arc::new(DashMap::new()),
            stops_areas: vec![],
            networks: Arc::new(DashMap::new()),
            routes_networks: Arc::new(DashMap::new()),
            shapes: Arc::new(DashMap::new()),
            frequencies: Arc::new(DashMap::new()),
            transfers: vec![],
            pathways: Arc::new(DashMap::new()),
            levels: Arc::new(DashMap::new()),
            location_groups: Arc::new(DashMap::new()),
            location_groups_stops: vec![],
            booking_rules: Arc::new(DashMap::new()),
            translations: vec![],
            feed_info: None,
            attributions: vec![],
        }
    }

    pub fn validate(&self) -> Result<()> {
        //
        // Validate individual fields.
        //

        // Validate agencies.
        // Note: There is nothing to validate at this level.

        // Validate stops.
        for stop in self.stops.iter() {
            stop.validate()?;
        }
        // Validate routes.
        for route in self.routes.iter() {
            route.validate()?;
        }
        // Validate trips.
        for trip in self.trips.iter() {
            trip.validate()?;
        }
        // Validate stop_times.
        for mut stop_time in self.stop_times.iter_mut() {
            stop_time.validate()?;
        }
        // Validate calendar.
        for calendar in self.calendar.iter() {
            calendar.validate()?;
        }
        // Validate calendar_dates.
        for calendar_date in self.calendar_dates.iter() {
            calendar_date.validate()?;
        }
        // Validate fare_attributes.
        for fare_attribute in self.fare_attributes.iter() {
            fare_attribute.validate()?;
        }
        // Validate fare_rules.
        for fare_rule in self.fare_rules.iter() {
            fare_rule.validate()?;
        }
        // Validate timeframes.
        for timeframe in self.timeframes.iter() {
            timeframe.validate()?;
        }
        // Validate fare_medias.
        for fare_media in self.fare_medias.iter() {
            fare_media.validate()?;
        }
        // Validate fare_products.
        for fare_product in self.fare_products.iter() {
            fare_product.validate()?;
        }
        // Validate fare_leg_rules.
        for fare_leg_rule in self.fare_leg_rules.iter() {
            fare_leg_rule.validate()?;
        }
        // Validate areas.
        for area in self.areas.iter() {
            area.validate()?;
        }
        // Validate stops_areas.
        for stop_area in &self.stops_areas {
            stop_area.validate()?;
        }
        // Validate networks.
        for network in self.networks.iter() {
            network.validate()?;
        }
        // Validate routes_networks.
        for route_network in self.routes_networks.iter() {
            route_network.validate()?;
        }
        // Validate shapes.
        for shape in self.shapes.iter() {
            shape.validate()?;
        }
        // Validate frequencies.
        for frequency in self.frequencies.iter() {
            frequency.validate()?;
        }
        // Validate transfers.
        for transfer in self.transfers.iter() {
            transfer.validate()?;
        }
        // Validate pathways.
        for pathway in self.pathways.iter() {
            pathway.validate()?;
        }
        // Validate levels.
        for mut level in self.levels.iter_mut() {
            level.validate()?;
        }
        // Validate location_groups.
        for location_group in self.location_groups.iter() {
            location_group.validate()?;
        }
        // Validate location_groups_stops.
        for location_group_stop in self.location_groups_stops.iter() {
            location_group_stop.validate()?;
        }
        // Validate booking_rules.
        for booking_rule in self.booking_rules.iter() {
            booking_rule.validate()?;
        }
        // Validate translations.
        for translation in self.translations.iter() {
            translation.validate()?;
        }
        // Validate feed_info.
        if let Some(feed_info) = self.feed_info.as_ref() {
            feed_info.validate()?;
        }
        // Validate attributions.
        for attribution in self.attributions.iter() {
            attribution.validate()?;
        }

        //
        // Validate the dataset as a whole.
        //

        // If there is more than one agency:
        // - agency_id must be present and unique.
        // - agency_timezone must be the same for all agencies.
        if self.agencies.len() > 1 {
            let mut agency_ids = HashSet::new();
            let agency_timezone = OnceCell::new();
            for agency in &self.agencies {
                // Validate agency_id's presence.
                if agency.agency_id.is_none() {
                    return Err(DatasetValidationError::new_missing_value(
                        "agency_id".to_string(),
                        Some("cannot be empty when there are multiple agencies".to_string()),
                        vec![agency.clone().into()],
                    )
                    .into());
                }
                // Validate agency_id's uniqueness.
                if agency_ids.contains(&agency.agency_id) {
                    // Collect all Agency with the same agency_id
                    let agencies = self
                        .agencies
                        .iter()
                        .filter(|a| a.agency_id == agency.agency_id)
                        .map(|a| Schema::from(a.clone()))
                        .collect::<Vec<_>>();
                    return Err(DatasetValidationError::new_primary_key_not_unique(
                        "agency_id".to_string(),
                        agency.clone().agency_id.unwrap().to_string(),
                        agencies,
                    )
                    .into());
                }
                agency_ids.insert(agency.agency_id.clone());
                // Validate agency_timezone.
                if agency_timezone.get().is_none() {
                    agency_timezone
                        .set(agency.agency_timezone)
                        .expect("Tried to set agency_timezone; but agency_timezone is already set");
                } else if agency_timezone.get() != Some(&agency.agency_timezone) {
                    return Err(DatasetValidationError::new_inconsistent_value(
                        "agency_timezone".to_string(),
                        agency.agency_timezone.to_string(),
                        Some(format!(
                            "must be the same for all agencies, expected {:?} but found {:?}",
                            agency_timezone.get(),
                            agency.agency_timezone
                        )),
                        vec![agency.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate stops:
        // - stop_id must be unique across stops.
        //   -> This is already taken care of because of the use of `Arc<DashMap<StopId, Stop>>`.
        // - parent_station must exist in stops.txt.
        // - level_id must exist in levels.txt.
        // - location_type=0 (or blank) stops with a parent_station must have a parent with location_type=1.
        // - Stops with location_type=1 (stations) must not have a parent_station.
        {
            let mut station_ids = HashSet::new();
            for stop in self.stops.iter() {
                // Validate parent_station
                if let Some(parent_station_id) = &stop.parent_station {
                    let mut current_parent_id = parent_station_id.clone();
                    let mut parent_chain = vec![stop.clone().into()];

                    loop {
                        let parent = self.stops.get(&current_parent_id).ok_or_else(|| {
                            DatasetValidationError::new_foreign_key_not_found(
                                "parent_station".to_string(),
                                current_parent_id.to_string(),
                                "stops.txt".to_string(),
                                parent_chain.clone(),
                            )
                        })?;

                        parent_chain.push(parent.clone().into());

                        if parent.location_type == Some(LocationType::Station) {
                            station_ids.insert(current_parent_id);
                            break;
                        } else if parent.parent_station.is_none() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                        "location_type".to_string(),
                        parent.clone().location_type.map(|loc| format!("{:?}", loc)).unwrap_or_default(),
                        Some("The parent station chain does not lead to a stop with location_type Station".to_string()),
                        parent_chain,
                    ).into());
                        }

                        current_parent_id = parent.parent_station.clone().unwrap();
                    }
                }

                // Validate level_id.
                if let Some(level_id) = &stop.level_id {
                    self.levels
                        .iter()
                        .find(|level| level.level_id == *level_id)
                        .ok_or_else(|| {
                            DatasetValidationError::new_foreign_key_not_found(
                                "level_id".to_string(),
                                level_id.to_string(),
                                "levels.txt".to_string(),
                                vec![stop.clone().into()],
                            )
                        })?;
                }

                // Validate location_type and parent_station relationship
                if stop.location_type == Some(LocationType::StopOrPlatform)
                    || stop.location_type.is_none()
                {
                    if let Some(parent_station_id) = &stop.parent_station {
                        if !station_ids.contains(parent_station_id) {
                            return Err(DatasetValidationError::new_foreign_key_not_found(
                                "parent_station".to_string(),
                                parent_station_id.to_string(),
                                "stops.txt".to_string(),
                                vec![stop.clone().into()],
                            )
                            .into());
                        }
                    }
                }
            }
        }

        // Validate routes:
        // - route_id must be unique accross routes.
        //   -> This is already taken care of because of the use of `Arc<DashMap<RouteId, Route>>`.
        // - agency_id must exist in agencies.txt (if there is more than one agency).
        // - continuous_pickup should not be defined if [`StopTime::start_pickup_drop_off_windown`] or
        //   [`StopTime::end_pickup_drop_off_window`] are defined for any trip of this route.
        // - continuous_drop_off should not be defined if [`StopTime::start_pickup_drop_off_windown`] or
        //   [`StopTime::end_pickup_drop_off_windown`] is defined.
        // - network_id should not be defined if route_networks.txt is present.
        {
            for route in self.routes.iter() {
                // Validate agency_id.
                if let Some(route_agency_id) = &route.agency_id {
                    if self.agencies.len() > 1 {
                        let exists = self.agencies.iter().any(|agency| {
                            agency
                                .agency_id
                                .as_ref()
                                .map_or(false, |agency_id| agency_id == route_agency_id)
                        });
                        if !exists {
                            return Err(DatasetValidationError::new_foreign_key_not_found(
                                "agency_id".to_string(),
                                route_agency_id.to_string(),
                                "agencies.txt".to_string(),
                                vec![route.clone().into()],
                            )
                            .into());
                        }
                    }
                }

                // Validate continuous_pickup and continuous_drop_off
                if route.continuous_pickup.is_some() || route.continuous_drop_off.is_some() {
                    let stop_times = self.stop_times_get_all_from_route(&route.route_id);
                    let has_pickup_drop_off_window = stop_times.iter().any(|stop_time| {
                        stop_time.start_pickup_drop_off_window.is_some()
                            || stop_time.end_pickup_drop_off_window.is_some()
                    });
                    if has_pickup_drop_off_window {
                        return Err(DatasetValidationError::new_inconsistent_value(
                            "continuous_pickup or continuous_drop_off".to_string(),
                            format!("{:?}", route.continuous_pickup.is_some() || route.continuous_drop_off.is_some()),
                            Some(format!(
                                "cannot be defined for route_id {:?} because at least one stop_time has start_pickup_drop_off_window or end_pickup_drop_off_window",
                                route.route_id
                            )),
                            vec![route.clone().into()],
                        ).into());
                    }
                }

                if route.network_id.is_some() && !self.routes_networks.is_empty() {
                    return Err(DatasetValidationError::new_inconsistent_value(
                        "network_id".to_string(),
                        format!("{:?}", route.network_id),
                        Some(format!(
                            "because route_networks.txt is present but network_id with value {:?} is defined for route_id: {:?}",
                            route.network_id.as_ref().unwrap(),
                            route.route_id
                        )),
                        vec![route.clone().into()],
                    ).into());
                }
            }
        }

        // Validate trips:
        // - trip_id must be unique across trips.
        //   -> This is already taken care of because of the use of `Arc<DashMap<TripId, Trip>>`.
        // - route_id must exist in routes.txt.
        // - service_id must exist in either calendar.txt or calendar_dates.txt.
        // - shape_id is required if the trip has a continuous pickup or drop-off behavior defined
        //   either in routes.txt or in stop_times.txt.
        {
            for trip in self.trips.iter() {
                if trip.shape_id.is_none() {
                    let as_continuous_pickup_or_drop_off =
                        self.routes.iter().any(|route| {
                            route.continuous_pickup.is_some() || route.continuous_drop_off.is_some()
                        }) || self.stop_times.iter().any(|stop_time| {
                            stop_time.start_pickup_drop_off_window.is_some()
                                || stop_time.end_pickup_drop_off_window.is_some()
                        });
                    if as_continuous_pickup_or_drop_off {
                        return Err(DatasetValidationError::new_missing_value(
                            "shape_id".to_string(),
                            Some(format!("because trip with id {:?} has a continuous pickup or drop-off behavior defined either in routes.txt or in stop_times.txt", trip.trip_id)),
                            vec![trip.clone().into()],
                        ).into());
                    }
                }

                // Validate route_id reference
                if !self.routes.contains_key(&trip.route_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "route_id".to_string(),
                        trip.route_id.to_string(),
                        "routes.txt".to_string(),
                        vec![trip.clone().into()],
                    )
                    .into());
                }

                // Validate service_id reference
                let service_id_valid = self.calendar.contains_key(&trip.service_id)
                    || self
                        .calendar_dates
                        .iter()
                        .any(|calendar_date| calendar_date.service_id == trip.service_id);
                if !service_id_valid {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "service_id".to_string(),
                        trip.service_id.to_string(),
                        "calendar.txt or calendar_dates.txt".to_string(),
                        vec![trip.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate stop_times:
        // - trip_id must reference a valid Trip.
        // - stop_id must reference a valid Stop (if specified).
        // - arrival_time and departure_time must be in the correct order and format.
        // - stop_sequence must increase along the trip.
        // - shape_dist_traveled must increase along the trip (if provided).
        {
            let trip_stop_sequences = DashMap::new();
            let trip_shape_distances = DashMap::new();

            // Sort all stop times by trip_id and arrival_time
            let mut sorted_stop_times: Vec<_> = self.stop_times.iter().map(|v| v.clone()).collect();
            sorted_stop_times.sort_by(|a, b| {
                a.trip_id
                    .cmp(&b.trip_id)
                    .then_with(|| a.arrival_time.cmp(&b.arrival_time))
            });

            for stop_time in sorted_stop_times.iter() {
                // Validate trip_id reference
                if !self.trips.contains_key(&stop_time.trip_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "trip_id".to_string(),
                        stop_time.trip_id.to_string(),
                        "trips.txt".to_string(),
                        vec![stop_time.clone().into()],
                    )
                    .into());
                }

                // Validate stop_id reference (if specified)
                if let Some(stop_id) = &stop_time.stop_id {
                    if !self.stops.contains_key(stop_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "stop_id".to_string(),
                            stop_id.to_string(),
                            "stops.txt".to_string(),
                            vec![stop_time.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate stop_sequence
                let mut stop_sequences = trip_stop_sequences
                    .entry(stop_time.trip_id.clone())
                    .or_insert_with(Vec::new);
                if !stop_sequences.is_empty()
                    && stop_time.stop_sequence <= *stop_sequences.last().unwrap()
                {
                    return Err(DatasetValidationError::new_inconsistent_value(
                        "stop_sequence".to_string(),
                        stop_time.stop_sequence.to_string(),
                        Some(format!("must increase along the trip. Found non-increasing sequence for trip_id: {:?}", stop_time.trip_id)),
                        vec![stop_time.clone().into()],
                    ).into());
                }
                stop_sequences.push(stop_time.stop_sequence);

                // Validate shape_dist_traveled
                if let Some(shape_dist) = stop_time.shape_dist_traveled {
                    let mut shape_distances = trip_shape_distances
                        .entry(stop_time.trip_id.clone())
                        .or_insert_with(Vec::new);
                    if !shape_distances.is_empty() && shape_dist <= *shape_distances.last().unwrap()
                    {
                        return Err(DatasetValidationError::new_inconsistent_value(
                            "shape_dist_traveled".to_string(),
                            shape_dist.to_string(),
                            Some(format!("must increase along the trip. Found non-increasing distance for trip_id: {:?}, stop_sequence: {}", 
                                stop_time.trip_id, stop_time.stop_sequence)),
                            vec![stop_time.clone().into()],
                        ).into());
                    }
                    shape_distances.push(shape_dist);
                }
            }
        }

        // Validate calendar:
        // - service_id must be unique across calendar entries.
        //   -> This is already taken care of because of the use of `Arc<DashMap<CalendarServiceId, Calendar>>`.

        // Validate calendar_dates:
        // - If calendar.txt is not provided, calendar_dates.txt must contain all dates of service.
        // - Each (service_id, date) pair should be unique.
        //   -> This is already taken care of because of the use of `Arc<DashMap<(CalendarServiceId, NaiveDate), CalendarDate>>`.
        {
            if self.calendar.is_empty() {
                let unique_service_ids: HashSet<_> = self
                    .calendar_dates
                    .iter()
                    .map(|entry| entry.service_id.clone())
                    .collect();

                if unique_service_ids.is_empty() {
                    // TEST ONLY: Ignore this error
                    let ignore_missing_calendar_dates = {
                        if let Ok(v) = env::var("__TEST__IGNORE_MISSING_CALENDAR_DATES") {
                            bool::from_str(&v).unwrap_or(true)
                        } else {
                            false
                        }
                    };

                    if !ignore_missing_calendar_dates {
                        return Err(DatasetValidationError::new_missing_value(
                            "calendar.txt and calendar_dates.txt".to_string(),
                            Some("calendar.txt is empty and calendar_dates.txt does not contain any entries".to_string()),
                            vec![],
                        ).into());
                    }
                }

                // Check if all trips have a corresponding service_id in calendar_dates
                for trip in self.trips.iter() {
                    if !unique_service_ids.contains(&trip.service_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "service_id".to_string(),
                            trip.service_id.to_string(),
                            "calendar_dates.txt".to_string(),
                            vec![trip.clone().into()],
                        )
                        .into());
                    }
                }
            }
        }

        // Validate fare_attributes:
        // - fare_id must be unique across fare attributes.
        //   -> This is already taken care of because of the use of `Arc<DashMap<FareId, FareAttribute>>`.
        // - If there are multiple agencies, agency_id must be provided and must reference a valid agency.
        {
            let multiple_agencies = self.agencies.len() > 1;

            for fare_attribute in self.fare_attributes.iter() {
                // Validate agency_id if there are multiple agencies
                if multiple_agencies {
                    if let Some(agency_id) = &fare_attribute.agency_id {
                        if !self
                            .agencies
                            .iter()
                            .any(|agency| agency.agency_id.as_ref() == Some(agency_id))
                        {
                            return Err(DatasetValidationError::new_foreign_key_not_found(
                                "agency_id".to_string(),
                                agency_id.to_string(),
                                "agency.txt".to_string(),
                                vec![fare_attribute.clone().into()],
                            )
                            .into());
                        }
                    } else {
                        return Err(DatasetValidationError::new_missing_value(
                            "agency_id".to_string(),
                            Some(format!("is required when there are multiple agencies. Missing for fare_id: {:?}",
                                fare_attribute.fare_id)),
                            vec![fare_attribute.clone().into()],
                        ).into());
                    }
                }
            }
        }

        // Validate fare_rules:
        // - fare_id must reference a valid fare_id in fare_attributes.txt.
        // - If provided, route_id must reference a valid route_id in routes.txt.
        // - If provided, origin_id, destination_id, and contains_id must reference valid zone_id values in stops.txt.
        {
            let valid_zone_ids: HashSet<_> = self
                .stops
                .iter()
                .filter_map(|stop| stop.zone_id.clone())
                .collect();

            for fare_rule in &self.fare_rules {
                // Validate fare_id reference
                if !self.fare_attributes.contains_key(&fare_rule.fare_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "fare_id".to_string(),
                        fare_rule.fare_id.to_string(),
                        "fare_attributes.txt".to_string(),
                        vec![fare_rule.clone().into()],
                    )
                    .into());
                }

                // Validate route_id reference if provided
                if let Some(route_id) = &fare_rule.route_id {
                    if !self.routes.contains_key(route_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "route_id".to_string(),
                            route_id.to_string(),
                            "routes.txt".to_string(),
                            vec![fare_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate origin_id reference if provided
                if let Some(origin_id) = &fare_rule.origin_id {
                    if !valid_zone_ids.contains(origin_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "origin_id".to_string(),
                            origin_id.to_string(),
                            "stops.txt (zone_id)".to_string(),
                            vec![fare_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate destination_id reference if provided
                if let Some(destination_id) = &fare_rule.destination_id {
                    if !valid_zone_ids.contains(destination_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "destination_id".to_string(),
                            destination_id.to_string(),
                            "stops.txt (zone_id)".to_string(),
                            vec![fare_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate contains_id reference if provided
                if let Some(contains_id) = &fare_rule.contains_id {
                    if !valid_zone_ids.contains(contains_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "contains_id".to_string(),
                            contains_id.to_string(),
                            "stops.txt (zone_id)".to_string(),
                            vec![fare_rule.clone().into()],
                        )
                        .into());
                    }
                }
            }
        }

        // Validate timeframes:
        // - service_id must reference a valid service_id in either calendar.txt or calendar_dates.txt.
        // - There must not be overlapping time intervals for the same timeframe_group_id and service_id values.
        {
            // Collect all valid service_ids from calendar and calendar_dates
            let valid_service_ids: HashSet<_> = self
                .calendar
                .iter()
                .map(|calendar| calendar.service_id.clone())
                .chain(
                    self.calendar_dates
                        .iter()
                        .map(|calendar_date| calendar_date.service_id.clone()),
                )
                .collect();

            // Group timeframes by timeframe_group_id and service_id
            let grouped_timeframes: DashMap<
                (&TimeframeGroupId, &CalendarServiceId),
                Vec<&Timeframe>,
            > = DashMap::new();

            for timeframe in &self.timeframes {
                // Validate service_id reference
                if !valid_service_ids.contains(&timeframe.service_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "service_id".to_string(),
                        timeframe.service_id.to_string(),
                        "calendar.txt or calendar_dates.txt".to_string(),
                        vec![timeframe.clone().into()],
                    )
                    .into());
                }

                // Group timeframes for overlap checking
                grouped_timeframes
                    .entry((&timeframe.timeframe_group_id, &timeframe.service_id))
                    .or_default()
                    .push(timeframe);
            }

            // Check for overlapping time intervals
            for ((timeframe_group_id, service_id), timeframes) in grouped_timeframes {
                for (i, timeframe1) in timeframes.iter().enumerate() {
                    for timeframe2 in timeframes.iter().skip(i + 1) {
                        if let (Some(start1), Some(end1), Some(start2), Some(end2)) = (
                            timeframe1.start_time,
                            timeframe1.end_time,
                            timeframe2.start_time,
                            timeframe2.end_time,
                        ) {
                            if (start1 < end2 && end1 > start2) || (start2 < end1 && end2 > start1)
                            {
                                return Err(DatasetValidationError::new_overlapping_intervals(
                                    format!(
                                        "Overlapping time intervals found for timeframe_group_id: {:?}, service_id: {:?}. \
                                        Interval 1: {:?}-{:?}, Interval 2: {:?}-{:?}",
                                        timeframe_group_id, service_id, start1, end1, start2, end2
                                    ),
                                    vec![timeframe1.to_owned().clone().into(), timeframe2.to_owned().clone().into()],
                                ).into());
                            }
                        }
                    }
                }
            }
        }

        // Validate fare_media:
        // - fare_media_id must be unique across all fare media.
        //   -> This is already taken care of because of the use of `Arc<DashMap<FareMediaId, FareMedia>>`.

        // Validate fare_products:
        // - The combination of fare_product_id and fare_media_id must be unique.
        //   -> This is already taken care of because of the use of `Arc<DashMap<(FareProductId, Option<FareMediaId>), FareProduct>>`.
        // - If provided, fare_media_id must reference a valid fare_media_id in fare_media.txt.
        {
            for fare_product in self.fare_products.iter() {
                if let Some(media_id) = fare_product.fare_media_id.clone() {
                    if !self.fare_medias.contains_key(&media_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "fare_media_id".to_string(),
                            media_id.to_string(),
                            "fare_media.txt".to_string(),
                            vec![fare_product.clone().into()],
                        )
                        .into());
                    }
                }
            }
        }

        // Validate fare_leg_rules:
        // - If provided, network_id must reference a valid network_id in either routes.txt or networks.txt.
        // - If provided, from_area_id and to_area_id must reference valid area_id values in areas.txt.
        // - If provided, from_timeframe_group_id and to_timeframe_group_id must reference valid timeframe_group_id values in timeframes.txt.
        // - fare_product_id must reference a valid fare_product_id in fare_products.txt.
        {
            let valid_network_ids: HashSet<_> = self
                .routes
                .iter()
                .filter_map(|route| route.network_id.clone())
                .chain(
                    self.networks
                        .iter()
                        .map(|network| network.network_id.clone()),
                )
                .collect();

            let valid_area_ids: HashSet<_> =
                self.areas.iter().map(|area| area.area_id.clone()).collect();

            let valid_timeframe_group_ids: HashSet<_> = self
                .timeframes
                .iter()
                .map(|timeframe| &timeframe.timeframe_group_id)
                .collect();

            for fare_leg_rule in &self.fare_leg_rules {
                // Validate network_id reference if provided
                if let Some(network_id) = &fare_leg_rule.network_id {
                    if !valid_network_ids.contains(network_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "network_id".to_string(),
                            network_id.to_string(),
                            "routes.txt or networks.txt".to_string(),
                            vec![fare_leg_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate from_area_id reference if provided
                if let Some(from_area_id) = &fare_leg_rule.from_area_id {
                    if !valid_area_ids.contains(from_area_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "from_area_id".to_string(),
                            from_area_id.to_string(),
                            "areas.txt".to_string(),
                            vec![fare_leg_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate to_area_id reference if provided
                if let Some(to_area_id) = &fare_leg_rule.to_area_id {
                    if !valid_area_ids.contains(to_area_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "to_area_id".to_string(),
                            to_area_id.to_string(),
                            "areas.txt".to_string(),
                            vec![fare_leg_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate from_timeframe_group_id reference if provided
                if let Some(from_timeframe_group_id) = &fare_leg_rule.from_timeframe_group_id {
                    if !valid_timeframe_group_ids.contains(from_timeframe_group_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "from_timeframe_group_id".to_string(),
                            from_timeframe_group_id.to_string(),
                            "timeframes.txt".to_string(),
                            vec![fare_leg_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate to_timeframe_group_id reference if provided
                if let Some(to_timeframe_group_id) = &fare_leg_rule.to_timeframe_group_id {
                    if !valid_timeframe_group_ids.contains(to_timeframe_group_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "to_timeframe_group_id".to_string(),
                            to_timeframe_group_id.to_string(),
                            "timeframes.txt".to_string(),
                            vec![fare_leg_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate fare_product_id reference
                if !self.fare_products.iter().any(|fare_product| {
                    fare_product.fare_product_id == fare_leg_rule.fare_product_id
                }) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "fare_product_id".to_string(),
                        fare_leg_rule.fare_product_id.to_string(),
                        "fare_products.txt".to_string(),
                        vec![fare_leg_rule.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate fare_transfer_rules:
        // - If provided, from_leg_group_id and to_leg_group_id must reference valid leg_group_id values in fare_leg_rules.txt.
        // - If provided, fare_product_id must reference a valid fare_product_id in fare_products.txt.
        {
            let valid_leg_group_ids: HashSet<_> = self
                .fare_leg_rules
                .iter()
                .filter_map(|rule| rule.leg_group_id.as_ref())
                .collect();

            for fare_transfer_rule in &self.fare_transfers {
                // Validate from_leg_group_id reference if provided
                if let Some(from_leg_group_id) = &fare_transfer_rule.from_leg_group_id {
                    if !valid_leg_group_ids.contains(from_leg_group_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "from_leg_group_id".to_string(),
                            from_leg_group_id.to_string(),
                            "fare_leg_rules.txt".to_string(),
                            vec![fare_transfer_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate to_leg_group_id reference if provided
                if let Some(to_leg_group_id) = &fare_transfer_rule.to_leg_group_id {
                    if !valid_leg_group_ids.contains(to_leg_group_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "to_leg_group_id".to_string(),
                            to_leg_group_id.to_string(),
                            "fare_leg_rules.txt".to_string(),
                            vec![fare_transfer_rule.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate fare_product_id reference if provided
                if let Some(fare_product_id) = &fare_transfer_rule.fare_product_id {
                    if !self
                        .fare_products
                        .iter()
                        .any(|fare_product| &fare_product.fare_product_id == fare_product_id)
                    {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "fare_product_id".to_string(),
                            fare_product_id.to_string(),
                            "fare_products.txt".to_string(),
                            vec![fare_transfer_rule.clone().into()],
                        )
                        .into());
                    }
                }
            }
        }

        // Validate areas:
        // - area_id must be unique across all areas.
        //   -> This is already taken care of because of the use of `Arc<DashMap<AreaId, Area>>`.

        // Validate stop_areas:
        // - area_id must reference a valid area_id in areas.txt.
        // - stop_id must reference a valid stop_id in stops.txt.
        {
            for stop_area in &self.stops_areas {
                // Validate area_id reference
                if !self.areas.contains_key(&stop_area.area_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "area_id".to_string(),
                        stop_area.area_id.to_string(),
                        "areas.txt".to_string(),
                        vec![stop_area.clone().into()],
                    )
                    .into());
                }

                // Validate stop_id reference
                if !self.stops.contains_key(&stop_area.stop_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "stop_id".to_string(),
                        stop_area.stop_id.to_string(),
                        "stops.txt".to_string(),
                        vec![stop_area.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate networks.txt
        // - network_id must be unique accross all networks
        //   -> This is already taken care of because of the use of `Arc<DashMap<AreaId, Area>>`.

        // Validate route_networks.txt
        // - network_id must reference a valid network_id in networks.txt
        // - route_id must reference a valid route_id in routes.txt
        {
            for route_network in self.routes_networks.iter() {
                // Validate network_id reference
                if !self.networks.contains_key(&route_network.network_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "network_id".to_string(),
                        route_network.network_id.to_string(),
                        "networks.txt".to_string(),
                        vec![route_network.clone().into()],
                    )
                    .into());
                }

                // Validate route_id reference
                if !self.routes.contains_key(&route_network.route_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "route_id".to_string(),
                        route_network.route_id.to_string(),
                        "routes.txt".to_string(),
                        vec![route_network.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate shapes.txt
        // - shape_id must be unique accross all shapes.txt
        //    -> This is already taken care of because of the use of `Arc<DashMap<ShapeId, Shape>>`.
        // - shape_dist_traveled must increase along with shape_pt_sequence for each shape_id.
        {
            let shape_distances: DashMap<ShapeId, Vec<(u32, f32)>> = DashMap::new();

            for shape in self.shapes.iter() {
                if let Some(shape_dist_traveled) = shape.shape_dist_traveled {
                    shape_distances
                        .entry(shape.shape_id.clone())
                        .or_default()
                        .push((shape.shape_pt_sequence, shape_dist_traveled));
                }
            }

            for (shape_id, distances) in shape_distances {
                let mut sorted_distances = distances;
                sorted_distances.sort_by_key(|(sequence, _)| *sequence);

                for window in sorted_distances.windows(2) {
                    if let [(prev_seq, prev_dist), (curr_seq, curr_dist)] = window {
                        if curr_dist <= prev_dist {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "shape_dist_traveled".to_string(),
                                format!("{}", curr_dist),
                                Some(format!(
                                    "shape_dist_traveled does not increase along shape_pt_sequence for shape_id: {:?}. \
                                    Previous distance: {} at sequence: {}, Current distance: {} at sequence: {}",
                                    shape_id, prev_dist, prev_seq, curr_dist, curr_seq
                                )),
                                vec![
                                    self.shapes.get(&(shape_id.clone(), *prev_seq)).unwrap().clone().into(),
                                    self.shapes.get(&(shape_id.clone(), *curr_seq)).unwrap().clone().into(),
                                ],
                            ).into());
                        }
                    }
                }
            }
        }

        // Validate frequencies.txt
        // - trip_id must reference a valid trip_id in trips.txt.
        {
            for frequency in self.frequencies.iter() {
                // Validate trip_id reference
                if !self.trips.contains_key(&frequency.trip_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "trip_id".to_string(),
                        frequency.trip_id.to_string(),
                        "trips.txt".to_string(),
                        vec![frequency.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate transfers:
        // - from_stop_id and to_stop_id must reference valid stop_id values in stops.txt.
        // - from_route_id and to_route_id (if provided) must reference valid route_id values in routes.txt.
        // - from_trip_id and to_trip_id (if provided) must reference valid trip_id values in trips.txt.
        {
            for transfer in &self.transfers {
                // Validate stop_id references
                if let Some(from_stop_id) = &transfer.from_stop_id {
                    if !self.stops.contains_key(from_stop_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "from_stop_id".to_string(),
                            from_stop_id.to_string(),
                            "stops.txt".to_string(),
                            vec![transfer.clone().into()],
                        )
                        .into());
                    }
                }
                if let Some(to_stop_id) = &transfer.to_stop_id {
                    if !self.stops.contains_key(to_stop_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "to_stop_id".to_string(),
                            to_stop_id.to_string(),
                            "stops.txt".to_string(),
                            vec![transfer.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate route_id references
                if let Some(from_route_id) = &transfer.from_route_id {
                    if !self.routes.contains_key(from_route_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "from_route_id".to_string(),
                            from_route_id.to_string(),
                            "routes.txt".to_string(),
                            vec![transfer.clone().into()],
                        )
                        .into());
                    }
                }
                if let Some(to_route_id) = &transfer.to_route_id {
                    if !self.routes.contains_key(to_route_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "to_route_id".to_string(),
                            to_route_id.to_string(),
                            "routes.txt".to_string(),
                            vec![transfer.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate trip_id references
                if let Some(from_trip_id) = &transfer.from_trip_id {
                    if !self.trips.contains_key(from_trip_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "from_trip_id".to_string(),
                            from_trip_id.to_string(),
                            "trips.txt".to_string(),
                            vec![transfer.clone().into()],
                        )
                        .into());
                    }
                }
                if let Some(to_trip_id) = &transfer.to_trip_id {
                    if !self.trips.contains_key(to_trip_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "to_trip_id".to_string(),
                            to_trip_id.to_string(),
                            "trips.txt".to_string(),
                            vec![transfer.clone().into()],
                        )
                        .into());
                    }
                }
            }
        }

        // Validate pathways:
        // - pathway_id must be unique across all pathways.
        //   -> This is already taken care of because of the use of `Arc<DashMap<PathwayId, Pathway>>`.
        // - from_stop_id and to_stop_id must reference valid stop_id values in stops.txt.
        // - Exit gates (pathway_mode=7) must not be bidirectional.
        // - from_stop_id and to_stop_id must reference stops with appropriate location types.
        {
            for pathway in self.pathways.iter() {
                // Validate from_stop_id reference
                let from_stop = self.stops.get(&pathway.from_stop_id).ok_or_else(|| {
                    DatasetValidationError::new_foreign_key_not_found(
                        "from_stop_id".to_string(),
                        pathway.from_stop_id.to_string(),
                        "stops.txt".to_string(),
                        vec![pathway.clone().into()],
                    )
                })?;

                // Validate to_stop_id reference
                let to_stop = self.stops.get(&pathway.to_stop_id).ok_or_else(|| {
                    DatasetValidationError::new_foreign_key_not_found(
                        "to_stop_id".to_string(),
                        pathway.to_stop_id.to_string(),
                        "stops.txt".to_string(),
                        vec![pathway.clone().into()],
                    )
                })?;

                // Validate is_bidirectional constraint for exit gates
                if pathway.pathway_mode == PathwayMode::ExitGate && pathway.is_bidirectional {
                    return Err(DatasetValidationError::new_inconsistent_value(
                        "is_bidirectional".to_string(),
                        "true".to_string(),
                        Some("Exit gates (pathway_mode=7) must not be bidirectional".to_string()),
                        vec![pathway.clone().into()],
                    )
                    .into());
                }

                // Validate location types
                let valid_location_types = [
                    LocationType::StopOrPlatform,
                    LocationType::Station,
                    LocationType::EntranceOrExit,
                    LocationType::GenericNode,
                    LocationType::BoardingArea,
                ];

                if !valid_location_types.contains(
                    &from_stop
                        .location_type
                        .clone()
                        .unwrap_or(LocationType::StopOrPlatform),
                ) {
                    return Err(DatasetValidationError::new_inconsistent_value(
                        "location_type".to_string(),
                        format!("{:?}", from_stop.location_type),
                        Some("Invalid location_type for from_stop_id in pathway".to_string()),
                        vec![pathway.clone().into(), from_stop.clone().into()],
                    )
                    .into());
                }

                if !valid_location_types.contains(
                    &to_stop
                        .location_type
                        .clone()
                        .unwrap_or(LocationType::StopOrPlatform),
                ) {
                    return Err(DatasetValidationError::new_inconsistent_value(
                        "location_type".to_string(),
                        format!("{:?}", to_stop.location_type),
                        Some("Invalid location_type for to_stop_id in pathway".to_string()),
                        vec![pathway.clone().into(), to_stop.clone().into()],
                    )
                    .into());
                }

                // Validate that pathways don't connect a station to itself
                if from_stop.location_type == Some(LocationType::Station)
                    && to_stop.location_type == Some(LocationType::Station)
                    && pathway.from_stop_id == pathway.to_stop_id
                {
                    return Err(DatasetValidationError::new_inconsistent_value(
                        "from_stop_id and to_stop_id".to_string(),
                        format!(
                            "from_stop_id: {}, to_stop_id: {}",
                            pathway.from_stop_id, pathway.to_stop_id
                        ),
                        Some("Pathway cannot connect a station to itself".to_string()),
                        vec![pathway.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate levels:
        // - level_id must be unique across all levels.
        //   -> This is already taken care of because of the use of `Arc<DashMap<LevelId, Level>>`.

        // Validate location_groups:
        // - location_group_id must be unique across all location groups.
        //   -> This is already taken care of because of the use of `Arc<DashMap<LocationGroupId, LocationGroup>>`.
        // - location_group_id must be unique across all stops.stop_id, locations.geojson id,
        //   and location_groups.location_group_id values.
        {
            let mut all_ids = HashSet::new();

            // Collect all stop_ids
            for stop in self.stops.iter() {
                if !all_ids.insert(stop.stop_id.to_string()) {
                    return Err(DatasetValidationError::new_primary_key_not_unique(
                        "stop_id".to_string(),
                        stop.stop_id.to_string(),
                        vec![stop.clone().into()],
                    )
                    .into());
                }
            }

            // Collect all location_group_ids
            for location_group in self.location_groups.iter() {
                if !all_ids.insert(location_group.location_group_id.to_string()) {
                    return Err(DatasetValidationError::new_primary_key_not_unique(
                        "location_group_id".to_string(),
                        location_group.location_group_id.to_string(),
                        vec![location_group.clone().into()],
                    )
                    .into());
                }
            }

            // Note: We can't check locations.geojson ids here because they're not part of the Dataset struct.
            // If locations.geojson is implemented in the future, its IDs should be checked here as well.
        }

        // Validate location_group_stops:
        // - location_group_id must reference a valid location_group_id in location_groups.txt.
        // - stop_id must reference a valid stop_id in stops.txt.
        {
            for location_group_stop in &self.location_groups_stops {
                // Validate location_group_id reference
                if !self
                    .location_groups
                    .contains_key(&location_group_stop.location_group_id)
                {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "location_group_id".to_string(),
                        location_group_stop.location_group_id.to_string(),
                        "location_groups.txt".to_string(),
                        vec![location_group_stop.clone().into()],
                    )
                    .into());
                }

                // Validate stop_id reference
                if !self.stops.contains_key(&location_group_stop.stop_id) {
                    return Err(DatasetValidationError::new_foreign_key_not_found(
                        "stop_id".to_string(),
                        location_group_stop.stop_id.to_string(),
                        "stops.txt".to_string(),
                        vec![location_group_stop.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate booking_rules:
        // - booking_rule_id must be unique across all booking rules.
        //   -> This is already taken care of because of the use of `Arc<DashMap<BookingRuleId, BookingRule>>`.
        // - prior_notice_service_id, if provided, must reference a valid service_id in either calendar.txt or calendar_dates.txt.
        {
            // Collect all valid service_ids from calendar and calendar_dates
            let valid_service_ids: HashSet<_> = self
                .calendar
                .iter()
                .map(|service| service.service_id.clone())
                .chain(
                    self.calendar_dates
                        .iter()
                        .map(|calendar_date| calendar_date.service_id.clone()),
                )
                .collect();

            for booking_rule in self.booking_rules.iter() {
                if let Some(prior_notice_service_id) = &booking_rule.prior_notice_service_id {
                    if !valid_service_ids.contains(prior_notice_service_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "prior_notice_service_id".to_string(),
                            prior_notice_service_id.to_string(),
                            "calendar.txt or calendar_dates.txt".to_string(),
                            vec![booking_rule.clone().into()],
                        )
                        .into());
                    }
                }
            }
        }

        // Validate translations:
        // - Validate references to other tables based on the table_name field.
        // - Ensure that record_sub_id is provided when required.
        // - Check that field_value is not used together with record_id and record_sub_id.
        // - Verify that translations for feed_info.txt don't use record_id, record_sub_id, or field_value.
        {
            for translation in &self.translations {
                match translation.table_name {
                    TableName::Agency => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.agencies.iter().any(|agency| {
                                agency.agency_id == Some(AgencyId::from(record_id.as_str()))
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "agency_id".to_string(),
                                    record_id.to_string(),
                                    "agency.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for agency translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::Stops => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.stops.contains_key(&StopId::from(record_id.as_str())) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "stop_id".to_string(),
                                    record_id.to_string(),
                                    "stops.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for stops translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::Routes => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.routes.contains_key(&RouteId::from(record_id.as_str())) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "route_id".to_string(),
                                    record_id.to_string(),
                                    "routes.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for routes translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::Trips => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.trips.contains_key(&TripId::from(record_id.as_str())) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "trip_id".to_string(),
                                    record_id.to_string(),
                                    "trips.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for trips translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::StopTimes => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.stop_times.iter().any(|stop_time| {
                                stop_time.trip_id == TripId::from(record_id.as_str())
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "trip_id".to_string(),
                                    record_id.to_string(),
                                    "stop_times.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if let Some(record_sub_id) = &translation.record_sub_id {
                            if !self.stop_times.iter().any(|stop_time| {
                                stop_time.stop_sequence
                                    == u32::from_str(record_sub_id.as_str())
                                        .expect("Could not parse stop_sequence from record_sub_id")
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "stop_sequence".to_string(),
                                    record_sub_id.to_string(),
                                    "stop_times.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                    }
                    TableName::Pathways => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.pathways.iter().any(|pathway| {
                                pathway.pathway_id == PathwayId::from(record_id.as_str())
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "pathway_id".to_string(),
                                    record_id.to_string(),
                                    "pathways.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for pathways translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::Levels => {
                        if let Some(record_id) = &translation.record_id {
                            if !self
                                .levels
                                .iter()
                                .any(|level| level.level_id == LevelId::from(record_id.as_str()))
                            {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "level_id".to_string(),
                                    record_id.to_string(),
                                    "levels.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for levels translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::FeedInfo => {
                        if translation.record_id.is_some()
                            || translation.record_sub_id.is_some()
                            || translation.field_value.is_some()
                        {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_id, record_sub_id, field_value".to_string(),
                                "".to_string(),
                                Some("record_id, record_sub_id, and field_value are not allowed for feed_info translations".to_string()),
                                vec![translation.clone().into()],
                            ).into());
                        }
                    }
                    TableName::Attributions => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.attributions.iter().any(|attribution| {
                                attribution.attribution_id
                                    == Some(AttributionId::from(record_id.as_str()))
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "attribution_id".to_string(),
                                    record_id.to_string(),
                                    "attributions.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for attributions translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::Calendar => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.calendar.iter().any(|calendar| {
                                calendar.service_id == CalendarServiceId::from(record_id.as_str())
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "service_id".to_string(),
                                    record_id.to_string(),
                                    "calendar.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for calendar translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::CalendarDates => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.calendar_dates.iter().any(|calendar_date| {
                                calendar_date.service_id
                                    == CalendarServiceId::from(record_id.as_str())
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "service_id".to_string(),
                                    record_id.to_string(),
                                    "calendar_dates.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if let Some(record_sub_id) = &translation.record_sub_id {
                            if !self.calendar_dates.iter().any(|calendar_date| {
                                calendar_date.date
                                    == NaiveDate::from_str(record_sub_id.as_str())
                                        .expect("Could not parse NaiveDate from record_sub_id")
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "date".to_string(),
                                    record_sub_id.to_string(),
                                    "calendar_dates.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                    }
                    TableName::FareAttributes => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.fare_attributes.iter().any(|fare_attribute| {
                                fare_attribute.fare_id == FareId::from(record_id.as_str())
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "fare_id".to_string(),
                                    record_id.to_string(),
                                    "fare_attributes.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for fare_attributes translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::FareRules => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.fare_rules.iter().any(|fare_rule| {
                                fare_rule.fare_id == FareId::from(record_id.as_str())
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "fare_id".to_string(),
                                    record_id.to_string(),
                                    "fare_rules.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if let Some(record_sub_id) = &translation.record_sub_id {
                            if !self.fare_rules.iter().any(|fare_rule| {
                                fare_rule.route_id == Some(RouteId::from(record_sub_id.as_str()))
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "route_id".to_string(),
                                    record_sub_id.to_string(),
                                    "fare_rules.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                    }
                    TableName::Shapes => {
                        if let Some(record_id) = &translation.record_id {
                            if !self
                                .shapes
                                .iter()
                                .any(|shape| shape.shape_id == ShapeId::from(record_id.as_str()))
                            {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "shape_id".to_string(),
                                    record_id.to_string(),
                                    "shapes.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if translation.record_sub_id.is_some() {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "record_sub_id".to_string(),
                                translation.record_sub_id.clone().unwrap_or_default(),
                                Some(
                                    "record_sub_id is not allowed for shapes translations"
                                        .to_string(),
                                ),
                                vec![translation.clone().into()],
                            )
                            .into());
                        }
                    }
                    TableName::Frequencies => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.frequencies.iter().any(|frequency| {
                                frequency.trip_id == TripId::from(record_id.as_str())
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "trip_id".to_string(),
                                    record_id.to_string(),
                                    "frequencies.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if let Some(record_sub_id) = &translation.record_sub_id {
                            if !self.frequencies.iter().any(|frequency| {
                                frequency.start_time
                                    == NaiveServiceTime::try_from(record_sub_id.as_str()).expect(
                                        "Could not parse NaiveServiceTime from record_sub_id",
                                    )
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "start_time".to_string(),
                                    record_sub_id.to_string(),
                                    "frequencies.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                    }
                    TableName::Transfers => {
                        if let Some(record_id) = &translation.record_id {
                            if !self.transfers.iter().any(|transfer| {
                                transfer.from_stop_id == Some(StopId::from(record_id.as_str()))
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "from_stop_id".to_string(),
                                    record_id.to_string(),
                                    "transfers.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                        if let Some(record_sub_id) = &translation.record_sub_id {
                            if !self.transfers.iter().any(|transfer| {
                                transfer.to_stop_id == Some(StopId::from(record_sub_id.as_str()))
                            }) {
                                return Err(DatasetValidationError::new_foreign_key_not_found(
                                    "to_stop_id".to_string(),
                                    record_sub_id.to_string(),
                                    "transfers.txt".to_string(),
                                    vec![translation.clone().into()],
                                )
                                .into());
                            }
                        }
                    }
                }

                // Check that field_value is not used together with record_id and record_sub_id
                if translation.field_value.is_some()
                    && (translation.record_id.is_some() || translation.record_sub_id.is_some())
                {
                    return Err(DatasetValidationError::new_inconsistent_value(
                        "field_value, record_id, record_sub_id".to_string(),
                        "".to_string(),
                        Some(
                            "field_value cannot be used together with record_id or record_sub_id"
                                .to_string(),
                        ),
                        vec![translation.clone().into()],
                    )
                    .into());
                }
            }
        }

        // Validate feed_info:
        // - feed_lang should be consistent with the language in translations.txt.
        // - if feed_lang is set to "mul" each translation should be translated
        //   to all languages found in translations.txt.
        // - feed_info.txt is required if translations.txt is provided.
        {
            match &self.feed_info {
                Some(feed_info) => {
                    // Check feed_start_date and feed_end_date
                    if let (Some(start_date), Some(end_date)) =
                        (feed_info.feed_start_date, feed_info.feed_end_date)
                    {
                        if start_date > end_date {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "feed_start_date, feed_end_date".to_string(),
                                format!("start_date: {}, end_date: {}", start_date, end_date),
                                Some("feed_start_date is after feed_end_date".to_string()),
                                vec![feed_info.clone().into()],
                            )
                            .into());
                        }
                    }

                    // Check feed_lang consistency
                    // 1. Collect all languages found in translations.txt
                    let languages: HashSet<LanguageTag<String>> = self
                        .translations
                        .iter()
                        .map(|translation| translation.language.clone())
                        .collect();

                    // 2. Check if feed_lang is set to "mul" and if each translation is translated to all languages found in translations.txt
                    if feed_info.feed_lang == LanguageTag::parse("mul").unwrap() {
                        // Split translations into groups of (record_id, record_sub_id) by language
                        let translations_by_language: DashMap<
                            LanguageTag<String>,
                            HashSet<(Option<String>, Option<String>)>,
                        > = DashMap::new();
                        for translation in &self.translations {
                            let language = translation.language.clone();
                            translations_by_language
                                .entry(language)
                                .or_default()
                                .insert((
                                    translation.record_id.clone(),
                                    translation.record_sub_id.clone(),
                                ));
                        }

                        // Check if each translation is translated to all languages found in translations.txt
                        // We just have to compare each HashSets
                        let translations_by_language = translations_by_language.into_read_only();
                        let mut peekable_translations_by_language =
                            translations_by_language.iter().peekable();
                        while let Some((language, translations)) =
                            peekable_translations_by_language.next()
                        {
                            let next = peekable_translations_by_language.peek();
                            if let Some((next_language, next_translations)) = next {
                                if *next_translations != translations {
                                    return Err(DatasetValidationError::new_inconsistent_value(
                                        "translations".to_string(),
                                        "".to_string(),
                                        Some(format!(
                                            "when feed_lang is set to \"mul\" each translation should be translated to all languages found in translations.txt; when comparing translations between languages {:?} and {:?}, translations differed",
                                            language, next_language,
                                        )),
                                        vec![feed_info.clone().into()],
                                    ).into());
                                }
                            }
                        }
                    } else {
                        // This means only one language should be in translations.txt (or none)
                        if languages.len() > 1
                            && languages.iter().next().unwrap() != &feed_info.feed_lang
                        {
                            return Err(DatasetValidationError::new_inconsistent_value(
                                "feed_lang".to_string(),
                                feed_info.feed_lang.to_string(),
                                Some(format!(
                                    "feed_lang is inconsistent with the languages found in translations.txt ({:?})",
                                    languages
                                )),
                                vec![feed_info.clone().into()],
                            ).into());
                        }
                    }
                }
                None => {
                    // Check if feed_info is required
                    if !self.translations.is_empty() {
                        return Err(DatasetValidationError::new_missing_value(
                            "feed_info.txt".to_string(),
                            Some(
                                "feed_info.txt is required when translations.txt is provided"
                                    .to_string(),
                            ),
                            vec![],
                        )
                        .into());
                    }
                }
            }
        }

        // Validate attributions:
        // - attribution_id must be unique across all attributions.
        //   -> This is already taken care of because of the use of `Arc<DashMap<AttributionId, Attribution>>`.
        // - agency_id, route_id, and trip_id (if provided) must reference valid IDs in their respective files.
        {
            for attribution in self.attributions.iter() {
                // Validate agency_id reference
                if let Some(agency_id) = &attribution.agency_id {
                    if !self
                        .agencies
                        .iter()
                        .any(|agency| agency.agency_id.as_ref() == Some(agency_id))
                    {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "agency_id".to_string(),
                            agency_id.to_string(),
                            "agency.txt".to_string(),
                            vec![attribution.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate route_id reference
                if let Some(route_id) = &attribution.route_id {
                    if !self.routes.contains_key(route_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "route_id".to_string(),
                            route_id.to_string(),
                            "routes.txt".to_string(),
                            vec![attribution.clone().into()],
                        )
                        .into());
                    }
                }

                // Validate trip_id reference
                if let Some(trip_id) = &attribution.trip_id {
                    if !self.trips.contains_key(trip_id) {
                        return Err(DatasetValidationError::new_foreign_key_not_found(
                            "trip_id".to_string(),
                            trip_id.to_string(),
                            "trips.txt".to_string(),
                            vec![attribution.clone().into()],
                        )
                        .into());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn from_csv(dir: &Path) -> Result<Self> {
        // Get all files in the directory matching the CSV_FILES
        let files = std::fs::read_dir(dir)
            .map_err(|e| ParseError::from(ParseErrorKind::from(e)))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .filter(|entry| {
                CSV_FILES
                    .iter()
                    .any(|file| entry.path().file_name().unwrap().to_str().unwrap() == *file)
            })
            .collect::<Vec<_>>();

        // Read each file and parse it.
        let mut dataset = Self::default();
        for file in files {
            let file_name = file.file_name();
            let file_name = file_name.to_str().unwrap();
            let mut reader = csv::Reader::from_path(file.path())
                .map_err(|e| ParseError::from(ParseErrorKind::from(e)))?;
            let header = reader
                .headers()
                .map_err(|e| ParseError::from(ParseErrorKind::from(e)))?
                .clone();
            for record in reader.records() {
                let record = record.map_err(|e| ParseError::from(ParseErrorKind::from(e)))?;
                let position = record.position().expect("Could not get position of record");
                let wrap_err_with_context = |f: &str| {
                    format!(
                        "Failed to deserialize {} at position: {:?}; Cell: {:?}",
                        f,
                        position,
                        record.get(position.record() as usize).unwrap()
                    )
                };
                match file_name {
                    "agency.txt" => {
                        let record: Agency = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.agencies.push(record);
                    }
                    "stops.txt" => {
                        let record: Stop = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.stops.insert(record.stop_id.clone(), record);
                    }
                    "routes.txt" => {
                        let record: Route = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.routes.insert(record.route_id.clone(), record);
                    }
                    "trips.txt" => {
                        let record: Trip = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.trips.insert(record.trip_id.clone(), record);
                    }
                    "stop_times.txt" => {
                        let record: StopTime = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset
                            .stop_times
                            .insert((record.trip_id.clone(), record.stop_sequence), record);
                    }
                    "calendar.txt" => {
                        let record: Calendar = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.calendar.insert(record.service_id.clone(), record);
                    }
                    "calendar_dates.txt" => {
                        let record: CalendarDate =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset
                            .calendar_dates
                            .insert((record.service_id.clone(), record.date), record);
                    }
                    "fare_attributes.txt" => {
                        let record: FareAttribute =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset
                            .fare_attributes
                            .insert(record.fare_id.clone(), record);
                    }
                    "fare_rules.txt" => {
                        let record: FareRule = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.fare_rules.push(record);
                    }
                    "timeframes.txt" => {
                        let record: Timeframe = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.timeframes.push(record);
                    }
                    "fare_media.txt" => {
                        let record: FareMedia = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset
                            .fare_medias
                            .insert(record.fare_media_id.clone(), record);
                    }
                    "fare_products.txt" => {
                        let record: FareProduct =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset.fare_products.insert(
                            (record.fare_product_id.clone(), record.fare_media_id.clone()),
                            record,
                        );
                    }
                    "fare_leg_rules.txt" => {
                        let record: FareLegRule =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset.fare_leg_rules.push(record);
                    }
                    "fare_transfers.txt" => {
                        let record: FareTransferRule =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset.fare_transfers.push(record);
                    }
                    "areas.txt" => {
                        let record: Area = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.areas.insert(record.area_id.clone(), record);
                    }
                    "stops_areas.txt" => {
                        let record: StopArea = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.stops_areas.push(record);
                    }
                    "networks.txt" => {
                        let record: Network = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.networks.insert(record.network_id.clone(), record);
                    }
                    "routes_networks.txt" => {
                        let record: RouteNetwork =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset
                            .routes_networks
                            .insert(record.route_id.clone(), record);
                    }
                    "shapes.txt" => {
                        let record: Shape = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset
                            .shapes
                            .insert((record.shape_id.clone(), record.shape_pt_sequence), record);
                    }
                    "frequencies.txt" => {
                        let record: Frequency = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset
                            .frequencies
                            .insert((record.trip_id.clone(), record.start_time), record);
                    }
                    "transfers.txt" => {
                        let record: Transfer = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.transfers.push(record);
                    }
                    "pathways.txt" => {
                        let record: Pathway = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.pathways.insert(record.pathway_id.clone(), record);
                    }
                    "levels.txt" => {
                        let record: Level = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.levels.insert(record.level_id.clone(), record);
                    }
                    "location_groups.txt" => {
                        let record: LocationGroup =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset
                            .location_groups
                            .insert(record.location_group_id.clone(), record);
                    }
                    "location_groups_stops.txt" => {
                        let record: LocationGroupStop =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset.location_groups_stops.push(record);
                    }
                    "booking_rules.txt" => {
                        let record: BookingRule =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset
                            .booking_rules
                            .insert(record.booking_rule_id.clone(), record);
                    }
                    "translations.txt" => {
                        let record: Translation =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset.translations.push(record);
                    }
                    "feed_info.txt" => {
                        let record: FeedInfo = record.deserialize(Some(&header)).map_err(|e| {
                            ParseError::from(ParseErrorKind::from(e))
                                .with_context(ErrorContext(wrap_err_with_context(file_name)))
                        })?;
                        dataset.feed_info = Some(record);
                    }
                    "attributions.txt" => {
                        let record: Attribution =
                            record.deserialize(Some(&header)).map_err(|e| {
                                ParseError::from(ParseErrorKind::from(e))
                                    .with_context(ErrorContext(wrap_err_with_context(file_name)))
                            })?;
                        dataset.attributions.push(record);
                    }
                    _ => {}
                }
            }
        }

        Ok(dataset)
    }

    pub fn stop_get_parent_station(&self, stop_id: &StopId) -> Option<Stop> {
        self.stops
            .iter()
            .find(|stop| stop.parent_station == Some(stop_id.clone()))
            .map(|stop| stop.clone().into())
    }

    pub fn stop_get_level(&self, stop_id: &StopId) -> Option<Level> {
        self.levels
            .iter()
            .find(|level| {
                self.stops
                    .get(&stop_id)
                    .map_or(false, |stop| stop.level_id == Some(level.level_id.clone()))
            })
            .map(|level| level.clone().into())
    }

    pub fn stop_get_all_location_groups(&self, stop_id: &StopId) -> Vec<LocationGroup> {
        let location_groups_ids: Vec<&LocationGroupId> = self
            .location_groups_stops
            .iter()
            .filter(|location_group| location_group.stop_id == *stop_id)
            .map(|location_group| location_group.location_group_id.as_wrapper())
            .collect();

        location_groups_ids
            .iter()
            .map(|location_group_id| self.location_groups.get(location_group_id).unwrap().clone())
            .collect()
    }

    pub fn trip_get_all_from_route(&self, route_id: &RouteId) -> Vec<Trip> {
        self.trips
            .iter()
            .filter(|trip| trip.route_id == *route_id)
            .map(|trip| trip.clone().into())
            .collect()
    }

    pub fn stop_times_get_all_from_trip(&self, trip_id: &TripId) -> Vec<StopTime> {
        self.stop_times
            .iter()
            .filter(|stop_time| stop_time.trip_id == *trip_id)
            .map(|stop_time| stop_time.clone().into())
            .collect()
    }

    pub fn stop_times_get_all_from_route(&self, route_id: &RouteId) -> Vec<StopTime> {
        let trips = self.trip_get_all_from_route(route_id);
        trips
            .iter()
            .flat_map(|trip| self.stop_times_get_all_from_trip(&trip.trip_id))
            .collect()
    }
}
