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
use sqlx::{MySql, QueryBuilder};
use sqlx::{MySqlPool, Row, prelude::FromRow};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UpdatePaskibrakaRequest {
    pub id: i32,
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

//helper untuk upload photo
async fn read_text_field(mut field: actix_multipart::Field) -> Result<String, actix_web::Error> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field
        .try_next()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        bytes.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8_lossy(&bytes).trim().to_string())
}

async fn save_photo_field(
    mut field: actix_multipart::Field,
    dir: &str,
) -> Result<String, actix_web::Error> {
    let upload_dir = Path::new(dir);
    if !upload_dir.exists() {
        fs::create_dir_all(upload_dir).map_err(actix_web::error::ErrorInternalServerError)?;
    }
    // deteksi ekstensi sederhana dari content-type
    let ext = field
        .content_type()
        .map(|ct| match (ct.type_().as_str(), ct.subtype().as_str()) {
            ("image", "png") => "png",
            ("image", "jpeg") | ("image", "jpg") => "jpg",
            ("image", "webp") => "webp",
            _ => "png",
        })
        .unwrap_or("png");

    let filename = format!(
        "paskibraka_nasional_{}.{}",
        chrono::Utc::now().timestamp_millis(),
        ext
    );
    let filepath = upload_dir.join(&filename);

    let mut f = fs::File::create(&filepath).map_err(actix_web::error::ErrorInternalServerError)?;
    while let Some(chunk) = field
        .try_next()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        f.write_all(&chunk)
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    Ok(format!("{}/{}", dir.trim_start_matches("./"), filename))
}

fn is_safe_upload_path(rel: &str) -> bool {
    // Hindari traversal: hanya izinkan path yang diawali "uploads/"
    rel.starts_with("uploads/")
}

fn to_fs_path(rel: &str) -> PathBuf {
    // Simpel: gabungkan dengan root project. Sesuaikan kalau foldernya beda.
    Path::new("./").join(rel)
}

