//! Server management for the OpenStack Cost Dashboard

use axum::{
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tracing::info;

use crate::config::Config;
use crate::handlers::{serve_index, get_chart_data, refresh_data, health_check, app_info};
use crate::AppState;

/// Server struct managing the web server and background tasks
pub struct Server {
    config: Config,
    app_state: AppState,
}

impl Server {
    /// Create a new server instance
    pub fn new(config: Config, app_state: AppState) -> Self {
        Self {
            config,
            app_state,
        }
    }

    /// Start the server and background tasks
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        // Start background refresh task
        self.start_background_refresh().await;
        
        // Build router
        let app = self.build_router();
        
        // Start server
        let listener = TcpListener::bind(self.config.server_address()).await?;
        info!("Server running on {}", self.config.public_url());
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }

    /// Build the Axum router with all routes
    fn build_router(&self) -> Router {
        Router::new()
            .route("/", get(serve_index))
            .route("/api/data", get(get_chart_data))
            .route("/api/refresh", get(refresh_data))
            .route("/api/health", get(health_check))
            .route("/api/info", get(app_info))
            .with_state(self.app_state.clone())
    }

    /// Start the background task for automatic data refresh
    async fn start_background_refresh(&self) {
        let bg_state = self.app_state.clone();
        let refresh_interval = self.config.refresh_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(refresh_interval);
            loop {
                interval.tick().await;
                info!("Background refresh triggered");
                
                let new_data = bg_state.data_service.fetch_data(None, None).await;
                let new_chart_data = bg_state.data_service.process_data(new_data);
                *bg_state.chart_data.write().await = new_chart_data;
                info!("Background refresh completed successfully");
            }
        });
    }
}
