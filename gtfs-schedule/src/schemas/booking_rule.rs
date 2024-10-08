//! Provides data structures and enumerations related to booking rules.
//!
//! The main types are:
//! - [`BookingRule`]: Defines the booking rules for rider-requested services.
//! - [`BookingRuleId`]: Identifies a rule.
//! - [`BookingType`]: Indicates how far in advance booking can be made.

use std::time::Duration;

use chrono::NaiveTime;
use gtfs_schedule_macros::StringWrapper;
use phonenumber::PhoneNumber;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use serde_with::skip_serializing_none;
use url::Url;

use crate::error::{Result, SchemaValidationError};

use super::{CalendarServiceId, Schema};

/// Identifies a rule.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct BookingRuleId(pub String);

/// Indicates how far in advance booking can be made.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum BookingType {
    /// Real time booking.
    RealTime = 0,
    /// Up to same-day booking with advance notice.
    SameDayWithNotice = 1,
    /// Up to prior day(s) booking.
    PriorDaysWithNotice = 2,
}

/// Defines the booking rules for rider-requested services.
///
/// See [booking_rules.txt](https://gtfs.org/schedule/reference/#booking_rulestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct BookingRule {
    /// Identifies a rule.
    pub booking_rule_id: BookingRuleId,
    /// Indicates how far in advance booking can be made.
    pub booking_type: BookingType,
    /// Minimum number of minutes before travel to make the request.
    ///
    /// **Conditionally Required:**
    /// - Required for [`BookingType::SameDayWithNotice`].
    /// - Forbidden otherwise.
    pub prior_notice_duration_min: Option<Duration>,
    /// Maximum number of minutes before travel to make the booking request.
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden for [`BookingType::RealTime`] and [`BookingType::PriorDaysWithNotice`].
    /// - Optional for [`BookingType::SameDayWithNotice`].
    pub prior_notice_duration_max: Option<Duration>,
    /// Last day before travel to make the booking request.
    ///
    /// Example: "Ride must be booked 1 day in advance before 5PM" will be encoded as
    /// [`BookingRule::prior_notice_last_day`] = `1`.
    ///
    /// **Conditionally Required:**
    /// - Required for [`BookingType::PriorDaysWithNotice`].
    /// - Forbidden otherwise.
    pub prior_notice_last_day: Option<u32>,
    /// Last time on the last day before travel to make the booking request.
    ///
    /// Example: "Ride must be booked 1 day in advance before 5PM" will be encoded as
    /// [`BookingRule::prior_notice_last_time`] = `17:00:00`.
    ///
    /// **Conditionally Required:**
    /// - Required if [`BookingRule::prior_notice_last_day`] is defined.
    /// - Forbidden otherwise.
    pub prior_notice_last_time: Option<NaiveTime>,
    /// Earliest day before travel to make the booking request.
    ///
    /// Example: "Ride can be booked at the earliest one week in advance at midnight" will be
    /// encoded as [`BookingRule::prior_notice_start_day`] = `7`.
    ///
    /// **Conditionally Forbidden:**
    /// - Forbidden for [`BookingType::RealTime`].
    /// - Forbidden for [`BookingType::SameDayWithNotice`] if [`BookingRule::prior_notice_duration_max`] is defined.
    /// - Optional otherwise.
    pub prior_notice_start_day: Option<u32>,
    /// Earliest time on the earliest day before travel to make the booking request.
    ///
    /// Example: "Ride can be booked at the earliest one week in advance at midnight" will be
    /// encoded as [`BookingRule::prior_notice_start_time`] = `00:00:00`.
    ///
    /// **Conditionally Required:**
    /// - Required if [`BookingRule::prior_notice_start_day`] is defined.
    /// - Forbidden otherwise.
    pub prior_notice_start_time: Option<NaiveTime>,
    /// Indicates the service days on which [`BookingRule::prior_notice_last_day`] or [`BookingRule::prior_notice_start_day`] are counted.
    ///
    /// Example: If empty, [`BookingRule::prior_notice_start_day`] = `2` will be two calendar days in advance.
    /// If defined as a [`CalendarServiceId`] containing only business days (weekdays without holidays),
    /// [`BookingRule::prior_notice_start_day`] = `2` will be two business days in advance.
    ///
    /// **Conditionally Forbidden:**
    /// - Optional if `BookingType::PriorDaysWithNotice`.
    /// - Forbidden otherwise.
    pub prior_notice_service_id: Option<CalendarServiceId>,
    /// Message to riders utilizing service at a [`crate::schemas::stop_time::StopTime`] when booking on-demand pickup and drop off.
    /// Meant to provide minimal information to be transmitted within a user interface about the action
    /// a rider must take in order to utilize the service.
    pub message: Option<String>,
    /// Functions in the same way as [`BookingRule::message`] but used when riders have on-demand pickup only.
    pub pickup_message: Option<String>,
    /// Functions in the same way as [`BookingRule::message`] but used when riders have on-demand drop off only.
    pub drop_off_message: Option<String>,
    /// Phone number to call to make the booking request.
    pub phone_number: Option<PhoneNumber>,
    /// URL providing information about the booking rule.
    pub info_url: Option<Url>,
    /// URL to an online interface or app where the booking request can be made.
    pub booking_url: Option<Url>,
}

