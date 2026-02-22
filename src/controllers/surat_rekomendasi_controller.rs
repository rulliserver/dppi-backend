<<<<<<< HEAD
use std::{fs, path::Path};

// src/controllers/surat-rekomendasi.rs
use actix_web::{Error, HttpRequest, HttpResponse, Responder, post, web};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::auth;

#[derive(Debug, Deserialize)]
pub struct UploadSuratRequest {
    pub nomor_surat: String,
    pub keterangan: Option<String>,
    pub file_surat: String, // base64 encoded PDF
    pub filename: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuratResponse {
    pub id: i32,
    pub nomor_surat: String,
    pub keterangan: Option<String>,
    pub file_surat: String,
    pub file_path: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[post("/api/upload-surat-rekomendasi")]
pub async fn upload_surat_rekomendasi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    surat_data: web::Json<UploadSuratRequest>,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    // Validasi input
    if surat_data.nomor_surat.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Nomor surat tidak boleh kosong",
        ));
    }

    if surat_data.file_surat.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "File surat tidak boleh kosong",
        ));
    }

    // Decode base64 file
    let file_data = match BASE64_STANDARD.decode(&surat_data.file_surat) {
        Ok(data) => data,
        Err(_) => {
            return Err(actix_web::error::ErrorBadRequest(
                "Format base64 tidak valid",
            ));
        }
    };

    // Validasi ukuran file (max 5MB)
    const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
    if file_data.len() > MAX_FILE_SIZE {
        return Err(actix_web::error::ErrorBadRequest(
            "Ukuran file maksimal 5MB",
        ));
    }

    // Validasi tipe file (harus PDF)
    if file_data.len() < 4 || &file_data[0..4] != b"%PDF" {
        return Err(actix_web::error::ErrorBadRequest(
            "File harus berformat PDF",
        ));
    }

    // Buat direktori upload jika belum ada
    let upload_dir = "./uploads/surat-rekomendasi";
    if !Path::new(upload_dir).exists() {
        fs::create_dir_all(upload_dir).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Gagal membuat direktori: {}", e))
        })?;
    }

    // Generate nama file unik
    let file_extension = "pdf";
    let filename = surat_data
        .filename
        .as_deref()
        .and_then(|name| Path::new(name).file_name().and_then(|n| n.to_str()))
        .unwrap_or("surat_rekomendasi.pdf");

    let sanitized_filename = filename.replace(" ", "_").replace("/", "_");
    let unique_filename = format!(
        "{}_{}.{}",
        Uuid::new_v4().to_string(),
        sanitized_filename
            .trim_end_matches(".pdf")
            .trim_end_matches(".PDF"),
        file_extension
    );

    let file_path = Path::new(upload_dir).join(&unique_filename);
    let file_path_str = file_path.to_string_lossy().to_string();

    // Simpan file ke disk
    fs::write(&file_path, &file_data).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal menyimpan file: {}", e))
    })?;

    // Simpan data ke database
    let query = r#"
        INSERT INTO surat_rekomendasi (nomor_surat, keterangan, file_surat, file_path, created_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, NOW(), NOW())
    "#;

    let result = sqlx::query(query)
        .bind(&surat_data.nomor_surat)
        .bind(&surat_data.keterangan)
        .bind(&unique_filename)
        .bind(&file_path_str)
        .bind(claims.sub) // user_id dari token JWT
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Gagal menyimpan ke database: {}",
                e
            ))
        })?;

    let inserted_id = result.last_insert_id() as i32;

    // Ambil data yang baru saja diinsert
    let inserted_surat = sqlx::query_as!(
        SuratResponse,
        r#"
        SELECT id, nomor_surat, keterangan, file_surat, file_path, created_at, updated_at
        FROM surat_rekomendasi
        WHERE id = ?
        "#,
        inserted_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal mengambil data: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Surat rekomendasi berhasil diupload",
        "data": {
            "id": inserted_surat.id,
            "nomor_surat": inserted_surat.nomor_surat,
            "keterangan": inserted_surat.keterangan,
            "file_name": inserted_surat.file_surat,
            "file_path": inserted_surat.file_path,
            "created_at": inserted_surat.created_at
        }
    })))
}

// Endpoint untuk mendapatkan daftar surat rekomendasi
#[post("/api/surat-rekomendasi/list")]
pub async fn get_surat_rekomendasi_list(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "User"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses",
        ));
    }

    let surat_list = sqlx::query_as!(
        SuratResponse,
        r#"
        SELECT id, nomor_surat, keterangan, file_surat, file_path, created_at, updated_at
        FROM surat_rekomendasi
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal mengambil data: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": surat_list
    })))
}

