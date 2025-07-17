//! Configuration management for the OpenStack Cost Dashboard

use std::env;
use std::time::Duration;
use tracing::{info, warn};

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Server bind address
    pub bind_address: String,
    /// Server port
    pub port: u16,
    /// Data refresh interval in seconds
    pub refresh_interval: Duration,
    /// Currency conversion rate (rating to currency)
    pub currency_rate: f64,
    /// OpenStack CLI command name
    pub openstack_command: String,
    /// OpenStack authentication URL
    pub os_auth_url: String,
    /// OpenStack username
    pub os_username: String,
    /// OpenStack password
    pub os_password: String,
    /// OpenStack project ID
    pub os_project_id: String,
    /// Openstack region name
    pub os_region_name: String,
    /// OpenStack user domain name
    pub os_user_domain_name: String,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl Config {
    /// Create a new configuration with defaults and environment overrides
    pub fn new() -> Self {
        info!("Loading configuration from environment variables...");
        
        let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| {
            info!("Using default BIND_ADDRESS: 0.0.0.0");
            "0.0.0.0".to_string()
        });
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| {
                info!("Using default PORT: 3001");
                "3001".to_string()
            })
            .parse()
            .unwrap_or_else(|e| {
                warn!("Invalid PORT value, using default 3001: {}", e);
                3001
            });
        
        let refresh_interval_secs = env::var("REFRESH_INTERVAL_SECONDS")
            .unwrap_or_else(|_| {
                info!("Using default REFRESH_INTERVAL_SECONDS: 300");
                "300".to_string()
            })
            .parse()
            .unwrap_or_else(|e| {
                warn!("Invalid REFRESH_INTERVAL_SECONDS value, using default 300: {}", e);
                300
            });
        
        let currency_rate = env::var("CURRENCY_RATE")
            .unwrap_or_else(|_| {
                info!("Using default CURRENCY_RATE: 55.5");
                "55.5".to_string()
            })
            .parse()
            .unwrap_or_else(|e| {
                warn!("Invalid CURRENCY_RATE value, using default 55.5: {}", e);
                55.5
            });
        
        let openstack_command = env::var("OPENSTACK_COMMAND").unwrap_or_else(|_| {
            info!("Using default OPENSTACK_COMMAND: openstack");
            "openstack".to_string()
        });
        
        // Load OpenStack authentication variables
        let os_auth_url = env::var("OS_AUTH_URL").unwrap_or_else(|_| {
            warn!("OS_AUTH_URL not set - OpenStack authentication may fail");
            String::new()
        });
        
        let os_username = env::var("OS_USERNAME").unwrap_or_else(|_| {
            warn!("OS_USERNAME not set - OpenStack authentication may fail");
            String::new()
        });
        
        let os_password = env::var("OS_PASSWORD").unwrap_or_else(|_| {
            warn!("OS_PASSWORD not set - OpenStack authentication may fail");
            String::new()
        });
        
        let os_project_id = env::var("OS_PROJECT_ID").unwrap_or_else(|_| {
            warn!("OS_PROJECT_ID not set - OpenStack authentication may fail");
            String::new()
        });
        
        let os_region_name = env::var("OS_REGION_NAME").unwrap_or_else(|_| {
            warn!("OS_REGION_NAME not set - OpenStack data may not be useful");
            String::new()
        });
        
        let os_user_domain_name = env::var("OS_USER_DOMAIN_NAME").unwrap_or_else(|_| {
            info!("Using default OS_USER_DOMAIN_NAME: Default");
            "Default".to_string()
        });
        
        let cache_ttl_seconds = env::var("CACHE_TTL_SECONDS")
            .unwrap_or_else(|_| {
                info!("Using default CACHE_TTL_SECONDS: 1800");
                "1800".to_string()
            })
            .parse()
            .unwrap_or_else(|e| {
                warn!("Invalid CACHE_TTL_SECONDS value, using default 1800: {}", e);
                1800
            });
        
        let config = Self {
            bind_address,
            port,
            refresh_interval: Duration::from_secs(refresh_interval_secs),
            currency_rate,
            openstack_command,
            os_auth_url,
            os_username,
            os_password,
            os_project_id,
            os_region_name,
            os_user_domain_name,
            cache_ttl_seconds,
        };
        
        info!("Configuration loaded successfully:");
        info!("  Server: {}", config.server_address());
        info!("  Refresh interval: {}s", refresh_interval_secs);
        info!("  Currency rate: {}", config.currency_rate);
        info!("  OpenStack command: {}", config.openstack_command);
        
        config
    }

    /// Get the full server bind address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }

    /// Get the public server URL for display
    pub fn public_url(&self) -> String {
        if self.bind_address == "0.0.0.0" {
            format!("http://localhost:{}", self.port)
        } else {
            format!("http://{}:{}", self.bind_address, self.port)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
