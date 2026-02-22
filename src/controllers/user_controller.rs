use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use bcrypt::{DEFAULT_COST, hash};
use chrono::DateTime;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, prelude::FromRow};
use std::fs;
use std::io::Write;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    auth,
    controllers::pelaksana_controller::remove_file_if_exists,
    models::user::{User, UserForm, UserProfile},
    utils::normalize_phone,
};

// Fungsi untuk menyimpan file avatar
async fn save_avatar_file(
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
        .unwrap_or_else(|| ".png".to_string()); // Changed to png to match frontend

    let filename = format!("avatar_{}{}", Uuid::new_v4(), ext);
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

    Ok(format!("uploads/assets/images/avatars/{}", filename)) // Remove leading slash
}

#[put("/api/userpanel/profile")]
pub async fn update_profile_user(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut multipart: Multipart,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // Ambil user berdasarkan email dari claim
    let user_email = claims.sub;

    // Ambil data user lama termasuk avatar
    let user_old = sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, password, address, avatar, phone, email_verified_at, remember_token, id_pdp, id_provinsi, id_kabupaten, created_at FROM users WHERE email = ?"
    )
    .bind(&user_email)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("User not found for email {}: {}", user_email, e);
        actix_web::error::ErrorNotFound("User tidak ditemukan")
    })?;
    let id_user = user_old.id.clone();
    let old_avatar = user_old.avatar.clone();

    // Baca multipart: form fields + photo file
    let mut name: Option<String> = None;
    let mut email: Option<String> = None;
    let mut address: Option<String> = None;
    let mut phone: Option<String> = None;
    let mut new_avatar_rel: Option<String> = None;
    let mut has_new_avatar = false;

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
            "name" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                name =
                    Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                        actix_web::error::ErrorBadRequest("name bukan UTF-8 valid")
                    })?);
            }
            "email" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                email =
                    Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                        actix_web::error::ErrorBadRequest("email bukan UTF-8 valid")
                    })?);
            }
            "address" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                address =
                    Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                        actix_web::error::ErrorBadRequest("address bukan UTF-8 valid")
                    })?);
            }
            "phone" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                phone =
                    Some(String::from_utf8(bytes.to_vec()).map_err(|_| {
                        actix_web::error::ErrorBadRequest("phone bukan UTF-8 valid")
                    })?);
            }
            "avatar" => {
                // Cek apakah benar-benar ada file yang diupload
                if let Some(filename) = cd.and_then(|c| c.get_filename().map(|s| s.to_string())) {
                    if !filename.trim().is_empty() {
                        let orig = Some(filename);
                        let rel = save_avatar_file(
                            field,
                            std::path::Path::new("uploads/assets/images/avatars"),
                            orig,
                        )
                        .await?;
                        new_avatar_rel = Some(rel);
                        has_new_avatar = true;
                    }
                } else {
                    // Drain field jika tidak ada file
                    while let Some(_chunk) = field
                        .try_next()
                        .await
                        .map_err(actix_web::error::ErrorBadRequest)?
                    {}
                }
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
    let new_name =
        name.ok_or_else(|| actix_web::error::ErrorBadRequest("Field name wajib diisi"))?;
    let new_email =
        email.ok_or_else(|| actix_web::error::ErrorBadRequest("Field email wajib diisi"))?;
    let new_address =
        address.ok_or_else(|| actix_web::error::ErrorBadRequest("Field address wajib diisi"))?;
    let new_phone =
        phone.ok_or_else(|| actix_web::error::ErrorBadRequest("Field phone wajib diisi"))?;

    // Validasi field tidak boleh kosong
    if new_name.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest("Nama tidak boleh kosong"));
    }
    if new_email.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Email tidak boleh kosong",
        ));
    }

    // Cek jika email diubah, pastikan email baru tidak duplikat
    if new_email != user_old.email {
        let existing_user: Option<(i32,)> =
            sqlx::query_as("SELECT id FROM users WHERE email = ? AND id != ?")
                .bind(&new_email)
                .bind(user_old.id)
                .fetch_optional(pool.get_ref())
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;

        if existing_user.is_some() {
            return Err(actix_web::error::ErrorBadRequest(
                "Email sudah digunakan oleh user lain",
            ));
        }
    }

    // Update user profile
    let result = sqlx::query(
        "UPDATE users SET name = ?, email = ?, address = ?, phone = ?, avatar = COALESCE(?, avatar), updated_at = NOW() WHERE id = ?"
    )
    .bind(&new_name)
    .bind(&new_email)
    .bind(&new_address)
    .bind(&new_phone)
    .bind(if has_new_avatar { new_avatar_rel.as_ref() } else { None })
    .bind(id_user)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("User tidak ditemukan"));
    }

    // Hapus avatar lama hanya jika ada avatar baru & path berbeda
    if has_new_avatar {
        if let (Some(new_rel), Some(old_rel)) = (new_avatar_rel.as_ref(), old_avatar.as_ref()) {
            if new_rel != old_rel && !old_rel.is_empty() {
                let old_abs = std::path::Path::new(".").join(old_rel.trim_start_matches('/'));
                if old_abs.exists() {
                    let _ = tokio::fs::remove_file(old_abs).await;
                }
            }
        }
    }

    #[derive(serde::Serialize)]
    struct UpdateProfileResponse {
        message: String,
        avatar: Option<String>,
        updated: bool,
    }

    Ok(HttpResponse::Ok().json(UpdateProfileResponse {
        message: "Profile berhasil diperbarui".into(),
        avatar: if has_new_avatar {
            new_avatar_rel
        } else {
            old_avatar
        },
        updated: true,
    }))
}

