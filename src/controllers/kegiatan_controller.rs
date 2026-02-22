use crate::{auth, utils::generate_slug};
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, Result, delete, error::ErrorInternalServerError,
    post, put, web,
};
use bytes::BytesMut;
use chrono::{NaiveDate, NaiveTime};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, prelude::FromRow, query, query_as};
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

async fn save_photo_file(
    mut field: actix_multipart::Field,
    dir: &std::path::Path,
    original_filename: Option<String>,
) -> Result<String, Error> {
    if !dir.exists() {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    let ext = original_filename
        .as_deref()
        .and_then(|n| std::path::Path::new(n).extension().and_then(|s| s.to_str()))
        .map(|s| format!(".{}", s))
        .unwrap_or_else(|| ".png".to_string());

    let filename = format!("photo_{}{}", Uuid::new_v4(), ext);
    let filepath = dir.join(&filename);

    let mut f = tokio::fs::File::create(&filepath)
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

    Ok(format!("uploads/assets/images/kegiatan/{}", filename))
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Kegiatan {
    id: i32,
    kategori: String,
    nama_kegiatan: String,
    slug: String,
    photo: String,
    biaya: i32,
    lokasi: String,
    tanggal: NaiveDate,
    jam: Option<NaiveTime>,
    batas_pendaftaran: Option<NaiveDate>,
    map: Option<String>,
    link_pendaftaran: Option<String>,
    status: String,
}

#[put("/api/adminpanel/kegiatan/{id}")]
pub async fn update_kegiatan(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    mut multipart: Multipart,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    // Ambil data kegiatan lama termasuk photo
    let old_kegiatan = sqlx::query_as::<_, Kegiatan>(
        "SELECT id, kategori, nama_kegiatan, slug, photo, biaya, lokasi, tanggal, jam, batas_pendaftaran, map, link_pendaftaran, status FROM kegiatan WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Kegiatan dengan id {}: {}", id, e);
        actix_web::error::ErrorNotFound("Kegiatan tidak ditemukan")
    })?;

    let old_photo = old_kegiatan.photo.clone();

    // Baca multipart: form fields + photo file
    let mut kategori: Option<String> = None;
    let mut nama_kegiatan: Option<String> = None;
    let mut biaya: Option<String> = None;
    let mut lokasi: Option<String> = None;
    let mut tanggal: Option<NaiveDate> = None;
    let mut jam: Option<NaiveTime> = None;
    let mut batas_pendaftaran: Option<NaiveDate> = None;
    let mut map: Option<String> = None;
    let mut link_pendaftaran: Option<String> = None;
    let mut slug: Option<String> = None;
    let mut status: Option<String> = None;
    let mut new_photo_rel: Option<String> = None;
    let mut has_new_photo = false;

    while let Some(mut field) = multipart
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().clone();
        let field_name = cd
            .as_ref()
            .and_then(|c| c.get_name())
            .unwrap_or_default()
            .to_string();

        // Handle file field (photo) secara terpisah
        if field_name == "photo" {
            if let Some(filename) = cd.and_then(|c| c.get_filename().map(|s| s.to_string())) {
                if !filename.trim().is_empty() {
                    let orig = Some(filename);
                    let rel = save_photo_file(
                        field,
                        std::path::Path::new("uploads/assets/images/kegiatan"),
                        orig,
                    )
                    .await?;
                    new_photo_rel = Some(rel);
                    has_new_photo = true;
                    continue; // Skip processing selanjutnya untuk field photo
                }
            }
            // Drain field jika tidak ada file
            while let Some(_chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {}
            continue;
        }

        // Handle text fields
        let mut data = BytesMut::new();
        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(actix_web::error::ErrorBadRequest)?
        {
            data.extend_from_slice(&chunk);
        }

        let val = String::from_utf8(data.to_vec()).map_err(|_| {
            actix_web::error::ErrorBadRequest(format!("{} bukan UTF-8 valid", field_name))
        })?;
        let val = val.trim().to_string();
        let val = if val.to_lowercase() == "null" {
            String::new()
        } else {
            val
        };

        match field_name.as_str() {
            "kategori" => kategori = Some(val),
            "nama_kegiatan" => nama_kegiatan = Some(val),
            "biaya" => biaya = Some(val),
            "lokasi" => lokasi = Some(val),
            "tanggal" => {
                if !val.is_empty() {
                    tanggal = NaiveDate::parse_from_str(&val, "%Y-%m-%d")
                        .map_err(|_| {
                            actix_web::error::ErrorBadRequest(
                                "Format tanggal tidak valid. Gunakan YYYY-MM-DD",
                            )
                        })?
                        .into();
                }
            }
            "jam" => {
                if !val.is_empty() {
                    // Coba beberapa format waktu
                    jam = NaiveTime::parse_from_str(&val, "%H:%M:%S")
                        .or_else(|_| NaiveTime::parse_from_str(&val, "%H:%M"))
                        .or_else(|_| NaiveTime::parse_from_str(&val, "%H-%M-%S"))
                        .ok();
                    if jam.is_none() {
                        log::warn!("Format jam tidak dikenali: {}", val);
                    }
                }
            }
            "batas_pendaftaran" => {
                if !val.is_empty() {
                    batas_pendaftaran = NaiveDate::parse_from_str(&val, "%Y-%m-%d")
                        .map_err(|_| {
                            actix_web::error::ErrorBadRequest(
                                "Format batas_pendaftaran tidak valid. Gunakan YYYY-MM-DD",
                            )
                        })?
                        .into();
                }
            }
            "map" => map = Some(val),
            "link_pendaftaran" => link_pendaftaran = Some(val),
            "slug" => slug = Some(val),
            "status" => status = Some(val),
            _ => {
                // ignore unknown fields
            }
        }
    }

    // Validasi required fields
    let kategori =
        kategori.ok_or_else(|| actix_web::error::ErrorBadRequest("kategori wajib diisi"))?;
    let nama_kegiatan = nama_kegiatan
        .ok_or_else(|| actix_web::error::ErrorBadRequest("nama_kegiatan wajib diisi"))?;
    let biaya = biaya.ok_or_else(|| actix_web::error::ErrorBadRequest("biaya wajib diisi"))?;
    let lokasi = lokasi.ok_or_else(|| actix_web::error::ErrorBadRequest("lokasi wajib diisi"))?;
    let tanggal =
        tanggal.ok_or_else(|| actix_web::error::ErrorBadRequest("tanggal wajib diisi"))?;

    // Parse biaya ke i32
    let biaya = biaya
        .parse::<i32>()
        .map_err(|_| actix_web::error::ErrorBadRequest("biaya harus angka"))?;

    // Generate slug jika tidak ada atau nama_kegiatan berubah
    let slug = if let Some(existing_slug) = slug {
        existing_slug
    } else {
        if nama_kegiatan != old_kegiatan.nama_kegiatan {
            // Anda perlu mengimplementasikan fungsi generate_slug ini
            generate_slug(&nama_kegiatan)
        } else {
            old_kegiatan.slug.clone()
        }
    };

    // Tentukan status
    let status = status.unwrap_or_else(|| "Pendaftaran Dibuka".to_string());

    // Tentukan photo yang akan digunakan
    // Tentukan photo yang akan digunakan tanpa clone yang tidak perlu
    let photo_ref = if has_new_photo {
        // Gunakan reference ke String dalam Option
        new_photo_rel.as_ref().unwrap()
    } else {
        &old_photo
    };

    // Update data di database menggunakan reference
    let result = sqlx::query!(
        r#"
    UPDATE kegiatan
    SET kategori = ?, nama_kegiatan = ?, slug = ?, photo = ?, biaya = ?,
        lokasi = ?, tanggal = ?, jam = ?, batas_pendaftaran = ?,
        map = ?, link_pendaftaran = ?, status = ?
    WHERE id = ?
    "#,
        kategori,
        nama_kegiatan,
        slug,
        photo_ref, // gunakan reference
        biaya,
        lokasi,
        tanggal,
        jam,
        batas_pendaftaran,
        map,
        link_pendaftaran,
        status,
        id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                return Err(actix_web::error::ErrorNotFound("Kegiatan tidak ditemukan"));
            }

            // Jika photo berubah, hapus photo lama
            if has_new_photo && !old_photo.is_empty() {
                let old_photo_path = std::path::Path::new(&old_photo);
                if old_photo_path.exists() {
                    if let Err(e) = tokio::fs::remove_file(old_photo_path).await {
                        log::warn!("Gagal menghapus photo lama: {}", e);
                    }
                }
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Kegiatan berhasil diperbarui"
            })))
        }
        Err(e) => {
            log::error!("Gagal update kegiatan: {}", e);

            // Jika gagal update dan sudah upload photo baru, hapus photo baru
            if has_new_photo {
                if let Some(new_photo) = &new_photo_rel {
                    let new_photo_path = std::path::Path::new(new_photo);
                    if new_photo_path.exists() {
                        let _ = tokio::fs::remove_file(new_photo_path).await;
                    }
                }
            }

            Err(actix_web::error::ErrorInternalServerError(
                "Gagal memperbarui kegiatan",
            ))
        }
    }
}