pub fn remove_file_if_exists(rel: &str) {
    if !is_safe_upload_path(rel) {
        return;
    }
    let p = to_fs_path(rel);
    if p.exists() {
        let _ = fs::remove_file(&p);
    }
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
    let tahun_filter = query.tahun_tugas;
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
    req: HttpRequest,
    path: web::Path<i32>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&_claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator",
        ));
    }

    let id = path.into_inner();

    let mut nama_lengkap: Option<String> = None;
    let mut jk: Option<String> = None;
    let mut id_provinsi: Option<i32> = None;
    let mut id_kabupaten: Option<i32> = None;
    let mut asal_sma: Option<String> = None;
    let mut tahun_tugas: Option<i32> = None;
    let mut photo_new_path: Option<String> = None;
    let mut photo_remove = false;

    while let Some(field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        let name = field
            .content_disposition()
            .and_then(|cd| cd.get_name())
            .unwrap_or("");

        match name {
            "nama_lengkap" => {
                let v = read_text_field(field).await?;
                nama_lengkap = Some(sanitize_input(&v));
            }
            "jk" => {
                let v = read_text_field(field).await?;
                jk = Some(sanitize_input(&v));
            }
            "id_provinsi" => {
                let v = read_text_field(field).await?;
                if !v.is_empty() {
                    if let Ok(val) = v.parse::<i32>() {
                        id_provinsi = Some(val);
                    }
                }
            }
            "id_kabupaten" => {
                let v = read_text_field(field).await?;
                if !v.is_empty() {
                    if let Ok(val) = v.parse::<i32>() {
                        id_kabupaten = Some(val);
                    }
                }
            }
            "asal_sma" => {
                let v = read_text_field(field).await?;
                asal_sma = Some(sanitize_input(&v));
            }
            "tahun_tugas" => {
                let v = read_text_field(field).await?;
                if !v.is_empty() {
                    if let Ok(val) = v.parse::<i32>() {
                        tahun_tugas = Some(val);
                    }
                }
            }
            "photo" => {
                photo_new_path =
                    Some(save_photo_field(field, "./uploads/assets/paskibraka-nasional").await?);
            }
            "photo_remove" => {
                let v = read_text_field(field).await?;
                photo_remove = v == "1" || v.eq_ignore_ascii_case("true");
            }
            _ => {}
        }
    }

    // Ambil path foto lama sebelum proses update
    let (old_photo_opt,): (Option<String>,) =
        sqlx::query_as("SELECT photo FROM paskibraka_nasional WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut qb: QueryBuilder<MySql> = QueryBuilder::new("UPDATE paskibraka_nasional SET ");
    let mut first = true;
    let mut has_any = false;

    if let Some(v) = nama_lengkap {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("nama_lengkap = ").push_bind(v);
    }

    if let Some(v) = jk {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("jk = ").push_bind(v);
    }

    if let Some(v) = id_provinsi {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("id_provinsi = ").push_bind(v);
    }

    if let Some(v) = id_kabupaten {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("id_kabupaten = ").push_bind(v);
    }

    if let Some(v) = asal_sma {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("asal_sma = ").push_bind(v);
    }

    if let Some(v) = tahun_tugas {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("tahun_tugas = ").push_bind(v);
    }

    let mut remove_old = false;

    if photo_remove {
        if !first {
            qb.push(", ");
        }
        has_any = true;
        qb.push("photo = NULL");
        if old_photo_opt.is_some() {
            remove_old = true;
        }
    } else if let Some(ref p) = photo_new_path {
        if !first {
            qb.push(", ");
        }
        has_any = true;
        qb.push("photo = ").push_bind(p);
        if let Some(ref oldp) = old_photo_opt {
            if oldp != p {
                remove_old = true;
            }
        }
    }

    if !has_any {
        return Ok(HttpResponse::BadRequest().body("Tidak ada field untuk diupdate"));
    }

    qb.push(" WHERE id = ").push_bind(id);

    // Eksekusi update
    qb.build()
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Hapus file lama
    if remove_old {
        if let Some(oldp) = old_photo_opt {
            remove_file_if_exists(&oldp);
        }
    }

    // Ambil data terbaru
    let updated = sqlx::query_as::<_, UpdatePaskibrakaRequest>(
        "SELECT id, nama_lengkap, jk, id_provinsi, id_kabupaten, asal_sma, tahun_tugas, photo FROM paskibraka_nasional WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(updated))
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

// ==========================================
// PASKIBRAKA MEMBER & ADMIN MANAGEMENT API
// ==========================================

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PhysicalRecord {
    pub id: String,
    pub user_id: String,
    pub id_capaska: Option<String>,
    pub bulan: String,
    pub tb: f64,
    pub bb: f64,
    pub catatan: Option<String>,
    pub tanggal: Option<chrono::NaiveDate>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct PhysicalRecordRequest {
    pub bulan: String,
    pub tb: f64,
    pub bb: f64,
    pub catatan: Option<String>,
    pub tanggal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaskibrakaTugas {
    pub id: String,
    pub judul: String,
    pub deskripsi: String,
    pub file_lampiran: Option<String>,
    pub deadline: chrono::NaiveDateTime,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct PaskibrakaTugasRequest {
    pub id: Option<String>,
    pub judul: String,
    pub deskripsi: String,
    pub deadline: String,
    pub file_lampiran: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaskibrakaInformasi {
    pub id: String,
    pub judul: String,
    pub konten: String,
    pub file_lampiran: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct PaskibrakaInformasiRequest {
    pub judul: String,
    pub konten: String,
    pub file_lampiran: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaskibrakaPengumpulan {
    pub id: String,
    pub id_tugas: String,
    pub user_id: String,
    pub id_capaska: Option<String>,
    pub nama_siswa: String,
    pub file_url: String,
    pub file_type: String,
    pub catatan_siswa: Option<String>,
    pub submitted_at: Option<chrono::NaiveDateTime>,
    pub judul_tugas: Option<String>,
    pub deadline_tugas: Option<chrono::NaiveDateTime>,
    pub nilai: Option<String>,
    pub catatan_admin: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct PaskibrakaPenilaian {
    pub id: String,
    pub id_pengumpulan: String,
    pub id_tugas: String,
    pub user_id: String,
    pub nilai: String,
    pub catatan_admin: Option<String>,
    pub graded_by: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct PenilaianRequest {
    pub id_pengumpulan: String,
    pub id_tugas: String,
    pub user_id: String,
    pub nilai: String,
    pub catatan_admin: Option<String>,
}

pub async fn init_paskibraka_tables(pool: &MySqlPool) {
    let queries = vec![
        r#"CREATE TABLE IF NOT EXISTS paskibraka_physical_records (
            id VARCHAR(36) PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            id_capaska VARCHAR(255) NULL,
            bulan VARCHAR(50) NOT NULL,
            tb DOUBLE NOT NULL,
            bb DOUBLE NOT NULL,
            catatan TEXT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,
        r#"CREATE TABLE IF NOT EXISTS paskibraka_tugas (
            id VARCHAR(36) PRIMARY KEY,
            judul VARCHAR(255) NOT NULL,
            deskripsi TEXT NOT NULL,
            file_lampiran VARCHAR(500) NULL,
            deadline DATETIME NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,
        r#"CREATE TABLE IF NOT EXISTS paskibraka_informasi (
            id VARCHAR(36) PRIMARY KEY,
            judul VARCHAR(255) NOT NULL,
            konten TEXT NOT NULL,
            file_lampiran VARCHAR(500) NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,
        r#"CREATE TABLE IF NOT EXISTS paskibraka_pengumpulan (
            id VARCHAR(36) PRIMARY KEY,
            id_tugas VARCHAR(36) NOT NULL,
            user_id VARCHAR(255) NOT NULL,
            id_capaska VARCHAR(255) NULL,
            nama_siswa VARCHAR(255) NOT NULL,
            file_url VARCHAR(500) NOT NULL,
            file_type VARCHAR(50) NOT NULL,
            catatan_siswa TEXT NULL,
            submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,
        r#"CREATE TABLE IF NOT EXISTS paskibraka_penilaian (
            id VARCHAR(36) PRIMARY KEY,
            id_pengumpulan VARCHAR(36) NOT NULL,
            id_tugas VARCHAR(36) NOT NULL,
            user_id VARCHAR(255) NOT NULL,
            nilai VARCHAR(50) NOT NULL,
            catatan_admin TEXT NULL,
            graded_by VARCHAR(255) NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"#,
    ];

    for q in queries {
        let _ = sqlx::query(q).execute(pool).await;
    }

    let alter_queries = vec![
        "ALTER TABLE users ADD COLUMN nama_sekolah VARCHAR(255) NULL",
        "ALTER TABLE users ADD COLUMN guru_pembimbing VARCHAR(255) NULL",
        "ALTER TABLE users ADD COLUMN no_hp_guru_pembimbing VARCHAR(50) NULL",
        "ALTER TABLE data_capaska ADD COLUMN guru_pembimbing VARCHAR(255) NULL",
        "ALTER TABLE data_capaska ADD COLUMN no_hp_guru_pembimbing VARCHAR(50) NULL",
        "ALTER TABLE paskibraka_physical_records ADD COLUMN tanggal DATE NULL",
    ];
    for q in alter_queries {
        let _ = sqlx::query(q).execute(pool).await;
    }
}

// 1. Sync Users from data_capaska
#[post("/api/paskibraka/sync-users")]
pub async fn sync_capaska_users(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if claims.role != "Superadmin"
        && claims.role != "Administrator"
        && claims.role != "Admin Kesbangpol"
    {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak: khusus admin",
        ));
    }

    let rows = sqlx::query(
        "SELECT id, nama_lengkap, email FROM data_capaska WHERE email IS NOT NULL AND email != ''",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Error fetching data_capaska: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal membaca data_capaska")
    })?;

    let default_pass_hash = bcrypt::hash("Paskibraka123!", bcrypt::DEFAULT_COST)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Gagal hash password"))?;

    let mut created_count = 0;
    let total_records = rows.len();

    for row in rows {
        let capaska_email: Option<String> = row.get("email");
        let email_str = capaska_email.unwrap_or_default();
        if email_str.trim().is_empty() {
            continue;
        }

        let existing = sqlx::query("SELECT id FROM users WHERE email = ?")
            .bind(&email_str)
            .fetch_optional(pool.get_ref())
            .await
            .unwrap_or(None);

        if existing.is_none() {
            let new_id = Uuid::new_v4().to_string();
            let capaska_id_raw: i64 = row.get::<i64, _>("id");
            let capaska_id = capaska_id_raw.to_string();
            let nama_lengkap: Option<String> = row.get("nama_lengkap");
            let name = nama_lengkap.unwrap_or_else(|| "Anggota Paskibraka".to_string());

            let res = sqlx::query(
                "INSERT INTO users (id, name, email, role, password, id_pdp, created_at) VALUES (?, ?, ?, 'Paskibraka', ?, ?, NOW())"
            )
            .bind(&new_id)
            .bind(&name)
            .bind(&email_str)
            .bind(&default_pass_hash)
            .bind(&capaska_id)
            .execute(pool.get_ref())
            .await;

            if res.is_ok() {
                created_count += 1;
            }
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": format!("Berhasil menyinkronkan data user: {} akun baru dibuat dari {} data capaska.", created_count, total_records),
        "created_count": created_count,
        "total_records": total_records
    })))
}

// 2. Profile Paskibraka & Monthly BB/TB Records
#[get("/api/paskibraka/profile")]
pub async fn get_paskibraka_profile(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let user_row = sqlx::query(
        "SELECT id, name, email, role, avatar, phone, address, nama_sekolah, guru_pembimbing, no_hp_guru_pembimbing, id_pdp FROM users WHERE id = ?",
    )
    .bind(&claims.user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorNotFound("User tidak ditemukan"))?;

    let u_id: String = user_row.get("id");
    let u_name: String = user_row.get("name");
    let u_email: String = user_row.get("email");
    let u_role: String = user_row.get("role");
    let u_avatar: Option<String> = user_row.get("avatar");
    let u_phone: Option<String> = user_row.get("phone");
    let u_address: Option<String> = user_row.get("address");
    let u_nama_sekolah: Option<String> = user_row.get("nama_sekolah");
    let u_guru_pembimbing: Option<String> = user_row.get("guru_pembimbing");
    let u_no_hp_guru_pembimbing: Option<String> = user_row.get("no_hp_guru_pembimbing");
    let u_id_pdp: Option<String> = user_row.get("id_pdp");

    let mut capaska_info = None;
    if let Some(ref id_pdp) = u_id_pdp {
        let capaska = sqlx::query("SELECT id, no_peserta, nama_lengkap, jk, photo, provinsi, kabupaten_kota, status, asal_sekolah, guru_pembimbing, no_hp_guru_pembimbing FROM data_capaska WHERE id = ?")
            .bind(id_pdp)
            .fetch_optional(pool.get_ref())
            .await
            .unwrap_or(None);

        if let Some(c) = capaska {
            let c_id: i64 = c.get("id");
            let no_peserta: Option<String> = c.get("no_peserta");
            let nama_lengkap: Option<String> = c.get("nama_lengkap");
            let jk: Option<String> = c.get("jk");
            let photo: Option<String> = c.get("photo");
            let provinsi: Option<String> = c.get("provinsi");
            let kabupaten_kota: Option<String> = c.get("kabupaten_kota");
            let status: Option<String> = c.get("status");
            let asal_sekolah: Option<String> = c.get("asal_sekolah");
            let guru_pembimbing: Option<String> = c.get("guru_pembimbing");
            let no_hp_guru_pembimbing: Option<String> = c.get("no_hp_guru_pembimbing");

            capaska_info = Some(json!({
                "id": c_id,
                "no_peserta": no_peserta,
                "nama_lengkap": nama_lengkap,
                "jk": jk,
                "photo": photo,
                "provinsi": provinsi,
                "kabupaten_kota": kabupaten_kota,
                "status": status,
                "asal_sekolah": asal_sekolah,
                "guru_pembimbing": guru_pembimbing,
                "no_hp_guru_pembimbing": no_hp_guru_pembimbing
            }));
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "user": {
            "id": u_id,
            "name": u_name,
            "email": u_email,
            "role": u_role,
            "avatar": u_avatar,
            "phone": u_phone,
            "address": u_address,
            "nama_sekolah": u_nama_sekolah,
            "guru_pembimbing": u_guru_pembimbing,
            "no_hp_guru_pembimbing": u_no_hp_guru_pembimbing,
            "id_pdp": u_id_pdp
        },
        "capaska_details": capaska_info
    })))
}

#[get("/api/paskibraka/physical-records")]
pub async fn get_physical_records(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let records = sqlx::query_as::<_, PhysicalRecord>(
        "SELECT id, user_id, id_capaska, bulan, tb, bb, catatan, tanggal, created_at FROM paskibraka_physical_records WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(&claims.user_id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": records
    })))
}

