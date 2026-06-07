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
use sqlx::{MySqlPool, prelude::FromRow};
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

    let mut nama_lengkap: Option<String> = None; // presence = Some
    let mut jk: Option<String> = None; // presence = Some
    let mut id_provinsi: Option<i32> = None; // presence = Some
    let mut id_kabupaten: Option<i32> = None; // presence = Some
    let mut asal_sma: Option<String> = None; // presence = Some
    let mut tahun_tugas: Option<i32> = None; // presence = Some
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
                nama_lengkap = Some(read_text_field(field).await?);
            }
            "jk" => {
                let v = read_text_field(field).await?;
                jk = Some(v);
            }
            "id_provinsi" => {
                id_provinsi = None;
            }
            "id_kabupaten" => {
                id_kabupaten = None;
            }
            "asal_sma" => {
                let v = read_text_field(field).await?;
                asal_sma = Some(v);
            }
            "tahun_tugas" => {
                tahun_tugas = None;
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
        "SELECT id, nama_lengkap, jk, id_provinsi, id_kabupaten, asal_sma, tahun_tugas,  photo FROM paskibraka_nasional WHERE id = ?",
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