#[post("/api/adminpanel/kegiatan")]
pub async fn create_kegiatan(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut multipart: Multipart,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    // Baca multipart: form fields + photo file
    let mut kategori: Option<String> = None;
    let mut nama_kegiatan: Option<String> = None;
    let mut biaya: Option<String> = None;
    let mut lokasi: Option<String> = None;
    let mut tanggal: Option<NaiveDate> = None;
    let mut jam: Option<NaiveTime> = None;
    let mut batas_pendaftaran: Option<NaiveDate> = None;
    let mut map: Option<String> = None;
    let mut link_pendaftaran: Option<String> = None;
    let mut status: Option<String> = None;
    let mut photo: Option<String> = None;

    while let Some(mut field) = multipart
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().clone();
        let field_name = cd
            .as_ref()
            .and_then(|c| c.get_name())
            .unwrap_or_default()
            .to_string();

        // Handle file field (photo) secara terpisah
        if field_name == "photo" {
            if let Some(filename) = cd.and_then(|c| c.get_filename().map(|s| s.to_string())) {
                if !filename.trim().is_empty() && filename.to_lowercase() != "null" {
                    let orig = Some(filename);
                    let rel = save_photo_file(
                        field,
                        std::path::Path::new("uploads/assets/images/kegiatan"),
                        orig,
                    )
                    .await?;
                    photo = Some(rel);
                    continue;
                }
            }
            // Drain field jika tidak ada file
            while let Some(_chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {}
            continue;
        }

        // Handle text fields dengan konversi null
        let mut data = BytesMut::new();
        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(actix_web::error::ErrorBadRequest)?
        {
            data.extend_from_slice(&chunk);
        }

        let val = String::from_utf8(data.to_vec()).map_err(|_| {
            actix_web::error::ErrorBadRequest(format!("{} bukan UTF-8 valid", field_name))
        })?;
        let val = val.trim().to_string();

        // Konversi "null" menjadi empty string
        let val = if val.to_lowercase() == "null" {
            String::new()
        } else {
            val
        };

        match field_name.as_str() {
            "kategori" => kategori = Some(val),
            "nama_kegiatan" => nama_kegiatan = Some(val),
            "biaya" => biaya = Some(val),
            "lokasi" => lokasi = Some(val),
            "tanggal" => {
                if !val.is_empty() {
                    tanggal = NaiveDate::parse_from_str(&val, "%Y-%m-%d")
                        .map_err(|_| {
                            actix_web::error::ErrorBadRequest(
                                "Format tanggal tidak valid. Gunakan YYYY-MM-DD",
                            )
                        })?
                        .into();
                }
            }
            "jam" => {
                if !val.is_empty() {
                    jam = NaiveTime::parse_from_str(&val, "%H:%M:%S")
                        .or_else(|_| NaiveTime::parse_from_str(&val, "%H:%M"))
                        .or_else(|_| NaiveTime::parse_from_str(&val, "%H-%M-%S"))
                        .ok();
                }
            }
            "batas_pendaftaran" => {
                if !val.is_empty() {
                    batas_pendaftaran = NaiveDate::parse_from_str(&val, "%Y-%m-%d")
                        .map_err(|_| {
                            actix_web::error::ErrorBadRequest(
                                "Format batas_pendaftaran tidak valid. Gunakan YYYY-MM-DD",
                            )
                        })?
                        .into();
                }
            }
            "map" => map = Some(val),
            "link_pendaftaran" => link_pendaftaran = Some(val),
            "status" => status = Some(val),
            _ => {}
        }
    }

    // Validasi required fields
    let kategori =
        kategori.ok_or_else(|| actix_web::error::ErrorBadRequest("kategori wajib diisi"))?;
    let nama_kegiatan = nama_kegiatan
        .ok_or_else(|| actix_web::error::ErrorBadRequest("nama_kegiatan wajib diisi"))?;
    let biaya = biaya.ok_or_else(|| actix_web::error::ErrorBadRequest("biaya wajib diisi"))?;
    let lokasi = lokasi.ok_or_else(|| actix_web::error::ErrorBadRequest("lokasi wajib diisi"))?;
    let tanggal =
        tanggal.ok_or_else(|| actix_web::error::ErrorBadRequest("tanggal wajib diisi"))?;

    // Parse biaya ke i32
    let biaya = biaya
        .parse::<i32>()
        .map_err(|_| actix_web::error::ErrorBadRequest("biaya harus angka"))?;

    // Generate slug otomatis
    let slug = generate_slug(&nama_kegiatan);

    // Tentukan status default jika tidak disediakan
    let status = status.unwrap_or_else(|| "Pendaftaran Dibuka".to_string());

    // Tentukan photo default jika tidak diupload
    let photo = photo.unwrap_or_else(|| "uploads/assets/images/kegiatan/default.jpg".to_string());

    // Konversi Option fields ke empty string jika None
    let map = map.unwrap_or_default();
    let link_pendaftaran = link_pendaftaran.unwrap_or_default();

    // Insert data ke database
    let result = sqlx::query!(
        r#"
        INSERT INTO kegiatan
        (kategori, nama_kegiatan, slug, photo, biaya, lokasi, tanggal, jam,
         batas_pendaftaran, map, link_pendaftaran, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())
        "#,
        kategori,
        nama_kegiatan,
        slug,
        photo,
        biaya,
        lokasi,
        tanggal,
        jam,
        batas_pendaftaran,
        map,
        link_pendaftaran,
        status,
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            let kegiatan_id = result.last_insert_id();

            Ok(HttpResponse::Created().json(serde_json::json!({
                "success": true,
                "message": "Kegiatan berhasil dibuat",
                "data": {
                    "id": kegiatan_id,
                    "kategori": kategori,
                    "nama_kegiatan": nama_kegiatan,
                    "slug": slug,
                    "tanggal": tanggal,
                    "status": status
                }
            })))
        }
        Err(e) => {
            log::error!("Gagal create kegiatan: {}", e);

            // Jika gagal insert dan sudah upload photo, hapus photo yang sudah diupload
            if photo != "uploads/assets/images/kegiatan/default.jpg" {
                let photo_file_path = std::path::Path::new(&photo);
                if photo_file_path.exists() {
                    if let Err(delete_err) = tokio::fs::remove_file(photo_file_path).await {
                        log::warn!("Gagal menghapus photo: {}", delete_err);
                    }
                }
            }

            // Cek error duplicate slug
            if e.to_string().contains("Duplicate entry") && e.to_string().contains("slug") {
                return Err(actix_web::error::ErrorBadRequest(
                    "Slug sudah digunakan, coba nama kegiatan yang berbeda",
                ));
            }

            Err(actix_web::error::ErrorInternalServerError(
                "Gagal membuat kegiatan",
            ))
        }
    }
}