#[post("/api/paskibraka/physical-records")]
pub async fn add_physical_record(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    body: web::Json<PhysicalRecordRequest>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let record_id = Uuid::new_v4().to_string();
    let parsed_tanggal = body.tanggal.as_deref().and_then(|t| chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d").ok());

    sqlx::query(
        "INSERT INTO paskibraka_physical_records (id, user_id, id_capaska, bulan, tb, bb, catatan, tanggal, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW())"
    )
    .bind(&record_id)
    .bind(&claims.user_id)
    .bind(&claims.id_pdp)
    .bind(&body.bulan)
    .bind(body.tb)
    .bind(body.bb)
    .bind(&body.catatan)
    .bind(parsed_tanggal)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Error saving physical record: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyimpan rekam medis/fisik")
    })?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Data BB & TB bulanan berhasil diperbarui."
    })))
}

// 3. Tugas Bulanan
#[get("/api/paskibraka/tugas")]
pub async fn get_paskibraka_tugas(
    _req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;

    let tasks = sqlx::query_as::<_, PaskibrakaTugas>(
        "SELECT id, judul, deskripsi, file_lampiran, deadline, created_at, updated_at FROM paskibraka_tugas ORDER BY deadline ASC"
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": tasks
    })))
}

#[post("/api/paskibraka/tugas")]
pub async fn create_or_update_tugas(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    body: web::Json<PaskibrakaTugasRequest>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if claims.role != "Superadmin"
        && claims.role != "Administrator"
        && claims.role != "Admin Kesbangpol"
    {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak: khusus admin",
        ));
    }

    let parsed_deadline =
        chrono::NaiveDateTime::parse_from_str(&body.deadline, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(
                    &format!("{} 23:59:59", body.deadline),
                    "%Y-%m-%d %H:%M:%S",
                )
            })
            .map_err(|_| {
                actix_web::error::ErrorBadRequest(
                    "Format deadline tidak valid (Gunakan format YYYY-MM-DD HH:MM:SS)",
                )
            })?;

    if let Some(ref task_id) = body.id {
        sqlx::query(
            "UPDATE paskibraka_tugas SET judul = ?, deskripsi = ?, file_lampiran = ?, deadline = ?, updated_at = NOW() WHERE id = ?"
        )
        .bind(&body.judul)
        .bind(&body.deskripsi)
        .bind(&body.file_lampiran)
        .bind(parsed_deadline)
        .bind(task_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Error updating task: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal memperbarui tugas")
        })?;
    } else {
        let task_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO paskibraka_tugas (id, judul, deskripsi, file_lampiran, deadline, created_at) VALUES (?, ?, ?, ?, ?, NOW())"
        )
        .bind(&task_id)
        .bind(&body.judul)
        .bind(&body.deskripsi)
        .bind(&body.file_lampiran)
        .bind(parsed_deadline)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Error creating task: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menambahkan tugas")
        })?;
    }

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Tugas bulanan berhasil disimpan."
    })))
}

