// src/controllers/visitor_advanced_controller.rs
use crate::models::visitor::AdvancedStats;
use actix_web::{HttpResponse, get, web};
use sqlx::MySqlPool;
use sqlx::Row;

#[get("/api/stats/advanced")]
pub async fn get_advanced_stats(
    pool: web::Data<MySqlPool>,
    query: web::Query<AdvancedStatsQuery>,
) -> HttpResponse {
    let range = query.range.clone().unwrap_or_default().to_string();

    // Query untuk hourly visits
    let hourly_query = match range.as_str() {
        "24h" => {
            "SELECT HOUR(created_at) as hour, COUNT(*) as count FROM visitors WHERE created_at >= NOW() - INTERVAL 24 HOUR GROUP BY HOUR(created_at) ORDER BY hour"
        }
        "7d" => {
            "SELECT HOUR(created_at) as hour, COUNT(*) as count FROM visitors WHERE created_at >= NOW() - INTERVAL 7 DAY GROUP BY HOUR(created_at) ORDER BY hour"
        }
        _ => {
            "SELECT HOUR(created_at) as hour, COUNT(*) as count FROM visitors WHERE created_at >= NOW() - INTERVAL 30 DAY GROUP BY HOUR(created_at) ORDER BY hour"
        }
    };

    // Query untuk top pages
    let _top_pages_query = match range.as_str() {
        "24h" => {
            "SELECT page_url, COUNT(*) as count FROM visitors WHERE created_at >= NOW() - INTERVAL 24 HOUR GROUP BY page_url ORDER BY count DESC LIMIT 10"
        }
        "7d" => {
            "SELECT page_url, COUNT(*) as count FROM visitors WHERE created_at >= NOW() - INTERVAL 7 DAY GROUP BY page_url ORDER BY count DESC LIMIT 10"
        }
        _ => {
            "SELECT page_url, COUNT(*) as count FROM visitors WHERE created_at >= NOW() - INTERVAL 30 DAY GROUP BY page_url ORDER BY count DESC LIMIT 10"
        }
    };

    match sqlx::query(hourly_query).fetch_all(pool.get_ref()).await {
        Ok(rows) => {
            let hourly_visits = rows
                .iter()
                .map(|row| {
                    let hour: i32 = row.get(0);
                    let count: i64 = row.get(1);
                    crate::models::visitor::HourlyVisit {
                        hour: hour as u32,
                        count,
                    }
                })
                .collect();

            // Lakukan query lain untuk top pages, avg time, dll...
            // Untuk sementara, return dummy data

            let stats = AdvancedStats {
                hourly_visits,
                top_pages: vec![],
                avg_time_on_page: 45.6,
                bounce_rate: 32.1,
                returning_visitors: 123,
                traffic_sources: vec![],
            };

            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct AdvancedStatsQuery {
    pub range: Option<String>,
}
