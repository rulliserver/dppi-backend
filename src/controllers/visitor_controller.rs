use crate::models::visitor::{
    BrowserStats, CountryStats, DailyVisit, OSStats, Visitor, VisitorData, VisitorStats,
};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use sqlx::MySqlPool;
use uuid::Uuid;

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

// Helper function untuk parsing User Agent
fn parse_user_agent(user_agent_str: &str) -> (String, String, String) {
    let ua = user_agent_str.to_lowercase();

    // Parse browser
    let browser = if ua.contains("chrome") && !ua.contains("chromium") && !ua.contains("edg") {
        "Chrome"
    } else if ua.contains("firefox") {
        "Firefox"
    } else if ua.contains("safari") && !ua.contains("chrome") && !ua.contains("android") {
        "Safari"
    } else if ua.contains("edge") || ua.contains("edg/") {
        "Edge"
    } else if ua.contains("opera") || ua.contains("opr/") {
        "Opera"
    } else if ua.contains("brave") {
        "Brave"
    } else if ua.contains("msie") || ua.contains("trident") {
        "Internet Explorer"
    } else if ua.contains("vivaldi") {
        "Vivaldi"
    } else if ua.contains("samsungbrowser") {
        "Samsung Browser"
    } else if ua.contains("ucbrowser") {
        "UC Browser"
    } else {
        "Unknown"
    };

    // Parse OS
    let os = if ua.contains("windows nt 10") || ua.contains("windows 10") {
        "Windows 10"
    } else if ua.contains("windows nt 11") || ua.contains("windows 11") {
        "Windows 11"
    } else if ua.contains("windows nt 6.3") {
        "Windows 8.1"
    } else if ua.contains("windows nt 6.2") {
        "Windows 8"
    } else if ua.contains("windows nt 6.1") {
        "Windows 7"
    } else if ua.contains("windows nt 6.0") {
        "Windows Vista"
    } else if ua.contains("windows nt 5.1") || ua.contains("windows xp") {
        "Windows XP"
    } else if ua.contains("mac os x") || ua.contains("macos") || ua.contains("darwin") {
        "macOS"
    } else if ua.contains("linux") && !ua.contains("android") {
        "Linux"
    } else if ua.contains("android") {
        "Android"
    } else if ua.contains("ios") || ua.contains("iphone") {
        "iOS"
    } else if ua.contains("ipad") {
        "iPadOS"
    } else if ua.contains("cros") {
        "Chrome OS"
    } else {
        "Unknown"
    };

    // Parse device type
    let device_type = get_device_type(user_agent_str);

    (browser.to_string(), os.to_string(), device_type)
}

// Helper function untuk device type
fn get_device_type(user_agent: &str) -> String {
    let ua = user_agent.to_lowercase();

    if ua.contains("mobile") && !ua.contains("tablet") && !ua.contains("ipad") {
        "Mobile"
    } else if ua.contains("tablet") || ua.contains("ipad") {
        "Tablet"
    } else if ua.contains("tv") || ua.contains("smart-tv") || ua.contains("smarttv") {
        "TV"
    } else if ua.contains("bot") || ua.contains("crawler") || ua.contains("spider") {
        "Bot"
    } else if ua.contains("playstation") || ua.contains("xbox") || ua.contains("nintendo") {
        "Gaming Console"
    } else if ua.contains("watch") || ua.contains("wearable") {
        "Wearable"
    } else if ua.contains("car") || ua.contains("automotive") {
        "Automotive"
    } else if ua.contains("iot") || ua.contains("embedded") {
        "IoT"
    } else {
        "Desktop"
    }
    .to_string()
}

// Helper function untuk geolocation (sederhana)
fn get_geolocation(ip: &str) -> (Option<String>, Option<String>, Option<f64>, Option<f64>) {
    // Skip localhost dan IP private
    let ip_lower = ip.to_lowercase();
    if ip_lower == "127.0.0.1"
        || ip_lower == "::1"
        || ip_lower == "localhost"
        || ip.starts_with("192.168.")
        || ip.starts_with("10.")
        || ip.starts_with("172.16.")
        || ip.starts_with("172.17.")
        || ip.starts_with("172.18.")
        || ip.starts_with("172.19.")
        || ip.starts_with("172.20.")
        || ip.starts_with("172.21.")
        || ip.starts_with("172.22.")
        || ip.starts_with("172.23.")
        || ip.starts_with("172.24.")
        || ip.starts_with("172.25.")
        || ip.starts_with("172.26.")
        || ip.starts_with("172.27.")
        || ip.starts_with("172.28.")
        || ip.starts_with("172.29.")
        || ip.starts_with("172.30.")
        || ip.starts_with("172.31.")
        || ip.starts_with("169.254.")
        || ip.starts_with("fc00:")
        || ip.starts_with("fe80:")
        || ip == "::"
        || ip == "0:0:0:0:0:0:0:1"
    {
        return (None, None, None, None);
    }

    // TODO: Implementasi dengan MaxMind DB jika diperlukan
    // Untuk sekarang, return None
    (None, None, None, None)
}