impl BookingRule {
    /// Validates if the BookingRule is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate booking_rule_id.
        if self.booking_rule_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "booking_rule_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate prior_notice_duration_min.
        if self.booking_type == BookingType::SameDayWithNotice
            && self.prior_notice_duration_min.is_none()
        {
            return Err(SchemaValidationError::new_missing_value(
                "prior_notice_duration_min".to_string(),
                Some("required when booking_type is SameDayWithNotice".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.booking_type != BookingType::SameDayWithNotice
            && self.prior_notice_duration_min.is_some()
        {
            return Err(SchemaValidationError::new_forbidden_value(
                "prior_notice_duration_min".to_string(),
                Some(
                    "should not be defined when booking_type is not SameDayWithNotice".to_string(),
                ),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate prior_notice_duration_max.
        if (self.booking_type == BookingType::RealTime
            || self.booking_type == BookingType::PriorDaysWithNotice)
            && self.prior_notice_duration_max.is_some()
        {
            return Err(SchemaValidationError::new_forbidden_value(
                "prior_notice_duration_max".to_string(),
                Some(
                    "should not be defined when booking_type is RealTime or PriorDaysWithNotice"
                        .to_string(),
                ),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate prior_notice_last_day.
        if self.booking_type == BookingType::PriorDaysWithNotice
            && self.prior_notice_last_day.is_none()
        {
            return Err(SchemaValidationError::new_missing_value(
                "prior_notice_last_day".to_string(),
                Some("required when booking_type is PriorDaysWithNotice".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.booking_type != BookingType::PriorDaysWithNotice
            && self.prior_notice_last_day.is_some()
        {
            return Err(SchemaValidationError::new_forbidden_value(
                "prior_notice_last_day".to_string(),
                Some(
                    "should not be defined when booking_type is not PriorDaysWithNotice"
                        .to_string(),
                ),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate prior_notice_last_time.
        if self.prior_notice_last_day.is_some() && self.prior_notice_last_time.is_none() {
            return Err(SchemaValidationError::new_missing_value(
                "prior_notice_last_time".to_string(),
                Some("required when prior_notice_last_day is defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.prior_notice_last_day.is_none() && self.prior_notice_last_time.is_some() {
            return Err(SchemaValidationError::new_forbidden_value(
                "prior_notice_last_time".to_string(),
                Some("should not be defined when prior_notice_last_day is not defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate prior_notice_start_day.
        if self.booking_type == BookingType::RealTime && self.prior_notice_start_day.is_some() {
            return Err(SchemaValidationError::new_forbidden_value(
                "prior_notice_start_day".to_string(),
                Some("should not be defined when booking_type is RealTime".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.booking_type == BookingType::PriorDaysWithNotice
            && self.prior_notice_duration_max.is_some()
        {
            return Err(SchemaValidationError::new_forbidden_value(
                "prior_notice_duration_max".to_string(),
                Some("should not be defined when booking_type is PriorDaysWithNotice ".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate prior_notice_start_time.
        if self.prior_notice_start_day.is_some() && self.prior_notice_start_time.is_none() {
            return Err(SchemaValidationError::new_missing_value(
                "prior_notice_start_time".to_string(),
                Some("required when prior_notice_start_day is defined".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }
        if self.prior_notice_start_day.is_none() && self.prior_notice_start_time.is_some() {
            return Err(SchemaValidationError::new_forbidden_value(
                "prior_notice_start_time".to_string(),
                Some(
                    "should not be defined when prior_notice_start_day is not defined".to_string(),
                ),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate prior_notice_service_id.
        if self.booking_type != BookingType::PriorDaysWithNotice
            && self.prior_notice_service_id.is_some()
        {
            return Err(SchemaValidationError::new_forbidden_value(
                "prior_notice_service_id".to_string(),
                Some(
                    "should not be defined when booking_type is not PriorDaysWithNotice"
                        .to_string(),
                ),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
