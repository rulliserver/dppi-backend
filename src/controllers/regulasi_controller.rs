use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Result, delete, post, put, web};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::auth;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Regulasi {
    pub id: i32,
    pub nama_regulasi: String,
    pub icon_regulasi: String,   // images
    pub file_regulasi: String,   // pdf
    pub created_by: String,         // dari claims.user_id
    pub updated_by: Option<String>, // dari claims.user_id
}

// Response struct
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

// Fungsi untuk menyimpan file icon regulasi
async fn save_icon_regulasi_file(
    mut field: actix_multipart::Field,
    dir: &std::path::Path,
) -> Result<String, Error> {
    if !dir.exists() {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    // Dapatkan informasi dari content-disposition header
    let content_disposition = field.content_disposition();
    let filename = content_disposition
        .and_then(|cd| cd.get_filename())
        .map(|s| s.to_string());

    // Dapatkan ekstensi dari nama file atau gunakan default
    let ext = if let Some(ref filename) = filename {
        Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{}", ext.to_ascii_lowercase()))
            .unwrap_or_else(|| ".png".to_string())
    } else {
        // Jika tidak ada nama file, gunakan ekstensi berdasarkan content-type
        let content_type = field.content_type();
        match content_type {
            Some(ct) if ct == &mime::IMAGE_JPEG => ".jpg",
            Some(ct) if ct == &mime::IMAGE_PNG => ".png",
            Some(ct) if ct == &mime::IMAGE_GIF => ".gif",
            Some(ct) if ct.type_() == mime::IMAGE => match ct.subtype().as_str() {
                "webp" => ".webp",
                "svg+xml" => ".svg",
                _ => ".png",
            },
            _ => ".png",
        }
        .to_string()
    };

    // Validasi bahwa file adalah image
    let allowed_extensions = [".jpg", ".jpeg", ".png", ".gif", ".webp", ".svg"];
    if !allowed_extensions
        .iter()
        .any(|&e| e == ext.to_ascii_lowercase())
    {
        return Err(actix_web::error::ErrorBadRequest(
            "File icon harus berupa gambar (jpg, png, gif, webp, svg)",
        ));
    }

    let filename = format!("icon_regulasi_{}{}", Uuid::new_v4(), ext);
    let filepath = dir.join(&filename);

    let mut f = tokio::fs::File::create(&filepath)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    while let Some(chunk) = field
        .try_next()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        f.write_all(&chunk)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    Ok(format!("uploads/assets/images/regulasi/{}", filename))
}

// Fungsi untuk menyimpan file regulasi (PDF)
async fn save_file_regulasi_pdf(
    mut field: actix_multipart::Field,
    dir: &std::path::Path,
) -> Result<String, Error> {
    if !dir.exists() {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    // Dapatkan informasi dari content-disposition header
    let content_disposition = field.content_disposition();
    let original_filename = content_disposition
        .and_then(|cd| cd.get_filename())
        .map(|s| s.to_string());

    // Validasi bahwa file adalah PDF berdasarkan content-type
    let content_type = field.content_type();
    if content_type != Some(&mime::APPLICATION_PDF) {
        return Err(actix_web::error::ErrorBadRequest("File harus berupa PDF"));
    }

    // Validasi berdasarkan ekstensi filename (jika ada)
    if let Some(ref filename) = original_filename {
        let path = Path::new(filename);
        if let Some(ext) = path.extension() {
            if ext.to_ascii_lowercase() != "pdf" {
                return Err(actix_web::error::ErrorBadRequest(
                    "File harus memiliki ekstensi .pdf",
                ));
            }
        }
    }

    let filename = format!("file_regulasi_{}.pdf", Uuid::new_v4());
    let filepath = dir.join(&filename);

    let mut f = tokio::fs::File::create(&filepath)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    while let Some(chunk) = field
        .try_next()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        f.write_all(&chunk)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    Ok(format!("uploads/assets/file/regulasi/{}", filename))
}

// POST - Create new regulasi dengan file upload
#[post("/api/adminpanel/regulasi")]
pub async fn create_regulasi(
    req: HttpRequest,
    mut payload: Multipart,
    pool: web::Data<sqlx::MySqlPool>,
) -> Result<HttpResponse> {
    // Verify JWT dan get claims
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let mut nama_regulasi = String::new();
    let mut icon_regulasi_path = String::new();
    let mut file_regulasi_path = String::new();

    // Process multipart form
    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "nama_regulasi" => {
                // Handle text field
                let mut data = Vec::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    data.extend_from_slice(&chunk);
                }
                nama_regulasi = String::from_utf8(data).map_err(|_| {
                    actix_web::error::ErrorBadRequest("Invalid UTF-8 in nama_regulasi")
                })?;
            }
            "icon_regulasi" => {
                // Handle icon file upload
                let icon_dir = Path::new("uploads/assets/images/regulasi");
                icon_regulasi_path = save_icon_regulasi_file(field, icon_dir).await?;
            }
            "file_regulasi" => {
                // Handle PDF file upload
                let pdf_dir = Path::new("uploads/assets/file/regulasi");
                file_regulasi_path = save_file_regulasi_pdf(field, pdf_dir).await?;
            }
            _ => {
                // Skip unknown fields
                continue;
            }
        }
    }

    // Validasi required fields
    if nama_regulasi.is_empty() {
        return Ok(HttpResponse::BadRequest()
            .json(ApiResponse::<Regulasi>::error("Nama regulasi harus diisi")));
    }
    if icon_regulasi_path.is_empty() {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<Regulasi>::error(
                "Icon regulasi harus diupload",
            )),
        );
    }
    if file_regulasi_path.is_empty() {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<Regulasi>::error(
                "File regulasi (PDF) harus diupload",
            )),
        );
    }

    let query = r#"
        INSERT INTO regulasi (nama_regulasi, icon_regulasi, file_regulasi, created_by, updated_by)
        VALUES (?, ?, ?, ?, ?)
    "#;

    let result = sqlx::query(query)
        .bind(&nama_regulasi)
        .bind(&icon_regulasi_path)
        .bind(&file_regulasi_path)
        .bind(claims.user_id.clone())
        .bind(claims.user_id.clone())
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            // Get the inserted regulasi
            let regulasi_query = r#"
                SELECT id, nama_regulasi, icon_regulasi, file_regulasi, created_by, updated_by
                FROM regulasi
                WHERE id = ?
            "#;

            let regulasi = sqlx::query_as::<_, Regulasi>(regulasi_query)
                .bind(result.last_insert_id() as i32)
                .fetch_one(pool.get_ref())
                .await;

            match regulasi {
                Ok(regulasi) => Ok(HttpResponse::Created()
                    .json(ApiResponse::success(regulasi, "Regulasi berhasil dibuat"))),
                Err(e) => {
                    log::error!("Failed to fetch created regulasi: {}", e);
                    // Clean up uploaded files jika gagal
                    let _ = tokio::fs::remove_file(&icon_regulasi_path).await;
                    let _ = tokio::fs::remove_file(&file_regulasi_path).await;

                    Ok(
                        HttpResponse::InternalServerError().json(ApiResponse::<Regulasi>::error(
                            "Gagal mengambil data regulasi yang baru dibuat",
                        )),
                    )
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create regulasi: {}", e);
            // Clean up uploaded files jika gagal
            let _ = tokio::fs::remove_file(&icon_regulasi_path).await;
            let _ = tokio::fs::remove_file(&file_regulasi_path).await;

            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<Regulasi>::error("Gagal membuat regulasi")))
        }
    }
}