// Function change_password tetap sama seperti sebelumnya
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[put("/api/userpanel/change-password")]
pub async fn change_password(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<ChangePasswordRequest>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let user_email = claims.sub;
    let password_data = payload.into_inner();

    // Validasi password
    if password_data.new_password != password_data.confirm_password {
        return Err(actix_web::error::ErrorBadRequest(
            "Password baru dan konfirmasi password tidak sama",
        ));
    }

    if password_data.new_password.len() < 6 {
        return Err(actix_web::error::ErrorBadRequest(
            "Password baru minimal 6 karakter",
        ));
    }

    // Ambil user dengan password
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, password, address, avatar, phone, email_verified_at, remember_token, id_pdp, id_provinsi, id_kabupaten, created_at FROM users WHERE email = ?"
    )
    .bind(&user_email)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("User not found for email {}: {}", user_email, e);
        actix_web::error::ErrorNotFound("User tidak ditemukan")
    })?;

    // Verifikasi password saat ini
    let is_valid =
        bcrypt::verify(&password_data.current_password, &user.password).map_err(|e| {
            log::error!("Password verification error: {}", e);
            actix_web::error::ErrorInternalServerError("Error verifikasi password")
        })?;

    if !is_valid {
        return Err(actix_web::error::ErrorBadRequest(
            "Password saat ini tidak valid",
        ));
    }

    // Hash password baru
    let hashed_new_password = hash(&password_data.new_password, DEFAULT_COST)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Update password
    sqlx::query("UPDATE users SET password = ?, updated_at = NOW() WHERE id = ?")
        .bind(&hashed_new_password)
        .bind(user.id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    #[derive(serde::Serialize)]
    struct ChangePasswordResponse {
        message: String,
        updated: bool,
    }

    Ok(HttpResponse::Ok().json(ChangePasswordResponse {
        message: "Password berhasil diubah".into(),
        updated: true,
    }))
}