// Helper function untuk extract pathname dari URL
fn extract_pathname(url: &str) -> String {
    // Coba parse URL
    if let Ok(parsed_url) = url::Url::parse(url) {
        let path = parsed_url.path();
        if path.is_empty() || path == "/" {
            "/".to_string()
        } else {
            path.to_string()
        }
    } else {
        // Jika parsing gagal, coba ekstrak secara manual
        if let Some(hash_index) = url.find('#') {
            let without_hash = &url[..hash_index];
            if let Some(query_index) = without_hash.find('?') {
                let path = &without_hash[..query_index];
                if let Some(domain_end) = path.find("://") {
                    if let Some(path_start) = path[domain_end + 3..].find('/') {
                        return path[domain_end + 3 + path_start..].to_string();
                    }
                }
                return path.to_string();
            }
            return without_hash.to_string();
        }

        // Default
        "/".to_string()
    }
}

#[post("/api/track")] // Ubah ke POST untuk menerima JSON body
pub async fn track_visitor(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    data: web::Json<VisitorData>,
) -> HttpResponse {
    // Get IP address menggunakan helper
    let ip_address = get_ip_address(&req);

    // Get user agent menggunakan helper
    let user_agent_str = get_user_agent(&req);

    // Parse browser, OS, dan device type
    let (browser, os, device_type) = parse_user_agent(&user_agent_str);

    // Get geolocation
    let (country, city, lat, lon) = get_geolocation(&ip_address);

    // Extract pathname dari page_url jika tidak disediakan
    let pathname = data
        .pathname
        .clone()
        .unwrap_or_else(|| extract_pathname(&data.page_url));

    // Buat visitor dengan tipe data yang tepat
    let visitor = Visitor {
        id: Uuid::new_v4().to_string(),
        session_id: data.session_id.clone(),
        ip_address: ip_address.clone(),
        user_agent: user_agent_str.clone(),
        browser: browser.clone(),
        os: os.clone(),
        device_type: device_type.clone(),
        referrer: data.referrer.clone(),
        page_url: data.page_url.clone(),
        country: country.clone(),
        city: city.clone(),
        latitude: lat,
        longitude: lon,
        screen_resolution: data.screen_resolution.clone(),
        language: data.language.clone(),
        time_on_page: data.time_on_page,
        created_at: Utc::now(),
    };

    // Debug log
    println!("📥 Tracking visitor:");
    println!("  Session: {}", visitor.session_id);
    println!("  IP: {}", visitor.ip_address);
    println!("  Path: {}", pathname);
    println!("  Browser: {}", visitor.browser);
    println!("  OS: {}", visitor.os);
    println!("  Device: {}", visitor.device_type);

    match sqlx::query(
        r#"
        INSERT INTO visitors (
            id, session_id, ip_address, user_agent, browser, os, device_type,
            referrer, page_url, country, city, latitude, longitude,
            screen_resolution, language, time_on_page, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&visitor.id)
    .bind(&visitor.session_id)
    .bind(&visitor.ip_address)
    .bind(&visitor.user_agent)
    .bind(&visitor.browser)
    .bind(&visitor.os)
    .bind(&visitor.device_type)
    .bind(&visitor.referrer)
    .bind(&visitor.page_url)
    .bind(&visitor.country)
    .bind(&visitor.city)
    .bind(&visitor.latitude)
    .bind(&visitor.longitude)
    .bind(&visitor.screen_resolution)
    .bind(&visitor.language)
    .bind(&visitor.time_on_page)
    .bind(&visitor.created_at)
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            println!(
                "✅ Visitor saved. Rows affected: {}",
                result.rows_affected()
            );

            // Response sukses
            HttpResponse::Created().json(serde_json::json!({
                "success": true,
                "message": "Visitor tracked successfully",
                "visitor_id": visitor.id,
                "session_id": visitor.session_id,
                "timestamp": visitor.created_at.to_rfc3339(),
                "pathname": pathname
            }))
        }
        Err(e) => {
            eprintln!("❌ Database error: {}", e);
            eprintln!("   Error details: {:?}", e);

            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Failed to save visitor data",
                "details": e.to_string()
            }))
        }
    }
}

