//! Driver registry lookups the frontend asks for before it can render a
//! connection form or a type picker.
//!
//! Extracted from the legacy `commands` module so `application/` no longer
//! depends on it. Behaviour is unchanged.

use std::sync::Arc;

use crate::drivers::driver_trait::{DatabaseDriver, PluginManifest};
use crate::models::DataTypeRegistry;
use crate::plugins::connection_metadata::DriverManifestDetails;

/// Resolve the driver from the registry or return a descriptive error.
async fn driver_for(id: &str) -> Result<Arc<dyn DatabaseDriver>, String> {
    crate::drivers::registry::get_driver(id)
        .await
        .ok_or_else(|| format!("Unsupported driver: {}", id))
}

/// Snapshot of connection ids currently open in the shared backend (across all
/// windows). Used by each window to render cross-window connection status.
pub async fn get_active_connections() -> Vec<String> {
    crate::health_check::active_connections().await
}

pub async fn get_data_types(driver: String) -> Result<DataTypeRegistry, String> {
    log::debug!("Fetching data types for driver: {}", driver);

    let drv = driver_for(&driver).await?;
    let types = drv.get_data_types();

    Ok(DataTypeRegistry { driver, types })
}

pub async fn get_registered_drivers() -> Vec<PluginManifest> {
    crate::drivers::registry::list_drivers().await
}

pub async fn get_driver_manifest(driver_id: String) -> Option<DriverManifestDetails> {
    crate::drivers::registry::get_driver(&driver_id)
        .await
        .map(|d| DriverManifestDetails {
            manifest: d.manifest().clone(),
            connection_metadata: d.has_connection_metadata().then_some(true),
        })
}
