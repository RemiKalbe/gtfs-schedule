//! Provides data structures related to networks.
//!
//! The main types are:
//! - [`Network`]: Defines network identifiers that apply for fare leg rules.
//! - [`NetworkId`]: Identifies a network.

use gtfs_schedule_macros::StringWrapper;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::Schema;
use crate::error::{Result, SchemaValidationError};

/// Identifies a network. Must be unique in [`Network`].
#[derive(Serialize, Deserialize, Debug, StringWrapper)]
pub struct NetworkId(pub String);

/// Defines network identifiers that apply for fare leg rules.
///
/// See [networks.txt](https://gtfs.org/schedule/reference/#networkstxt) for more details.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[skip_serializing_none]
pub struct Network {
    /// Identifies a network.
    pub network_id: NetworkId,
    /// The name of the network that apply for fare leg rules, as used by the local agency and its riders.
    pub network_name: Option<String>,
}

impl Network {
    /// Validates if the Network is valid in regards to the GTFS specification constraints.
    pub fn validate(&self) -> Result<()> {
        // Validate network_id.
        if self.network_id.is_empty() {
            return Err(SchemaValidationError::new_missing_value(
                "network_id".to_string(),
                Some("can never be empty".to_string()),
                Schema::from(self.clone()),
            )
            .into());
        }

        Ok(())
    }
}
