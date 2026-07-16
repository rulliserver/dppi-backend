use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, Error, HttpRequest, HttpResponse};
use chrono::{DateTime, Local};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row};
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::auth;

#[derive(Serialize, Deserialize)]
pub struct SipenaModul {
    pub id: String,
    pub title: String,
    pub source_type: String,
    pub file_path: Option<String>,
    pub youtube_url: Option<String>,
    pub extracted_text: String,
    pub created_at: Option<DateTime<Local>>,
}

#[derive(Deserialize)]
pub struct YoutubeInput {
    pub title: String,
    pub youtube_url: String,
    pub transcript: String,
}

#[derive(Deserialize)]
pub struct ChatInput {
    pub message: String,
}

// 1. GET: List all modules
#[get("/api/adminpanel/sipena/modul")]
pub async fn get_modules(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, Error> {
    // Auth Check
    let claims = auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let rows = sqlx::query("SELECT id, title, source_type, file_path, youtube_url, extracted_text, created_at FROM sipena_modul ORDER BY created_at DESC")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let modules: Vec<SipenaModul> = rows
        .into_iter()
        .map(|row| SipenaModul {
            id: row.get("id"),
            title: row.get("title"),
            source_type: row.get("source_type"),
            file_path: row.get("file_path"),
            youtube_url: row.get("youtube_url"),
            extracted_text: row.get("extracted_text"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(HttpResponse::Ok().json(modules))
}

// Helper to save PDF
async fn save_pdf_file(
    mut field: actix_multipart::Field,
    dir: &Path,
) -> Result<String, Error> {
    if !dir.exists() {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    let filename = format!("sipena_{}.pdf", Uuid::new_v4());
    let filepath = dir.join(&filename);

    let mut f = tokio::fs::File::create(&filepath)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    while let Some(chunk) = field
        .try_next()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        use tokio::io::AsyncWriteExt;
        f.write_all(&chunk)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    Ok(format!("uploads/assets/file/sipena/{}", filename))
}

// 2. POST: Upload PDF module
#[post("/api/adminpanel/sipena/upload-modul")]
pub async fn upload_pdf_module(
    req: HttpRequest,
    mut payload: Multipart,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, Error> {
    // Auth Check
    let claims = auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let mut title = String::new();
    let mut file_path = String::new();

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "title" => {
                let mut data = Vec::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    data.extend_from_slice(&chunk);
                }
                title = String::from_utf8(data).map_err(|_| {
                    actix_web::error::ErrorBadRequest("Invalid UTF-8 in title")
                })?;
            }
            "file" => {
                let pdf_dir = Path::new("uploads/assets/file/sipena");
                file_path = save_pdf_file(field, pdf_dir).await?;
            }
            _ => continue,
        }
    }

    if title.is_empty() || file_path.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Title dan file PDF wajib diisi"
        })));
    }

    // Extract text from the PDF file using pdf-extract crate
    let pdf_path = Path::new(&file_path);
    let extracted_text = match pdf_extract::extract_text(pdf_path) {
        Ok(text) => text,
        Err(e) => {
            log::error!("Failed to extract text from PDF: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Gagal mengekstrak teks PDF: {:?}", e)
            })));
        }
    };

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sipena_modul (id, title, source_type, file_path, youtube_url, extracted_text) VALUES (?, ?, 'pdf', ?, NULL, ?)"
    )
    .bind(&id)
    .bind(&title)
    .bind(&file_path)
    .bind(&extracted_text)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Modul PDF berhasil diunggah dan diekstrak",
        "id": id
    })))
}

// 3. POST: Add YouTube link module
#[post("/api/adminpanel/sipena/add-youtube")]
pub async fn add_youtube_module(
    req: HttpRequest,
    input: web::Json<YoutubeInput>,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, Error> {
    // Auth Check
    let claims = auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let title = input.title.trim();
    let youtube_url = input.youtube_url.trim();
    let transcript = input.transcript.trim();

    if title.is_empty() || youtube_url.is_empty() || transcript.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Semua field (Title, YouTube URL, dan Transkrip) wajib diisi"
        })));
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sipena_modul (id, title, source_type, file_path, youtube_url, extracted_text) VALUES (?, ?, 'youtube', NULL, ?, ?)"
    )
    .bind(&id)
    .bind(title)
    .bind(youtube_url)
    .bind(transcript)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Materi YouTube berhasil disimpan",
        "id": id
    })))
}

// 4. DELETE: Delete module
#[delete("/api/adminpanel/sipena/modul/{id}")]
pub async fn delete_module(
    req: HttpRequest,
    id: web::Path<String>,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, Error> {
    // Auth Check
    let claims = auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_str = id.into_inner();

    // Check if module exists and get file path
    let row = sqlx::query("SELECT file_path FROM sipena_modul WHERE id = ?")
        .bind(&id_str)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if let Some(r) = row {
        let file_path: Option<String> = r.get("file_path");
        if let Some(fp) = file_path {
            let path = Path::new(&fp);
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }

        sqlx::query("DELETE FROM sipena_modul WHERE id = ?")
            .bind(&id_str)
            .execute(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Modul berhasil dihapus"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Modul tidak ditemukan"
        })))
    }
}

// 5. POST: Chatbot endpoint (Publicly Accessible)
#[post("/api/sipena/chat")]
pub async fn chat(
    input: web::Json<ChatInput>,
) -> Result<HttpResponse, Error> {
    let user_message = input.message.trim();
    if user_message.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Pesan tidak boleh kosong"
        })));
    }

    let client = reqwest::Client::new();
    let response = match client.post("http://127.0.0.1:8080/chat")
        .json(&serde_json::json!({ "message": user_message }))
        .send()
        .await {
            Ok(res) => res,
            Err(e) => {
                log::error!("Gagal menghubungkan ke Microservice SiPena: {:?}", e);
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "answer": "Mohon maaf, sistem chatbot SiPena sedang offline. Harap hubungi administrator untuk menyalakan microservice."
                })));
            }
        };

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        log::error!("Microservice SiPena Error ({}): {}", status, body_text);
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "answer": "Mohon maaf, terjadi kesalahan pada pemrosesan AI di server chatbot."
        })));
    }

    let result: serde_json::Value = match serde_json::from_str(&body_text) {
        Ok(val) => val,
        Err(_) => {
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "answer": "Gagal mendekode respons dari server chatbot."
            })));
        }
    };

    Ok(HttpResponse::Ok().json(result))
}