#[get("/api/userpanel/user")]
pub async fn get_current_user(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let user_email = claims.sub;

    let user = sqlx::query_as::<_, UserProfile>(
        "SELECT id, name, email, role, address, avatar, phone, id_pdp, id_provinsi, id_kabupaten, created_at FROM users WHERE email = ?"
    )
    .bind(&user_email)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("User not found for email {}: {}", user_email, e);
        actix_web::error::ErrorNotFound("User tidak ditemukan")
    })?;

    Ok(HttpResponse::Ok().json(user))
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pelaksana {
    pub id: i32,
    pub id_pdp: Option<String>,
    pub nama_lengkap: String,
    pub photo: Option<String>,
    pub jabatan: Option<String>,
    pub id_provinsi: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub nama_provinsi: Option<String>,
    pub nama_kabupaten: Option<String>,
    pub tingkat_kepengurusan: Option<String>,
}
#[get("/api/userpanel/get-pelaksana")]
pub async fn get_current_pelaksana(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let id_pdp = match claims.id_pdp {
        Some(id) => id,
        None => {
            return Err(actix_web::error::ErrorUnauthorized(
                "User tidak memiliki akses PDP",
            ));
        }
    };

    // Pertama, ambil data user/pdp untuk mengetahui tingkat kepengurusan dan wilayah
    let user_data = sqlx::query!(
        r#"
        SELECT
            p.id,
            p.tingkat_kepengurusan,
            p.id_provinsi,
            p.id_kabupaten
        FROM pdp p
        WHERE p.id = ?
        "#,
        id_pdp
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Error fetching user data for id_pdp {}: {}", id_pdp, e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let user_data = match user_data {
        Some(data) => data,
        None => {
            return Err(actix_web::error::ErrorNotFound("Data user tidak ditemukan"));
        }
    };

    // Handle Option<String> untuk tingkat_kepengurusan
    let tingkat_kepengurusan = match user_data.tingkat_kepengurusan {
        Some(tingkat) => tingkat,
        None => {
            return Err(actix_web::error::ErrorBadRequest(
                "Tingkat kepengurusan tidak ditemukan untuk user",
            ));
        }
    };

    // Query berdasarkan tingkat kepengurusan
    let pelaksana = match tingkat_kepengurusan.as_str() {
        "Pelaksana Tingkat Kabupaten/Kota" => {
            // Handle Option untuk id_kabupaten
            let id_kabupaten = match user_data.id_kabupaten {
                Some(id) => id,
                None => {
                    return Err(actix_web::error::ErrorBadRequest(
                        "User kabupaten tidak memiliki id_kabupaten",
                    ));
                }
            };

            // Ambil data pelaksana_kabupaten berdasarkan id_kabupaten user
            sqlx::query_as::<_, Pelaksana>(
                r#"
                SELECT
                    pk.id,
                    pk.id_pdp,
                    pk.nama_lengkap,
                    pk.photo,
                    pk.jabatan,
                    pk.id_provinsi,
                    pk.id_kabupaten,
                    prov.nama_provinsi,
                    kab.nama_kabupaten,
                    'Pelaksana Tingkat Kabupaten/Kota' as tingkat_kepengurusan
                FROM pelaksana_kabupaten pk
                LEFT JOIN provinsi prov ON pk.id_provinsi = prov.id
                LEFT JOIN kabupaten kab ON pk.id_kabupaten = kab.id
                WHERE pk.id_kabupaten = ?
                ORDER BY pk.nama_lengkap
                "#,
            )
            .bind(id_kabupaten)
            .fetch_all(pool.get_ref())
            .await
        }
        "Pelaksana Tingkat Provinsi" => {
            // Handle Option untuk id_provinsi
            let id_provinsi = match user_data.id_provinsi {
                Some(id) => id,
                None => {
                    return Err(actix_web::error::ErrorBadRequest(
                        "User provinsi tidak memiliki id_provinsi",
                    ));
                }
            };

            // Ambil data pelaksana_provinsi berdasarkan id_provinsi user
            sqlx::query_as::<_, Pelaksana>(
                r#"
                SELECT
                    pp.id,
                    pp.id_pdp,
                    pp.nama_lengkap,
                    pp.photo,
                    pp.jabatan,
                    pp.id_provinsi,
                    NULL as id_kabupaten,
                    prov.nama_provinsi,
                    NULL as nama_kabupaten,
                    'Pelaksana Tingkat Provinsi' as tingkat_kepengurusan
                FROM pelaksana_provinsi pp
                LEFT JOIN provinsi prov ON pp.id_provinsi = prov.id
                WHERE pp.id_provinsi = ?
                ORDER BY pp.nama_lengkap
                "#,
            )
            .bind(id_provinsi)
            .fetch_all(pool.get_ref())
            .await
        }
        "Pelaksana Tingkat Pusat" => {
            // Ambil semua data pelaksana_pusat
            sqlx::query_as::<_, Pelaksana>(
                r#"
                SELECT
                    pps.id,
                    pps.id_pdp,
                    pps.nama_lengkap,
                    pps.photo,
                    pps.jabatan,
                    NULL as id_provinsi,
                    NULL as id_kabupaten,
                    NULL as nama_provinsi,
                    NULL as nama_kabupaten,
                    'Pelaksana Tingkat Pusat' as tingkat_kepengurusan
                FROM pelaksana_pusat pps
                ORDER BY pps.nama_lengkap
                "#,
            )
            .fetch_all(pool.get_ref())
            .await
        }
        _ => {
            return Err(actix_web::error::ErrorBadRequest(format!(
                "Tingkat kepengurusan tidak valid: {}",
                tingkat_kepengurusan
            )));
        }
    }
    .map_err(|e| {
        log::error!("Error fetching pelaksana: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::Ok().json(pelaksana))
}
// Alternatif: Versi dengan dynamic query (jika struktur tabel sama)
#[get("/api/userpanel/get-pelaksana-dynamic")]
pub async fn get_current_pelaksana_dynamic(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let id_pdp = match claims.id_pdp {
        Some(id) => id.clone(),
        None => {
            return Err(actix_web::error::ErrorUnauthorized(
                "User tidak memiliki akses PDP",
            ));
        }
    };

    // Cari di semua tabel pelaksana
    let pelaksana = sqlx::query_as::<_, Pelaksana>(
        r#"
        SELECT id, id_pdp, nama_lengkap, photo, jabatan, 'Kabupaten/Kota' as tingkat
        FROM pelaksana_kabupaten WHERE id_pdp = ?
        UNION ALL
        SELECT id, id_pdp, nama_lengkap, photo, jabatan, 'Provinsi' as tingkat
        FROM pelaksana_provinsi WHERE id_pdp = ?
        UNION ALL
        SELECT id, id_pdp, nama_lengkap, photo, jabatan, 'Pusat' as tingkat
        FROM pelaksana_pusat WHERE id_pdp = ?
        LIMIT 1
        "#,
    )
    .bind(&id_pdp)
    .bind(&id_pdp)
    .bind(&id_pdp)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Pelaksana not found for id_pdp {}: {}", id_pdp, e);
        actix_web::error::ErrorNotFound("Data pelaksana tidak ditemukan")
    })?;

    match pelaksana {
        Some(data) => Ok(HttpResponse::Ok().json(data)),
        None => Err(actix_web::error::ErrorNotFound(
            "Data pelaksana tidak ditemukan untuk PDP ini",
        )),
    }
}

#[post("/api/userpanel/pelaksana")]
pub async fn create_pelaksana(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // Ambil data user untuk mengetahui tingkat kepengurusan
    let user_data = sqlx::query!(
        r#"
        SELECT
            p.id,
            p.tingkat_kepengurusan,
            p.id_provinsi,
            p.id_kabupaten
        FROM pdp p
        WHERE p.id = ?
        "#,
        claims.id_pdp
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Error fetching user data: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let user_data = match user_data {
        Some(data) => data,
        None => {
            return Err(actix_web::error::ErrorNotFound("Data user tidak ditemukan"));
        }
    };

    let tingkat_kepengurusan = match user_data.tingkat_kepengurusan {
        Some(tingkat) => tingkat,
        None => {
            return Err(actix_web::error::ErrorBadRequest(
                "Tingkat kepengurusan tidak ditemukan",
            ));
        }
    };

    // Parse multipart form data
    let mut nama_lengkap = String::new();
    let mut jabatan = String::new();
    let mut id_pdp = String::new();
    let mut id_provinsi = None;
    let mut id_kabupaten = None;
    let mut photo_data = None;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "nama_lengkap" => {
                let bytes = field_to_bytes(&mut field).await?;
                nama_lengkap = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid nama_lengkap: {}", e))
                })?;
            }
            "jabatan" => {
                let bytes = field_to_bytes(&mut field).await?;
                jabatan = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid jabatan: {}", e))
                })?;
            }
            "id_pdp" => {
                let bytes = field_to_bytes(&mut field).await?;
                id_pdp = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid id_pdp: {}", e))
                })?;
            }
            "id_provinsi" => {
                let bytes = field_to_bytes(&mut field).await?;
                let provinsi_str = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid id_provinsi: {}", e))
                })?;
                if !provinsi_str.is_empty() {
                    id_provinsi = Some(provinsi_str.parse::<i32>().map_err(|e| {
                        actix_web::error::ErrorBadRequest(format!(
                            "Invalid id_provinsi format: {}",
                            e
                        ))
                    })?);
                }
            }
            "id_kabupaten" => {
                let bytes = field_to_bytes(&mut field).await?;
                let kabupaten_str = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid id_kabupaten: {}", e))
                })?;
                if !kabupaten_str.is_empty() {
                    id_kabupaten = Some(kabupaten_str.parse::<i32>().map_err(|e| {
                        actix_web::error::ErrorBadRequest(format!(
                            "Invalid id_kabupaten format: {}",
                            e
                        ))
                    })?);
                }
            }
            "photo" => {
                let content_type = field.content_type().map(|ct| ct.to_string());
                if let Some(ct) = content_type {
                    if ct.starts_with("image/") {
                        let bytes = field_to_bytes(&mut field).await?;
                        if !bytes.is_empty() {
                            photo_data = Some(bytes);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Validasi required fields
    if nama_lengkap.is_empty() || jabatan.is_empty() || id_pdp.is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "nama_lengkap, jabatan, dan id_pdp harus diisi",
        ));
    }

    let id_pdp = id_pdp
        .parse::<i32>()
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid id_pdp format: {}", e)))?;

    // Handle photo upload
    let photo_path = if let Some(photo_bytes) = photo_data {
        Some(save_image(&photo_bytes, "pelaksana").await?)
    } else {
        None
    };

    // Insert berdasarkan tingkat kepengurusan
    match tingkat_kepengurusan.as_str() {
        "Pelaksana Tingkat Kabupaten/Kota" => {
            let id_kabupaten = id_kabupaten.ok_or_else(|| {
                actix_web::error::ErrorBadRequest("id_kabupaten diperlukan untuk tingkat kabupaten")
            })?;

            let id_provinsi = id_provinsi.ok_or_else(|| {
                actix_web::error::ErrorBadRequest("id_provinsi diperlukan untuk tingkat kabupaten")
            })?;

            let result = sqlx::query!(
                r#"
                INSERT INTO pelaksana_kabupaten
                (id_pdp, nama_lengkap, photo, jabatan, id_provinsi, id_kabupaten)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
                id_pdp,
                nama_lengkap,
                photo_path,
                jabatan,
                id_provinsi,
                id_kabupaten
            )
            .execute(pool.get_ref())
            .await
            .map_err(|e| {
                log::error!("Error creating pelaksana kabupaten: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Pelaksana kabupaten berhasil dibuat",
                "id": result.last_insert_id()
            })))
        }
        "Pelaksana Tingkat Provinsi" => {
            let id_provinsi = id_provinsi.ok_or_else(|| {
                actix_web::error::ErrorBadRequest("id_provinsi diperlukan untuk tingkat provinsi")
            })?;

            let result = sqlx::query!(
                r#"
                INSERT INTO pelaksana_provinsi
                (id_pdp, nama_lengkap, photo, jabatan, id_provinsi)
                VALUES (?, ?, ?, ?, ?)
                "#,
                id_pdp,
                nama_lengkap,
                photo_path,
                jabatan,
                id_provinsi
            )
            .execute(pool.get_ref())
            .await
            .map_err(|e| {
                log::error!("Error creating pelaksana provinsi: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Pelaksana provinsi berhasil dibuat",
                "id": result.last_insert_id()
            })))
        }
        "Pelaksana Tingkat Pusat" => {
            let result = sqlx::query!(
                r#"
                INSERT INTO pelaksana_pusat
                (id_pdp, nama_lengkap, photo, jabatan)
                VALUES (?, ?, ?, ?)
                "#,
                id_pdp,
                nama_lengkap,
                photo_path,
                jabatan
            )
            .execute(pool.get_ref())
            .await
            .map_err(|e| {
                log::error!("Error creating pelaksana pusat: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Pelaksana pusat berhasil dibuat",
                "id": result.last_insert_id()
            })))
        }
        _ => Err(actix_web::error::ErrorBadRequest(
            "Tingkat kepengurusan tidak valid",
        )),
    }
}

