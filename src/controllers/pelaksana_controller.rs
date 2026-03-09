// src/controllers/pelaksana_controller.rs
use crate::auth;
use crate::controllers::pdp_controller::{EncryptedPdp, decrypt_pdp_row};
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, delete, get, post, put, web, web::Data,
};
use futures_util::TryStreamExt as _;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::{MySql, QueryBuilder};
use sqlx::{MySqlPool, prelude::FromRow};
use std::cmp::{max, min};
use std::path::PathBuf;
use std::{fs, io::Write, path::Path};

//pagination dan pencarian
#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    q: Option<String>,
    id_provinsi: Option<i32>,
}

#[derive(Serialize)]
struct PaginationLink {
    url: Option<String>,
    label: String,
    active: bool,
}

#[derive(Serialize)]
struct PaginatedResponse<T> {
    links: Vec<PaginationLink>,
    data: Vec<T>,
    current_page: u32,
    total_pages: u32,
    total_items: u64,
    per_page: u32,
    from: u64,
    to: u64,
    query: String,
}

fn make_url(base_path: &str, page: u32, per_page: u32, q: &Option<String>) -> String {
    match q {
        Some(s) if !s.is_empty() => format!(
            "{}?page={}&per_page={}&q={}",
            base_path,
            page,
            per_page,
            urlencoding::encode(s)
        ),
        _ => format!("{}?page={}&per_page={}", base_path, page, per_page),
    }
}

fn build_links(
    base_path: &str,
    current: u32,
    total_pages: u32,
    per_page: u32,
    q: &Option<String>,
) -> Vec<PaginationLink> {
    let mut links: Vec<PaginationLink> = Vec::new();

    // Prev
    if current > 1 {
        links.push(PaginationLink {
            url: Some(make_url(base_path, current - 1, per_page, q)),
            label: "&laquo; Previous".into(),
            active: false,
        });
    } else {
        links.push(PaginationLink {
            url: None,
            label: "&laquo; Previous".into(),
            active: false,
        });
    }

    // Pages 1..N
    for p in 1..=total_pages {
        if p == current {
            links.push(PaginationLink {
                url: None,
                label: p.to_string(),
                active: true,
            });
        } else {
            links.push(PaginationLink {
                url: Some(make_url(base_path, p, per_page, q)),
                label: p.to_string(),
                active: false,
            });
        }
    }

    // Next
    if current < total_pages {
        links.push(PaginationLink {
            url: Some(make_url(base_path, current + 1, per_page, q)),
            label: "Next &raquo;".into(),
            active: false,
        });
    } else {
        links.push(PaginationLink {
            url: None,
            label: "Next &raquo;".into(),
            active: false,
        });
    }

    links
}

fn build_links_pelaksana_provinsi(
    base_path: &str,
    current: u32,
    total_pages: u32,
    per_page: u64,
    q: &Option<String>,
    id_provinsi: Option<i32>, // TAMBAHAN
) -> Vec<PaginationLink> {
    let mut links = Vec::new();

    // Previous page
    if current > 1 {
        let mut query_params = format!("page={}&per_page={}", current - 1, per_page);
        if let Some(q) = q {
            if !q.is_empty() {
                query_params.push_str(&format!("&q={}", urlencoding::encode(q)));
            }
        }
        // TAMBAHAN: Include id_provinsi
        if let Some(prov_id) = id_provinsi {
            query_params.push_str(&format!("&id_provinsi={}", prov_id));
        }
        links.push(PaginationLink {
            url: Some(format!("{}?{}", base_path, query_params)),
            label: "« Previous".to_string(),
            active: false,
        });
    }

    // Numbered pages
    for p in 1..=total_pages {
        let mut query_params = format!("page={}&per_page={}", p, per_page);
        if let Some(q) = q {
            if !q.is_empty() {
                query_params.push_str(&format!("&q={}", urlencoding::encode(q)));
            }
        }
        // TAMBAHAN: Include id_provinsi
        if let Some(prov_id) = id_provinsi {
            query_params.push_str(&format!("&id_provinsi={}", prov_id));
        }

        links.push(PaginationLink {
            url: Some(format!("{}?{}", base_path, query_params)),
            label: p.to_string(),
            active: p == current,
        });
    }

    // Next page
    if current < total_pages {
        let mut query_params = format!("page={}&per_page={}", current + 1, per_page);
        if let Some(q) = q {
            if !q.is_empty() {
                query_params.push_str(&format!("&q={}", urlencoding::encode(q)));
            }
        }
        // TAMBAHAN: Include id_provinsi
        if let Some(prov_id) = id_provinsi {
            query_params.push_str(&format!("&id_provinsi={}", prov_id));
        }
        links.push(PaginationLink {
            url: Some(format!("{}?{}", base_path, query_params)),
            label: "Next »".to_string(),
            active: false,
        });
    }

    links
}
//helper uploads
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
        "pelaksana_{}.{}",
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
#[derive(Serialize, FromRow, Debug)]
struct Jabatan {
    id: i32,
    nama_jabatan: String,
}

