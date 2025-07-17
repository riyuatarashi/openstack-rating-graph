//! Data fetching and processing for the OpenStack Cost Dashboard

use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{info, warn};
use chrono::Local;

use crate::models::{ChartData, ResourceWrapper};
use crate::config::Config;
use crate::cache::OpenStackCache;

/// Data service for fetching and processing OpenStack data
#[derive(Clone)]
pub struct DataService {
    config: Config,
    cache: Arc<OpenStackCache>,
}

impl DataService {
    /// Create a new data service
    pub fn new(config: Config, cache: Arc<OpenStackCache>) -> Self {
        Self { config, cache }
    }

    /// Fetch data from OpenStack CLI with caching
    pub async fn fetch_data(&self, begin_at: Option<String>, end_at: Option<String>) -> HashMap<String, f64> {
        // Generate the date string in the same format as the shell command
        let begin_at_date_string = self.get_date_string(begin_at);
        let end_at_date_string = self.get_date_string(match end_at {
            Some(end_at) => Some(end_at),
            None => Some(Local::now().format("%Y-%m-%d").to_string()),
        });
        
        // Build arguments with authentication parameters
        let mut args = Vec::new();
        
        // Add authentication parameters if available
        if !self.config.os_auth_url.is_empty() {
            args.push("--os-auth-url".to_string());
            args.push(self.config.os_auth_url.clone());
        }
        
        if !self.config.os_username.is_empty() {
            args.push("--os-username".to_string());
            args.push(self.config.os_username.clone());
        }
        
        if !self.config.os_password.is_empty() {
            args.push("--os-password".to_string());
            args.push(self.config.os_password.clone());
        }
        
        if !self.config.os_project_id.is_empty() {
            args.push("--os-project-id".to_string());
            args.push(self.config.os_project_id.clone());
        }
        
        if !self.config.os_region_name.is_empty() {
            args.push("--os-region-name".to_string());
            args.push(self.config.os_region_name.clone());
        }
        
        if !self.config.os_user_domain_name.is_empty() {
            args.push("--os-user-domain-name".to_string());
            args.push(self.config.os_user_domain_name.clone());
        }
        
        // Add the main command arguments
        args.extend([
            "rating".to_string(),
            "dataframes".to_string(),
            "get".to_string(),
            "-b".to_string(),
            begin_at_date_string,
            "-e".to_string(),
            end_at_date_string,
            "-c".to_string(),
            "Resources".to_string(),
            "-f".to_string(),
            "json".to_string(),
        ]);
        
        // Generate a cache key from command and args
        let cache_key = self.cache.generate_key(&self.config.openstack_command, &args);
        
        // Check cache first
        if let Some(cached_data) = self.cache.get(&cache_key).await {
            info!("Using cached data for OpenStack query");
            return cached_data;
        }
        
        // Create a redacted version of args for logging
        let redacted_args = self.redact_sensitive_args(&args);
        info!("Executing command: {} {}", self.config.openstack_command, redacted_args.join(" "));
        
        let output = Command::new(&self.config.openstack_command)
            .args(&args)
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                let json_str = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<Vec<ResourceWrapper>>(&json_str) {
                    Ok(resources) => {
                        let data_map = self.process_resources(resources);
                        info!("Successfully fetched data for {} services", data_map.len());
                        
                        // Cache the result with configured TTL
                        self.cache.set(
                            cache_key.clone(),
                            data_map.clone()
                        ).await;
                        
                        data_map
                    }
                    Err(e) => {
                        warn!("Failed to parse JSON data: {}", e);
                        warn!("Raw output: {}", json_str);
                        HashMap::new()
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                warn!("OpenStack command failed with status: {}", output.status);
                if !stderr.is_empty() {
                    warn!("Error output: {}", stderr.trim());
                }
                if !stdout.is_empty() {
                    warn!("Standard output: {}", stdout.trim());
                }
                
                // Check for common authentication errors
                if stderr.contains("auth-url") || stderr.contains("Missing value") {
                    warn!("OpenStack authentication not configured. Please set up your OpenStack credentials.");
                    warn!("You can do this by sourcing an OpenStack RC file or setting environment variables.");
                    warn!("Example: source ~/openstack-rc.sh");
                }
                
                HashMap::new()
            }
            Err(e) => {
                warn!("Failed to execute OpenStack command: {}", e);
                warn!("Make sure the OpenStack CLI is installed and in your PATH");
                HashMap::new()
            }
        }
    }