#[delete("/api/paskibraka/tugas/{id}")]
pub async fn delete_tugas(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if claims.role != "Superadmin"
        && claims.role != "Administrator"
        && claims.role != "Admin Kesbangpol"
    {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak: khusus admin",
        ));
    }

    let task_id = path.into_inner();
    sqlx::query("DELETE FROM paskibraka_tugas WHERE id = ?")
        .bind(&task_id)
        .execute(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Gagal menghapus tugas"))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Tugas berhasil dihapus."
    })))
}

// 4. Informasi
#[get("/api/paskibraka/informasi")]
pub async fn get_paskibraka_informasi(
    _req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;

    let items = sqlx::query_as::<_, PaskibrakaInformasi>(
        "SELECT id, judul, konten, file_lampiran, created_at FROM paskibraka_informasi ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": items
    })))
}

#[post("/api/paskibraka/informasi")]
pub async fn create_informasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    body: web::Json<PaskibrakaInformasiRequest>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if claims.role != "Superadmin"
        && claims.role != "Administrator"
        && claims.role != "Admin Kesbangpol"
    {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak: khusus admin",
        ));
    }

    let info_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO paskibraka_informasi (id, judul, konten, file_lampiran, created_at) VALUES (?, ?, ?, ?, NOW())"
    )
    .bind(&info_id)
    .bind(&body.judul)
    .bind(&body.konten)
    .bind(&body.file_lampiran)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Error creating info: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menambah informasi")
    })?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Informasi berhasil ditambahkan."
    })))
}

