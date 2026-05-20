// src/controllers/paskibraka_controller.rs
use crate::auth;
use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use ammonia::clean;
use chrono::Utc;
use futures::TryStreamExt;
use mime::{IMAGE_JPEG, IMAGE_PNG};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{MySqlPool, prelude::FromRow};
use std::fs;
use std::io::Write;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaskibrakaNasional {
    pub id: i32,
    pub nama_lengkap: String,
    pub jk: String,
    pub id_provinsi: i32,
    pub id_kabupaten: Option<i32>,
    pub asal_sma: Option<String>,
    pub tahun_tugas: Option<i32>,
    pub photo: Option<String>,
    // Tambahan field dari join
    pub nama_provinsi: Option<String>,
    pub nama_kabupaten: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Validate)]
pub struct PaskibrakaNasionalRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Nama lengkap harus diisi dan maksimal 100 karakter"
    ))]
    pub nama_lengkap: String,
    pub jk: String,
    pub id_provinsi: i32,
    pub id_kabupaten: Option<i32>,
    #[validate(length(max = 200, message = "Nama SMA maksimal 200 karakter"))]
    pub asal_sma: Option<String>,
    pub tahun_tugas: Option<i32>,
    pub photo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePaskibrakaRequest {
    pub nama_lengkap: Option<String>,
    pub jk: Option<String>,
    pub id_provinsi: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub asal_sma: Option<String>,
    pub tahun_tugas: Option<i32>,
    pub photo: Option<String>,
}

//pagination dan pencarian
#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    q: Option<String>,
    tahun_tugas: Option<i32>,
}

#[derive(Serialize)]
struct PaginatedResponse<T> {
    data: Vec<T>,
    current_page: u32,
    total_pages: u32,
    total_items: u64,
    per_page: u32,
    from: u64,
    to: u64,
    query: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}

// Helper function untuk validasi dan sanitasi input
fn sanitize_input(input: &str) -> String {
    clean(input).to_string()
}

