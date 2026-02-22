<<<<<<< HEAD
use crate::auth;
use crate::utils::{GALLERY_DIR, delete_gallery_images_all, ensure_gallery_dir, save_gallery_images};
use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Result, delete, error, post, put, web};
use bytes::BytesMut;
use chrono::NaiveDate;
use futures::TryStreamExt;
use futures_util::StreamExt;
use sqlx::MySqlPool;
use sqlx::types::Json;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

#[post("/api/adminpanel/galeri-foto")]
pub async fn create_gallery(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    ensure_gallery_dir().map_err(|e| error::ErrorInternalServerError(e))?;

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let (mut kegiatan, mut keterangan, mut status,
        mut tanggal, mut created_by, mut updated_by) = (
        String::new(),
        None,
        String::new(),
        None,
        String::new(),
        String::new(),
    );
    let mut saved_files: Vec<String> = Vec::new();

    // Process multipart form data
    while let Some(field) = payload.try_next().await.map_err(error::ErrorBadRequest)? {
        let name = field.name().unwrap_or_default().to_string();

        // Handle file uploads - perhatikan pola nama field dari frontend
        if name.starts_with("foto[") || name == "foto" {
            // Process image files
            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_default();

            if !content_type.starts_with("image/") {
                return Err(error::ErrorBadRequest("File bukan image"));
            }

            let cd = field.content_disposition().cloned();
            let orig = cd
                .and_then(|d| d.get_filename().map(|s| s.to_string()))
                .unwrap_or_else(|| "image".to_string());

            let ext = Path::new(&orig)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpg");

            let filename = format!("{}.{}", Uuid::new_v4(), ext);
            let safe = sanitize_filename::sanitize(&filename);
            let filepath = Path::new(GALLERY_DIR).join(&safe);

            let mut f = std::fs::File::create(&filepath)
                .map_err(|e| error::ErrorInternalServerError(format!("Create file: {}", e)))?;

            // Write file chunks
            let mut field_stream = field;
            while let Some(chunk) = field_stream
                .try_next()
                .await
                .map_err(error::ErrorBadRequest)?
            {
                f.write_all(&chunk)
                    .map_err(|e| error::ErrorInternalServerError(format!("Write file: {}", e)))?;
            }

            saved_files.push(safe);
        } else {
            // Process text fields
            let mut data = BytesMut::new();
            let mut field_stream = field;

            while let Some(chunk) = field_stream.next().await {
                let chunk = chunk.map_err(error::ErrorBadRequest)?;
                data.extend_from_slice(&chunk);
            }

            let val = String::from_utf8_lossy(&data).trim().to_string();

            match name.as_str() {
                "kegiatan" => kegiatan = val,
                "keterangan" => {
                    if !val.is_empty() {
                        keterangan = Some(val);
                    }
                }
                "status" => status = val,
                "tanggal" => {
                    if !val.is_empty() {
                        tanggal = NaiveDate::parse_from_str(&val, "%Y-%m-%d").ok();
                    }
                }
                "created_by" => created_by = val,
                "updated_by" => updated_by = val,
                _ => {
                    // Debug: log unexpected fields
                    eprintln!("Unexpected field: {} = {}", name, val);
                }
            }
        }
    }

    // Validation
    if kegiatan.trim().is_empty() {
        return Err(error::ErrorBadRequest("kegiatan wajib diisi"));
    }

    if saved_files.is_empty() {
        return Err(error::ErrorBadRequest("Minimal satu foto harus diupload"));
    }

    // Set default values if empty
    if status.is_empty() {
        status = "Tayang".to_string();
    }
    if created_by.is_empty() {
        created_by = claims.nama_user.clone();
    }
    if updated_by.is_empty() {
        updated_by = claims.nama_user.clone();
    }

    // Insert into database
    let rec = sqlx::query!(
        r#"
        INSERT INTO galleries (kegiatan, foto, keterangan, tanggal, status, created_at, updated_at, created_by, updated_by)
        VALUES (?, ?, ?, ?, ?, NOW(), NOW(), ?, ?)
        "#,
        kegiatan,
        Json(saved_files), // Store files as JSON array
        keterangan,
        tanggal,
        status,
        created_by,
        updated_by
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        error::ErrorInternalServerError("Gagal menyimpan data ke database")
    })?;

    let new_id = rec.last_insert_id();

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Galeri foto berhasil dibuat",
        "id": new_id
    })))
}

