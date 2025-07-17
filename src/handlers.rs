//! HTTP handlers for the OpenStack Cost Dashboard API

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
};
use axum::extract::Query;
use serde::Deserialize;
use tracing::info;
use crate::models::ChartData;
use crate::AppState;

#[derive(Deserialize)]
pub struct DateRange {
    begin_at: Option<String>,
    end_at: Option<String>,
}

/// Serve the main HTML page
pub async fn serve_index() -> Html<String> {
    Html(include_str!("../templates/index.html").to_string())
}

/// Get current chart data
pub async fn get_chart_data(State(state): State<AppState>) -> Json<ChartData> {
    let data = state.chart_data.read().await;
    Json(data.clone())
}

/// Refresh data manually
pub async fn refresh_data(State(state): State<AppState>, Query(date_range): Query<DateRange>) -> Json<ChartData> {
    info!("Manual refresh requested");
    
    let new_data = state.data_service.fetch_data(date_range.begin_at, date_range.end_at).await;
    let new_chart_data = state.data_service.process_data(new_data);
    *state.chart_data.write().await = new_chart_data.clone();
    Json(new_chart_data)
}

/// Health check endpoint
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Get application information
pub async fn app_info() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "OpenStack Cost Dashboard",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "A web dashboard for OpenStack cost visualization"
    }))
}