#[get("/api/adminpanel/paskibraka-nasional")]
pub async fn get_pasnas(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    // Params
    let mut page = query.page.unwrap_or(1);
    if page == 0 {
        page = 1;
    }
    let mut per_page = query.per_page.unwrap_or(10);
    per_page = per_page.clamp(1, 100);

    let q = query.q.clone();
    let tahun_filter = match query.tahun_tugas {
        Some(tahun) => Some(tahun),
        None => {
            // Get latest tahun_tugas as default
            let latest_tahun: Option<i32> = sqlx::query_scalar(
                "SELECT DISTINCT tahun_tugas FROM paskibraka_nasional
                 WHERE tahun_tugas IS NOT NULL
                 ORDER BY tahun_tugas DESC LIMIT 1",
            )
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Error getting latest tahun: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
            latest_tahun
        }
    };

    // Count query
    let (total_items,): (i64,) = match (&q, tahun_filter) {
        (Some(keyword), Some(tahun)) => {
            let like = format!("%{}%", keyword);
            sqlx::query_as::<_, (i64,)>(
                "SELECT COUNT(*) FROM paskibraka_nasional
                 WHERE nama_lengkap LIKE ? AND tahun_tugas = ?",
            )
            .bind(&like)
            .bind(tahun)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Count error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
        }
        (Some(keyword), None) => {
            let like = format!("%{}%", keyword);
            sqlx::query_as::<_, (i64,)>(
                "SELECT COUNT(*) FROM paskibraka_nasional WHERE nama_lengkap LIKE ?",
            )
            .bind(&like)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Count error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
        }
        (None, Some(tahun)) => sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM paskibraka_nasional WHERE tahun_tugas = ?",
        )
        .bind(tahun)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Count error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?,
        (None, None) => sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM paskibraka_nasional")
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Count error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?,
    };

    let total_items = total_items.max(0) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + (per_page as u64) - 1) / (per_page as u64)) as u32
    };
    let current = if page > total_pages {
        total_pages
    } else {
        page
    };
    let offset = ((current - 1) as u64) * (per_page as u64);

    // Data query dengan JOIN
    let data: Vec<PaskibrakaNasional> = match (&q, tahun_filter) {
        (Some(keyword), Some(tahun)) => {
            let like = format!("%{}%", keyword);
            sqlx::query_as::<_, PaskibrakaNasional>(
                r#"SELECT
                    pn.id,
                    pn.nama_lengkap,
                    pn.jk,
                    pn.id_provinsi,
                    pn.id_kabupaten,
                    pn.asal_sma,
                    pn.tahun_tugas,
                    pn.photo,
                    p.nama_provinsi,
                    k.nama_kabupaten
                 FROM paskibraka_nasional pn
                 LEFT JOIN provinsi p ON pn.id_provinsi = p.id
                 LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
                 WHERE pn.nama_lengkap LIKE ? AND pn.tahun_tugas = ?
                 ORDER BY pn.id ASC
                 LIMIT ? OFFSET ?"#,
            )
            .bind(&like)
            .bind(tahun)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Data error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
        }
        (Some(keyword), None) => {
            let like = format!("%{}%", keyword);
            sqlx::query_as::<_, PaskibrakaNasional>(
                r#"SELECT
                    pn.id,
                    pn.nama_lengkap,
                    pn.jk,
                    pn.id_provinsi,
                    pn.id_kabupaten,
                    pn.asal_sma,
                    pn.tahun_tugas,
                    pn.photo,
                    p.nama_provinsi,
                    k.nama_kabupaten
                 FROM paskibraka_nasional pn
                 LEFT JOIN provinsi p ON pn.id_provinsi = p.id
                 LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
                 WHERE pn.nama_lengkap LIKE ?
                 ORDER BY pn.id ASC
                 LIMIT ? OFFSET ?"#,
            )
            .bind(&like)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Data error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
        }
        (None, Some(tahun)) => sqlx::query_as::<_, PaskibrakaNasional>(
            r#"SELECT
                    pn.id,
                    pn.nama_lengkap,
                    pn.jk,
                    pn.id_provinsi,
                    pn.id_kabupaten,
                    pn.asal_sma,
                    pn.tahun_tugas,
                    pn.photo,
                    p.nama_provinsi,
                    k.nama_kabupaten
                 FROM paskibraka_nasional pn
                 LEFT JOIN provinsi p ON pn.id_provinsi = p.id
                 LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
                 WHERE pn.tahun_tugas = ?
                 ORDER BY pn.id ASC
                 LIMIT ? OFFSET ?"#,
        )
        .bind(tahun)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Data error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?,
        (None, None) => sqlx::query_as::<_, PaskibrakaNasional>(
            r#"SELECT
                    pn.id,
                    pn.nama_lengkap,
                    pn.jk,
                    pn.id_provinsi,
                    pn.id_kabupaten,
                    pn.asal_sma,
                    pn.tahun_tugas,
                    pn.photo,
                    p.nama_provinsi,
                    k.nama_kabupaten
                 FROM paskibraka_nasional pn
                 LEFT JOIN provinsi p ON pn.id_provinsi = p.id
                 LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
                 ORDER BY pn.id ASC
                 LIMIT ? OFFSET ?"#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Data error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?,
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        (offset + data.len() as u64).min(total_items)
    };

    let resp = PaginatedResponse {
        data,
        current_page: current,
        total_pages,
        total_items,
        per_page,
        from,
        to,
        query: q.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(resp))
}
// CREATE Paskibraka Nasional
#[post("/api/adminpanel/paskibraka-nasional")]
pub async fn create_pasnas(
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // Auth check
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let upload_dir = "uploads/assets/file/piagam";
    fs::create_dir_all(upload_dir).map_err(|e| {
        eprintln!("Gagal buat folder upload: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal membuat direktori")
    })?;

    let mut form_data = PaskibrakaNasionalRequest {
        nama_lengkap: String::new(),
        jk: String::new(),
        id_provinsi: 0,
        id_kabupaten: None,
        asal_sma: None,
        tahun_tugas: None,
        photo: None,
    };

    // Baca seluruh field dari multipart form
    while let Ok(Some(mut field)) = payload.try_next().await {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "photo" => {
                eprintln!("📸 menerima field photo...");
                let ct = field.content_type();

                if let Some(content_type) = ct {
                    if content_type == &IMAGE_JPEG || content_type == &IMAGE_PNG {
                        let ext = if content_type == &IMAGE_PNG {
                            "png"
                        } else {
                            "jpg"
                        };
                        let filename = format!(
                            "photo_{}_{}.{}",
                            Utc::now().timestamp(),
                            Uuid::new_v4(),
                            ext
                        );

                        let upload_photo_dir = "uploads/assets/images/paskibraka";
                        fs::create_dir_all(upload_photo_dir).ok();
                        let filepath = format!("{}/{}", upload_photo_dir, filename);

                        let mut f = fs::File::create(&filepath).map_err(|e| {
                            eprintln!("❌ Gagal membuat file: {:?}", e);
                            actix_web::error::ErrorInternalServerError("Gagal membuat file foto")
                        })?;

                        let mut file_size: u64 = 0;
                        while let Ok(Some(chunk)) = field.try_next().await {
                            file_size += chunk.len() as u64;
                            if file_size > 10 * 1024 * 1024 {
                                return Ok(HttpResponse::BadRequest().json(ApiResponse {
                                    success: false,
                                    message: "Ukuran foto maksimal 10MB".to_string(),
                                    data: None::<()>,
                                }));
                            }
                            f.write_all(&chunk).map_err(|_| {
                                actix_web::error::ErrorInternalServerError("Gagal menulis foto")
                            })?;
                        }
                        eprintln!("📸 tersimpan di {:?}", filepath);
                        form_data.photo = Some(filepath);
                    } else {
                        eprintln!("⚠️ Tipe foto tidak valid: {:?}", content_type);
                        return Ok(HttpResponse::BadRequest().json(ApiResponse {
                            success: false,
                            message: "Tipe file harus JPEG atau PNG".to_string(),
                            data: None::<()>,
                        }));
                    }
                }
            }
            "nama_lengkap" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                form_data.nama_lengkap = sanitize_input(&text);
            }
            "jk" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                form_data.jk = sanitize_input(&text);
            }
            "id_provinsi" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                form_data.id_provinsi = text.parse().unwrap_or(0);
            }
            "id_kabupaten" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !text.is_empty() {
                    form_data.id_kabupaten = Some(text.parse().unwrap_or(0));
                }
            }
            "asal_sma" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                form_data.asal_sma = Some(sanitize_input(&text));
            }
            "tahun_tugas" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !text.is_empty() {
                    form_data.tahun_tugas = Some(text.parse().unwrap_or(0));
                }
            }
            _ => {}
        }
    }

    // Validasi
    if form_data.nama_lengkap.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Nama lengkap harus diisi".to_string(),
            data: None::<()>,
        }));
    }

    // Insert ke database
    let result = sqlx::query(
        "INSERT INTO paskibraka_nasional (nama_lengkap, jk, id_provinsi, id_kabupaten, asal_sma, tahun_tugas, photo)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&form_data.nama_lengkap)
    .bind(form_data.jk)
    .bind(form_data.id_provinsi)
    .bind(form_data.id_kabupaten)
    .bind(&form_data.asal_sma)
    .bind(form_data.tahun_tugas)
    .bind(&form_data.photo)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyimpan data")
    })?;

    Ok(HttpResponse::Created().json(ApiResponse {
        success: true,
        message: "Data Paskibraka Nasional berhasil ditambahkan".to_string(),
        data: Some(json!({ "id": result.last_insert_id() })),
    }))
}