/// UPDATE metadata
#[put("/api/adminpanel/galeri-foto/ubah/{id}")]
pub async fn update_gallery_meta_spoof(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
    mut multipart: Multipart,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    // Baca multipart: form fields
    let mut kegiatan: Option<String> = None;
    let mut tanggal: Option<NaiveDate> = None;
    let mut keterangan: Option<String> = None;

    while let Some(mut field) = multipart
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().cloned();
        let field_name = cd
            .as_ref()
            .and_then(|c| c.get_name())
            .unwrap_or_default()
            .to_string();

        match field_name.as_str() {
            "kegiatan" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                kegiatan = Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                    actix_web::error::ErrorBadRequest("kegiatan bukan UTF-8 valid")
                })?);
            }

            "tanggal" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                let s = String::from_utf8(bytes.to_vec())
                    .map_err(|_| actix_web::error::ErrorBadRequest("tanggal bukan UTF-8 valid"))?;
                tanggal = Some(
                    NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                        .map_err(|_| actix_web::error::ErrorBadRequest("format tanggal salah"))?,
                );
            }

            "keterangan" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                keterangan =
                    Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                        actix_web::error::ErrorBadRequest("address bukan UTF-8 valid")
                    })?);
            }

            _ => {
                // drain unknown field
                while let Some(_chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {}
            }
        }
    }

    // Validasi required fields
    let kegiatan =
        kegiatan.ok_or_else(|| actix_web::error::ErrorBadRequest("Field kegiatan wajib diisi"))?;
    let tanggal =
        tanggal.ok_or_else(|| actix_web::error::ErrorBadRequest("Field tanggal wajib diisi"))?;

    // Validasi field tidak boleh kosong
    if kegiatan.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Nama kegiatan tidak boleh kosong",
        ));
    }

    // Update Galery
    let result =
        sqlx::query("UPDATE galleries SET kegiatan = ?, tanggal = ?, keterangan = ? WHERE id = ?")
            .bind(&kegiatan)
            .bind(&tanggal)
            .bind(&keterangan)
            .bind(id)
            .execute(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Galeri Kegiatan tidak ditemukan",
        ));
    }

    #[derive(serde::Serialize)]
    struct UpdateGaleriResponse {
        message: String,
        updated: bool,
    }

    Ok(HttpResponse::Ok().json(UpdateGaleriResponse {
        message: "Galeri berhasil diperbarui".into(),
        updated: true,
    }))
}

/// APPEND foto[] (multipart)
#[put("/api/adminpanel/galeri-foto/update/{id}")]
pub async fn append_gallery_photos(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
    req: actix_web::HttpRequest,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    // simpan file fisik
    let mut new_files = save_gallery_images(payload).await?;
    if new_files.is_empty() {
        return Err(error::ErrorBadRequest("Tidak ada file foto[] yang dikirim"));
    }

    // ambil list lama
    let mut current: Option<Json<Vec<String>>> =
        sqlx::query_scalar("SELECT foto FROM galleries WHERE id = ?")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?;

    let mut merged: Vec<String> = current.take().map(|Json(v)| v).unwrap_or_default();
    merged.append(&mut new_files);

    sqlx::query("UPDATE galleries SET foto = ?, updated_at = NOW() WHERE id = ?")
        .bind(Json(merged) as Json<Vec<String>>)
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message":"Foto ditambahkan"})))
}

