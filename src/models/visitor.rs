// src/models/visitor.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Visitor {
    pub id: String,
    pub session_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub browser: String,
    pub os: String,
    pub device_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,

    pub page_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_resolution: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_on_page: Option<i32>,

    pub created_at: DateTime<Utc>,
}

// src/models/visitor.rs
#[derive(Debug, Deserialize)]
pub struct VisitorData {
    pub session_id: String,
    pub page_url: String,
    pub pathname: Option<String>,
    pub referrer: Option<String>,
    pub screen_resolution: Option<String>,
    pub language: Option<String>,
    pub time_on_page: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct VisitorStats {
    pub total_visitors: i64,
    pub unique_visitors: i64,
    pub page_views: i64,
    pub browsers: Vec<BrowserStats>,
    pub os_stats: Vec<OSStats>,
    pub countries: Vec<CountryStats>,
    pub daily_visits: Vec<DailyVisit>,
}

#[derive(Debug, Serialize)]
pub struct BrowserStats {
    pub browser: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct OSStats {
    pub os: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CountryStats {
    pub country: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct DailyVisit {
    pub date: String,
    pub visitors: i64,
    pub page_views: i64,
}

// Tambahkan struct untuk advanced stats
#[derive(Debug, Serialize)]
pub struct HourlyVisit {
    pub hour: u32,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct PageStats {
    pub page_url: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct TrafficSource {
    pub source: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct AdvancedStats {
    pub hourly_visits: Vec<HourlyVisit>,
    pub top_pages: Vec<PageStats>,
    pub avg_time_on_page: f64,
    pub bounce_rate: f64,
    pub returning_visitors: i64,
    pub traffic_sources: Vec<TrafficSource>,
}
