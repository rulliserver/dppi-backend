<<<<<<< HEAD
// src/controllers/majelis_pertimbangan.rs
use crate::{auth, models::majelis_pertimbangan::MajelisPertimbangan};
use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use futures_util::TryStreamExt as _;
use sqlx::{MySql, MySqlPool, QueryBuilder};

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[get("/api/adminpanel/majelis-pertimbangan")]
pub async fn get_all_majelis_pertimbangan(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let data = sqlx::query_as::<_, MajelisPertimbangan>(
        r#"
        SELECT id, id_pdp, nama_lengkap, photo, jabatan
        FROM majelis_pertimbangan
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(data))
}

#[post("/api/adminpanel/majelis-pertimbangan")]
pub async fn create_majelis_pertimbangan(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let mut nama_lengkap: Option<String> = None;
    let mut jabatan: Option<String> = None;
    let mut id_pdp_val: Option<i64> = None;
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
                    v.parse::<i64>().ok()
                };
            }

            "photo" => {
                photo_path =
                    Some(save_photo_field(field, "./uploads/assets/majelis-pertimbangan").await?);
            }
            _ => { /* ignore */ }
        }
    }

    let nama =
        nama_lengkap.ok_or_else(|| actix_web::error::ErrorBadRequest("nama_lengkap wajib"))?;

    sqlx::query(
        "INSERT INTO majelis_pertimbangan (id_pdp, nama_lengkap, photo, jabatan)
         VALUES (?, ?, ?, ?)",
    )
    .bind(id_pdp_val)
    .bind(&nama)
    .bind(&photo_path)
    .bind(&jabatan)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let created = sqlx::query_as::<_, MajelisPertimbangan>(
        "SELECT
            id,
            id_pdp,
            nama_lengkap,
            photo,
            jabatan
         FROM majelis_pertimbangan
        ",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(created))
}

#[put("/api/adminpanel/majelis-pertimbangan/{id}")]
pub async fn update_majelis_pertimbangan(
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
                photo_new_path =
                    Some(save_photo_field(field, "./uploads/assets/majelis-pertimbangan").await?);
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
        sqlx::query_as("SELECT photo FROM majelis_pertimbangan WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut qb: QueryBuilder<MySql> = QueryBuilder::new("UPDATE majelis_pertimbangan SET ");
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
    let updated = sqlx::query_as::<_, MajelisPertimbangan>(
        "SELECT id, id_pdp, nama_lengkap, photo, jabatan FROM majelis_pertimbangan WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/api/adminpanel/majelis-pertimbangan/{id}")]
pub async fn delete_majelis_pertimbangan(
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
        sqlx::query_as("SELECT photo FROM majelis_pertimbangan WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // Hapus row
    let result = sqlx::query("DELETE FROM majelis_pertimbangan WHERE id = ?")
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
        "majelis_pertimbangan_{}.{}",
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

#[get("/api/majelis-pertimbangan")]
pub async fn get_all_mp(
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let data = sqlx::query_as::<_, MajelisPertimbangan>(
        r#"
        SELECT id, id_pdp, nama_lengkap, photo, jabatan
        FROM majelis_pertimbangan
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(data))
}
=======
// src/controllers/majelis_pertimbangan.rs
use crate::{auth, models::majelis_pertimbangan::MajelisPertimbangan};
use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use futures_util::TryStreamExt as _;
use sqlx::{MySql, MySqlPool, QueryBuilder};

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[get("/api/adminpanel/majelis-pertimbangan")]
pub async fn get_all_majelis_pertimbangan(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let data = sqlx::query_as::<_, MajelisPertimbangan>(
        r#"
        SELECT id, id_pdp, nama_lengkap, photo, jabatan
        FROM majelis_pertimbangan
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(data))
}

#[post("/api/adminpanel/majelis-pertimbangan")]
pub async fn create_majelis_pertimbangan(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let mut nama_lengkap: Option<String> = None;
    let mut jabatan: Option<String> = None;
    let mut id_pdp_val: Option<i64> = None;
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
                    v.parse::<i64>().ok()
                };
            }

            "photo" => {
                photo_path =
                    Some(save_photo_field(field, "./uploads/assets/majelis-pertimbangan").await?);
            }
            _ => { /* ignore */ }
        }
    }

    let nama =
        nama_lengkap.ok_or_else(|| actix_web::error::ErrorBadRequest("nama_lengkap wajib"))?;

    sqlx::query(
        "INSERT INTO majelis_pertimbangan (id_pdp, nama_lengkap, photo, jabatan)
         VALUES (?, ?, ?, ?)",
    )
    .bind(id_pdp_val)
    .bind(&nama)
    .bind(&photo_path)
    .bind(&jabatan)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let created = sqlx::query_as::<_, MajelisPertimbangan>(
        "SELECT
            id,
            id_pdp,
            nama_lengkap,
            photo,
            jabatan
         FROM majelis_pertimbangan
        ",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(created))
}

#[put("/api/adminpanel/majelis-pertimbangan/{id}")]
pub async fn update_majelis_pertimbangan(
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
                photo_new_path =
                    Some(save_photo_field(field, "./uploads/assets/majelis-pertimbangan").await?);
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
        sqlx::query_as("SELECT photo FROM majelis_pertimbangan WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    let mut qb: QueryBuilder<MySql> = QueryBuilder::new("UPDATE majelis_pertimbangan SET ");
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
    let updated = sqlx::query_as::<_, MajelisPertimbangan>(
        "SELECT id, id_pdp, nama_lengkap, photo, jabatan FROM majelis_pertimbangan WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/api/adminpanel/majelis-pertimbangan/{id}")]
pub async fn delete_majelis_pertimbangan(
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
        sqlx::query_as("SELECT photo FROM majelis_pertimbangan WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // Hapus row
    let result = sqlx::query("DELETE FROM majelis_pertimbangan WHERE id = ?")
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
        "majelis_pertimbangan_{}.{}",
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

#[get("/api/majelis-pertimbangan")]
pub async fn get_all_mp(
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let data = sqlx::query_as::<_, MajelisPertimbangan>(
        r#"
        SELECT id, id_pdp, nama_lengkap, photo, jabatan
        FROM majelis_pertimbangan
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(data))
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