/// HAPUS 1 foto ///
#[delete("/api/adminpanel/galeri-foto/delete/{id}/{filename}")]
pub async fn delete_one_photo(
    pool: web::Data<MySqlPool>,
    path: web::Path<(u64, String)>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    let (id, filename_raw) = path.into_inner();
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    // Decode percent-encoding -> Cow<str> -> String
    let filename: String = match urlencoding::decode(&filename_raw) {
        Ok(cow) => cow.into_owned(),
        Err(_) => filename_raw, // fallback kalau decode gagal
    };

    // Hardening: tolak nama file yang mencurigakan
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(error::ErrorBadRequest("Nama file tidak valid"));
    }

    // Ambil array foto yang ada
    let Json(mut arr): Json<Vec<String>> =
        sqlx::query_scalar("SELECT foto FROM galleries WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?;

    // Filter keluar file yang dihapus
    let old_len = arr.len();
    arr.retain(|x| x != &filename);
    if arr.len() == old_len {
        return Err(error::ErrorNotFound("Foto tidak ditemukan di galeri"));
    }

    // Update DB
    sqlx::query("UPDATE galleries SET foto = ?, updated_at = NOW() WHERE id = ?")
        .bind(Json(arr) as Json<Vec<String>>)
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    // Hapus file fisik (abaikan error kalau file sudah tidak ada)
    let _ = crate::utils::delete_gallery_image(&filename);

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Foto dihapus" })))
}

/// DELETE seluruh galeri
#[delete("/api/adminpanel/galeri-foto/{id}")]
pub async fn delete_gallery(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    // Ambil daftar file untuk dihapus
    let maybe_fotos: Option<Json<Vec<String>>> =
        sqlx::query_scalar("SELECT foto FROM galleries WHERE id = ?")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?;

    // Delete row
    let res = sqlx::query("DELETE FROM galleries WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    if res.rows_affected() == 0 {
        return Err(error::ErrorNotFound("Galeri tidak ditemukan"));
    }

    if let Some(Json(files)) = maybe_fotos {
        delete_gallery_images_all(&files);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"message":"Galeri dihapus"})))
}
=======
use crate::auth;
use crate::utils::{GALLERY_DIR, delete_gallery_images_all, ensure_gallery_dir, save_gallery_images};
use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Result, delete, error, post, put, web};
use bytes::BytesMut;
use chrono::NaiveDate;
use futures::TryStreamExt;
use futures_util::StreamExt;
use sqlx::MySqlPool;
use sqlx::types::Json;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