#[get("/api/adminpanel/jabatan")]
pub async fn get_jabatan(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let jabatan: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(jabatan))
}

#[get("/api/adminpanel/jabatan-kabupaten")]
pub async fn get_jabatan_kabupaten(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let jabatan: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan_kabupaten")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(jabatan))
}

#[get("/api/adminpanel/jabatan-provinsi")]
pub async fn get_jabatan_provinsi(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let jabatan: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan_provinsi")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(jabatan))
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct PelaksanaPusat {
    id: i32,
    id_pdp: Option<String>,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
}

#[get("/api/adminpanel/pelaksana-pusat")]
pub async fn get_pelaksana_pusat(
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
    let base_path = "/api/adminpanel/pelaksana-pusat";

    // Count
    let (total_items,): (i64,) = if let Some(ref keyword) = q {
        let like = format!("%{}%", keyword);
        sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM pelaksana_pusat
             WHERE nama_lengkap LIKE ? OR jabatan LIKE ?",
        )
        .bind(&like)
        .bind(&like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM pelaksana_pusat")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let total_items = max(0, total_items) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + (per_page as u64) - 1) / (per_page as u64)) as u32
    };
    let current = min(page, total_pages);
    let offset = ((current - 1) as u64) * (per_page as u64);

    // Data
    let data: Vec<PelaksanaPusat> = if let Some(ref keyword) = q {
        let like = format!("%{}%", keyword);
        sqlx::query_as::<_, PelaksanaPusat>(
            "SELECT id, id_pdp, nama_lengkap, photo, jabatan
             FROM pelaksana_pusat
             WHERE nama_lengkap LIKE ? OR jabatan LIKE ?
             ORDER BY id ASC
             LIMIT ? OFFSET ?",
        )
        .bind(&like)
        .bind(&like)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, PelaksanaPusat>(
            "SELECT id, id_pdp, nama_lengkap, photo, jabatan
             FROM pelaksana_pusat
             ORDER BY id ASC
             LIMIT ? OFFSET ?",
        )
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        std::cmp::min(offset + data.len() as u64, total_items)
    };

    // 🔗 Links yang kompatibel dengan Components/Pagination.tsx
    let links = build_links(base_path, current, total_pages, per_page, &q);

    let resp = PaginatedResponse {
        links,
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

#[post("/api/adminpanel/pelaksana-pusat")]
pub async fn create_pelaksana_pusat(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&_claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator",
        ));
    }

    let mut nama_lengkap: Option<String> = None;
    let mut jabatan: Option<String> = None;
    let mut id_pdp_val: Option<i32> = None;
    let mut photo_path: Option<String> = None;

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
            "jabatan" => {
                let v = read_text_field(field).await?;
                jabatan = if v.is_empty() { None } else { Some(v) };
            }
            "id_pdp" => {
                let v = read_text_field(field).await?;
                id_pdp_val = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "photo" => {
                photo_path = Some(save_photo_field(field, "./uploads/assets/pelaksana").await?);
            }
            _ => { /* ignore */ }
        }
    }

    let nama =
        nama_lengkap.ok_or_else(|| actix_web::error::ErrorBadRequest("nama_lengkap wajib"))?;

    let result = sqlx::query(
        "INSERT INTO pelaksana_pusat (id_pdp, nama_lengkap, photo, jabatan)
         VALUES (?, ?, ?, ?)",
    )
    .bind(id_pdp_val)
    .bind(&nama)
    .bind(&photo_path)
    .bind(&jabatan)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let id = result.last_insert_id() as i32;

    let created = sqlx::query_as::<_, PelaksanaPusat>(
        "SELECT id, id_pdp, nama_lengkap, photo, jabatan FROM pelaksana_pusat WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(created))
}