// GET by ID
#[get("/api/adminpanel/paskibraka-nasional/{id}")]
pub async fn get_pasnas_by_id(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    // Auth check
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let data = sqlx::query_as::<_, PaskibrakaNasional>(
        r#"SELECT
            pn.id,
            pn.nama_lengkap,
            pn.jk,
            pn.id_provinsi,
            pn.id_kabupaten,
            pn.asal_sma,
            pn.tahun_tugas,
            pn.photo,
            p.nama_provinsi,
            k.nama_kabupaten
         FROM paskibraka_nasional pn
         LEFT JOIN provinsi p ON pn.id_provinsi = p.id
         LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
         WHERE pn.id = ?"#,
    )
    .bind(id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal mengambil data")
    })?;

    match data {
        Some(data) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: "Data ditemukan".to_string(),
            data: Some(data),
        })),
        None => Ok(HttpResponse::NotFound().json(ApiResponse {
            success: false,
            message: "Data tidak ditemukan".to_string(),
            data: None::<PaskibrakaNasional>,
        })),
    }
}

#[put("/api/adminpanel/paskibraka-nasional/{id}")]
pub async fn update_pasnas(
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
    req: HttpRequest,
    id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    // Auth check
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let item_id = id.into_inner();

    // Check if exists
    let exists = sqlx::query("SELECT id FROM paskibraka_nasional WHERE id = ?")
        .bind(item_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
        .is_some();

    if !exists {
        return Ok(HttpResponse::NotFound().json(ApiResponse {
            success: false,
            message: "Data tidak ditemukan".to_string(),
            data: None::<()>,
        }));
    }

    let mut update_data = UpdatePaskibrakaRequest {
        nama_lengkap: None,
        jk: None,
        id_provinsi: None,
        id_kabupaten: None,
        asal_sma: None,
        tahun_tugas: None,
        photo: None,
    };

    // Parse multipart form
    while let Ok(Some(mut field)) = payload.try_next().await {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "photo" => {
                let ct = field.content_type();
                if let Some(content_type) = ct {
                    if content_type == &IMAGE_JPEG || content_type == &IMAGE_PNG {
                        let ext = if content_type == &IMAGE_PNG {
                            "png"
                        } else {
                            "jpg"
                        };
                        let filename = format!(
                            "photo_{}_{}.{}",
                            Utc::now().timestamp(),
                            Uuid::new_v4(),
                            ext
                        );

                        let upload_photo_dir = "uploads/assets/images/paskibraka";
                        fs::create_dir_all(upload_photo_dir).ok();
                        let filepath = format!("{}/{}", upload_photo_dir, filename);

                        // Hapus photo lama jika ada
                        let old_photo: Option<String> = sqlx::query_scalar(
                            "SELECT photo FROM paskibraka_nasional WHERE id = ?",
                        )
                        .bind(item_id)
                        .fetch_optional(pool.get_ref())
                        .await
                        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

                        if let Some(old_path) = old_photo {
                            if fs::metadata(&old_path).is_ok() {
                                let _ = fs::remove_file(&old_path);
                            }
                        }

                        let mut f = fs::File::create(&filepath).map_err(|_e| {
                            actix_web::error::ErrorInternalServerError("Gagal membuat file")
                        })?;

                        let mut file_size: u64 = 0;
                        while let Ok(Some(chunk)) = field.try_next().await {
                            file_size += chunk.len() as u64;
                            if file_size > 10 * 1024 * 1024 {
                                return Ok(HttpResponse::BadRequest().json(ApiResponse {
                                    success: false,
                                    message: "Ukuran foto maksimal 10MB".to_string(),
                                    data: None::<()>,
                                }));
                            }
                            f.write_all(&chunk).map_err(|e| {
                                eprintln!("Gagal write file: {:?}", e);
                                actix_web::error::ErrorInternalServerError("Gagal menulis foto")
                            })?;
                        }
                        update_data.photo = Some(filepath);
                    } else {
                        return Ok(HttpResponse::BadRequest().json(ApiResponse {
                            success: false,
                            message: "Format foto harus JPEG atau PNG".to_string(),
                            data: None::<()>,
                        }));
                    }
                }
            }
            "nama_lengkap" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !text.is_empty() {
                    update_data.nama_lengkap = Some(sanitize_input(&text));
                }
            }
            "jk" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !text.is_empty() {
                    update_data.jk = Some(sanitize_input(&text));
                }
            }
            "id_provinsi" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !text.is_empty() {
                    update_data.id_provinsi = Some(text.parse().unwrap_or(0));
                }
            }
            "id_kabupaten" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !text.is_empty() {
                    update_data.id_kabupaten = Some(text.parse().unwrap_or(0));
                }
            }
            "asal_sma" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !text.is_empty() {
                    update_data.asal_sma = Some(sanitize_input(&text));
                }
            }
            "tahun_tugas" => {
                let mut text = String::new();
                while let Ok(Some(chunk)) = field.try_next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                }
                if !text.is_empty() {
                    update_data.tahun_tugas = Some(text.parse().unwrap_or(0));
                }
            }
            _ => {}
        }
    }

    // Check if any field to update
    if update_data.nama_lengkap.is_none()
        && update_data.jk.is_none()
        && update_data.id_provinsi.is_none()
        && update_data.id_kabupaten.is_none()
        && update_data.asal_sma.is_none()
        && update_data.tahun_tugas.is_none()
        && update_data.photo.is_none()
    {
        return Ok(HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            message: "Tidak ada data yang diupdate".to_string(),
            data: None::<()>,
        }));
    }

    // Build dynamic update query - Cara manual yang lebih aman
    let mut set_clauses = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(nama) = &update_data.nama_lengkap {
        set_clauses.push("nama_lengkap = ?".to_string());
        params.push(nama.clone());
    }
    if let Some(jk) = &update_data.jk {
        set_clauses.push("jk = ?".to_string());
        params.push(jk.clone());
    }
    if let Some(provinsi) = &update_data.id_provinsi {
        set_clauses.push("id_provinsi = ?".to_string());
        params.push(provinsi.to_string());
    }
    if let Some(kabupaten) = &update_data.id_kabupaten {
        set_clauses.push("id_kabupaten = ?".to_string());
        params.push(kabupaten.to_string());
    }
    if let Some(sma) = &update_data.asal_sma {
        set_clauses.push("asal_sma = ?".to_string());
        params.push(sma.clone());
    }
    if let Some(tahun) = &update_data.tahun_tugas {
        set_clauses.push("tahun_tugas = ?".to_string());
        params.push(tahun.to_string());
    }
    if let Some(photo) = &update_data.photo {
        set_clauses.push("photo = ?".to_string());
        params.push(photo.clone());
    }

    let set_clause = set_clauses.join(", ");
    let query_str = format!("UPDATE paskibraka_nasional SET {} WHERE id = ?", set_clause);

    // Execute dengan sqlx query builder
    let mut query = sqlx::query(&query_str);

    // Bind semua parameter
    for param in params {
        query = query.bind(param);
    }
    // Bind ID
    query = query.bind(item_id);

    // Execute query
    query.execute(pool.get_ref()).await.map_err(|e| {
        eprintln!("Update error: {:?}", e);
        actix_web::error::ErrorInternalServerError(format!("Gagal mengupdate data: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Data berhasil diupdate".to_string(),
        data: Some(json!({ "id": item_id })),
    }))
}

