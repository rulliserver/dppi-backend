use crate::auth;
use crate::models::rating::{
    RatingCount, RatingFilter, RatingRequest, RatingStats, RatingWithName, VisitorRating,
};

use actix_web::{Error, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use chrono::Utc;

use rust_decimal::Decimal;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;
// Submit rating baru
#[post("/api/ratings")]
pub async fn submit_rating(
    req: actix_web::HttpRequest,
    pool: web::Data<MySqlPool>,
    data: web::Json<RatingRequest>,
) -> HttpResponse {
    // Validasi rating (1-5 stars)
    if data.rating < 1 || data.rating > 5 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Rating must be between 1 and 5 stars"
        }));
    }

    // Validasi suggestion tidak kosong
    if data.suggestion.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Suggestion cannot be empty"
        }));
    }

    // Cegah spam: maksimal 1 rating per session per halaman per hari
    let today = Utc::now().date_naive();

    let existing_rating = sqlx::query!(
        "SELECT id FROM rating_limits
         WHERE session_id = ?
         AND DATE(created_at) = ?",
        data.session_id,
        today
    )
    .fetch_optional(pool.get_ref())
    .await;

    if let Ok(Some(_)) = existing_rating {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "You have already submitted a rating for this page today",
            "code": "RATE_LIMIT_EXCEEDED"
        }));
    }

    // Get IP address
    let ip_address = get_ip_address(&req);
    let user_agent = get_user_agent(&req);

    let now = Utc::now();

    // Mulai transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to begin transaction: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Insert rating
    let rating_result = sqlx::query!(
        r#"
        INSERT INTO visitor_ratings
        (session_id, visitor_ip, email, name, rating, suggestion, user_agent, is_approved)
        VALUES (?, ?, ?, ?, ?, ?, ?, FALSE)
        "#,
        data.session_id,
        ip_address,
        data.email,
        data.name,
        data.rating,
        data.suggestion,
        user_agent
    )
    .execute(&mut *tx)
    .await;

    match rating_result {
        Ok(result) => {
            let rating_id = result.last_insert_id();

            // Insert ke rating_limits untuk cegah spam
            let limit_result = sqlx::query!(
                "INSERT INTO rating_limits (session_id, rating_id, created_at)
                 VALUES (?, ?, ?)",
                data.session_id,
                rating_id,
                now
            )
            .execute(&mut *tx)
            .await;

            match limit_result {
                Ok(_) => {
                    // Commit transaction
                    if let Err(e) = tx.commit().await {
                        eprintln!("Failed to commit transaction: {}", e);
                        return HttpResponse::InternalServerError().finish();
                    }

                    HttpResponse::Created().json(serde_json::json!({
                        "success": true,
                        "message": "Thank you for your feedback!",
                        "rating_id": rating_id,
                        "rating": data.rating,
                        "timestamp": now.to_rfc3339()
                    }))
                }
                Err(e) => {
                    eprintln!("Failed to insert rating limit: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to save rating"
                    }))
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to save rating to database"
            }))
        }
    }
}

// Get rating statistics
#[get("/api/ratings/stats")]
pub async fn get_rating_stats(
    pool: web::Data<MySqlPool>,
    query: web::Query<RatingFilter>,
) -> HttpResponse {
    let approved_only = query.approved_only.unwrap_or(true);

    // ---------- BUILD WHERE + BINDS ----------
    let mut conditions: Vec<&str> = Vec::new();
    let mut binds: Vec<i32> = Vec::new();

    if approved_only {
        conditions.push("is_approved = TRUE");
    }

    if let Some(min) = query.min_rating {
        conditions.push("rating >= ?");
        binds.push(min);
    }

    let where_sql = if conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // ---------- TOTAL RATINGS ----------
    let total_sql = format!("SELECT COUNT(*) FROM visitor_ratings {}", where_sql);
    let mut total_q = sqlx::query_scalar::<_, i64>(&total_sql);

    for b in &binds {
        total_q = total_q.bind(b);
    }

    let total_ratings = match total_q.fetch_one(pool.get_ref()).await {
        Ok(v) => v,
        Err(e) => return sql_error("total_ratings", e),
    };

    // ---------- AVERAGE RATING (ANTI NULL) ----------
    let avg_sql = format!(
        "SELECT COALESCE(AVG(rating), 0) FROM visitor_ratings {}",
        where_sql
    );
    let mut avg_q = sqlx::query_scalar::<_, Decimal>(&avg_sql);

    for b in &binds {
        avg_q = avg_q.bind(b);
    }

    let average_rating = match avg_q.fetch_one(pool.get_ref()).await {
        Ok(v) => v,
        Err(e) => return sql_error("average_rating", e),
    };

    // ---------- DISTRIBUTION ----------
    let dist_sql = format!(
        r#"
    SELECT rating AS stars, COUNT(*) AS count
    FROM visitor_ratings {}
    GROUP BY rating
    "#,
        where_sql
    );

    let mut dist_q = sqlx::query(&dist_sql);

    for b in &binds {
        dist_q = dist_q.bind(b);
    }

    let rows = match dist_q.fetch_all(pool.get_ref()).await {
        Ok(v) => v,
        Err(e) => return sql_error("distribution", e),
    };

    let mut distribution = (1..=5)
        .map(|stars| RatingCount {
            stars,
            count: 0,
            percentage: 0.0,
        })
        .collect::<Vec<_>>();

    for row in rows {
        let stars: i32 = row.get("stars");
        let count: i64 = row.get("count");

        if let Some(r) = distribution.iter_mut().find(|r| r.stars == stars) {
            r.count = count;
            r.percentage = if total_ratings > 0 {
                (count as f64 / total_ratings as f64) * 100.0
            } else {
                0.0
            };
        }
    }

    // ---------- RECENT RATINGS ----------
    let recent = match sqlx::query_as::<_, RatingWithName>(
        r#"
        SELECT id,
               COALESCE(name, 'Anonymous') AS name,
               rating,
               suggestion,
               created_at
        FROM visitor_ratings
        WHERE is_approved = TRUE
        ORDER BY created_at DESC
        LIMIT 10
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(v) => v,
        Err(e) => return sql_error("recent_ratings", e),
    };

    // ---------- RESPONSE ----------
    HttpResponse::Ok().json(RatingStats {
        average_rating,
        total_ratings,
        rating_distribution: distribution,
        recent_ratings: recent,
        total_suggestions: total_ratings,
    })
}