// Endpoint untuk menghapus surat rekomendasi
#[post("/api/surat-rekomendasi/delete/{id}")]
pub async fn delete_surat_rekomendasi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    surat_id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat menghapus",
        ));
    }

    // Ambil info file sebelum delete
    let surat = sqlx::query!(
        r#"
        SELECT file_path, file_surat
        FROM surat_rekomendasi
        WHERE id = ?
        "#,
        *surat_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal mengambil data: {}", e))
    })?;

    if let Some(surat_data) = surat {
        // Hapus file dari disk
        if let Err(e) = fs::remove_file(&surat_data.file_path) {
            eprintln!("Gagal menghapus file: {}", e);
            // Tetap lanjutkan untuk hapus dari database meskipun file sudah tidak ada
        }

        // Hapus dari database
        sqlx::query!(
            r#"
            DELETE FROM surat_rekomendasi
            WHERE id = ?
            "#,
            *surat_id
        )
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Gagal menghapus data: {}", e))
        })?;

        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Surat rekomendasi berhasil dihapus"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Surat rekomendasi tidak ditemukan"
        })))
    }
}

// Endpoint untuk mendownload surat rekomendasi
#[post("/api/surat-rekomendasi/download/{id}")]
pub async fn download_surat_rekomendasi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    surat_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "User"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses",
        ));
    }

    // Ambil data surat
    let surat = sqlx::query!(
        r#"
        SELECT file_path, nomor_surat, file_surat
        FROM surat_rekomendasi
        WHERE id = ?
        "#,
        *surat_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal mengambil data: {}", e))
    })?;

    if let Some(surat_data) = surat {
        // Baca file
        let file_content = fs::read(&surat_data.file_path).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Gagal membaca file: {}", e))
        })?;

        // Buat nama file untuk download
        let download_filename = format!(
            "{}_{}.pdf",
            surat_data.nomor_surat.replace("/", "_").replace(" ", "_"),
            surat_data.file_surat
        );

        Ok(HttpResponse::Ok()
            .content_type("application/pdf")
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", download_filename),
            ))
            .body(file_content))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Surat rekomendasi tidak ditemukan"
        })))
    }
}
=======
use std::{fs, path::Path};

// src/controllers/surat-rekomendasi.rs
use actix_web::{Error, HttpRequest, HttpResponse, Responder, post, web};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::auth;

#[derive(Debug, Deserialize)]
pub struct UploadSuratRequest {
    pub nomor_surat: String,
    pub keterangan: Option<String>,
    pub file_surat: String, // base64 encoded PDF
    pub filename: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuratResponse {
    pub id: i32,
    pub nomor_surat: String,
    pub keterangan: Option<String>,
    pub file_surat: String,
    pub file_path: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[post("/api/upload-surat-rekomendasi")]
pub async fn upload_surat_rekomendasi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    surat_data: web::Json<UploadSuratRequest>,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    // Validasi input
    if surat_data.nomor_surat.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Nomor surat tidak boleh kosong",
        ));
    }

    if surat_data.file_surat.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "File surat tidak boleh kosong",
        ));
    }

    // Decode base64 file
    let file_data = match BASE64_STANDARD.decode(&surat_data.file_surat) {
        Ok(data) => data,
        Err(_) => {
            return Err(actix_web::error::ErrorBadRequest(
                "Format base64 tidak valid",
            ));
        }
    };

    // Validasi ukuran file (max 5MB)
    const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
    if file_data.len() > MAX_FILE_SIZE {
        return Err(actix_web::error::ErrorBadRequest(
            "Ukuran file maksimal 5MB",
        ));
    }

    // Validasi tipe file (harus PDF)
    if file_data.len() < 4 || &file_data[0..4] != b"%PDF" {
        return Err(actix_web::error::ErrorBadRequest(
            "File harus berformat PDF",
        ));
    }

    // Buat direktori upload jika belum ada
    let upload_dir = "./uploads/surat-rekomendasi";
    if !Path::new(upload_dir).exists() {
        fs::create_dir_all(upload_dir).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Gagal membuat direktori: {}", e))
        })?;
    }

    // Generate nama file unik
    let file_extension = "pdf";
    let filename = surat_data
        .filename
        .as_deref()
        .and_then(|name| Path::new(name).file_name().and_then(|n| n.to_str()))
        .unwrap_or("surat_rekomendasi.pdf");

    let sanitized_filename = filename.replace(" ", "_").replace("/", "_");
    let unique_filename = format!(
        "{}_{}.{}",
        Uuid::new_v4().to_string(),
        sanitized_filename
            .trim_end_matches(".pdf")
            .trim_end_matches(".PDF"),
        file_extension
    );

    let file_path = Path::new(upload_dir).join(&unique_filename);
    let file_path_str = file_path.to_string_lossy().to_string();

    // Simpan file ke disk
    fs::write(&file_path, &file_data).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal menyimpan file: {}", e))
    })?;

    // Simpan data ke database
    let query = r#"
        INSERT INTO surat_rekomendasi (nomor_surat, keterangan, file_surat, file_path, created_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, NOW(), NOW())
    "#;

    let result = sqlx::query(query)
        .bind(&surat_data.nomor_surat)
        .bind(&surat_data.keterangan)
        .bind(&unique_filename)
        .bind(&file_path_str)
        .bind(claims.sub) // user_id dari token JWT
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Gagal menyimpan ke database: {}",
                e
            ))
        })?;

    let inserted_id = result.last_insert_id() as i32;

    // Ambil data yang baru saja diinsert
    let inserted_surat = sqlx::query_as!(
        SuratResponse,
        r#"
        SELECT id, nomor_surat, keterangan, file_surat, file_path, created_at, updated_at
        FROM surat_rekomendasi
        WHERE id = ?
        "#,
        inserted_id
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal mengambil data: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Surat rekomendasi berhasil diupload",
        "data": {
            "id": inserted_surat.id,
            "nomor_surat": inserted_surat.nomor_surat,
            "keterangan": inserted_surat.keterangan,
            "file_name": inserted_surat.file_surat,
            "file_path": inserted_surat.file_path,
            "created_at": inserted_surat.created_at
        }
    })))
}