#[put("/api/adminpanel/regulasi/{id}")]
pub async fn update_regulasi(
    req: HttpRequest,
    path: web::Path<i32>,
    mut payload: Multipart,
    pool: web::Data<sqlx::MySqlPool>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    // Verify JWT dan get claims
    let claims = match crate::auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            return Ok(
                HttpResponse::Unauthorized().json(ApiResponse::<Regulasi>::error(&e.to_string()))
            );
        }
    };

    // Get existing regulasi data
    let existing_regulasi = sqlx::query_as::<_, Regulasi>("SELECT * FROM regulasi WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    let existing_regulasi = match existing_regulasi {
        Some(regulasi) => regulasi,
        None => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::<Regulasi>::error("Regulasi tidak ditemukan")));
        }
    };

    let mut nama_regulasi = None;
    let mut icon_regulasi_path = None;
    let mut file_regulasi_path = None;

    // Process multipart form
    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "nama_regulasi" => {
                let mut data = Vec::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    data.extend_from_slice(&chunk);
                }
                nama_regulasi = Some(String::from_utf8(data).map_err(|_| {
                    actix_web::error::ErrorBadRequest("Invalid UTF-8 in nama_regulasi")
                })?);
            }
            "icon_regulasi" => {
                // Hanya proses jika file benar-benar diupload (bukan empty)
                let content_disposition = field.content_disposition();
                if let Some(cd) = content_disposition {
                    if let Some(filename) = cd.get_filename() {
                        if !filename.is_empty() {
                            let icon_dir = Path::new("uploads/assets/images/regulasi");
                            let path = save_icon_regulasi_file(field, icon_dir).await?;
                            icon_regulasi_path = Some(path);
                        }
                    }
                }
            }
            "file_regulasi" => {
                // Hanya proses jika file benar-benar diupload (bukan empty)
                let content_disposition = field.content_disposition();
                if let Some(cd) = content_disposition {
                    if let Some(filename) = cd.get_filename() {
                        if !filename.is_empty() {
                            let pdf_dir = Path::new("uploads/assets/file/regulasi");
                            let path = save_file_regulasi_pdf(field, pdf_dir).await?;
                            file_regulasi_path = Some(path);
                        }
                    }
                }
            }
            _ => continue,
        }
    }

    // Build dynamic update query
    let mut update_fields = Vec::new();
    let mut bind_params: Vec<String> = Vec::new();

    if let Some(nama) = &nama_regulasi {
        update_fields.push("nama_regulasi = ?");
        bind_params.push(nama.clone());
    }

    if let Some(icon_path) = &icon_regulasi_path {
        update_fields.push("icon_regulasi = ?");
        bind_params.push(icon_path.clone());
        // Delete old icon file hanya jika upload baru berhasil
        let _ = tokio::fs::remove_file(&existing_regulasi.icon_regulasi).await;
    }

    if let Some(file_path) = &file_regulasi_path {
        update_fields.push("file_regulasi = ?");
        bind_params.push(file_path.clone());
        // Delete old PDF file hanya jika upload baru berhasil
        let _ = tokio::fs::remove_file(&existing_regulasi.file_regulasi).await;
    }

    // Jika tidak ada field yang diupdate selain updated_by, return error
    if update_fields.is_empty()
        && nama_regulasi.is_none()
        && icon_regulasi_path.is_none()
        && file_regulasi_path.is_none()
    {
        return Ok(
            HttpResponse::BadRequest().json(ApiResponse::<Regulasi>::error(
                "Tidak ada data yang diupdate",
            )),
        );
    }

    // Selalu update updated_by, tapi created_by TIDAK diubah
    update_fields.push("updated_by = ?");
    bind_params.push(claims.user_id.to_string());

    let query = format!(
        "UPDATE regulasi SET {} WHERE id = ?",
        update_fields.join(", ")
    );

    let mut sql_query = sqlx::query(&query);

    // Bind semua parameter
    for param in &bind_params {
        sql_query = sql_query.bind(param);
    }

    // Bind ID
    sql_query = sql_query.bind(id);

    let result = sql_query.execute(pool.get_ref()).await;

    match result {
        Ok(_) => {
            // Get updated regulasi
            let updated_regulasi =
                sqlx::query_as::<_, Regulasi>("SELECT * FROM regulasi WHERE id = ?")
                    .bind(id)
                    .fetch_one(pool.get_ref())
                    .await;

            match updated_regulasi {
                Ok(regulasi) => Ok(HttpResponse::Ok()
                    .json(ApiResponse::success(regulasi, "Regulasi berhasil diupdate"))),
                Err(e) => {
                    log::error!("Failed to fetch updated regulasi: {}", e);

                    // Rollback: hapus file yang baru diupload jika gagal mengambil data
                    if let Some(icon_path) = icon_regulasi_path {
                        let _ = tokio::fs::remove_file(icon_path).await;
                    }
                    if let Some(file_path) = file_regulasi_path {
                        let _ = tokio::fs::remove_file(file_path).await;
                    }

                    Ok(
                        HttpResponse::InternalServerError().json(ApiResponse::<Regulasi>::error(
                            "Gagal mengambil data regulasi yang diupdate",
                        )),
                    )
                }
            }
        }
        Err(e) => {
            log::error!("Failed to update regulasi: {}", e);

            // Rollback: hapus file yang baru diupload jika gagal update database
            if let Some(icon_path) = icon_regulasi_path {
                let _ = tokio::fs::remove_file(icon_path).await;
            }
            if let Some(file_path) = file_regulasi_path {
                let _ = tokio::fs::remove_file(file_path).await;
            }

            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<Regulasi>::error("Gagal mengupdate regulasi")))
        }
    }
}

