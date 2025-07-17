//! OpenStack Cost Dashboard
//!
//! A web application that fetches OpenStack rating data and displays it in a Chart.js dashboard.
//! Features include real-time data fetching, interactive charts, and auto-refresh capabilities.

mod models;
mod handlers;
mod data;
mod config;
mod server;
mod cache;

use std::sync::Arc;
use chrono::Local;
use tokio::sync::RwLock;

use crate::models::ChartData;
use crate::data::DataService;
use crate::server::Server;
use crate::config::Config;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub chart_data: Arc<RwLock<ChartData>>,
    pub data_service: DataService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::new();
    
    // Initialize cache
    let cache = Arc::new(cache::OpenStackCache::new(std::time::Duration::from_secs(300)));
    
    // Initialize data service
    let data_service = DataService::new(config.clone(), cache.clone());
    
    // Fetch initial data
    let initial_data = data_service.fetch_data(
        Some(Local::now().format("%Y-%m-01").to_string()),
        Some(Local::now().format("%Y-%m-%d").to_string())
    ).await;

    let chart_data = data_service.process_data(initial_data);
    let chart_data_state = Arc::new(RwLock::new(chart_data));
    
    // Create combined app state
    let app_state = AppState {
        chart_data: chart_data_state,
        data_service: data_service.clone(),
    };
    
    // Start the server
    let server = Server::new(config, app_state);
    server.start().await?;
    
    Ok(())
}
