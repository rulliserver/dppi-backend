<<<<<<< HEAD
use crate::auth;
use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, get, put, web};
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt as _;
use serde::Serialize;
use sqlx::{MySqlPool, Row, prelude::FromRow};
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

#[derive(Serialize, FromRow, Debug)]
struct Video {
    id: i32,
    file_video: String,
    created_at: DateTime<Utc>,
}

#[get("/api/video")]
pub async fn get_video(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let video: Video = sqlx::query_as::<_, Video>(
        r#"
        SELECT id, file_video, created_at
        FROM videos
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(video))
}

/// Ganti file video (upload multipart), swap atomik, update DB, hapus file lama
/// Form-field: `file` (video/mp4|video/webm)
#[put("/api/adminpanel/videos/{id}")]
pub async fn update_video(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();

    // 1) Auth & role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    // 2) Ambil path lama dari DB
    let (old_path,): (String,) = sqlx::query("SELECT file_video FROM videos WHERE id = ?")
        .bind(id)
        .fetch_one(pool.get_ref())
        .await
        .map(|row| (row.get::<String, _>(0),))
        .map_err(|e| {
            log::error!("Gagal mengambil video {}: {:?}", id, e);
            actix_web::error::ErrorNotFound("Video tidak ditemukan")
        })?;

    // 3) Siapkan folder simpan
    let base_dir = Path::new("./uploads/assets/videos");
    if !base_dir.exists() {
        fs::create_dir_all(base_dir)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    // 4) Terima 1 part bernama "file" → tulis ke file sementara
    let mut temp_path: Option<PathBuf> = None;
    let final_name = format!("video-{}.mp4", Uuid::new_v4());
    let final_path = base_dir.join(&final_name);

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = match field.content_disposition() {
            Some(cd) => cd,
            None => continue,
        };
        if cd.get_name() != Some("file") {
            continue;
        }

        // Validasi Content-Type sederhana (header)
        let content_type_ok = field
            .content_type()
            .map(|ct| {
                let mt = ct.essence_str();
                mt == "video/mp4" || mt == "video/webm" || mt == "application/octet-stream"
            })
            .unwrap_or(true);
        if !content_type_ok {
            return Err(actix_web::error::ErrorBadRequest(
                "Tipe file tidak didukung (hanya mp4/webm)",
            ));
        }

        // Tulis ke temporary file
        let tmp = base_dir.join(format!("{}.uploading", Uuid::new_v4()));
        let mut f = fs::File::create(&tmp)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(actix_web::error::ErrorBadRequest)?
        {
            f.write_all(&chunk)
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;
        }
        f.flush().await.ok();
        temp_path = Some(tmp);
    }

    let Some(tmp) = temp_path else {
        return Err(actix_web::error::ErrorBadRequest(
            "Tidak ada part bernama `file` pada multipart form-data",
        ));
    };

    // 5) Swap atomik: temp -> final
    if let Err(e) = fs::rename(&tmp, &final_path).await {
        // cleanup tmp bila gagal
        let _ = fs::remove_file(&tmp).await;
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    // Path relatif untuk diserve front-end
    let final_rel = format!("/uploads/assets/videos/{}", final_name);

    // 6) Update DB
    let tx = pool
        .begin()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;


    sqlx::query(r#" UPDATE videos SET file_video = ? WHERE id = ? "#)
        .bind(&final_rel)
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    tx.commit()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // 7) Hapus file lama (best-effort; normalisasi path)
    if !old_path.is_empty() {
        // contoh: "/uploads/assets/videos/abc.mp4" -> "./uploads/assets/videos/abc.mp4"
        let old_abs = PathBuf::from(".").join(old_path.trim_start_matches('/'));
        if old_abs.exists() {
            if let Err(e) = fs::remove_file(&old_abs).await {
                log::warn!("Gagal hapus file lama {:?}: {:?}", old_abs, e);
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Video berhasil diupdate",
        "file_video": final_rel
    })))
}
=======
use crate::auth;
use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, get, put, web};
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt as _;
use serde::Serialize;
use sqlx::{MySqlPool, Row, prelude::FromRow};
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