// DELETE - Delete regulasi
#[delete("/api/adminpanel/regulasi/{id}")]
pub async fn delete_regulasi(
    req: HttpRequest,
    path: web::Path<i32>,
    pool: web::Data<sqlx::MySqlPool>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    // Verify JWT
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    // Get regulasi data untuk menghapus file
    let regulasi = sqlx::query_as::<_, Regulasi>("SELECT * FROM regulasi WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;

    let regulasi_data = match regulasi {
        Ok(Some(regulasi)) => regulasi,
        Ok(None) => {
            return Ok(
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Regulasi tidak ditemukan"))
            );
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Terjadi kesalahan database")));
        }
    };

    // Delete dari database
    let result = sqlx::query("DELETE FROM regulasi WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Ok(HttpResponse::NotFound()
                    .json(ApiResponse::<()>::error("Regulasi tidak ditemukan")))
            } else {
                // Delete associated files
                let _ = tokio::fs::remove_file(&regulasi_data.icon_regulasi).await;
                let _ = tokio::fs::remove_file(&regulasi_data.file_regulasi).await;

                Ok(HttpResponse::Ok().json(ApiResponse::success((), "Regulasi berhasil dihapus")))
            }
        }
        Err(e) => {
            log::error!("Failed to delete regulasi: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Gagal menghapus regulasi")))
        }
    }
}