#[post("/api/adminpanel/galeri-foto")]
pub async fn create_gallery(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    ensure_gallery_dir().map_err(|e| error::ErrorInternalServerError(e))?;

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let (mut kegiatan, mut keterangan, mut status,
        mut tanggal, mut created_by, mut updated_by) = (
        String::new(),
        None,
        String::new(),
        None,
        String::new(),
        String::new(),
    );
    let mut saved_files: Vec<String> = Vec::new();

    // Process multipart form data
    while let Some(field) = payload.try_next().await.map_err(error::ErrorBadRequest)? {
        let name = field.name().unwrap_or_default().to_string();

        // Handle file uploads - perhatikan pola nama field dari frontend
        if name.starts_with("foto[") || name == "foto" {
            // Process image files
            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_default();

            if !content_type.starts_with("image/") {
                return Err(error::ErrorBadRequest("File bukan image"));
            }

            let cd = field.content_disposition().cloned();
            let orig = cd
                .and_then(|d| d.get_filename().map(|s| s.to_string()))
                .unwrap_or_else(|| "image".to_string());

            let ext = Path::new(&orig)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("jpg");

            let filename = format!("{}.{}", Uuid::new_v4(), ext);
            let safe = sanitize_filename::sanitize(&filename);
            let filepath = Path::new(GALLERY_DIR).join(&safe);

            let mut f = std::fs::File::create(&filepath)
                .map_err(|e| error::ErrorInternalServerError(format!("Create file: {}", e)))?;

            // Write file chunks
            let mut field_stream = field;
            while let Some(chunk) = field_stream
                .try_next()
                .await
                .map_err(error::ErrorBadRequest)?
            {
                f.write_all(&chunk)
                    .map_err(|e| error::ErrorInternalServerError(format!("Write file: {}", e)))?;
            }

            saved_files.push(safe);
        } else {
            // Process text fields
            let mut data = BytesMut::new();
            let mut field_stream = field;

            while let Some(chunk) = field_stream.next().await {
                let chunk = chunk.map_err(error::ErrorBadRequest)?;
                data.extend_from_slice(&chunk);
            }

            let val = String::from_utf8_lossy(&data).trim().to_string();

            match name.as_str() {
                "kegiatan" => kegiatan = val,
                "keterangan" => {
                    if !val.is_empty() {
                        keterangan = Some(val);
                    }
                }
                "status" => status = val,
                "tanggal" => {
                    if !val.is_empty() {
                        tanggal = NaiveDate::parse_from_str(&val, "%Y-%m-%d").ok();
                    }
                }
                "created_by" => created_by = val,
                "updated_by" => updated_by = val,
                _ => {
                    // Debug: log unexpected fields
                    eprintln!("Unexpected field: {} = {}", name, val);
                }
            }
        }
    }

    // Validation
    if kegiatan.trim().is_empty() {
        return Err(error::ErrorBadRequest("kegiatan wajib diisi"));
    }

    if saved_files.is_empty() {
        return Err(error::ErrorBadRequest("Minimal satu foto harus diupload"));
    }

    // Set default values if empty
    if status.is_empty() {
        status = "Tayang".to_string();
    }
    if created_by.is_empty() {
        created_by = claims.nama_user.clone();
    }
    if updated_by.is_empty() {
        updated_by = claims.nama_user.clone();
    }

    // Insert into database
    let rec = sqlx::query!(
        r#"
        INSERT INTO galleries (kegiatan, foto, keterangan, tanggal, status, created_at, updated_at, created_by, updated_by)
        VALUES (?, ?, ?, ?, ?, NOW(), NOW(), ?, ?)
        "#,
        kegiatan,
        Json(saved_files), // Store files as JSON array
        keterangan,
        tanggal,
        status,
        created_by,
        updated_by
    )
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Database error: {}", e);
        error::ErrorInternalServerError("Gagal menyimpan data ke database")
    })?;

    let new_id = rec.last_insert_id();

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Galeri foto berhasil dibuat",
        "id": new_id
    })))
}

/// UPDATE metadata
#[put("/api/adminpanel/galeri-foto/ubah/{id}")]
pub async fn update_gallery_meta_spoof(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
    mut multipart: Multipart,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    // Baca multipart: form fields
    let mut kegiatan: Option<String> = None;
    let mut tanggal: Option<NaiveDate> = None;
    let mut keterangan: Option<String> = None;

    while let Some(mut field) = multipart
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().cloned();
        let field_name = cd
            .as_ref()
            .and_then(|c| c.get_name())
            .unwrap_or_default()
            .to_string();

        match field_name.as_str() {
            "kegiatan" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                kegiatan = Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                    actix_web::error::ErrorBadRequest("kegiatan bukan UTF-8 valid")
                })?);
            }

            "tanggal" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                let s = String::from_utf8(bytes.to_vec())
                    .map_err(|_| actix_web::error::ErrorBadRequest("tanggal bukan UTF-8 valid"))?;
                tanggal = Some(
                    NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                        .map_err(|_| actix_web::error::ErrorBadRequest("format tanggal salah"))?,
                );
            }

            "keterangan" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                keterangan =
                    Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                        actix_web::error::ErrorBadRequest("address bukan UTF-8 valid")
                    })?);
            }

            _ => {
                // drain unknown field
                while let Some(_chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {}
            }
        }
    }

    // Validasi required fields
    let kegiatan =
        kegiatan.ok_or_else(|| actix_web::error::ErrorBadRequest("Field kegiatan wajib diisi"))?;
    let tanggal =
        tanggal.ok_or_else(|| actix_web::error::ErrorBadRequest("Field tanggal wajib diisi"))?;

    // Validasi field tidak boleh kosong
    if kegiatan.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Nama kegiatan tidak boleh kosong",
        ));
    }

    // Update Galery
    let result =
        sqlx::query("UPDATE galleries SET kegiatan = ?, tanggal = ?, keterangan = ? WHERE id = ?")
            .bind(&kegiatan)
            .bind(&tanggal)
            .bind(&keterangan)
            .bind(id)
            .execute(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Galeri Kegiatan tidak ditemukan",
        ));
    }

    #[derive(serde::Serialize)]
    struct UpdateGaleriResponse {
        message: String,
        updated: bool,
    }

    Ok(HttpResponse::Ok().json(UpdateGaleriResponse {
        message: "Galeri berhasil diperbarui".into(),
        updated: true,
    }))
}

