use crate::{auth, models::pengumuman::Pengumuman};

use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, Result, delete, error::ErrorInternalServerError,
    post, put, web,
};
use futures_util::TryStreamExt as _;
use serde::Serialize;
use sqlx::{MySqlPool, prelude::FromRow, query, query_as};
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

// Response structure
#[derive(Debug, Serialize)]
pub struct UpdateAnnouncementResponse {
    pub success: bool,
    pub message: String,
    pub id: i32,
}

#[put("/api/adminpanel/pengumuman/{id}")]
pub async fn put_announcement(
    req: HttpRequest,
    path: web::Path<i32>,
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let id_pengumuman = path.into_inner();

    // Verify JWT and check permissions
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    // Check if announcement exists
    let existing_announcement: Option<Pengumuman> = sqlx::query_as!(
        Pengumuman,
        "SELECT * FROM pengumuman WHERE id = ?",
        id_pengumuman as i32
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if existing_announcement.is_none() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "message": "Pengumuman tidak ditemukan"
        })));
    }

    // Parse multipart form data
    let mut new_image_path: Option<String> = None;
    let mut new_link: Option<String> = None;

    while let Some(mut field) = payload.try_next().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to parse multipart: {}", e))
    })? {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition
            .and_then(|cd| cd.get_name())
            .unwrap_or("");

        match field_name {
            "image" => {
                // Generate unique filename
                let filename = format!(
                    "{}_{}",
                    Uuid::new_v4().to_string(),
                    content_disposition
                        .and_then(|cd| cd.get_filename())
                        .unwrap_or("image.jpg")
                        .replace(" ", "_")
                );

                // Create uploads directory if it doesn't exist
                let upload_dir = "./uploads/assets/pengumuman";
                tokio::fs::create_dir_all(upload_dir).await.map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!(
                        "Failed to create directory: {}",
                        e
                    ))
                })?;

                let filepath = format!("{}/{}", upload_dir, filename);

                // Save the file
                let mut f = tokio::fs::File::create(&filepath)
                    .await
                    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

                while let Some(chunk) = field.try_next().await.map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!("File read error: {}", e))
                })? {
                    f.write_all(&chunk).await.map_err(|e| {
                        actix_web::error::ErrorInternalServerError(format!(
                            "File write error: {}",
                            e
                        ))
                    })?;
                }

                // Store relative path for database
                new_image_path = Some(format!("uploads/assets/pengumuman/{}", filename));
            }
            "link" => {
                // Read link as text
                let mut link_bytes = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!("Link read error: {}", e))
                })? {
                    link_bytes.extend_from_slice(&chunk);
                }

                let link_text = String::from_utf8(link_bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid link encoding: {}", e))
                })?;

                if !link_text.trim().is_empty() {
                    new_link = Some(link_text.trim().to_string());
                }
            }
            _ => {}
        }
    }

    // If no image provided in update, use existing image
    let image_to_update = match new_image_path.as_ref() {
        Some(path) => path.clone(),
        None => existing_announcement.as_ref().unwrap().image.clone(),
    };

    // If no link provided in update, use existing link or clear it
    let link_to_update = match new_link {
        Some(link) => Some(link),
        None => existing_announcement.as_ref().unwrap().link.clone(),
    };

    // Update the announcement in database
    let updated_rows = sqlx::query!(
        "UPDATE pengumuman SET image = ?, link = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        image_to_update,
        link_to_update,
        id_pengumuman
    )
    .execute(pool.as_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    if updated_rows.rows_affected() == 0 {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": "Gagal memperbarui pengumuman"
        })));
    }

    // If a new image was uploaded, delete the old image file
    if new_image_path.is_some() {
        let old_image_path = &existing_announcement.unwrap().image;
        // Remove leading slash to get filesystem path
        let old_filepath = old_image_path.trim_start_matches('/');

        // Try to delete old file, but don't fail if it doesn't exist
        if let Err(e) = tokio::fs::remove_file(old_filepath).await {
            println!("Warning: Failed to delete old image file: {}", e);
        }
    }

    Ok(HttpResponse::Ok().json(UpdateAnnouncementResponse {
        success: true,
        message: "Pengumuman berhasil diperbarui".to_string(),
        id: id_pengumuman as i32,
    }))
}

#[derive(Serialize, FromRow, Debug)]
struct PengumumanDelete {
    id: i32,
    image: Option<String>,
}