#[put("/api/userpanel/pelaksana/{id}")]
pub async fn update_pelaksana(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // Ambil data user untuk mengetahui tingkat kepengurusan
    let user_data = sqlx::query!(
        r#"
        SELECT
            p.tingkat_kepengurusan,
            p.id_provinsi,
            p.id_kabupaten
        FROM pdp p
        WHERE p.id = ?
        "#,
        claims.id_pdp
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Error fetching user data: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let user_data = match user_data {
        Some(data) => data,
        None => {
            return Err(actix_web::error::ErrorNotFound("Data user tidak ditemukan"));
        }
    };

    let tingkat_kepengurusan = match user_data.tingkat_kepengurusan {
        Some(tingkat) => tingkat,
        None => {
            return Err(actix_web::error::ErrorBadRequest(
                "Tingkat kepengurusan tidak ditemukan",
            ));
        }
    };

    // Parse multipart form data
    let mut nama_lengkap = None;
    let mut jabatan = None;
    let mut id_pdp = None;
    let mut id_provinsi = None;
    let mut id_kabupaten = None;
    let mut photo_data = None;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "nama_lengkap" => {
                let bytes = field_to_bytes(&mut field).await?;
                let value = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid nama_lengkap: {}", e))
                })?;
                if !value.is_empty() {
                    nama_lengkap = Some(value);
                }
            }
            "jabatan" => {
                let bytes = field_to_bytes(&mut field).await?;
                let value = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid jabatan: {}", e))
                })?;
                if !value.is_empty() {
                    jabatan = Some(value);
                }
            }
            "id_pdp" => {
                let bytes = field_to_bytes(&mut field).await?;
                let value = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid id_pdp: {}", e))
                })?;
                if !value.is_empty() {
                    id_pdp = Some(value.parse::<String>().map_err(|e| {
                        actix_web::error::ErrorBadRequest(format!("Invalid id_pdp format: {}", e))
                    })?);
                }
            }
            "id_provinsi" => {
                let bytes = field_to_bytes(&mut field).await?;
                let value = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid id_provinsi: {}", e))
                })?;
                if !value.is_empty() {
                    id_provinsi = Some(value.parse::<i32>().map_err(|e| {
                        actix_web::error::ErrorBadRequest(format!(
                            "Invalid id_provinsi format: {}",
                            e
                        ))
                    })?);
                }
            }
            "id_kabupaten" => {
                let bytes = field_to_bytes(&mut field).await?;
                let value = String::from_utf8(bytes).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Invalid id_kabupaten: {}", e))
                })?;
                if !value.is_empty() {
                    id_kabupaten = Some(value.parse::<i32>().map_err(|e| {
                        actix_web::error::ErrorBadRequest(format!(
                            "Invalid id_kabupaten format: {}",
                            e
                        ))
                    })?);
                }
            }
            "photo" => {
                let content_type = field.content_type().map(|ct| ct.to_string());
                if let Some(ct) = content_type {
                    if ct.starts_with("image/") {
                        let bytes = field_to_bytes(&mut field).await?;
                        if !bytes.is_empty() {
                            photo_data = Some(bytes);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Handle photo upload
    let photo_path = if let Some(photo_bytes) = photo_data {
        Some(save_image(&photo_bytes, "pelaksana").await?)
    } else {
        None
    };

    // Update berdasarkan tingkat kepengurusan dengan dynamic query builder
    match tingkat_kepengurusan.as_str() {
        "Pelaksana Tingkat Kabupaten/Kota" => {
            let mut query = "UPDATE pelaksana_kabupaten SET ".to_string();
            let mut params: Vec<String> = Vec::new();
            let mut bind_values: Vec<String> = Vec::new();

            if let Some(nama) = &nama_lengkap {
                params.push("nama_lengkap = ?".to_string());
                bind_values.push(nama.clone());
            }
            if let Some(jab) = &jabatan {
                params.push("jabatan = ?".to_string());
                bind_values.push(jab.clone());
            }
            if let Some(pdp) = id_pdp {
                params.push("id_pdp = ?".to_string());
                bind_values.push(pdp.to_string());
            }
            if let Some(prov) = id_provinsi {
                params.push("id_provinsi = ?".to_string());
                bind_values.push(prov.to_string());
            }
            if let Some(kab) = id_kabupaten {
                params.push("id_kabupaten = ?".to_string());
                bind_values.push(kab.to_string());
            }
            if let Some(photo) = &photo_path {
                params.push("photo = ?".to_string());
                bind_values.push(photo.clone());
            }

            if params.is_empty() {
                return Err(actix_web::error::ErrorBadRequest(
                    "Tidak ada data yang diupdate",
                ));
            }

            query.push_str(&params.join(", "));
            query.push_str(" WHERE id = ?");
            bind_values.push(id.to_string());

            // Build query dengan bind parameters
            let mut dynamic_query = sqlx::query(&query);

            for value in bind_values {
                dynamic_query = dynamic_query.bind(value);
            }

            let result = dynamic_query.execute(pool.get_ref()).await.map_err(|e| {
                log::error!("Error updating pelaksana kabupaten: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

            if result.rows_affected() == 0 {
                return Err(actix_web::error::ErrorNotFound(
                    "Pelaksana kabupaten tidak ditemukan",
                ));
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Pelaksana kabupaten berhasil diupdate"
            })))
        }
        "Pelaksana Tingkat Provinsi" => {
            let mut query = "UPDATE pelaksana_provinsi SET ".to_string();
            let mut params: Vec<String> = Vec::new();
            let mut bind_values: Vec<String> = Vec::new();

            if let Some(nama) = &nama_lengkap {
                params.push("nama_lengkap = ?".to_string());
                bind_values.push(nama.clone());
            }
            if let Some(jab) = &jabatan {
                params.push("jabatan = ?".to_string());
                bind_values.push(jab.clone());
            }
            if let Some(pdp) = id_pdp {
                params.push("id_pdp = ?".to_string());
                bind_values.push(pdp.to_string());
            }
            if let Some(prov) = id_provinsi {
                params.push("id_provinsi = ?".to_string());
                bind_values.push(prov.to_string());
            }
            if let Some(photo) = &photo_path {
                params.push("photo = ?".to_string());
                bind_values.push(photo.clone());
            }

            if params.is_empty() {
                return Err(actix_web::error::ErrorBadRequest(
                    "Tidak ada data yang diupdate",
                ));
            }

            query.push_str(&params.join(", "));
            query.push_str(" WHERE id = ?");
            bind_values.push(id.to_string());

            let mut dynamic_query = sqlx::query(&query);

            for value in bind_values {
                dynamic_query = dynamic_query.bind(value);
            }

            let result = dynamic_query.execute(pool.get_ref()).await.map_err(|e| {
                log::error!("Error updating pelaksana provinsi: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

            if result.rows_affected() == 0 {
                return Err(actix_web::error::ErrorNotFound(
                    "Pelaksana provinsi tidak ditemukan",
                ));
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Pelaksana provinsi berhasil diupdate"
            })))
        }
        "Pelaksana Tingkat Pusat" => {
            let mut query = "UPDATE pelaksana_pusat SET ".to_string();
            let mut params: Vec<String> = Vec::new();
            let mut bind_values: Vec<String> = Vec::new();

            if let Some(nama) = &nama_lengkap {
                params.push("nama_lengkap = ?".to_string());
                bind_values.push(nama.clone());
            }
            if let Some(jab) = &jabatan {
                params.push("jabatan = ?".to_string());
                bind_values.push(jab.clone());
            }
            if let Some(pdp) = id_pdp {
                params.push("id_pdp = ?".to_string());
                bind_values.push(pdp.to_string());
            }
            if let Some(photo) = &photo_path {
                params.push("photo = ?".to_string());
                bind_values.push(photo.clone());
            }

            if params.is_empty() {
                return Err(actix_web::error::ErrorBadRequest(
                    "Tidak ada data yang diupdate",
                ));
            }

            query.push_str(&params.join(", "));
            query.push_str(" WHERE id = ?");
            bind_values.push(id.to_string());

            let mut dynamic_query = sqlx::query(&query);

            for value in bind_values {
                dynamic_query = dynamic_query.bind(value);
            }

            let result = dynamic_query.execute(pool.get_ref()).await.map_err(|e| {
                log::error!("Error updating pelaksana pusat: {}", e);
                actix_web::error::ErrorInternalServerError("Database error")
            })?;

            if result.rows_affected() == 0 {
                return Err(actix_web::error::ErrorNotFound(
                    "Pelaksana pusat tidak ditemukan",
                ));
            }

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Pelaksana pusat berhasil diupdate"
            })))
        }
        _ => Err(actix_web::error::ErrorBadRequest(
            "Tingkat kepengurusan tidak valid",
        )),
    }
}
// Helper function untuk membaca field multipart menjadi bytes
async fn field_to_bytes(field: &mut actix_multipart::Field) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.next().await {
        let data = chunk?;
        bytes.extend_from_slice(&data);
    }
    Ok(bytes)
}