#[delete("/api/paskibraka/informasi/{id}")]
pub async fn delete_informasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if claims.role != "Superadmin"
        && claims.role != "Administrator"
        && claims.role != "Admin Kesbangpol"
    {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak: khusus admin",
        ));
    }

    let info_id = path.into_inner();
    sqlx::query("DELETE FROM paskibraka_informasi WHERE id = ?")
        .bind(&info_id)
        .execute(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Gagal menghapus informasi"))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Informasi berhasil dihapus."
    })))
}

// 5. Pengumpulan Tugas (PDF, DOCX, MP4)
#[post("/api/paskibraka/pengumpulan")]
pub async fn upload_pengumpulan_tugas(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let upload_dir = "uploads/paskibraka/submissions";
    if let Err(e) = fs::create_dir_all(upload_dir) {
        log::error!("Gagal membuat folder upload: {:?}", e);
    }

    let mut id_tugas = String::new();
    let mut catatan_siswa = None;
    let mut file_url = String::new();
    let mut file_type = String::new();

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let content_disposition = field.content_disposition().cloned();
        let field_name = content_disposition
            .as_ref()
            .and_then(|cd| cd.get_name())
            .unwrap_or("");

        if field_name == "id_tugas" {
            let mut value_bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                value_bytes.extend_from_slice(&chunk);
            }
            id_tugas = String::from_utf8(value_bytes)
                .unwrap_or_default()
                .trim()
                .to_string();
        } else if field_name == "catatan_siswa" {
            let mut value_bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                value_bytes.extend_from_slice(&chunk);
            }
            let text = String::from_utf8(value_bytes)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !text.is_empty() {
                catatan_siswa = Some(text);
            }
        } else if field_name == "file" {
            let filename = content_disposition
                .as_ref()
                .and_then(|cd| cd.get_filename())
                .unwrap_or("file");
            let ext = Path::new(filename)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            if ext != "pdf" && ext != "docx" && ext != "doc" && ext != "mp4" {
                return Err(actix_web::error::ErrorBadRequest(
                    "Format file tidak didukung. Harus berupa PDF, DOCX, atau MP4.",
                ));
            }

            file_type = ext.clone();
            let new_filename =
                format!("{}_{}_{}.{}", claims.user_id, Uuid::new_v4(), id_tugas, ext);
            let filepath = format!("{}/{}", upload_dir, new_filename);

            let mut f = fs::File::create(&filepath).map_err(|e| {
                log::error!("Gagal membuat file upload: {:?}", e);
                actix_web::error::ErrorInternalServerError("Gagal menyimpan berkas")
            })?;

            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                f.write_all(&chunk).map_err(|e| {
                    log::error!("Gagal menulis chunk file: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Gagal menyimpan berkas")
                })?;
            }

            file_url = format!("{}/{}", upload_dir, new_filename);
        }
    }

    if id_tugas.is_empty() {
        return Err(actix_web::error::ErrorBadRequest("Tugas harus dipilih"));
    }

    if file_url.is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "File pengumpulan tugas tidak boleh kosong",
        ));
    }

    let task_row = sqlx::query("SELECT deadline FROM paskibraka_tugas WHERE id = ?")
        .bind(&id_tugas)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?;

    if let Some(t) = task_row {
        let deadline: chrono::NaiveDateTime = t.get("deadline");
        let now_naive = chrono::Local::now().naive_local();
        if now_naive > deadline {
            return Err(actix_web::error::ErrorBadRequest(
                "Batas waktu pengumpulan tugas ini telah berakhir.",
            ));
        }
    } else {
        return Err(actix_web::error::ErrorNotFound("Tugas tidak ditemukan"));
    }

    let existing =
        sqlx::query("SELECT id FROM paskibraka_pengumpulan WHERE id_tugas = ? AND user_id = ?")
            .bind(&id_tugas)
            .bind(&claims.user_id)
            .fetch_optional(pool.get_ref())
            .await
            .unwrap_or(None);

    let submission_id = if let Some(ext) = existing {
        let ext_id: String = ext.get("id");
        sqlx::query(
            "UPDATE paskibraka_pengumpulan SET file_url = ?, file_type = ?, catatan_siswa = ?, submitted_at = NOW() WHERE id = ?"
        )
        .bind(&file_url)
        .bind(&file_type)
        .bind(&catatan_siswa)
        .bind(&ext_id)
        .execute(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Gagal memperbarui pengumpulan"))?;
        ext_id
    } else {
        let new_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO paskibraka_pengumpulan (id, id_tugas, user_id, id_capaska, nama_siswa, file_url, file_type, catatan_siswa, submitted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW())"
        )
        .bind(&new_id)
        .bind(&id_tugas)
        .bind(&claims.user_id)
        .bind(&claims.id_pdp)
        .bind(&claims.nama_user)
        .bind(&file_url)
        .bind(&file_type)
        .bind(&catatan_siswa)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Error saving submission: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menyimpan pengumpulan tugas")
        })?;
        new_id
    };

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Tugas berhasil dikumpulkan!",
        "submission_id": submission_id,
        "file_url": file_url
    })))
}