#[delete("/api/adminpanel/pengumuman/{id}")]
pub async fn delete_pengumuman(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let id_to_delete = path.into_inner();
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    // Langkah 1: Ambil path file dari database
    let pengumuman_to_delete: Option<PengumumanDelete> = query_as!(
        PengumumanDelete,
        "SELECT id, image FROM pengumuman WHERE id = ?",
        id_to_delete
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    if pengumuman_to_delete.is_none() {
        return Ok(
            HttpResponse::NotFound().body(format!("Pengumuman with id {} not found", id_to_delete))
        );
    }

    let image_path: Option<PathBuf> = pengumuman_to_delete.and_then(|c| c.image).map(|filename| {
        // Gabungkan nama file dengan direktori upload
        Path::new("./").join(filename)
    });

    // Langkah 2: Hapus entri dari database
    let result = query!("DELETE FROM pengumuman WHERE id = ?", id_to_delete)
        .execute(pool.get_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(
            HttpResponse::NotFound().body(format!("Pengumuman with id {} not found", id_to_delete))
        );
    }

    // Langkah 3: Hapus file dari sistem file (jika ada)
    if let Some(path_to_delete) = image_path {
        if path_to_delete.exists() {
            if let Err(e) = fs::remove_file(&path_to_delete).await {
                eprintln!("Failed to delete file {}: {}", path_to_delete.display(), e);
            }
        }
    }

    Ok(HttpResponse::Ok().body(format!(
        "Pengumuman with id {} and its evidence deleted successfully",
        id_to_delete
    )))
}

//create pengumuman
#[post("/api/adminpanel/pengumuman")]
pub async fn create_pengumuman(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    // Verify JWT and check permissions
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let user_id = claims.user_id;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let mut image_path: Option<String> = None;
    let mut link: Option<String> = None;
    let mut has_image = false;

    while let Some(mut field) = payload.try_next().await.map_err(|e| {
        println!("❌ Error next field: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Failed to parse multipart: {}", e))
    })? {
        let content_disposition = field.content_disposition();

        let field_name = content_disposition
            .and_then(|cd| cd.get_name())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string());

        let filename = content_disposition
            .and_then(|cd| cd.get_filename())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "no_filename".to_string());

        println!("📦 Field: name='{}', filename='{}'", field_name, filename);

        match field_name.as_str() {
            "image" => {
                println!("🖼️ Processing image field with filename: {}", filename);
                has_image = true;

                // Gunakan extension dari filename atau default png
                let ext = if filename.contains('.') && filename != "no_filename" {
                    filename.split('.').last().unwrap_or("png")
                } else {
                    "png" // Default untuk blob
                };

                let new_filename = format!("{}.{}", Uuid::new_v4(), ext);
                let upload_dir = "./uploads/assets/pengumuman";

                // Create directory
                match tokio::fs::create_dir_all(upload_dir).await {
                    Ok(_) => println!("✅ Directory created/verified"),
                    Err(e) => {
                        println!("❌ Directory error: {}", e);
                        return Err(actix_web::error::ErrorInternalServerError(format!(
                            "Failed to create directory: {}",
                            e
                        )));
                    }
                }

                let filepath = format!("{}/{}", upload_dir, new_filename);
                println!("💾 Saving to: {}", filepath);

                // Save file
                let mut f = match tokio::fs::File::create(&filepath).await {
                    Ok(file) => {
                        println!("✅ File created");
                        file
                    }
                    Err(e) => {
                        println!("❌ File create error: {}", e);
                        return Err(actix_web::error::ErrorInternalServerError(e.to_string()));
                    }
                };

                let mut total_bytes = 0;

                // Baca dan tulis data
                while let Some(chunk) = field.try_next().await.map_err(|e| {
                    println!("❌ Chunk read error: {}", e);
                    actix_web::error::ErrorInternalServerError(format!("File read error: {}", e))
                })? {
                    total_bytes += chunk.len();
                    match f.write_all(&chunk).await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("❌ Write error: {}", e);
                            return Err(actix_web::error::ErrorInternalServerError(format!(
                                "File write error: {}",
                                e
                            )));
                        }
                    }
                }

                println!("✅ Image saved: {} bytes", total_bytes);

                if total_bytes == 0 {
                    println!("⚠️ WARNING: Image file is empty!");
                    return Err(actix_web::error::ErrorBadRequest("Image file is empty"));
                }

                image_path = Some(format!("uploads/assets/pengumuman/{}", new_filename));
                println!("📁 Image path: {:?}", image_path);
            }
            "link" => {
                println!("🔗 Processing link field...");

                let mut link_bytes = Vec::new();
                while let Some(chunk) = field.try_next().await.map_err(|e| {
                    println!("❌ Link chunk error: {}", e);
                    actix_web::error::ErrorInternalServerError(format!("Link read error: {}", e))
                })? {
                    link_bytes.extend_from_slice(&chunk);
                }

                if !link_bytes.is_empty() {
                    match String::from_utf8(link_bytes) {
                        Ok(link_text) => {
                            let trimmed = link_text.trim();
                            if !trimmed.is_empty() {
                                link = Some(trimmed.to_string());
                                println!("✅ Link: {}", trimmed);
                            }
                        }
                        Err(e) => println!("⚠️ Link UTF-8 error: {}", e),
                    }
                }
            }
            _ => println!("⚠️ Unknown field: {}", field_name),
        }
    }

    println!("🔚 Finished processing fields");
    println!(
        "📊 Summary: has_image={}, image_path={:?}, link={:?}",
        has_image, image_path, link
    );

    // Validate
    if !has_image || image_path.is_none() {
        println!("❌ Validation failed: Image required but not found");
        return Err(actix_web::error::ErrorBadRequest("Image is required"));
    }

    // Insert to database
    let image_path_str = image_path.unwrap();
    println!("💾 Inserting to database...");

    match sqlx::query!(
        "INSERT INTO pengumuman (image, link, created_by, created_at, updated_at)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        image_path_str,
        link,
        user_id
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            let new_id = result.last_insert_id();
            println!("✅ Database insert successful, ID: {}", new_id);

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Pengumuman berhasil dibuat",
                "id": new_id
            })))
        }
        Err(e) => {
            println!("❌ Database error: {}", e);
            Err(actix_web::error::ErrorInternalServerError(format!(
                "Database error: {}",
                e
            )))
        }
    }
}