#[put("/api/adminpanel/pelaksana-pusat/{id}")]
pub async fn update_pelaksana_pusat(
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

    // tri-state utk id_pdp & photo:
    // present(false/true), value(None/Some)
    let mut id_pdp_present = false;
    let mut id_pdp_value: Option<i32> = None;

    let mut nama_lengkap: Option<String> = None; // presence = Some
    let mut jabatan: Option<String> = None; // presence = Some

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
            "jabatan" => {
                let v = read_text_field(field).await?;
                jabatan = Some(v); // empty string => set empty string; ubah ke None kalau mau treat kosong jadi NULL
            }
            "id_pdp" => {
                id_pdp_present = true;
                let v = read_text_field(field).await?;
                id_pdp_value = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "photo" => {
                photo_new_path = Some(save_photo_field(field, "./uploads/assets/pelaksana").await?);
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
        sqlx::query_as("SELECT photo FROM pelaksana_pusat WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut qb: QueryBuilder<MySql> = QueryBuilder::new("UPDATE pelaksana_pusat SET ");
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
    if let Some(v) = jabatan {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("jabatan = ").push_bind(v);
    }

    if id_pdp_present {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("id_pdp = ");
        match id_pdp_value {
            Some(val) => {
                qb.push_bind(val);
            }
            None => {
                qb.push("NULL");
            }
        }
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
    let updated = sqlx::query_as::<_, PelaksanaPusat>(
        "SELECT id, id_pdp, nama_lengkap, photo, jabatan FROM pelaksana_pusat WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/api/adminpanel/pelaksana-pusat/{id}")]
pub async fn delete_pelaksana_pusat(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let id = path.into_inner();

    // Ambil foto lama sebelum delete
    let (old_photo_opt,): (Option<String>,) =
        sqlx::query_as("SELECT photo FROM pelaksana_pusat WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // Hapus row
    let result = sqlx::query("DELETE FROM pelaksana_pusat WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().body("Data tidak ditemukan"));
    }

    // Hapus file fisik
    if let Some(oldp) = old_photo_opt {
        remove_file_if_exists(&oldp);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Berhasil dihapus",
        "id": id
    })))
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct PelaksanaProvinsi {
    id: i32,
    id_pdp: Option<String>,
    id_provinsi: i32,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_provinsi: Option<String>,
}

#[get("/api/adminpanel/pelaksana-provinsi")]
pub async fn get_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
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

    let q_trimmed = query.q.as_ref().map(|s| s.trim().to_string());
    let has_q = q_trimmed.as_ref().is_some_and(|s| !s.is_empty());

    // TAMBAHAN BARU: Filter provinsi
    let id_provinsi = query.id_provinsi;
    let has_provinsi_filter = id_provinsi.is_some();

    let base_path = "/api/adminpanel/pelaksana-provinsi";

    // ===== COUNT =====
    let (total_items,): (i64,) = if has_q && has_provinsi_filter {
        // CASE 1: Ada pencarian DAN filter provinsi
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE (pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?)
            AND pp.id_provinsi = ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q {
        // CASE 2: Hanya ada pencarian
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter {
        // CASE 3: Hanya ada filter provinsi
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.id_provinsi = ?
            "#,
        )
        .bind(id_provinsi.unwrap())
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // CASE 4: Tidak ada filter
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            "#,
        )
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let total_items = max(0, total_items) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + per_page as u64 - 1) / per_page as u64) as u32
    };
    let current = min(page, total_pages);
    let offset = ((current - 1) as u64) * (per_page as u64);

    // ===== DATA =====
    let data: Vec<PelaksanaProvinsi> = if has_q && has_provinsi_filter {
        // CASE 1: Ada pencarian DAN filter provinsi
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE (pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?)
            AND pp.id_provinsi = ?
            ORDER BY pp.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q {
        // CASE 2: Hanya ada pencarian
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?
            ORDER BY pp.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter {
        // CASE 3: Hanya ada filter provinsi
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.id_provinsi = ?
            ORDER BY pp.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(id_provinsi.unwrap())
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // CASE 4: Tidak ada filter
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            ORDER BY pp.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        std::cmp::min(offset + data.len() as u64, total_items)
    };

    // TAMBAHAN: Include id_provinsi dalam build_links jika ada
    let links = build_links_pelaksana_provinsi(
        base_path,
        current,
        total_pages,
        per_page.into(),
        &q_trimmed,
        id_provinsi,
    );

    let resp = PaginatedResponse {
        links,
        data,
        current_page: current,
        total_pages,
        total_items,
        per_page,
        from,
        to,
        query: q_trimmed.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[post("/api/adminpanel/pelaksana-provinsi")]
pub async fn create_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&_claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator",
        ));
    }

    let mut nama_lengkap: Option<String> = None;
    let mut jabatan: Option<String> = None;
    let mut id_pdp_val: Option<i32> = None;
    let mut id_provinsi_val: Option<i32> = None;
    let mut photo_path: Option<String> = None;

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
            "jabatan" => {
                let v = read_text_field(field).await?;
                jabatan = if v.is_empty() { None } else { Some(v) };
            }
            "id_pdp" => {
                let v = read_text_field(field).await?;
                id_pdp_val = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "id_provinsi" => {
                let v = read_text_field(field).await?;
                id_provinsi_val = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "photo" => {
                photo_path = Some(save_photo_field(field, "./uploads/assets/pelaksana").await?);
            }
            _ => { /* ignore */ }
        }
    }

    let nama =
        nama_lengkap.ok_or_else(|| actix_web::error::ErrorBadRequest("nama_lengkap wajib"))?;

    let result = sqlx::query(
        "INSERT INTO pelaksana_provinsi (id_pdp, id_provinsi, nama_lengkap, photo, jabatan)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id_pdp_val)
    .bind(id_provinsi_val)
    .bind(&nama)
    .bind(&photo_path)
    .bind(&jabatan)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let id = result.last_insert_id() as i32;

    let created = sqlx::query_as::<_, PelaksanaProvinsi>(
        "SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
             FROM pelaksana_provinsi pp
             LEFT JOIN provinsi p ON pp.id_provinsi = p.id
             WHERE pp.id = ?",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(created))
}

#[put("/api/adminpanel/pelaksana-provinsi/{id}")]
pub async fn update_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&_claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator",
        ));
    }

    let id = path.into_inner();
    let mut id_pdp_present = false;
    let mut id_pdp_value: Option<i32> = None;

    let mut id_provinsi_present = false;
    let mut id_provinsi_value: Option<i32> = None;

    let mut nama_lengkap: Option<String> = None; // presence = Some
    let mut jabatan: Option<String> = None; // presence = Some

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
            "jabatan" => {
                let v = read_text_field(field).await?;
                jabatan = Some(v); // empty string => set empty string; ubah ke None kalau mau treat kosong jadi NULL
            }
            "id_pdp" => {
                id_pdp_present = true;
                let v = read_text_field(field).await?;
                id_pdp_value = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "id_provinsi" => {
                id_provinsi_present = true;
                let v = read_text_field(field).await?;
                id_provinsi_value = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "photo" => {
                photo_new_path = Some(save_photo_field(field, "./uploads/assets/pelaksana").await?);
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
        sqlx::query_as("SELECT photo FROM pelaksana_provinsi WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut qb: QueryBuilder<MySql> = QueryBuilder::new("UPDATE pelaksana_provinsi SET ");
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
    if let Some(v) = jabatan {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("jabatan = ").push_bind(v);
    }

    if id_pdp_present {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("id_pdp = ");
        match id_pdp_value {
            Some(val) => {
                qb.push_bind(val);
            }
            None => {
                qb.push("NULL");
            }
        }
    }
    if id_provinsi_present {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("id_provinsi = ");
        match id_provinsi_value {
            Some(val) => {
                qb.push_bind(val);
            }
            None => {
                qb.push("NULL");
            }
        }
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
    let updated = sqlx::query_as::<_, PelaksanaProvinsi>(
        "SELECT
            pp.id,
            pp.id_pdp,
            pp.id_provinsi,
            pp.nama_lengkap,
            pp.photo,
            pp.jabatan,
            p.nama_provinsi
         FROM pelaksana_provinsi pp
         LEFT JOIN provinsi p ON pp.id_provinsi = p.id
         WHERE pp.id = ?",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/api/adminpanel/pelaksana-provinsi/{id}")]
pub async fn delete_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let id = path.into_inner();

    // Ambil foto lama sebelum delete
    let (old_photo_opt,): (Option<String>,) =
        sqlx::query_as("SELECT photo FROM pelaksana_provinsi WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // Hapus row
    let result = sqlx::query("DELETE FROM pelaksana_provinsi WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().body("Data tidak ditemukan"));
    }

    // Hapus file fisik
    if let Some(oldp) = old_photo_opt {
        remove_file_if_exists(&oldp);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Berhasil dihapus",
        "id": id
    })))
}

#[derive(Serialize, FromRow, Debug)]
struct PelaksanaKabupaten {
    id: i32,
    id_pdp: Option<String>,
    id_provinsi: i32,
    id_kabupaten: i32,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_provinsi: Option<String>,
    nama_kabupaten: Option<String>,
}

#[derive(Deserialize)]
struct ListQueryKabupaten {
    page: Option<u32>,
    per_page: Option<u32>,
    q: Option<String>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
}

// Update fungsi get_pelaksana_kabupaten
#[get("/api/adminpanel/pelaksana-kabupaten")]
pub async fn get_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQueryKabupaten>,
) -> Result<impl Responder, Error> {
    // Auth
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Administrator atau Pelaksana yang dapat mengakses",
        ));
    }

    use std::cmp::{max, min};

    // Params
    let mut page = query.page.unwrap_or(1);
    if page == 0 {
        page = 1;
    }
    let mut per_page = query.per_page.unwrap_or(10);
    per_page = per_page.clamp(1, 100);

    let q_trimmed = query.q.as_ref().map(|s| s.trim().to_string());
    let has_q = q_trimmed.as_ref().is_some_and(|s| !s.is_empty());

    // TAMBAHAN BARU: Filter provinsi & kabupaten
    let id_provinsi = query.id_provinsi;
    let has_provinsi_filter = id_provinsi.is_some();
    let id_kabupaten = query.id_kabupaten;
    let has_kabupaten_filter = id_kabupaten.is_some();

    let base_path = "/api/adminpanel/pelaksana-kabupaten";

    // COUNT
    let (total_items,): (i64,) = if has_q && has_provinsi_filter && has_kabupaten_filter {
        // CASE 1: Ada pencarian + provinsi + kabupaten
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_provinsi = ? AND pk.id_kabupaten = ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .bind(id_kabupaten.unwrap())
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q && has_provinsi_filter {
        // CASE 2: Ada pencarian + provinsi
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_provinsi = ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q && has_kabupaten_filter {
        // CASE 3: Ada pencarian + kabupaten
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_kabupaten = ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_kabupaten.unwrap())
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter && has_kabupaten_filter {
        // CASE 4: Provinsi + kabupaten (tanpa pencarian)
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_provinsi = ? AND pk.id_kabupaten = ?
            "#,
        )
        .bind(id_provinsi.unwrap())
        .bind(id_kabupaten.unwrap())
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter {
        // CASE 5: Hanya provinsi (tanpa pencarian) - CASE BARU YANG DITAMBAHKAN
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_provinsi = ?
            "#,
        )
        .bind(id_provinsi.unwrap())
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_kabupaten_filter {
        // CASE 6: Hanya kabupaten (tanpa pencarian)
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_kabupaten = ?
            "#,
        )
        .bind(id_kabupaten.unwrap())
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q {
        // CASE 7: Hanya pencarian
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // CASE 8: Tidak ada filter
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            "#,
        )
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let total_items = max(0, total_items) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + per_page as u64 - 1) / per_page as u64) as u32
    };
    let current = min(page, total_pages);
    let offset = ((current - 1) as u64) * (per_page as u64);

    // DATA - Implementasi semua case yang sama dengan COUNT
    let data: Vec<PelaksanaKabupaten> = if has_q && has_provinsi_filter && has_kabupaten_filter {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_provinsi = ? AND pk.id_kabupaten = ?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .bind(id_kabupaten.unwrap())
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q && has_provinsi_filter {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_provinsi = ?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q && has_kabupaten_filter {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_kabupaten = ?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_kabupaten.unwrap())
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter && has_kabupaten_filter {
        // CASE 4: Provinsi + kabupaten (tanpa pencarian)
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_provinsi = ? AND pk.id_kabupaten = ?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(id_provinsi.unwrap())
        .bind(id_kabupaten.unwrap())
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter {
        // CASE 5: Hanya provinsi (tanpa pencarian) - CASE BARU YANG DITAMBAHKAN
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_provinsi = ?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(id_provinsi.unwrap())
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_kabupaten_filter {
        // CASE 6: Hanya kabupaten (tanpa pencarian)
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_kabupaten = ?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(id_kabupaten.unwrap())
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        std::cmp::min(offset + data.len() as u64, total_items)
    };

    // Update build_links untuk include filter provinsi & kabupaten
    let links = build_links_pelaksana_kabupaten(
        base_path,
        current,
        total_pages,
        per_page.into(),
        &q_trimmed,
        id_provinsi,
        id_kabupaten,
    );

    let resp = PaginatedResponse {
        links,
        data,
        current_page: current,
        total_pages,
        total_items,
        per_page,
        from,
        to,
        query: q_trimmed.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[post("/api/adminpanel/pelaksana-kabupaten")]
pub async fn create_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&_claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator yang dapat mengakses",
        ));
    }

    let mut nama_lengkap: Option<String> = None;
    let mut jabatan: Option<String> = None;
    let mut id_pdp_val: Option<i32> = None;
    let mut id_kabupaten_val: Option<i32> = None;
    let mut id_provinsi_val: Option<i32> = None; // opsional: kalau dikirim
    let mut photo_path: Option<String> = None;

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
            "jabatan" => {
                let v = read_text_field(field).await?;
                jabatan = if v.is_empty() { None } else { Some(v) };
            }
            "id_pdp" => {
                let v = read_text_field(field).await?;
                id_pdp_val = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "id_kabupaten" => {
                let v = read_text_field(field).await?;
                id_kabupaten_val = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "id_provinsi" => {
                let v = read_text_field(field).await?;
                id_provinsi_val = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "photo" => {
                photo_path = Some(save_photo_field(field, "./uploads/assets/pelaksana").await?);
            }
            _ => {}
        }
    }

    let nama =
        nama_lengkap.ok_or_else(|| actix_web::error::ErrorBadRequest("nama_lengkap wajib"))?;
    let id_kab =
        id_kabupaten_val.ok_or_else(|| actix_web::error::ErrorBadRequest("id_kabupaten wajib"))?;

    let result = sqlx::query(
        "INSERT INTO pelaksana_kabupaten (id_pdp, id_provinsi, id_kabupaten, nama_lengkap, photo, jabatan)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(id_pdp_val)
    .bind(id_provinsi_val)  // ganti ke derive jika pakai opsi derive di atas
    .bind(id_kab)
    .bind(&nama)
    .bind(&photo_path)
    .bind(&jabatan)
    .execute(pool.get_ref()).await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let id = result.last_insert_id() as i32;

    let created = sqlx::query_as::<_, PelaksanaKabupaten>(
        r#"
        SELECT
            pk.id,
            pk.id_pdp,
            pk.id_provinsi,
            pk.id_kabupaten,
            pk.nama_lengkap,
            pk.photo,
            pk.jabatan,
            p.nama_provinsi,
            k.nama_kabupaten
        FROM pelaksana_kabupaten pk
        LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
        LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
        WHERE pk.id = ?
        "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(created))
}

#[put("/api/adminpanel/pelaksana-kabupaten/{id}")]
pub async fn update_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    path: web::Path<i32>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&_claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator yang dapat mengakses",
        ));
    }
    let id = path.into_inner();

    let mut id_pdp_present = false;
    let mut id_pdp_value: Option<i32> = None;

    let mut id_kabupaten_present = false;
    let mut id_kabupaten_value: Option<i32> = None;

    let mut id_provinsi_present = false;
    let mut id_provinsi_value: Option<i32> = None;

    let mut nama_lengkap: Option<String> = None;
    let mut jabatan: Option<String> = None;

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
            "jabatan" => {
                let v = read_text_field(field).await?;
                jabatan = Some(v);
            }
            "id_pdp" => {
                id_pdp_present = true;
                let v = read_text_field(field).await?;
                id_pdp_value = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "id_kabupaten" => {
                id_kabupaten_present = true;
                let v = read_text_field(field).await?;
                id_kabupaten_value = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "id_provinsi" => {
                id_provinsi_present = true;
                let v = read_text_field(field).await?;
                id_provinsi_value = if v.is_empty() {
                    None
                } else {
                    v.parse::<i32>().ok()
                };
            }
            "photo" => {
                photo_new_path = Some(save_photo_field(field, "./uploads/assets/pelaksana").await?);
            }
            "photo_remove" => {
                let v = read_text_field(field).await?;
                photo_remove = v == "1" || v.eq_ignore_ascii_case("true");
            }
            _ => {}
        }
    }

    // foto lama
    let (old_photo_opt,): (Option<String>,) =
        sqlx::query_as("SELECT photo FROM pelaksana_kabupaten WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    use sqlx::{MySql, QueryBuilder};
    let mut qb: QueryBuilder<MySql> = QueryBuilder::new("UPDATE pelaksana_kabupaten SET ");
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
    if let Some(v) = jabatan {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("jabatan = ").push_bind(v);
    }

    if id_pdp_present {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("id_pdp = ");
        match id_pdp_value {
            Some(val) => qb.push_bind(val),
            None => qb.push("NULL"),
        };
    }
    if id_kabupaten_present {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("id_kabupaten = ");
        match id_kabupaten_value {
            Some(val) => qb.push_bind(val),
            None => qb.push("NULL"),
        };
    }
    if id_provinsi_present {
        if !first {
            qb.push(", ");
        }
        first = false;
        has_any = true;
        qb.push("id_provinsi = ");
        match id_provinsi_value {
            Some(val) => qb.push_bind(val),
            None => qb.push("NULL"),
        };
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

    qb.build()
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if remove_old {
        if let Some(oldp) = old_photo_opt {
            remove_file_if_exists(&oldp);
        }
    }

    // SELECT akhir — alias & kolom diperbaiki
    let updated = sqlx::query_as::<_, PelaksanaKabupaten>(
        r#"
        SELECT
            pk.id,
            pk.id_pdp,
            pk.id_provinsi,
            pk.id_kabupaten,
            pk.nama_lengkap,
            pk.photo,
            pk.jabatan,
            p.nama_provinsi,
            k.nama_kabupaten
        FROM pelaksana_kabupaten pk
        LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
        LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
        WHERE pk.id = ?
        "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/api/adminpanel/pelaksana-kabupaten/{id}")]
pub async fn delete_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Administrator atau Pelaksana yang dapat mengakses",
        ));
    }
    let id = path.into_inner();
    // Ambil foto lama sebelum delete
    let (old_photo_opt,): (Option<String>,) =
        sqlx::query_as("SELECT photo FROM pelaksana_kabupaten WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?; // Hapus row
    let result = sqlx::query("DELETE FROM pelaksana_kabupaten WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().body("Data tidak ditemukan"));
    } // Hapus file fisik
    if let Some(oldp) = old_photo_opt {
        remove_file_if_exists(&oldp);
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Berhasil dihapus", "id": id })))
}

//ambil data pdp sesuai dengan claims.id_pdp
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub q: Option<String>,
}