#[derive(sqlx::FromRow)]
struct TempRecord {
    photo: Option<String>,
}

#[delete("/api/adminpanel/paskibraka-nasional/{id}")]
pub async fn delete_pasnas(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    // Auth check
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let item_id = id.into_inner();

    let record =
        sqlx::query_as::<_, TempRecord>("SELECT photo FROM paskibraka_nasional WHERE id = ?")
            .bind(item_id)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Error fetching record: {:?}", e);
                actix_web::error::ErrorInternalServerError(e.to_string())
            })?;

    // Delete record
    let rows_affected = sqlx::query("DELETE FROM paskibraka_nasional WHERE id = ?")
        .bind(item_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Error deleting: {:?}", e);
            actix_web::error::ErrorInternalServerError(format!("Delete error: {}", e))
        })?
        .rows_affected();

    if rows_affected == 0 {
        return Ok(HttpResponse::NotFound().json(ApiResponse {
            success: false,
            message: "Data tidak ditemukan".to_string(),
            data: None::<()>,
        }));
    }
    if let Some(record) = record {
        if let Some(photo_path) = record.photo {
            if !photo_path.is_empty() && fs::metadata(&photo_path).is_ok() {
                let _ = fs::remove_file(&photo_path);
            }
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Data berhasil dihapus".to_string(),
        data: Some(json!({ "id": item_id })),
    }))
}