    /// Process fetched resources into a hashmap
    fn process_resources(&self, resources: Vec<ResourceWrapper>) -> HashMap<String, f64> {
        let mut data_map = HashMap::new();
        for wrapped in resources.into_iter() {
            for resource in wrapped.resources.into_iter() {
                if let Ok(rating) = resource.rating.parse::<f64>() {
                    let cost = rating / self.config.currency_rate;
                    *data_map.entry(resource.service).or_insert(0.0) += cost;
                }
            }
        }
        data_map
    }

    /// Process data into chart-ready format
    pub fn process_data(&self, data: HashMap<String, f64>) -> ChartData {
        let mut sorted_data: Vec<(String, f64)> = data.iter().map(|(k, v)| (k.clone(), *v)).collect();
        sorted_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let labels: Vec<String> = sorted_data.iter().map(|(service, _)| service.clone()).collect();
        let values: Vec<f64> = sorted_data.iter().map(|(_, cost)| *cost).collect();
        let total_cost: f64 = values.iter().sum();
        let service_count = labels.len();
        let average_cost = if service_count > 0 {
            total_cost / service_count as f64
        } else {
            0.0
        };

        ChartData {
            labels,
            values,
            total_cost,
            service_count,
            average_cost,
            last_updated: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    /// Get the formatted date string that would be used in the OpenStack command
    pub fn get_date_string(&self, date: Option<String>) -> String {
        match date {
            Some(d) => {
                // Parse the provided date string
                match chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d") {
                    Ok(date) => date.format("%Y-%m-%dT00:00:00+00:00").to_string(),
                    Err(e) => {
                        warn!("Invalid date format '{}', using current date: {}", d, e);
                        Local::now().format("%Y-%m-01T00:00:00+00:00").to_string()
                    }
                }
            }
            None => Local::now().format("%Y-%m-01T00:00:00+00:00").to_string(),
        }
    }
    
    /// Create a redacted version of command arguments for safe logging
    fn redact_sensitive_args(&self, args: &[String]) -> Vec<String> {
        let mut redacted_args = Vec::new();
        let mut i = 0;
        
        while i < args.len() {
            let arg = &args[i];
            
            // Check if this is a sensitive parameter flag
            if arg == "--os-password" || arg == "--os-project-id" {
                redacted_args.push(arg.clone());
                // If there's a next argument (the value), replace it with [REDACTED]
                if i + 1 < args.len() {
                    redacted_args.push("[REDACTED]".to_string());
                    i += 1; // Skip the next argument since we've processed it
                }
            } else {
                redacted_args.push(arg.clone());
            }
            
            i += 1;
        }
        
        redacted_args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_date_formatting() {
        let config = Config {
            bind_address: "0.0.0.0".to_string(),
            port: 3001,
            refresh_interval: std::time::Duration::from_secs(300),
            currency_rate: 55.5,
            openstack_command: "openstack".to_string(),
            os_auth_url: String::new(),
            os_username: String::new(),
            os_password: String::new(),
            os_project_id: String::new(),
            os_region_name: "rc3-a".to_string(),
            os_user_domain_name: "Default".to_string(),
            cache_ttl_seconds: 300,
        };
        let cache = Arc::new(OpenStackCache::new(std::time::Duration::from_secs(300)));
        let service = DataService::new(config, cache.clone());
        let date_string = service.get_date_string(None);
        
        // Test that the date matches the expected format: YYYY-MM-01T00:00:00+00:00
        let date_regex = Regex::new(r"^\d{4}-\d{2}-01T00:00:00\+00:00$").unwrap();
        assert!(date_regex.is_match(&date_string), "Date format should match YYYY-MM-01T00:00:00+00:00, got: {}", date_string);
    }
}
