//! High-level discovery API
//!
//! Coming soon: `NginxDiscovery` implementation

use crate::error::Result;

/// High-level NGINX configuration discovery
pub struct NginxDiscovery {
    // TODO: Add fields
}

impl NginxDiscovery {
    /// Discover from running NGINX instance (stub)
    #[cfg(feature = "system")]
    pub fn from_running_instance() -> Result<Self> {
        // TODO: Implement
        unimplemented!("Coming soon")
    }

    /// Discover from config file (stub)
    pub fn from_config_file(_path: impl AsRef<std::path::Path>) -> Result<Self> {
        // TODO: Implement
        unimplemented!("Coming soon")
    }

    /// Discover from config text (stub)
    pub fn from_config_text(_text: &str) -> Result<Self> {
        // TODO: Implement
        unimplemented!("Coming soon")
    }
}