#[get("/api/adminpanel/tahun-list/paskibraka-nasional")]
pub async fn get_tahun_list(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // Auth check
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    // Get unique tahun_tugas, sort descending
    let tahun_list: Vec<i32> = sqlx::query_scalar(
        "SELECT DISTINCT tahun_tugas
         FROM paskibraka_nasional
         ORDER BY tahun_tugas DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Error getting tahun list: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal mengambil daftar tahun")
    })?;

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Daftar tahun berhasil diambil".to_string(),
        data: Some(tahun_list),
    }))
}

//publik

#[get("/api/public/paskibraka-nasional")]
pub async fn get_pasnas_public(
    pool: web::Data<MySqlPool>,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, Error> {
    let mut page = query.page.unwrap_or(1);
    if page == 0 {
        page = 1;
    }
    let mut per_page = query.per_page.unwrap_or(10);
    per_page = per_page.clamp(1, 100);

    let q = query.q.clone();
    let tahun_filter = match query.tahun_tugas {
        Some(tahun) => Some(tahun),
        None => {
            // Get latest tahun_tugas as default
            let latest_tahun: Option<i32> = sqlx::query_scalar(
                "SELECT DISTINCT tahun_tugas FROM paskibraka_nasional
                 WHERE tahun_tugas IS NOT NULL
                 ORDER BY tahun_tugas DESC LIMIT 1",
            )
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Error getting latest tahun: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;
            latest_tahun
        }
    };

    // Count query
    let (total_items,): (i64,) = match (&q, tahun_filter) {
        (Some(keyword), Some(tahun)) => {
            let like = format!("%{}%", keyword);
            sqlx::query_as::<_, (i64,)>(
                "SELECT COUNT(*) FROM paskibraka_nasional
                 WHERE nama_lengkap LIKE ? AND tahun_tugas = ?",
            )
            .bind(&like)
            .bind(tahun)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Count error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
        }
        (Some(keyword), None) => {
            let like = format!("%{}%", keyword);
            sqlx::query_as::<_, (i64,)>(
                "SELECT COUNT(*) FROM paskibraka_nasional WHERE nama_lengkap LIKE ?",
            )
            .bind(&like)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Count error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
        }
        (None, Some(tahun)) => sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM paskibraka_nasional WHERE tahun_tugas = ?",
        )
        .bind(tahun)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Count error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?,
        (None, None) => sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM paskibraka_nasional")
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Count error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?,
    };

    let total_items = total_items.max(0) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + (per_page as u64) - 1) / (per_page as u64)) as u32
    };
    let current = if page > total_pages {
        total_pages
    } else {
        page
    };
    let offset = ((current - 1) as u64) * (per_page as u64);

    // Data query dengan JOIN
    let data: Vec<PaskibrakaNasional> = match (&q, tahun_filter) {
        (Some(keyword), Some(tahun)) => {
            let like = format!("%{}%", keyword);
            sqlx::query_as::<_, PaskibrakaNasional>(
                r#"SELECT
                    pn.id,
                    pn.nama_lengkap,
                    pn.jk,
                    pn.id_provinsi,
                    pn.id_kabupaten,
                    pn.asal_sma,
                    pn.tahun_tugas,
                    pn.photo,
                    p.nama_provinsi,
                    k.nama_kabupaten
                 FROM paskibraka_nasional pn
                 LEFT JOIN provinsi p ON pn.id_provinsi = p.id
                 LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
                 WHERE pn.nama_lengkap LIKE ? AND pn.tahun_tugas = ?
                 ORDER BY pn.id ASC
                 LIMIT ? OFFSET ?"#,
            )
            .bind(&like)
            .bind(tahun)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Data error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
        }
        (Some(keyword), None) => {
            let like = format!("%{}%", keyword);
            sqlx::query_as::<_, PaskibrakaNasional>(
                r#"SELECT
                    pn.id,
                    pn.nama_lengkap,
                    pn.jk,
                    pn.id_provinsi,
                    pn.id_kabupaten,
                    pn.asal_sma,
                    pn.tahun_tugas,
                    pn.photo,
                    p.nama_provinsi,
                    k.nama_kabupaten
                 FROM paskibraka_nasional pn
                 LEFT JOIN provinsi p ON pn.id_provinsi = p.id
                 LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
                 WHERE pn.nama_lengkap LIKE ?
                 ORDER BY pn.id ASC
                 LIMIT ? OFFSET ?"#,
            )
            .bind(&like)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| {
                eprintln!("Data error: {:?}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?
        }
        (None, Some(tahun)) => sqlx::query_as::<_, PaskibrakaNasional>(
            r#"SELECT
                    pn.id,
                    pn.nama_lengkap,
                    pn.jk,
                    pn.id_provinsi,
                    pn.id_kabupaten,
                    pn.asal_sma,
                    pn.tahun_tugas,
                    pn.photo,
                    p.nama_provinsi,
                    k.nama_kabupaten
                 FROM paskibraka_nasional pn
                 LEFT JOIN provinsi p ON pn.id_provinsi = p.id
                 LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
                 WHERE pn.tahun_tugas = ?
                 ORDER BY pn.id ASC
                 LIMIT ? OFFSET ?"#,
        )
        .bind(tahun)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Data error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?,
        (None, None) => sqlx::query_as::<_, PaskibrakaNasional>(
            r#"SELECT
                    pn.id,
                    pn.nama_lengkap,
                    pn.jk,
                    pn.id_provinsi,
                    pn.id_kabupaten,
                    pn.asal_sma,
                    pn.tahun_tugas,
                    pn.photo,
                    p.nama_provinsi,
                    k.nama_kabupaten
                 FROM paskibraka_nasional pn
                 LEFT JOIN provinsi p ON pn.id_provinsi = p.id
                 LEFT JOIN kabupaten k ON pn.id_kabupaten = k.id
                 ORDER BY pn.id ASC
                 LIMIT ? OFFSET ?"#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            eprintln!("Data error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?,
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        (offset + data.len() as u64).min(total_items)
    };

    let resp = PaginatedResponse {
        data,
        current_page: current,
        total_pages,
        total_items,
        per_page,
        from,
        to,
        query: q.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[get("/api/public/tahun-list/paskibraka-nasional")]
pub async fn get_tahun_list_public(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let tahun_list: Vec<i32> = sqlx::query_scalar(
        "SELECT DISTINCT tahun_tugas
         FROM paskibraka_nasional
         ORDER BY tahun_tugas DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Error getting tahun list: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal mengambil daftar tahun")
    })?;

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Daftar tahun berhasil diambil".to_string(),
        data: Some(tahun_list),
    }))
}