#[get("/api/stats")]
pub async fn get_stats(pool: web::Data<MySqlPool>) -> HttpResponse {
    // Gunakan query yang lebih aman dengan Result handling
    let total_result = sqlx::query!("SELECT COUNT(*) as count FROM visitors")
        .fetch_one(pool.get_ref())
        .await;

    let unique_result = sqlx::query!("SELECT COUNT(DISTINCT session_id) as count FROM visitors")
        .fetch_one(pool.get_ref())
        .await;

    let browser_stats_result: Result<Vec<_>, _> =
        sqlx::query!("SELECT browser, COUNT(*) as count FROM visitors GROUP BY browser")
            .fetch_all(pool.get_ref())
            .await;

    let os_stats_result: Result<Vec<_>, _> =
        sqlx::query!("SELECT os, COUNT(*) as count FROM visitors GROUP BY os")
            .fetch_all(pool.get_ref())
            .await;

    let country_stats_result: Result<Vec<_>, _> = sqlx::query!(
        "SELECT country, COUNT(*) as count FROM visitors WHERE country IS NOT NULL GROUP BY country"
    )
    .fetch_all(pool.get_ref())
    .await;

    let daily_stats_result: Result<Vec<_>, _> = sqlx::query!(
        r#"
        SELECT DATE(created_at) as date,
               COUNT(DISTINCT session_id) as visitors,
               COUNT(*) as page_views
        FROM visitors
        GROUP BY DATE(created_at)
        ORDER BY date DESC
        LIMIT 30
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    // Handle semua hasil query
    match (
        total_result,
        unique_result,
        browser_stats_result,
        os_stats_result,
        country_stats_result,
        daily_stats_result,
    ) {
        (Ok(total), Ok(unique), Ok(browsers), Ok(os_stats), Ok(countries), Ok(daily)) => {
            // Fix: Handle i64 untuk count
            let total_count = total.count as i64;

            let browser_stats: Vec<BrowserStats> = browsers
                .into_iter()
                .map(|b| {
                    let count = b.count as i64;
                    let percentage = if total_count > 0 {
                        (count as f64 / total_count as f64) * 100.0
                    } else {
                        0.0
                    };
                    BrowserStats {
                        browser: b.browser.to_string(),
                        count,
                        percentage,
                    }
                })
                .collect();

            let os_stats: Vec<OSStats> = os_stats
                .into_iter()
                .map(|o| {
                    let count = o.count as i64;
                    let percentage = if total_count > 0 {
                        (count as f64 / total_count as f64) * 100.0
                    } else {
                        0.0
                    };
                    OSStats {
                        os: o.os.to_string(),
                        count,
                        percentage,
                    }
                })
                .collect();

            let country_stats: Vec<CountryStats> = countries
                .into_iter()
                .map(|c| {
                    let count = c.count as i64;
                    let percentage = if total_count > 0 {
                        (count as f64 / total_count as f64) * 100.0
                    } else {
                        0.0
                    };
                    CountryStats {
                        country: c.country.unwrap_or_else(|| "Unknown".to_string()),
                        count,
                        percentage,
                    }
                })
                .collect();

            let daily_visits: Vec<DailyVisit> = daily
                .into_iter()
                .map(|d| DailyVisit {
                    date: d.date.unwrap_or_default().to_string(),
                    visitors: d.visitors as i64,
                    page_views: d.page_views as i64,
                })
                .collect();

            let stats = VisitorStats {
                total_visitors: total_count,
                unique_visitors: unique.count as i64,
                page_views: total_count, // Karena setiap insert adalah satu page view
                browsers: browser_stats,
                os_stats: os_stats,
                countries: country_stats,
                daily_visits,
            };

            HttpResponse::Ok().json(stats)
        }
        _ => HttpResponse::InternalServerError().body("Failed to fetch stats"),
    }
}

#[get("/api/visitors/recent")]
pub async fn get_recent_visitors(pool: web::Data<MySqlPool>) -> HttpResponse {
    match sqlx::query_as::<_, Visitor>(
        r#"
        SELECT
            id,
            session_id,
            ip_address,
            user_agent,
            browser,
            os,
            device_type,
            referrer,
            page_url,
            country,
            city,
            latitude,
            longitude,
            screen_resolution,
            language,
            time_on_page,
            created_at
        FROM visitors
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(visitors) => HttpResponse::Ok().json(visitors),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch recent visitors")
        }
    }
}