async fn save_image(bytes: &[u8], prefix: &str) -> Result<String, Error> {
    // Determine image format
    let image_format = if bytes.len() >= 3 {
        match &bytes[0..3] {
            [0xFF, 0xD8, 0xFF] => "jpg",
            [0x89, 0x50, 0x4E] => "png",
            [0x47, 0x49, 0x46] => "gif",
            [0x52, 0x49, 0x46] => "webp",
            _ => "webp", // default to webp
        }
    } else {
        "webp"
    };

    let filename = format!(
        "{}_{}.{}",
        prefix,
        chrono::Utc::now().timestamp(),
        image_format
    );
    let directory = "./uploads/assets/pelaksana";
    let filepath = format!("{}/{}", directory, filename);

    // Create uploads directory if not exists
    tokio::fs::create_dir_all(&directory).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to create directory: {}", e))
    })?;

    // Save file
    tokio::fs::write(&filepath, bytes).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to save image: {}", e))
    })?;

    // Return path yang disimpan di database (tanpa ./uploads)
    Ok(format!("uploads/assets/pelaksana/{}", filename))
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GetUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub address: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub id_pdp: Option<String>,
    pub id_provinsi: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub nama_provinsi: Option<String>,
    pub nama_kabupaten: Option<String>,
    pub created_at: DateTime<chrono::Local>,
}