#[get("/api/paskibraka/pengumpulan")]
pub async fn get_pengumpulan_tugas(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let is_admin = claims.role == "Superadmin"
        || claims.role == "Administrator"
        || claims.role == "Admin Kesbangpol";

    let rows = if is_admin {
        sqlx::query_as::<_, PaskibrakaPengumpulan>(
            r#"SELECT
                p.id, p.id_tugas, p.user_id, p.id_capaska, p.nama_siswa, p.file_url, p.file_type, p.catatan_siswa, p.submitted_at,
                t.judul as judul_tugas, t.deadline as deadline_tugas,
                pen.nilai, pen.catatan_admin
               FROM paskibraka_pengumpulan p
               LEFT JOIN paskibraka_tugas t ON p.id_tugas = t.id
               LEFT JOIN paskibraka_penilaian pen ON p.id = pen.id_pengumpulan
               ORDER BY p.submitted_at DESC"#
        )
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, PaskibrakaPengumpulan>(
            r#"SELECT
                p.id, p.id_tugas, p.user_id, p.id_capaska, p.nama_siswa, p.file_url, p.file_type, p.catatan_siswa, p.submitted_at,
                t.judul as judul_tugas, t.deadline as deadline_tugas,
                pen.nilai, pen.catatan_admin
               FROM paskibraka_pengumpulan p
               LEFT JOIN paskibraka_tugas t ON p.id_tugas = t.id
               LEFT JOIN paskibraka_penilaian pen ON p.id = pen.id_pengumpulan
               WHERE p.user_id = ?
               ORDER BY p.submitted_at DESC"#,
        )
        .bind(&claims.user_id)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default()
    };

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": rows
    })))
}