#[get("/api/visitors/session/{session_id}")]
pub async fn get_visitor_by_session(
    pool: web::Data<MySqlPool>,
    session_id: web::Path<String>,
) -> HttpResponse {
    match sqlx::query_as::<_, Visitor>(
        r#"
        SELECT
            id,
            session_id,
            ip_address,
            user_agent,
            browser,
            os,
            device_type,
            referrer,
            page_url,
            country,
            city,
            latitude,
            longitude,
            screen_resolution,
            language,
            time_on_page,
            created_at
        FROM visitors
        WHERE session_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(session_id.to_string())
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(visitors) => HttpResponse::Ok().json(visitors),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch visitor data")
        }
    }
}

#[get("/api/stats/summary")]
pub async fn get_stats_summary(pool: web::Data<MySqlPool>) -> HttpResponse {
    // Query untuk stats summary
    let today_result =
        sqlx::query!("SELECT COUNT(*) as count FROM visitors WHERE DATE(created_at) = CURDATE()")
            .fetch_one(pool.get_ref())
            .await;

    let month_result = sqlx::query!(
        "SELECT COUNT(*) as count FROM visitors WHERE MONTH(created_at) = MONTH(CURDATE()) AND YEAR(created_at) = YEAR(CURDATE())"
    )
    .fetch_one(pool.get_ref())
    .await;

    let year_result = sqlx::query!(
        "SELECT COUNT(*) as count FROM visitors WHERE YEAR(created_at) = YEAR(CURDATE())"
    )
    .fetch_one(pool.get_ref())
    .await;

    let total_result = sqlx::query!("SELECT COUNT(*) as count FROM visitors")
        .fetch_one(pool.get_ref())
        .await;

    // Unique visitors
    let today_unique_result = sqlx::query!(
        "SELECT COUNT(DISTINCT session_id) as count FROM visitors WHERE DATE(created_at) = CURDATE()"
    )
    .fetch_one(pool.get_ref())
    .await;

    let month_unique_result = sqlx::query!(
        "SELECT COUNT(DISTINCT session_id) as count FROM visitors WHERE MONTH(created_at) = MONTH(CURDATE()) AND YEAR(created_at) = YEAR(CURDATE())"
    )
    .fetch_one(pool.get_ref())
    .await;

    match (
        today_result,
        month_result,
        year_result,
        total_result,
        today_unique_result,
        month_unique_result,
    ) {
        (Ok(today), Ok(month), Ok(year), Ok(total), Ok(today_unique), Ok(month_unique)) => {
            #[derive(Debug, Serialize)]
            struct StatsSummary {
                today: i64,
                month: i64,
                year: i64,
                total: i64,
                today_unique: i64,
                month_unique: i64,
                online_now: i64,
            }

            let stats = StatsSummary {
                today: today.count as i64,
                month: month.count as i64,
                year: year.count as i64,
                total: total.count as i64,
                today_unique: today_unique.count as i64,
                month_unique: month_unique.count as i64,
                online_now: 0, // Bisa diisi dengan real-time tracking
            };

            HttpResponse::Ok().json(stats)
        }
        _ => HttpResponse::InternalServerError().body("Failed to fetch stats"),
    }
}

// Alternative: Single query lebih efisien
#[get("/api/stats/summary2")]
pub async fn get_stats_summary2(pool: web::Data<MySqlPool>) -> HttpResponse {
    match sqlx::query!(
        r#"
        SELECT
            (SELECT COUNT(*) FROM visitors WHERE DATE(created_at) = CURDATE()) as today,
            (SELECT COUNT(DISTINCT session_id) FROM visitors WHERE DATE(created_at) = CURDATE()) as today_unique,
            (SELECT COUNT(*) FROM visitors WHERE MONTH(created_at) = MONTH(CURDATE()) AND YEAR(created_at) = YEAR(CURDATE())) as month,
            (SELECT COUNT(DISTINCT session_id) FROM visitors WHERE MONTH(created_at) = MONTH(CURDATE()) AND YEAR(created_at) = YEAR(CURDATE())) as month_unique,
            (SELECT COUNT(*) FROM visitors WHERE YEAR(created_at) = YEAR(CURDATE())) as year,
            (SELECT COUNT(*) FROM visitors) as total
        "#
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(stats) => {
            #[derive(Debug, Serialize)]
            struct SummaryResponse {
                today: i64,
                today_unique: i64,
                month: i64,
                month_unique: i64,
                year: i64,
                total: i64,
            }

            let response = SummaryResponse {
                today: stats.today.unwrap_or_default() as i64,
                today_unique: stats.today_unique.unwrap_or_default() as i64,
                month: stats.month.unwrap_or_default() as i64,
                month_unique: stats.month_unique.unwrap_or_default() as i64,
                year: stats.year.unwrap_or_default() as i64,
                total: stats.total.unwrap_or_default() as i64,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch summary stats"
            }))
        }
    }
}
