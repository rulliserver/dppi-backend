use crate::auth;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, get, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Deserialize)]
pub struct AuditLogQueryParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub module: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub role: Option<String>,
    pub action: String,
    pub module: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

#[get("/api/adminpanel/audit-logs")]
pub async fn get_audit_logs(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    query: web::Query<AuditLogQueryParams>,
) -> Result<impl Responder, Error> {
    // Verify authentication and authorization
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if claims.role != "Superadmin" {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    // Pagination parameters
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    // Build dynamic conditions
    let mut conditions = Vec::new();
    let mut binds: Vec<String> = Vec::new();

    if let Some(ref search) = query.search {
        if !search.is_empty() {
            conditions.push("(username LIKE ? OR action LIKE ? OR module LIKE ? OR details LIKE ?)".to_string());
            let pat = format!("%{}%", search);
            binds.push(pat.clone());
            binds.push(pat.clone());
            binds.push(pat.clone());
            binds.push(pat.clone());
        }
    }

    if let Some(ref status) = query.status {
        if !status.is_empty() {
            conditions.push("status = ?".to_string());
            binds.push(status.clone());
        }
    }

    if let Some(ref module) = query.module {
        if !module.is_empty() {
            conditions.push("module = ?".to_string());
            binds.push(module.clone());
        }
    }

    if let Some(ref start_date) = query.start_date {
        if !start_date.is_empty() {
            conditions.push("created_at >= ?".to_string());
            binds.push(format!("{} 00:00:00", start_date));
        }
    }

    if let Some(ref end_date) = query.end_date {
        if !end_date.is_empty() {
            conditions.push("created_at <= ?".to_string());
            binds.push(format!("{} 23:59:59", end_date));
        }
    }

    let where_clause = if conditions.is_empty() {
        "".to_string()
    } else {
        format!("AND {}", conditions.join(" AND "))
    };

    // Calculate total count
    let count_sql = format!("SELECT COUNT(*) FROM activity_logs WHERE 1=1 {}", where_clause);
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for bind_val in &binds {
        count_q = count_q.bind(bind_val);
    }
    
    let total = count_q
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Fetch data with pagination
    let data_sql = format!(
        "SELECT id, user_id, username, role, action, module, ip_address, user_agent, status, details, created_at FROM activity_logs WHERE 1=1 {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );
    let mut data_q = sqlx::query_as::<_, AuditLog>(&data_sql);
    for bind_val in &binds {
        data_q = data_q.bind(bind_val);
    }
    data_q = data_q.bind(limit as i64).bind(offset as i64);

    let logs = data_q
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let total_pages = (total as f64 / limit as f64).ceil() as u64;

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: logs,
        total,
        page,
        limit,
        total_pages,
    }))
}