#[get("/api/adminpanel/get-all-user")]
pub async fn get_all_user(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !"Superadmin".contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let user = sqlx::query_as::<_, GetUser>(
        "SELECT
        u.id,
        u.name,
        u.email,
        u.role,
        u.address,
        u.avatar,
        u.phone,
        u.id_pdp,
        u.id_provinsi,
        u.id_kabupaten,
        u.created_at,
        p.nama_provinsi,
        k.nama_kabupaten
        FROM users u
        LEFT JOIN provinsi p ON u.id_provinsi = p.id
        LEFT JOIN kabupaten k ON u.id_kabupaten = k.id
        WHERE role != 'Superadmin' AND role !='Anggota'",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("User not found {}", e);
        actix_web::error::ErrorNotFound("User tidak ditemukan")
    })?;

    Ok(HttpResponse::Ok().json(user))
}

#[put("/api/adminpanel/edit-user/{id}")]
pub async fn update_user_by_id(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    mut multipart: Multipart,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }

    let id = path.into_inner();
    // Ambil data user lama termasuk avatar
    let user_old = sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, password, address, avatar, phone, email_verified_at, remember_token, id_pdp, id_provinsi, id_kabupaten, created_at FROM users WHERE id = ?"
    )
    .bind(id.clone())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("User not found for id {}: {}", id, e);
        actix_web::error::ErrorNotFound("User tidak ditemukan")
    })?;

    let id_user = user_old.id.clone();
    let old_avatar = user_old.avatar.clone();

    // Baca multipart: form fields + photo file
    let mut name: Option<String> = None;
    let mut email: Option<String> = None;
    let mut address: Option<String> = None;
    let mut phone: Option<String> = None;
    let mut id_pdp: Option<String> = None;
    let mut id_provinsi: Option<i32> = None;
    let mut id_kabupaten: Option<i32> = None;
    let mut role: Option<String> = None;
    let mut new_avatar_rel: Option<String> = None;
    let mut has_new_avatar = false;

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
            "name" | "email" | "address" | "phone" | "role" | "id_pdp" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                let value = String::from_utf8(bytes.to_vec())
                    .map_err(|_| actix_web::error::ErrorBadRequest("Field bukan UTF-8 valid"))?;

                match field_name.as_str() {
                    "name" => name = Some(value),
                    "email" => email = Some(value),
                    "address" => address = Some(value),
                    "phone" => phone = Some(value),
                    "role" => role = Some(value),
                    "id_pdp" => id_pdp = Some(value),
                    _ => unreachable!(),
                }
            }
            "id_provinsi" | "id_kabupaten" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field
                    .try_next()
                    .await
                    .map_err(actix_web::error::ErrorBadRequest)?
                {
                    bytes.extend_from_slice(&chunk);
                }
                let value_str = String::from_utf8(bytes.to_vec())
                    .map_err(|_| actix_web::error::ErrorBadRequest("Field bukan UTF-8 valid"))?;

                if !value_str.trim().is_empty() {
                    let value = value_str.parse::<i32>().map_err(|_| {
                        actix_web::error::ErrorBadRequest("Field numerik tidak valid")
                    })?;

                    match field_name.as_str() {
                        "id_provinsi" => id_provinsi = Some(value),
                        "id_kabupaten" => id_kabupaten = Some(value),
                        _ => unreachable!(),
                    }
                }
            }
            "avatar" => {
                // Validasi tipe file
                let content_type = field
                    .content_type()
                    .map(|mime| mime.essence_str().to_string())
                    .ok_or_else(|| {
                        actix_web::error::ErrorBadRequest("Tipe file tidak ditemukan")
                    })?;
                if !["image/jpeg", "image/png"].contains(&content_type.as_str()) {
                    return Err(actix_web::error::ErrorBadRequest(
                        "File harus berupa jpg, jpeg, atau png",
                    ));
                }

                // Validasi ukuran file
                let max_size = 2 * 1024 * 1024; // 2MB
                let mut file_size = 0;
                let mut file_data = Vec::new();

                while let Some(chunk) = field.try_next().await? {
                    file_size += chunk.len();
                    if file_size > max_size {
                        return Err(actix_web::error::ErrorBadRequest(
                            "Ukuran file melebihi 2MB",
                        ));
                    }
                    file_data.extend_from_slice(&chunk);
                }

                // Simpan file baru
                let extension = if content_type == "image/jpeg" {
                    "jpg"
                } else {
                    "png"
                };
                let filename = format!(
                    "uploads/assets/images/avatars/{}.{}",
                    uuid::Uuid::new_v4(),
                    extension
                );
                let filepath = std::path::Path::new(&filename);
                let mut f = fs::File::create(filepath).map_err(|e| {
                    log::error!("Gagal membuat file: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Gagal menyimpan file")
                })?;
                f.write_all(&file_data).map_err(|e| {
                    log::error!("Gagal menulis file: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Gagal menulis file")
                })?;
                new_avatar_rel = Some(filename);
                has_new_avatar = true;
                continue;
            }
            _ => {
                return Err(actix_web::error::ErrorBadRequest(format!(
                    "Field tidak dikenal: {}",
                    field_name
                )));
            }
        }
    }

    // Validasi required fields
    let new_name = name
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Field name wajib diisi"))?;
    let new_email = email
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Field email wajib diisi"))?;
    let new_address = address
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Field address wajib diisi"))?;
    let new_role = role
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Field role wajib diisi"))?;
    let new_phone = phone
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Field phone wajib diisi"))?;

    // Validasi format email
    if !new_email.contains('@') || new_email.trim().len() < 3 {
        return Err(actix_web::error::ErrorBadRequest(
            "Format email tidak valid",
        ));
    }

    // Cek jika email diubah, pastikan email baru tidak duplikat
    if new_email != user_old.email {
        let existing_user: Option<(i32,)> =
            sqlx::query_as("SELECT id FROM users WHERE email = ? AND id != ?")
                .bind(&new_email)
                .bind(user_old.id)
                .fetch_optional(pool.get_ref())
                .await
                .map_err(|e| {
                    log::error!("Database error checking email: {}", e);
                    actix_web::error::ErrorInternalServerError("Database error")
                })?;

        if existing_user.is_some() {
            return Err(actix_web::error::ErrorBadRequest(
                "Email sudah digunakan oleh user lain",
            ));
        }
    }

    // Validasi role
    if ![
        "Superadmin",
        "Administrator",
        "Anggota",
        "Pelaksana",
        "Admin Kesbangpol",
        "Jurnalis",
        "Majelis Pertimbangan DPPI",
    ]
    .contains(&new_role.as_str())
    {
        return Err(actix_web::error::ErrorBadRequest("Role tidak valid"));
    }

    // Update user profile
    let result = sqlx::query(
        "UPDATE users SET name = ?, email = ?, address = ?, role = ?, phone = ?, id_pdp = ?, id_provinsi = ?, id_kabupaten = ?, avatar = COALESCE(?, avatar), updated_at = NOW() WHERE id = ?"
    )
    .bind(&new_name)
    .bind(&new_email)
    .bind(&new_address)
    .bind(&new_role)
    .bind(&new_phone)
    .bind(id_pdp)
    .bind(id_provinsi)
    .bind(id_kabupaten)
    .bind(if has_new_avatar { new_avatar_rel.as_ref() } else { None })
    .bind(id_user)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Failed to update user: {}", e);
        actix_web::error::ErrorInternalServerError("Gagal memperbarui user")
    })?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("User tidak ditemukan"));
    }

    // Hapus avatar lama hanya jika ada avatar baru & path berbeda
    if has_new_avatar {
        if let (Some(new_rel), Some(old_rel)) = (new_avatar_rel.as_ref(), old_avatar.as_ref()) {
            if new_rel != old_rel && !old_rel.is_empty() {
                let old_abs = std::path::Path::new(".").join(old_rel.trim_start_matches('/'));
                if old_abs.exists() {
                    if let Err(e) = tokio::fs::remove_file(&old_abs).await {
                        log::warn!(
                            "Failed to delete old avatar file {}: {}",
                            old_abs.display(),
                            e
                        );
                    }
                }
            }
        }
    }

    #[derive(serde::Serialize)]
    struct UpdateProfileResponse {
        message: String,
        avatar: Option<String>,
        updated: bool,
    }

    Ok(HttpResponse::Ok().json(UpdateProfileResponse {
        message: "User berhasil diperbarui".into(),
        avatar: if has_new_avatar {
            new_avatar_rel
        } else {
            old_avatar
        },
        updated: true,
    }))
}