#[get("/api/userpanel/get-pdp")]
pub async fn list_pdp_by_claims(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 50);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    // WHERE dasar
    let mut where_clause = String::from(
        " WHERE (p.status = 'Belum Diverifikasi' or p.status = 'Verified' or p.status = 'Simental') ",
    );

    // Kumpulkan placeholder dan bind-nya terpisah agar urutan konsisten
    let mut search_binds: Vec<String> = vec![];
    // Bind untuk filter level pelaksana (campur String/i32)
    enum FB {
        S(String),
        I(i32),
    }
    let mut filter_binds: Vec<FB> = vec![];

    // Filter berbasis role "Pelaksana" sesuai level kepengurusan
    if claims.role == "Pelaksana" {
        // Ambil PDP pemilik token untuk menentukan cakupan
        let (lvl_kep, lvl_tugas, id_prov, id_kab) = sqlx::query_as::<
            _,
            (String, String, Option<i32>, Option<i32>),
        >(
            r#"
        SELECT tingkat_kepengurusan, tingkat_penugasan, id_provinsi, id_kabupaten
        FROM pdp
        WHERE id = ?
        "#,
        )
        .bind(claims.id_pdp)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|_| actix_web::error::ErrorUnauthorized("PDP pemilik token tidak ditemukan"))?;

        // Semua level pelaksana minimal dibatasi oleh tingkat_penugasan
        where_clause.push_str(" AND p.tingkat_penugasan = ? ");
        filter_binds.push(FB::S(lvl_tugas));

        match lvl_kep.as_str() {
            "Pelaksana Tingkat Provinsi" => {
                where_clause.push_str(" AND p.id_provinsi = ? ");
                let prov = id_prov
                    .ok_or_else(|| actix_web::error::ErrorUnauthorized("id_provinsi PDP kosong"))?;
                filter_binds.push(FB::I(prov));
            }
            "Pelaksana Tingkat Kabupaten/Kota" => {
                where_clause.push_str(" AND p.id_provinsi = ? AND p.id_kabupaten = ? ");
                let prov = id_prov
                    .ok_or_else(|| actix_web::error::ErrorUnauthorized("id_provinsi PDP kosong"))?;
                let kab = id_kab.ok_or_else(|| {
                    actix_web::error::ErrorUnauthorized("id_kabupaten PDP kosong")
                })?;
                filter_binds.push(FB::I(prov));
                filter_binds.push(FB::I(kab));
            }
            // "Pelaksana Tingkat Pusat": cukup tingkat_penugasan saja (sudah ditambah di atas)
            _ => {}
        }
    }

    // Pencarian (q)
    if !keyword.is_empty() {
        where_clause.push_str(
            " AND (p.no_piagam LIKE ? OR p.nama_lengkap LIKE ? OR p.no_simental LIKE ? OR p.jk LIKE ? \
               OR p.tingkat_penugasan LIKE ? OR CAST(p.thn_tugas AS CHAR) LIKE ? \
               OR p.email LIKE ? OR p.telepon LIKE ? OR p.nik LIKE ? \
               OR pd.nama_provinsi LIKE ? OR kd.nama_kabupaten LIKE ? \
               OR pp.nama_provinsi LIKE ? OR kp.nama_kabupaten LIKE ?)",
        );
        let needle = format!("%{}%", keyword);
        for _ in 0..13 {
            search_binds.push(needle.clone());
        }
    }

    // ---------- COUNT ----------
    let count_sql = format!(
        "SELECT COUNT(*) AS cnt
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         {where_clause}"
    );

    let mut count_query = sqlx::query(&count_sql);
    // urutan bind MUST match urutan placeholder:
    // 1) search, 2) filter pelaksana
    for b in &search_binds {
        count_query = count_query.bind(b);
    }
    for fb in &filter_binds {
        count_query = match fb {
            FB::S(s) => count_query.bind(s),
            FB::I(i) => count_query.bind(i),
        };
    }

    let total: i64 = count_query
        .fetch_one(pool.get_ref())
        .await
        .map(|row| row.get::<i64, _>("cnt")) // butuh use sqlx::Row;
        .map_err(|e| {
            log::error!("Error counting PDP: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // ---------- DATA ----------
    let data_sql = format!(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.jabatan,
            p.tingkat_kepengurusan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) AS thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi  AS provinsi_domisili_nama,
            kd.nama_kabupaten AS kabupaten_domisili_nama,
            pp.nama_provinsi  AS provinsi_penugasan_nama,
            kp.nama_kabupaten AS kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         {where_clause}
         ORDER BY p.status ASC
         LIMIT ? OFFSET ?"
    );

    let mut data_query = sqlx::query_as::<_, EncryptedPdp>(&data_sql);

    // urutan bind: search -> filter pelaksana -> limit -> offset
    for b in &search_binds {
        data_query = data_query.bind(b);
    }
    for fb in &filter_binds {
        data_query = match fb {
            FB::S(s) => data_query.bind(s),
            FB::I(i) => data_query.bind(i),
        };
    }
    data_query = data_query.bind(limit as i64).bind(offset as i64);

    let encrypted_rows: Vec<EncryptedPdp> =
        data_query.fetch_all(pool.get_ref()).await.map_err(|e| {
            log::error!("Error fetching PDP data: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Dekripsi
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::error!("Gagal mendekripsi data PDP: {:?}", e);
                continue;
            }
        }
    }

    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };
    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + limit, total as u32);

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: decrypted_rows,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