// 6. Penilaian
#[get("/api/paskibraka/penilaian")]
pub async fn get_penilaian_paskibraka(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let is_admin = claims.role == "Superadmin"
        || claims.role == "Administrator"
        || claims.role == "Admin Kesbangpol";

    let rows = if is_admin {
        sqlx::query(
            r#"SELECT pen.id, pen.id_pengumpulan, pen.id_tugas, pen.user_id, pen.nilai, pen.catatan_admin, pen.created_at,
                      p.nama_siswa, p.file_url, p.file_type, t.judul as judul_tugas
               FROM paskibraka_penilaian pen
               LEFT JOIN paskibraka_pengumpulan p ON pen.id_pengumpulan = p.id
               LEFT JOIN paskibraka_tugas t ON pen.id_tugas = t.id
               ORDER BY pen.created_at DESC"#
        )
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default()
    } else {
        sqlx::query(
            r#"SELECT pen.id, pen.id_pengumpulan, pen.id_tugas, pen.user_id, pen.nilai, pen.catatan_admin, pen.created_at,
                      p.nama_siswa, p.file_url, p.file_type, t.judul as judul_tugas
               FROM paskibraka_penilaian pen
               LEFT JOIN paskibraka_pengumpulan p ON pen.id_pengumpulan = p.id
               LEFT JOIN paskibraka_tugas t ON pen.id_tugas = t.id
               WHERE pen.user_id = ?
               ORDER BY pen.created_at DESC"#
        )
        .bind(&claims.user_id)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default()
    };

    let data = rows
        .into_iter()
        .map(|r| {
            let id: String = r.get("id");
            let id_pengumpulan: String = r.get("id_pengumpulan");
            let id_tugas: String = r.get("id_tugas");
            let user_id: String = r.get("user_id");
            let nilai: String = r.get("nilai");
            let catatan_admin: Option<String> = r.get("catatan_admin");
            let created_at: Option<chrono::NaiveDateTime> = r.get("created_at");
            let nama_siswa: Option<String> = r.get("nama_siswa");
            let file_url: Option<String> = r.get("file_url");
            let file_type: Option<String> = r.get("file_type");
            let judul_tugas: Option<String> = r.get("judul_tugas");
            json!({
                "id": id,
                "id_pengumpulan": id_pengumpulan,
                "id_tugas": id_tugas,
                "user_id": user_id,
                "nilai": nilai,
                "catatan_admin": catatan_admin,
                "created_at": created_at,
                "nama_siswa": nama_siswa,
                "file_url": file_url,
                "file_type": file_type,
                "judul_tugas": judul_tugas
            })
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": data
    })))
}

#[post("/api/paskibraka/penilaian")]
pub async fn submit_penilaian(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    body: web::Json<PenilaianRequest>,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if claims.role != "Superadmin"
        && claims.role != "Administrator"
        && claims.role != "Admin Kesbangpol"
    {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak: khusus admin",
        ));
    }

    let existing = sqlx::query("SELECT id FROM paskibraka_penilaian WHERE id_pengumpulan = ?")
        .bind(&body.id_pengumpulan)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap_or(None);

    if let Some(ext) = existing {
        let ext_id: String = ext.get("id");
        sqlx::query("UPDATE paskibraka_penilaian SET nilai = ?, catatan_admin = ?, graded_by = ?, created_at = NOW() WHERE id = ?")
            .bind(&body.nilai)
            .bind(&body.catatan_admin)
            .bind(&claims.nama_user)
            .bind(&ext_id)
            .execute(pool.get_ref())
            .await
            .map_err(|_| actix_web::error::ErrorInternalServerError("Gagal memperbarui nilai"))?;
    } else {
        let new_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO paskibraka_penilaian (id, id_pengumpulan, id_tugas, user_id, nilai, catatan_admin, graded_by, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, NOW())"
        )
        .bind(&new_id)
        .bind(&body.id_pengumpulan)
        .bind(&body.id_tugas)
        .bind(&body.user_id)
        .bind(&body.nilai)
        .bind(&body.catatan_admin)
        .bind(&claims.nama_user)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Error saving grade: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menyimpan nilai")
        })?;
    }

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Penilaian berhasil disimpan!"
    })))
}