// Endpoint untuk mendapatkan daftar surat rekomendasi
#[post("/api/surat-rekomendasi/list")]
pub async fn get_surat_rekomendasi_list(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "User"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses",
        ));
    }

    let surat_list = sqlx::query_as!(
        SuratResponse,
        r#"
        SELECT id, nomor_surat, keterangan, file_surat, file_path, created_at, updated_at
        FROM surat_rekomendasi
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal mengambil data: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": surat_list
    })))
}

// Endpoint untuk menghapus surat rekomendasi
#[post("/api/surat-rekomendasi/delete/{id}")]
pub async fn delete_surat_rekomendasi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    surat_id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat menghapus",
        ));
    }

    // Ambil info file sebelum delete
    let surat = sqlx::query!(
        r#"
        SELECT file_path, file_surat
        FROM surat_rekomendasi
        WHERE id = ?
        "#,
        *surat_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal mengambil data: {}", e))
    })?;

    if let Some(surat_data) = surat {
        // Hapus file dari disk
        if let Err(e) = fs::remove_file(&surat_data.file_path) {
            eprintln!("Gagal menghapus file: {}", e);
            // Tetap lanjutkan untuk hapus dari database meskipun file sudah tidak ada
        }

        // Hapus dari database
        sqlx::query!(
            r#"
            DELETE FROM surat_rekomendasi
            WHERE id = ?
            "#,
            *surat_id
        )
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Gagal menghapus data: {}", e))
        })?;

        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Surat rekomendasi berhasil dihapus"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Surat rekomendasi tidak ditemukan"
        })))
    }
}

// Endpoint untuk mendownload surat rekomendasi
#[post("/api/surat-rekomendasi/download/{id}")]
pub async fn download_surat_rekomendasi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    surat_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "User"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses",
        ));
    }

    // Ambil data surat
    let surat = sqlx::query!(
        r#"
        SELECT file_path, nomor_surat, file_surat
        FROM surat_rekomendasi
        WHERE id = ?
        "#,
        *surat_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Gagal mengambil data: {}", e))
    })?;

    if let Some(surat_data) = surat {
        // Baca file
        let file_content = fs::read(&surat_data.file_path).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Gagal membaca file: {}", e))
        })?;

        // Buat nama file untuk download
        let download_filename = format!(
            "{}_{}.pdf",
            surat_data.nomor_surat.replace("/", "_").replace(" ", "_"),
            surat_data.file_surat
        );

        Ok(HttpResponse::Ok()
            .content_type("application/pdf")
            .append_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", download_filename),
            ))
            .body(file_content))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Surat rekomendasi tidak ditemukan"
        })))
    }
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