// ================== DELETE Kegiatan ==================
#[derive(Serialize, FromRow, Debug)]
struct KegiatanDelete {
    id: i32,
    photo: Option<String>,
}
#[delete("/api/adminpanel/kegiatan/{id}")]
pub async fn delete_kegiatan(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let id_to_delete = path.into_inner();
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    // Langkah 1: Ambil path file dari database
    let kegiatan_to_delete: Option<KegiatanDelete> = query_as!(
        KegiatanDelete,
        "SELECT id, photo FROM kegiatan WHERE id = ?",
        id_to_delete
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if kegiatan_to_delete.is_none() {
        return Ok(HttpResponse::NotFound().body(format!(
            "Kegiatan dengan id {} tidak ditemukan",
            id_to_delete
        )));
    }

    let photo_path: Option<PathBuf> = kegiatan_to_delete.and_then(|c| c.photo).map(|filename| {
        // Gabungkan nama file dengan direktori upload
        Path::new("./").join(filename)
    });

    // Langkah 2: Hapus entri dari database
    let result = query!("DELETE FROM kegiatan WHERE id = ?", id_to_delete)
        .execute(pool.get_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(
            HttpResponse::NotFound().body(format!("Kegiatan with id {} not found", id_to_delete))
        );
    }

    // Langkah 3: Hapus file dari sistem file (jika ada)
    if let Some(path_to_delete) = photo_path {
        if path_to_delete.exists() {
            if let Err(e) = fs::remove_file(&path_to_delete).await {
                eprintln!("Failed to delete file {}: {}", path_to_delete.display(), e);
            }
        }
    }

    Ok(HttpResponse::Ok().body(format!(
        "Kegiatan with id {} and its evidence deleted successfully",
        id_to_delete
    )))
}