#[put("/api/paskibraka/profile")]
pub async fn update_paskibraka_profile(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    init_paskibraka_tables(pool.get_ref()).await;

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let upload_dir = "uploads/assets/images/avatars";
    if let Err(e) = fs::create_dir_all(upload_dir) {
        log::error!("Gagal membuat folder avatar: {:?}", e);
    }

    let mut phone = None;
    let mut address = None;
    let mut nama_sekolah = None;
    let mut guru_pembimbing = None;
    let mut no_hp_guru_pembimbing = None;
    let mut avatar_url = None;

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().cloned();
        let field_name = cd.as_ref().and_then(|c| c.get_name()).unwrap_or("");

        if field_name == "phone" {
            let mut value_bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                value_bytes.extend_from_slice(&chunk);
            }
            let text = String::from_utf8(value_bytes)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !text.is_empty() {
                phone = Some(text);
            }
        } else if field_name == "address" {
            let mut value_bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                value_bytes.extend_from_slice(&chunk);
            }
            let text = String::from_utf8(value_bytes)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !text.is_empty() {
                address = Some(text);
            }
        } else if field_name == "nama_sekolah" || field_name == "asal_sekolah" {
            let mut value_bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                value_bytes.extend_from_slice(&chunk);
            }
            let text = String::from_utf8(value_bytes)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !text.is_empty() {
                nama_sekolah = Some(text);
            }
        } else if field_name == "guru_pembimbing" {
            let mut value_bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                value_bytes.extend_from_slice(&chunk);
            }
            let text = String::from_utf8(value_bytes)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !text.is_empty() {
                guru_pembimbing = Some(text);
            }
        } else if field_name == "no_hp_guru_pembimbing" {
            let mut value_bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                value_bytes.extend_from_slice(&chunk);
            }
            let text = String::from_utf8(value_bytes)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !text.is_empty() {
                no_hp_guru_pembimbing = Some(text);
            }
        } else if field_name == "photo" || field_name == "avatar" {
            let filename = cd
                .as_ref()
                .and_then(|c| c.get_filename())
                .unwrap_or("avatar.png");
            let ext = Path::new(filename)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("png")
                .to_lowercase();

            let safe_ext = match ext.as_str() {
                "png" | "jpg" | "jpeg" | "webp" => ext,
                _ => "png".to_string(),
            };

            let new_filename = format!(
                "avatar_paskibraka_{}_{}.{}",
                claims.user_id,
                Uuid::new_v4(),
                safe_ext
            );
            let filepath = format!("{}/{}", upload_dir, new_filename);

            let mut f = fs::File::create(&filepath).map_err(|e| {
                log::error!("Gagal membuat file foto: {:?}", e);
                actix_web::error::ErrorInternalServerError("Gagal menyimpan foto profil")
            })?;

            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                f.write_all(&chunk).map_err(|e| {
                    log::error!("Gagal menulis photo chunk: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Gagal menyimpan foto profil")
                })?;
            }

            avatar_url = Some(format!("{}/{}", upload_dir, new_filename));
        }
    }

    // Resolve id_pdp from users table if not in claims
    let user_row = sqlx::query("SELECT id_pdp FROM users WHERE id = ?")
        .bind(&claims.user_id)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap_or(None);

    let id_pdp: Option<String> = user_row.and_then(|r| r.get("id_pdp"));
    let target_id_pdp = claims.id_pdp.clone().or(id_pdp);

    // Update users table with COALESCE so None fields retain their existing values
    sqlx::query(
        r#"UPDATE users SET 
            phone = COALESCE(?, phone), 
            address = COALESCE(?, address), 
            nama_sekolah = COALESCE(?, nama_sekolah), 
            guru_pembimbing = COALESCE(?, guru_pembimbing), 
            no_hp_guru_pembimbing = COALESCE(?, no_hp_guru_pembimbing), 
            avatar = COALESCE(?, avatar) 
        WHERE id = ?"#
    )
    .bind(&phone)
    .bind(&address)
    .bind(&nama_sekolah)
    .bind(&guru_pembimbing)
    .bind(&no_hp_guru_pembimbing)
    .bind(&avatar_url)
    .bind(&claims.user_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Gagal memperbarui profil user di DB: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memperbarui profil")
    })?;

    // Synchronize to data_capaska table if linked
    if let Some(ref pdp_id) = target_id_pdp {
        let _ = sqlx::query(
            r#"UPDATE data_capaska SET 
                photo = COALESCE(?, photo), 
                asal_sekolah = COALESCE(?, asal_sekolah), 
                guru_pembimbing = COALESCE(?, guru_pembimbing), 
                no_hp_guru_pembimbing = COALESCE(?, no_hp_guru_pembimbing) 
            WHERE id = ?"#
        )
        .bind(&avatar_url)
        .bind(&nama_sekolah)
        .bind(&guru_pembimbing)
        .bind(&no_hp_guru_pembimbing)
        .bind(pdp_id)
        .execute(pool.get_ref())
        .await;
    }

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Profil Paskibraka berhasil diperbarui",
        "avatar": avatar_url,
        "phone": phone,
        "address": address,
        "nama_sekolah": nama_sekolah,
        "guru_pembimbing": guru_pembimbing,
        "no_hp_guru_pembimbing": no_hp_guru_pembimbing
    })))
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[post("/api/paskibraka/change-password")]
pub async fn change_paskibraka_password(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    body: web::Json<ChangePasswordRequest>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if body.new_password.len() < 6 {
        return Ok(HttpResponse::Ok().json(json!({
            "success": false,
            "message": "Password baru minimal 6 karakter!"
        })));
    }

    if body.new_password != body.confirm_password {
        return Ok(HttpResponse::Ok().json(json!({
            "success": false,
            "message": "Konfirmasi password baru tidak cocok!"
        })));
    }

    let user_row = sqlx::query("SELECT password FROM users WHERE id = ?")
        .bind(&claims.user_id)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorNotFound("User tidak ditemukan"))?;

    let db_pass: String = user_row.get("password");

    let is_valid = bcrypt::verify(&body.old_password, &db_pass).unwrap_or(false);
    if !is_valid {
        return Ok(HttpResponse::Ok().json(json!({
            "success": false,
            "message": "Password lama Anda salah!"
        })));
    }

    let new_hash = bcrypt::hash(&body.new_password, bcrypt::DEFAULT_COST)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Gagal memproses password baru"))?;

    sqlx::query("UPDATE users SET password = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(&claims.user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Gagal memperbarui password"))?;

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Password berhasil diperbarui! Silakan gunakan password baru ini untuk login berikutnya."
    })))
}
