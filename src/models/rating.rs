<<<<<<< HEAD
// src/models/rating.rs
use chrono::DateTime;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VisitorRating {
    pub id: i64,
    pub session_id: String,
    pub visitor_ip: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub rating: i32,
    pub suggestion: String,
    pub user_agent: Option<String>,
    pub is_approved: bool,
    pub created_at: DateTime<chrono::Local>,
    pub updated_at: DateTime<chrono::Local>,
}

#[derive(Debug, Deserialize)]
pub struct RatingRequest {
    pub session_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub rating: i32,
    pub suggestion: String,
}

#[derive(Debug, Serialize)]
pub struct RatingStats {
    pub average_rating: Decimal,
    pub total_ratings: i64,
    pub rating_distribution: Vec<RatingCount>,
    pub recent_ratings: Vec<RatingWithName>,
    pub total_suggestions: i64,
}

#[derive(Debug, Serialize)]
pub struct RatingCount {
    pub stars: i32,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RatingWithName {
    pub id: i64,
    pub name: String,
    pub rating: i32,
    pub suggestion: String,
    pub created_at: DateTime<chrono::Local>,
}

#[derive(Debug, Deserialize)]
pub struct RatingFilter {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub min_rating: Option<i32>,
    pub approved_only: Option<bool>,
}
=======
// src/models/rating.rs
use chrono::DateTime;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VisitorRating {
    pub id: i64,
    pub session_id: String,
    pub visitor_ip: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub rating: i32,
    pub suggestion: String,
    pub user_agent: Option<String>,
    pub is_approved: bool,
    pub created_at: DateTime<chrono::Local>,
    pub updated_at: DateTime<chrono::Local>,
}

#[derive(Debug, Deserialize)]
pub struct RatingRequest {
    pub session_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub rating: i32,
    pub suggestion: String,
}

#[derive(Debug, Serialize)]
pub struct RatingStats {
    pub average_rating: Decimal,
    pub total_ratings: i64,
    pub rating_distribution: Vec<RatingCount>,
    pub recent_ratings: Vec<RatingWithName>,
    pub total_suggestions: i64,
}

#[derive(Debug, Serialize)]
pub struct RatingCount {
    pub stars: i32,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RatingWithName {
    pub id: i64,
    pub name: String,
    pub rating: i32,
    pub suggestion: String,
    pub created_at: DateTime<chrono::Local>,
}

#[derive(Debug, Deserialize)]
pub struct RatingFilter {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub min_rating: Option<i32>,
    pub approved_only: Option<bool>,
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