/// APPEND foto[] (multipart)
#[put("/api/adminpanel/galeri-foto/update/{id}")]
pub async fn append_gallery_photos(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
    req: actix_web::HttpRequest,
    payload: Multipart,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    // simpan file fisik
    let mut new_files = save_gallery_images(payload).await?;
    if new_files.is_empty() {
        return Err(error::ErrorBadRequest("Tidak ada file foto[] yang dikirim"));
    }

    // ambil list lama
    let mut current: Option<Json<Vec<String>>> =
        sqlx::query_scalar("SELECT foto FROM galleries WHERE id = ?")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?;

    let mut merged: Vec<String> = current.take().map(|Json(v)| v).unwrap_or_default();
    merged.append(&mut new_files);

    sqlx::query("UPDATE galleries SET foto = ?, updated_at = NOW() WHERE id = ?")
        .bind(Json(merged) as Json<Vec<String>>)
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message":"Foto ditambahkan"})))
}

/// HAPUS 1 foto ///
#[delete("/api/adminpanel/galeri-foto/delete/{id}/{filename}")]
pub async fn delete_one_photo(
    pool: web::Data<MySqlPool>,
    path: web::Path<(u64, String)>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    let (id, filename_raw) = path.into_inner();
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    // Decode percent-encoding -> Cow<str> -> String
    let filename: String = match urlencoding::decode(&filename_raw) {
        Ok(cow) => cow.into_owned(),
        Err(_) => filename_raw, // fallback kalau decode gagal
    };

    // Hardening: tolak nama file yang mencurigakan
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(error::ErrorBadRequest("Nama file tidak valid"));
    }

    // Ambil array foto yang ada
    let Json(mut arr): Json<Vec<String>> =
        sqlx::query_scalar("SELECT foto FROM galleries WHERE id = ?")
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?;

    // Filter keluar file yang dihapus
    let old_len = arr.len();
    arr.retain(|x| x != &filename);
    if arr.len() == old_len {
        return Err(error::ErrorNotFound("Foto tidak ditemukan di galeri"));
    }

    // Update DB
    sqlx::query("UPDATE galleries SET foto = ?, updated_at = NOW() WHERE id = ?")
        .bind(Json(arr) as Json<Vec<String>>)
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    // Hapus file fisik (abaikan error kalau file sudah tidak ada)
    let _ = crate::utils::delete_gallery_image(&filename);

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Foto dihapus" })))
}

/// DELETE seluruh galeri
#[delete("/api/adminpanel/galeri-foto/{id}")]
pub async fn delete_gallery(
    pool: web::Data<MySqlPool>,
    path: web::Path<u64>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    // Ambil daftar file untuk dihapus
    let maybe_fotos: Option<Json<Vec<String>>> =
        sqlx::query_scalar("SELECT foto FROM galleries WHERE id = ?")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(|e| error::ErrorInternalServerError(e))?;

    // Delete row
    let res = sqlx::query("DELETE FROM galleries WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    if res.rows_affected() == 0 {
        return Err(error::ErrorNotFound("Galeri tidak ditemukan"));
    }

    if let Some(Json(files)) = maybe_fotos {
        delete_gallery_images_all(&files);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"message":"Galeri dihapus"})))
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