#[delete("/api/adminpanel/delete-user/{id}")]
pub async fn delete_user(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    path: web::Path<String>,
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
    let (old_avatar_opt,): (Option<String>,) =
        sqlx::query_as("SELECT avatar FROM users WHERE id = ?")
            .bind(&id)
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // Hapus row
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().body("Data tidak ditemukan"));
    }

    // Hapus file fisik
    if let Some(oldp) = old_avatar_opt {
        remove_file_if_exists(&oldp);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Berhasil dihapus",
        "id": id
    })))
}

#[post("/api/adminpanel/new-user")]
pub async fn new_add_user(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    // Verifikasi token dan pastikan pengguna adalah admin
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya superadmin atau administrator yang dapat mengakses",
        ));
    }
    let mut form = UserForm {
        name: String::new(),
        email: String::new(),
        password: None,
        role: String::new(),
        address: None,
        phone: None,
        id_pdp: None,
        id_provinsi: None,
        id_kabupaten: None,
    };
    let mut avatar_path: Option<String> = None;

    // Proses multipart data
    while let Some(mut field) = payload.try_next().await? {
        let field_name = field
            .name()
            .ok_or_else(|| actix_web::error::ErrorBadRequest("Field name tidak ditemukan"))?;

        match field_name {
            "avatar" => {
                // Validasi tipe file
                let content_type = field
                    .content_type()
                    .map(|mime| mime.essence_str().to_string())
                    .ok_or_else(|| {
                        actix_web::error::ErrorBadRequest("Tipe file tidak ditemukan")
                    })?;
                if !["image/jpeg", "image/png", "image/jpeg"].contains(&content_type.as_str()) {
                    return Err(actix_web::error::ErrorBadRequest(
                        "File harus berupa jpg, jpeg, atau png",
                    ));
                }

                // Validasi ukuran file
                let max_size = 40 * 1024 * 1024; // 4MB
                let mut file_size = 0;
                let mut file_data = Vec::new();

                while let Some(chunk) = field.try_next().await? {
                    file_size += chunk.len();
                    if file_size > max_size {
                        return Err(actix_web::error::ErrorBadRequest(
                            "Ukuran file melebihi 40MB",
                        ));
                    }
                    file_data.extend_from_slice(&chunk);
                }

                // Simpan file
                let extension = if content_type == "image/jpeg" {
                    "jpg"
                } else {
                    "png"
                };
                let filename = format!(
                    "uploads/assets/images/avatars/{}.{}",
                    uuid::Uuid::new_v4(),
                    extension
                );
                let filepath = std::path::Path::new(&filename);
                let mut f = fs::File::create(filepath).map_err(|e| {
                    log::error!("Gagal membuat file: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Gagal menyimpan file")
                })?;
                f.write_all(&file_data).map_err(|e| {
                    log::error!("Gagal menulis file: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Gagal menulis file")
                })?;
                avatar_path = Some(filename);
            }

            "name" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca name: {}", e))
                })?;
                form.name = String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode name: {}", e))
                })?;
            }
            "email" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca email: {}", e))
                })?;
                form.email = String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode email: {}", e))
                })?;
            }
            "password" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca password: {}", e))
                })?;
                form.password = Some(String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode password: {}", e))
                })?);
            }
            "role" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca role: {}", e))
                })?;
                form.role = String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode role: {}", e))
                })?;
            }
            "address" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca alamat: {}", e))
                })?;
                form.address = Some(String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode alamat: {}", e))
                })?);
            }
            "phone" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca phone: {}", e))
                })?;
                let phone = String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode phone: {}", e))
                })?;
                form.phone = Some(normalize_phone(&phone));
            }
            "id_pdp" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca id_pdp: {}", e))
                })?;
                let id_pdp = String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode id_pdp: {}", e))
                })?;
                form.id_pdp = id_pdp.parse::<String>().ok();
            }
            "id_provinsi" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca id_provinsi: {}", e))
                })?;
                let id_provinsi = String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode id_provinsi: {}", e))
                })?;
                form.id_provinsi = id_provinsi.parse::<i32>().ok();
            }
            "id_kabupaten" => {
                let data = field.try_collect::<Vec<_>>().await.map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal membaca id_kabupaten: {}", e))
                })?;
                let id_kabupaten = String::from_utf8(data.concat()).map_err(|e| {
                    actix_web::error::ErrorBadRequest(format!("Gagal decode id_kabupaten: {}", e))
                })?;
                form.id_kabupaten = id_kabupaten.parse::<i32>().ok();
            }
            _ => {
                return Err(actix_web::error::ErrorBadRequest(format!(
                    "Field tidak dikenal: {}",
                    field_name
                )));
            }
        }
    }

    // Validasi input
    if form.name.is_empty() || form.email.is_empty() || form.role.is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Name, email, dan role harus diisi",
        ));
    }

    // Hash password jika ada
    let hashed_password = match form.password {
        Some(password) if !password.is_empty() => {
            Some(hash(&password, DEFAULT_COST).map_err(|e| {
                log::error!("Gagal menghash password: {:?}", e);
                actix_web::error::ErrorInternalServerError("Gagal menghash password")
            })?)
        }
        _ => None,
    };
    let new_id = generate_short_uuid();
    // Insert user ke database
    sqlx::query(
        r#"
        INSERT INTO users (id, name, password, email, role, address, avatar, phone, id_pdp, id_provinsi, id_kabupaten)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(new_id.clone())
    .bind(&form.name)
    .bind(hashed_password.unwrap_or_default())
    .bind(&form.email)
    .bind(&form.role)
    .bind(&form.address)
    .bind(&avatar_path)
    .bind(&form.phone)
    .bind(&form.id_pdp)
    .bind(&form.id_provinsi)
    .bind(&form.id_kabupaten)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Gagal menyimpan pengguna: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyimpan pengguna")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "User berhasil ditambahkan"
    })))
}

fn generate_short_uuid() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    raw[..10].to_uppercase()
}
