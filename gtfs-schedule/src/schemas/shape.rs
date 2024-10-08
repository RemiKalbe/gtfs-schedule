//! Provides data structures related to shapes.
//!
//! The main types are:
//! - [`Shape`]: Represents a shape.
//! - [`ShapeId`]: Identifies a shape.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{coord_type, GtfsCoord, Schema};
use crate::error::{Result, SchemaValidationError};

/// Identifies a shape.
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct ShapeId(pub String);

/// Represents a shape.
///
/// Shapes describe the path that a vehicle travels along a route alignment, and are defined in the file [`Shape`].
/// Shapes are associated with Trips, and consist of a sequence of points through which the vehicle passes in order.
/// Shapes do not need to intercept the location of Stops exactly, but all Stops on a trip should lie within a
/// small distance of the shape for that trip, i.e. close to straight line segments connecting the shape points.
///
/// See [shapes.txt](https://gtfs.org/schedule/reference/#shapestxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Shape {
    /// Identifies a shape.
    pub shape_id: ShapeId,
    /// Geographic coordinate of the shape point.
    #[serde(flatten)]
    pub shape_pt: GtfsCoord<{ coord_type::SHAPE }>,
    /// Sequence in which the shape points connect to form the shape. Values must increase along the trip but do not need to be consecutive.
    pub shape_pt_sequence: u32,
    /// Actual distance traveled along the shape from the first shape point to the point specified in this record.
    /// Used by trip planners to show the correct portion of the shape on a map. Values must increase along with
    /// [`Shape::shape_pt_sequence`]; they must not be used to show reverse travel along a route.
    /// Distance units must be consistent with those used in [`crate::schemas::stop_time::StopTime`].
    ///
    /// Recommended for routes that have looping or inlining (the vehicle crosses or travels over the same portion of alignment in one trip).
    ///
    /// Example: If a bus travels a distance of 5.25 kilometers from the start of the shape to the stop, [`Shape::shape_dist_traveled`] = `5.25`.
    pub shape_dist_traveled: Option<f32>,
}

impl Shape {
    /// Validates if the Shape is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate shape_id.
        if self.shape_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "shape_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        // Validate shape_dist_traveled.
        if let Some(dist) = self.shape_dist_traveled {
            if dist < 0.0 {
                return Err(SchemaValidationError::new_invalid_value(
                    "shape_dist_traveled".to_string(),
                    Some("must be positive".to_string()),
                    Schema::from(self.clone()),
                )
                .into());
            }
        }

        Ok(())
    }
}
