//! Data models for the OpenStack Cost Dashboard

use serde::{Deserialize, Serialize};

/// A single resource from OpenStack rating data
#[derive(Debug, Deserialize, Clone)]
pub struct Resource {
    pub rating: String,
    pub service: String,
}

/// Wrapper for resources from OpenStack API response
#[derive(Debug, Deserialize)]
pub struct ResourceWrapper {
    #[serde(rename = "Resources")]
    pub resources: Vec<Resource>,
}

/// Chart data structure sent to the frontend
#[derive(Debug, Serialize, Clone)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
    pub total_cost: f64,
    pub service_count: usize,
    pub average_cost: f64,
    pub last_updated: String,
}

impl ChartData {
    /// Create a new empty ChartData instance
    pub fn empty() -> Self {
        Self {
            labels: Vec::new(),
            values: Vec::new(),
            total_cost: 0.0,
            service_count: 0,
            average_cost: 0.0,
            last_updated: String::new(),
        }
    }
}