fn build_links_pelaksana_kabupaten(
    base_path: &str,
    current: u32,
    total_pages: u32,
    per_page: u64,
    q: &Option<String>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Vec<PaginationLink> {
    let mut links = Vec::new();

    // Previous page
    if current > 1 {
        let mut query_params = format!("page={}&per_page={}", current - 1, per_page);
        if let Some(q) = q {
            if !q.is_empty() {
                query_params.push_str(&format!("&q={}", urlencoding::encode(q)));
            }
        }
        if let Some(prov_id) = id_provinsi {
            query_params.push_str(&format!("&id_provinsi={}", prov_id));
        }
        if let Some(kab_id) = id_kabupaten {
            query_params.push_str(&format!("&id_kabupaten={}", kab_id));
        }
        links.push(PaginationLink {
            url: Some(format!("{}?{}", base_path, query_params)),
            label: "« Previous".to_string(),
            active: false,
        });
    }

    // Numbered pages
    for p in 1..=total_pages {
        let mut query_params = format!("page={}&per_page={}", p, per_page);
        if let Some(q) = q {
            if !q.is_empty() {
                query_params.push_str(&format!("&q={}", urlencoding::encode(q)));
            }
        }
        if let Some(prov_id) = id_provinsi {
            query_params.push_str(&format!("&id_provinsi={}", prov_id));
        }
        if let Some(kab_id) = id_kabupaten {
            query_params.push_str(&format!("&id_kabupaten={}", kab_id));
        }

        links.push(PaginationLink {
            url: Some(format!("{}?{}", base_path, query_params)),
            label: p.to_string(),
            active: p == current,
        });
    }

    // Next page
    if current < total_pages {
        let mut query_params = format!("page={}&per_page={}", current + 1, per_page);
        if let Some(q) = q {
            if !q.is_empty() {
                query_params.push_str(&format!("&q={}", urlencoding::encode(q)));
            }
        }
        if let Some(prov_id) = id_provinsi {
            query_params.push_str(&format!("&id_provinsi={}", prov_id));
        }
        if let Some(kab_id) = id_kabupaten {
            query_params.push_str(&format!("&id_kabupaten={}", kab_id));
        }
        links.push(PaginationLink {
            url: Some(format!("{}?{}", base_path, query_params)),
            label: "Next »".to_string(),
            active: false,
        });
    }

    links
}

//SEMUA PELAKSANA
#[get("/api/adminpanel/pelaksana-provinsi-all")]
pub async fn get_all_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    // Params
    let q_trimmed = query.q.as_ref().map(|s| s.trim().to_string());
    let has_q = q_trimmed.as_ref().is_some_and(|s| !s.is_empty());

    // TAMBAHAN BARU: Filter provinsi
    let id_provinsi = query.id_provinsi;
    let has_provinsi_filter = id_provinsi.is_some();

    // ===== DATA =====
    let data: Vec<PelaksanaProvinsi> = if has_q && has_provinsi_filter {
        // CASE 1: Ada pencarian DAN filter provinsi
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE (pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?)
            AND pp.id_provinsi = ?
            ORDER BY pp.id ASC
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q {
        // CASE 2: Hanya ada pencarian
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?
            ORDER BY pp.id ASC

            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter {
        // CASE 3: Hanya ada filter provinsi
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.id_provinsi = ?
            ORDER BY pp.id ASC
            "#,
        )
        .bind(id_provinsi.unwrap())
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // CASE 4: Tidak ada filter
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            ORDER BY pp.id ASC
            "#,
        )
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    Ok(HttpResponse::Ok().json(data))
}

#[get("/api/adminpanel/pelaksana-kabupaten-all")]
pub async fn get_all_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQueryKabupaten>,
) -> Result<impl Responder, Error> {
    // Auth
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Administrator atau Pelaksana yang dapat mengakses",
        ));
    }

    let q_trimmed = query.q.as_ref().map(|s| s.trim().to_string());
    let has_q = q_trimmed.as_ref().is_some_and(|s| !s.is_empty());

    // TAMBAHAN BARU: Filter provinsi & kabupaten
    let id_provinsi = query.id_provinsi;
    let has_provinsi_filter = id_provinsi.is_some();
    let id_kabupaten = query.id_kabupaten;
    let has_kabupaten_filter = id_kabupaten.is_some();

    // DATA - Implementasi semua case yang sama dengan COUNT
    let data: Vec<PelaksanaKabupaten> = if has_q && has_provinsi_filter && has_kabupaten_filter {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_provinsi = ? AND pk.id_kabupaten = ?
            ORDER BY pk.id ASC
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .bind(id_kabupaten.unwrap())
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q && has_provinsi_filter {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_provinsi = ?
            ORDER BY pk.id ASC
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi.unwrap())
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q && has_kabupaten_filter {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?)
            AND pk.id_kabupaten = ?
            ORDER BY pk.id ASC
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_kabupaten.unwrap())
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter && has_kabupaten_filter {
        // CASE 4: Provinsi + kabupaten (tanpa pencarian)
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_provinsi = ? AND pk.id_kabupaten = ?
            ORDER BY pk.id ASC
            "#,
        )
        .bind(id_provinsi.unwrap())
        .bind(id_kabupaten.unwrap())
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_provinsi_filter {
        // CASE 5: Hanya provinsi (tanpa pencarian) - CASE BARU YANG DITAMBAHKAN
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_provinsi = ?
            ORDER BY pk.id ASC
            "#,
        )
        .bind(id_provinsi.unwrap())
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_kabupaten_filter {
        // CASE 6: Hanya kabupaten (tanpa pencarian)
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.id_kabupaten = ?
            ORDER BY pk.id ASC
            "#,
        )
        .bind(id_kabupaten.unwrap())
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ?
               OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?
            ORDER BY pk.id ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            ORDER BY pk.id ASC
            "#,
        )
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };
    Ok(HttpResponse::Ok().json(data))
}