// ================= ERROR HELPER =================
fn sql_error(step: &str, e: sqlx::Error) -> HttpResponse {
    eprintln!("SQL ERROR [{}]: {:#?}", step, e);
    HttpResponse::InternalServerError().json(json!({
        "error": "Database error",
        "step": step,
        "details": e.to_string()
    }))
}

#[get("/api/ratings")]

pub async fn get_ratings(
    pool: web::Data<MySqlPool>,
    query: web::Query<RatingFilter>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // 1. Verifikasi JWT
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // 2. Cek otorisasi role
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;

    // 3. Query untuk data rating
    let ratings_query = r#"
        SELECT
            id,
            session_id,
            visitor_ip,
            email,
            name,
            rating,
            suggestion,
            user_agent,
            is_approved,
            created_at,
            updated_at
        FROM visitor_ratings
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
    "#;

    let ratings = sqlx::query_as::<_, VisitorRating>(ratings_query)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to fetch ratings: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengambil data rating")
        })?;

    // 4. Query untuk total count
    let count_query = "SELECT COUNT(*) as count FROM visitor_ratings";
    let count_result = sqlx::query(count_query)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to fetch count: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menghitung total data")
        })?;

    let total_count: i64 = count_result.get("count");

    // 5. Hitung pagination
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

    // 6. Build response
    let response = json!({
        "ratings": ratings,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total_count,
            "total_pages": total_pages,
            "has_next": page < total_pages,
            "has_prev": page > 1
        },
        "user_info": {
            "user_id": claims.user_id,
            "nama_user": claims.nama_user,
            "role": claims.role
        }
    });

    Ok(HttpResponse::Ok().json(response))
}

// Admin: Update rating approval status
#[put("/api/ratings/{id}/approve")]
pub async fn approve_rating(pool: web::Data<MySqlPool>, id: web::Path<i64>) -> HttpResponse {
    match sqlx::query!(
        "UPDATE visitor_ratings SET is_approved = TRUE WHERE id = ?",
        *id
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "Rating approved"
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Rating not found"
                }))
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to approve rating"
            }))
        }
    }
}

// Admin: Delete rating
#[delete("/api/ratings/{id}")]
pub async fn delete_rating(pool: web::Data<MySqlPool>, id: web::Path<i64>) -> HttpResponse {
    match sqlx::query!("DELETE FROM visitor_ratings WHERE id = ?", *id)
        .execute(pool.get_ref())
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "Rating deleted"
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Rating not found"
                }))
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete rating"
            }))
        }
    }
}

// Helper function untuk mendapatkan IP address
fn get_ip_address(req: &HttpRequest) -> String {
    // Coba ambil dari X-Forwarded-For (jika di belakang proxy)
    if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_ip) = forwarded_for.to_str() {
            // Ambil IP pertama (client asli) dari daftar
            if let Some(client_ip) = forwarded_ip.split(',').next() {
                let trimmed_ip = client_ip.trim();
                if !trimmed_ip.is_empty() {
                    return trimmed_ip.to_string();
                }
            }
        }
    }

    // Coba dari X-Real-IP
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip) = real_ip.to_str() {
            if !ip.trim().is_empty() {
                return ip.trim().to_string();
            }
        }
    }

    // Coba dari CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = req.headers().get("CF-Connecting-IP") {
        if let Ok(ip) = cf_ip.to_str() {
            if !ip.trim().is_empty() {
                return ip.trim().to_string();
            }
        }
    }

    // Fallback ke connection info
    req.connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string()
}

// Helper function untuk mendapatkan User Agent
fn get_user_agent(req: &HttpRequest) -> String {
    req.headers()
        .get("User-Agent")
        .map(|h| h.to_str().unwrap_or("unknown"))
        .unwrap_or("unknown")
        .to_string()
}