#[derive(Serialize, FromRow, Debug)]
struct Video {
    id: i32,
    file_video: String,
    created_at: DateTime<Utc>,
}

#[get("/api/video")]
pub async fn get_video(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let video: Video = sqlx::query_as::<_, Video>(
        r#"
        SELECT id, file_video, created_at
        FROM videos
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(video))
}

/// Ganti file video (upload multipart), swap atomik, update DB, hapus file lama
/// Form-field: `file` (video/mp4|video/webm)
#[put("/api/adminpanel/videos/{id}")]
pub async fn update_video(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();

    // 1) Auth & role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    // 2) Ambil path lama dari DB
    let (old_path,): (String,) = sqlx::query("SELECT file_video FROM videos WHERE id = ?")
        .bind(id)
        .fetch_one(pool.get_ref())
        .await
        .map(|row| (row.get::<String, _>(0),))
        .map_err(|e| {
            log::error!("Gagal mengambil video {}: {:?}", id, e);
            actix_web::error::ErrorNotFound("Video tidak ditemukan")
        })?;

    // 3) Siapkan folder simpan
    let base_dir = Path::new("./uploads/assets/videos");
    if !base_dir.exists() {
        fs::create_dir_all(base_dir)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    // 4) Terima 1 part bernama "file" → tulis ke file sementara
    let mut temp_path: Option<PathBuf> = None;
    let final_name = format!("video-{}.mp4", Uuid::new_v4());
    let final_path = base_dir.join(&final_name);

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = match field.content_disposition() {
            Some(cd) => cd,
            None => continue,
        };
        if cd.get_name() != Some("file") {
            continue;
        }

        // Validasi Content-Type sederhana (header)
        let content_type_ok = field
            .content_type()
            .map(|ct| {
                let mt = ct.essence_str();
                mt == "video/mp4" || mt == "video/webm" || mt == "application/octet-stream"
            })
            .unwrap_or(true);
        if !content_type_ok {
            return Err(actix_web::error::ErrorBadRequest(
                "Tipe file tidak didukung (hanya mp4/webm)",
            ));
        }

        // Tulis ke temporary file
        let tmp = base_dir.join(format!("{}.uploading", Uuid::new_v4()));
        let mut f = fs::File::create(&tmp)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(actix_web::error::ErrorBadRequest)?
        {
            f.write_all(&chunk)
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;
        }
        f.flush().await.ok();
        temp_path = Some(tmp);
    }

    let Some(tmp) = temp_path else {
        return Err(actix_web::error::ErrorBadRequest(
            "Tidak ada part bernama `file` pada multipart form-data",
        ));
    };

    // 5) Swap atomik: temp -> final
    if let Err(e) = fs::rename(&tmp, &final_path).await {
        // cleanup tmp bila gagal
        let _ = fs::remove_file(&tmp).await;
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    // Path relatif untuk diserve front-end
    let final_rel = format!("/uploads/assets/videos/{}", final_name);

    // 6) Update DB
    let tx = pool
        .begin()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;


    sqlx::query(r#" UPDATE videos SET file_video = ? WHERE id = ? "#)
        .bind(&final_rel)
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    tx.commit()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // 7) Hapus file lama (best-effort; normalisasi path)
    if !old_path.is_empty() {
        // contoh: "/uploads/assets/videos/abc.mp4" -> "./uploads/assets/videos/abc.mp4"
        let old_abs = PathBuf::from(".").join(old_path.trim_start_matches('/'));
        if old_abs.exists() {
            if let Err(e) = fs::remove_file(&old_abs).await {
                log::warn!("Gagal hapus file lama {:?}: {:?}", old_abs, e);
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Video berhasil diupdate",
        "file_video": final_rel
    })))
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
