// src/controllers/post_controller.rs
use crate::auth;
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, Result, delete, get, post, put,
    web::{self, Data, Path},
};
use chrono::Utc;
use futures::TryStreamExt;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row, prelude::FromRow};
use std::path::{Path as FsPath, PathBuf};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Serialize, FromRow, Debug)]
struct Pendidikan {
    id: i32,
    id_pdp: String,
    jenjang_pendidikan: String,
    nama_instansi_pendidikan: String,
    jurusan: Option<String>,
    tahun_masuk: u32,
    tahun_lulus: u32,
}

// DTO khusus input dari JSON (tanpa id & id_pdp)
#[derive(Deserialize, Debug)]
struct PendidikanIn {
    jenjang_pendidikan: String,
    nama_instansi_pendidikan: String,
    jurusan: Option<String>,
    tahun_masuk: i32,         // gunakan i32 agar mulus di MySQL
    tahun_lulus: Option<i32>, // sama
}

#[get("/api/userpanel/pendidikan/{id}")]
pub async fn get_pendidikan(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Anggota",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id = path.into_inner();

    let data_pendidikan=  sqlx::query_as::<_, Pendidikan>(
        "SELECT id, id_pdp, jenjang_pendidikan, nama_instansi_pendidikan, jurusan, tahun_masuk, tahun_lulus FROM pendidikan WHERE id_pdp = ? ",
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    Ok(HttpResponse::Ok().json(data_pendidikan))
}

#[post("/api/userpanel/pendidikan/{id}")]
pub async fn add_pendidikan(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    form: web::Json<PendidikanIn>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let pdp_id = path.into_inner();

    // ===== AuthZ =====
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let is_owner = claims.id_pdp.map(|pid| pid == pdp_id).unwrap_or(false);
    if !is_owner {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    // ===== Validasi sederhana =====
    if form.jenjang_pendidikan.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Jenjang pendidikan wajib diisi",
        ));
    }
    if form.nama_instansi_pendidikan.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Nama instansi pendidikan wajib diisi",
        ));
    }

    // ===== Mulai transaction =====
    let mut transaction = pool.begin().await.map_err(|e| {
        error!("DB transaction error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memulai transaksi database")
    })?;

    // ===== Insert data pendidikan =====
    let query = r#"
        INSERT INTO pendidikan
        (id_pdp, jenjang_pendidikan, nama_instansi_pendidikan, jurusan, tahun_masuk, tahun_lulus)
        VALUES (?, ?, ?, ?, ?, ?)
    "#;

    let res = sqlx::query(query)
        .bind(&pdp_id)
        .bind(form.jenjang_pendidikan.trim())
        .bind(form.nama_instansi_pendidikan.trim())
        .bind(&form.jurusan)
        .bind(form.tahun_masuk)
        .bind(form.tahun_lulus)
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            error!("DB insert pendidikan error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menyimpan data pendidikan")
        })?;

    let inserted_id = res.last_insert_id();

    // ===== Update status PDP menjadi "Simental" =====
    let update_status_query = r#"
        UPDATE pdp
        SET status = ?
        WHERE id = ?
    "#;

    sqlx::query(update_status_query)
        .bind("Simental")
        .bind(&pdp_id)
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            error!("DB update status PDP error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengupdate status PDP")
        })?;

    // ===== Commit transaction =====
    transaction.commit().await.map_err(|e| {
        error!("DB commit error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyelesaikan transaksi")
    })?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "inserted_id": inserted_id,
        "message": "Data pendidikan berhasil disimpan dan status PDP diperbarui"
    })))
}

// ===== DTO Update (tri-state untuk jurusan & tahun_lulus) =====
#[derive(Deserialize, Debug)]
pub struct PendidikanUpdate {
    pub jenjang_pendidikan: Option<String>, // None -> biarkan
    pub nama_instansi_pendidikan: Option<String>, // None -> biarkan
    pub jurusan: Option<Option<String>>, // Some(Some(v)) -> set v; Some(None) -> set NULL; None -> biarkan
    pub tahun_masuk: Option<i32>,        // None -> biarkan
    pub tahun_lulus: Option<Option<i32>>, // Some(Some(v)) -> set v; Some(None) -> set NULL; None -> biarkan
}

// ===== Helper: cek kepemilikan row pendidikan =====
async fn ensure_owner(req: &HttpRequest, pool: &MySqlPool, pendidikan_id: i32) -> Result<()> {
    let claims =
        auth::verify_jwt(req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let row = sqlx::query("SELECT id_pdp FROM pendidikan WHERE id = ?")
        .bind(pendidikan_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error!("DB select pendidikan.id_pdp error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengambil data")
        })?;

    let owner_id = row
        .ok_or_else(|| actix_web::error::ErrorNotFound("Data pendidikan tidak ditemukan"))?
        .get::<String, _>(0);

    let is_owner = claims.id_pdp.map(|pid| pid == owner_id).unwrap_or(false);
    if !is_owner {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }
    Ok(())
}

// ===== PUT: Update pendidikan by id =====
#[put("/api/userpanel/pendidikan/{id}")]
pub async fn update_pendidikan(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    form: web::Json<PendidikanUpdate>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let pendidikan_id = path.into_inner();

    // AuthZ & ownership
    ensure_owner(&req, pool.get_ref(), pendidikan_id).await?;

    // Normalisasi & validasi ringan
    let jenjang = form
        .jenjang_pendidikan
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned);

    let instansi = form
        .nama_instansi_pendidikan
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned);

    // jurusan: tri-state
    let jurusan_set = form.jurusan.is_some(); // apakah field dikirim?
    let jurusan_val: Option<String> = match &form.jurusan {
        Some(Some(s)) => {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_owned())
            }
        }
        Some(None) => None, // explicit NULL
        None => None,       // tidak dikirim → diabaikan di SQL via flag
    };

    // tahun_masuk: optional update + validasi range
    if let Some(tm) = form.tahun_masuk {
        if !(1900..=2100).contains(&tm) {
            return Err(actix_web::error::ErrorBadRequest("Tahun masuk tidak valid"));
        }
    }

    // tahun_lulus: tri-state + validasi
    let tahun_lulus_set = form.tahun_lulus.is_some();
    let tahun_lulus_val: Option<i32> = match form.tahun_lulus {
        Some(Some(v)) => {
            if !(1900..=2100).contains(&v) {
                return Err(actix_web::error::ErrorBadRequest("Tahun lulus tidak valid"));
            }
            Some(v)
        }
        Some(None) => None, // explicit NULL
        None => None,       // tidak dikirim
    };

    // Eksekusi update dengan flag tri-state
    let res = sqlx::query(
        r#"
        UPDATE pendidikan
        SET
            jenjang_pendidikan       = IFNULL(?, jenjang_pendidikan),
            nama_instansi_pendidikan = IFNULL(?, nama_instansi_pendidikan),
            jurusan                  = IF(? = 1, ?, jurusan),
            tahun_masuk              = IFNULL(?, tahun_masuk),
            tahun_lulus              = IF(? = 1, ?, tahun_lulus),
            updated_at               = NOW()
        WHERE id = ?
        "#,
    )
    .bind(jenjang) // ?
    .bind(instansi) // ?
    .bind(if jurusan_set { 1 } else { 0 }) // ? = flag set jurusan
    .bind(jurusan_val) // ?
    .bind(form.tahun_masuk) // ?
    .bind(if tahun_lulus_set { 1 } else { 0 }) // ? = flag set tahun_lulus
    .bind(tahun_lulus_val) // ?
    .bind(pendidikan_id) // ?
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        error!("DB update pendidikan error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memperbarui data pendidikan")
    })?;

    if res.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Data pendidikan tidak ditemukan",
        ));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "updated": res.rows_affected()
    })))
}

// ===== DELETE: Hapus pendidikan by id =====
#[delete("/api/userpanel/pendidikan/{id}")]
pub async fn delete_pendidikan(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let pendidikan_id = path.into_inner();

    // AuthZ & ownership
    ensure_owner(&req, pool.get_ref(), pendidikan_id).await?;

    let res = sqlx::query("DELETE FROM pendidikan WHERE id = ?")
        .bind(pendidikan_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            error!("DB delete pendidikan error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menghapus data pendidikan")
        })?;

    if res.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Data pendidikan tidak ditemukan",
        ));
    }

    // 204 No Content (kosong), atau 200 JSON — pilih salah satu.
    Ok(HttpResponse::NoContent().finish())
}
// ========== Model ==========
#[derive(Serialize, FromRow, Debug)]
struct DiklatResponse {
    id: i32,
    id_pdp: String,
    keterangan_diklat: String,
    sertifikat_diklat: String, // path relatif
    tahun_diklat: u32,
}

// ========== Konstanta ==========
const DIR_SERTIF: &str = "uploads/assets/sertifikat-diklat";
const MAX_FILE_BYTES: usize = 5 * 1024 * 1024; // 5MB
const ALLOWED_EXT: &[&str] = &["pdf", "jpg", "jpeg", "png", "webp"];

// ========== Helper Ownership ==========
async fn ensure_owner_diklat(req: &HttpRequest, pool: &MySqlPool, diklat_id: i32) -> Result<()> {
    let claims =
        auth::verify_jwt(req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let row = sqlx::query("SELECT id_pdp FROM diklat WHERE id = ?")
        .bind(diklat_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error!("DB select diklat.id_pdp error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengambil data")
        })?;

    let owner_id = row
        .ok_or_else(|| actix_web::error::ErrorNotFound("Data diklat tidak ditemukan"))?
        .get::<String, _>(0);

    let is_owner = claims.id_pdp.map(|pid| pid == owner_id).unwrap_or(false);
    if !is_owner {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }
    Ok(())
}

// ========== Helper Simpan Upload ==========
async fn save_upload_sertifikat(
    mut payload: Multipart,
) -> Result<(Option<String>, Option<String>, Option<i32>)> {
    // Pastikan folder ada
    if !FsPath::new(DIR_SERTIF).exists() {
        fs::create_dir_all(DIR_SERTIF).await.map_err(|e| {
            error!("Gagal membuat folder {}: {:?}", DIR_SERTIF, e);
            actix_web::error::ErrorInternalServerError("Gagal menyiapkan folder upload")
        })?;
    }

    let mut rel_path: Option<String> = None;
    let mut ket: Option<String> = None;
    let mut tahun: Option<i32> = None;

    while let Some(field) = payload.try_next().await.map_err(|e| {
        error!("Multipart stream error: {:?}", e);
        actix_web::error::ErrorBadRequest("Multipart tidak valid")
    })? {
        let cd = field.content_disposition().cloned();
        let name = cd.as_ref().and_then(|d| d.get_name()).unwrap_or("");

        // Text fields
        if name == "keterangan_diklat" {
            let bytes = field
                .try_fold(Vec::new(), |mut acc, b| async move {
                    acc.extend(b);
                    Ok(acc)
                })
                .await
                .map_err(|e| {
                    error!("Baca field keterangan_diklat gagal: {:?}", e);
                    actix_web::error::ErrorBadRequest("Field keterangan_diklat tidak valid")
                })?;
            ket = Some(String::from_utf8_lossy(&bytes).trim().to_string());
            continue;
        }
        if name == "tahun_diklat" {
            let bytes = field
                .try_fold(Vec::new(), |mut acc, b| async move {
                    acc.extend(b);
                    Ok(acc)
                })
                .await
                .map_err(|e| {
                    error!("Baca field tahun_diklat gagal: {:?}", e);
                    actix_web::error::ErrorBadRequest("Field tahun_diklat tidak valid")
                })?;
            let s = String::from_utf8_lossy(&bytes).trim().to_string();
            let v: i32 = s
                .parse()
                .map_err(|_| actix_web::error::ErrorBadRequest("tahun_diklat harus angka"))?;
            tahun = Some(v);
            continue;
        }

        // File field
        if name == "sertifikat_diklat" {
            // Nama file original (opsional)
            let filename = cd.as_ref().and_then(|d| d.get_filename()).unwrap_or("file");
            // cegah path traversal
            let sanitized = filename.replace(['\\', '/', ':', ';', '\0'], "_");

            // Ambil ekstensi
            let ext = FsPath::new(&sanitized)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_else(|| "bin".to_string());

            if !ALLOWED_EXT.contains(&ext.as_str()) {
                return Err(actix_web::error::ErrorBadRequest(
                    "Ekstensi file tidak diizinkan",
                ));
            }

            // Nama unik
            let ts = Utc::now().timestamp_millis();
            let unique = format!("{ts}_{sanitized}");
            let full_path = PathBuf::from(DIR_SERTIF).join(&unique);

            // Tulis file dengan limit ukuran
            let mut size: usize = 0;
            let mut f = fs::File::create(&full_path).await.map_err(|e| {
                error!("Gagal membuat file: {:?}", e);
                actix_web::error::ErrorInternalServerError("Gagal menyimpan file")
            })?;

            let mut stream = field;
            while let Some(chunk) = stream.try_next().await.map_err(|e| {
                error!("Gagal membaca chunk upload: {:?}", e);
                actix_web::error::ErrorBadRequest("Upload gagal")
            })? {
                size += chunk.len();
                if size > MAX_FILE_BYTES {
                    // hapus partial file
                    let _ = fs::remove_file(&full_path).await;
                    return Err(actix_web::error::ErrorBadRequest(
                        "Ukuran file melebihi 5MB",
                    ));
                }
                f.write_all(&chunk).await.map_err(|e| {
                    error!("Gagal menulis file: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Gagal menyimpan file")
                })?;
            }
            f.flush().await.ok();

            let rel = format!("{}/{}", DIR_SERTIF, unique);
            rel_path = Some(rel);
        }
    }

    Ok((rel_path, ket, tahun))
}

// ========== GET: list diklat by id_pdp ==========
#[get("/api/userpanel/diklat/{id}")]
pub async fn get_diklat(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<HttpResponse> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Anggota",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id_pdp = path.into_inner();

    // Jika role Anggota/Pelaksana, batasi ke pemilik saja
    if ["Pelaksana", "Anggota"].contains(&claims.role.as_str()) {
        let is_owner = claims.id_pdp.map(|pid| pid == id_pdp).unwrap_or(false);
        if !is_owner {
            return Err(actix_web::error::ErrorForbidden(
                "Anda tidak memiliki akses ke data ini!",
            ));
        }
    }

    let data = sqlx::query_as::<_, DiklatResponse>(
        "SELECT id, id_pdp, keterangan_diklat, sertifikat_diklat, tahun_diklat
         FROM diklat WHERE id_pdp = ? ORDER BY tahun_diklat DESC, id DESC",
    )
    .bind(id_pdp)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    Ok(HttpResponse::Ok().json(data))
}

// ========== POST: multipart (buat baru) ==========
#[post("/api/userpanel/diklat/{id}")]
pub async fn add_diklat(
    pool: Data<MySqlPool>,
    path: Path<String>,
    req: HttpRequest,
    payload: Multipart,
) -> Result<HttpResponse> {
    let id_pdp = path.into_inner();

    // Auth: pemilik saja
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    let is_owner = claims.id_pdp.map(|pid| pid == id_pdp).unwrap_or(false);
    if !is_owner {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let (rel_path, ket, tahun) = save_upload_sertifikat(payload).await?;

    let ket =
        ket.ok_or_else(|| actix_web::error::ErrorBadRequest("keterangan_diklat wajib diisi"))?;
    let tahun =
        tahun.ok_or_else(|| actix_web::error::ErrorBadRequest("tahun_diklat wajib diisi"))?;
    let sert = rel_path.ok_or_else(|| {
        actix_web::error::ErrorBadRequest("sertifikat_diklat (file) wajib diunggah")
    })?;

    if !(1900..=2100).contains(&tahun) {
        return Err(actix_web::error::ErrorBadRequest(
            "Tahun diklat tidak valid",
        ));
    }

    // Perbaikan: jumlah kolom & placeholder HARUS cocok (4 kolom -> 4 placeholder)
    let res = sqlx::query(
        r#"
        INSERT INTO diklat (id_pdp, keterangan_diklat, sertifikat_diklat, tahun_diklat)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&id_pdp)
    .bind(ket.trim())
    .bind(sert) // path relatif
    .bind(tahun)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        error!("DB insert diklat error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyimpan data diklat")
    })?;

    // ===== Mulai transaction =====
    let mut transaction = pool.begin().await.map_err(|e| {
        error!("DB transaction error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memulai transaksi database")
    })?;

    // ===== Update status PDP menjadi "Simental" =====
    let update_status_query = r#"
        UPDATE pdp
        SET status = ?
        WHERE id = ?
    "#;

    sqlx::query(update_status_query)
        .bind("Simental")
        .bind(&id_pdp)
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            error!("DB update status PDP error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengupdate status PDP")
        })?;

    // ===== Commit transaction =====
    transaction.commit().await.map_err(|e| {
        error!("DB commit error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyelesaikan transaksi")
    })?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "inserted_id": res.last_insert_id()
    })))
}

// ========== PUT: multipart (update sebagian, opsional ganti file) ==========
#[derive(Debug)]
struct DiklatUpdateFields {
    keterangan_diklat: Option<String>,
    tahun_diklat: Option<i32>,
    new_file_rel: Option<String>, // jika ada file baru
}

async fn parse_update_multipart(mut payload: Multipart) -> Result<DiklatUpdateFields> {
    let mut fields = DiklatUpdateFields {
        keterangan_diklat: None,
        tahun_diklat: None,
        new_file_rel: None,
    };

    if !FsPath::new(DIR_SERTIF).exists() {
        fs::create_dir_all(DIR_SERTIF).await.map_err(|e| {
            error!("Gagal membuat folder {}: {:?}", DIR_SERTIF, e);
            actix_web::error::ErrorInternalServerError("Gagal menyiapkan folder upload")
        })?;
    }

    while let Some(field) = payload.try_next().await.map_err(|e| {
        error!("Multipart stream error: {:?}", e);
        actix_web::error::ErrorBadRequest("Multipart tidak valid")
    })? {
        let cd = field.content_disposition().cloned();
        let name = cd.as_ref().and_then(|d| d.get_name()).unwrap_or("");

        if name == "keterangan_diklat" {
            let bytes = field
                .try_fold(Vec::new(), |mut acc, b| async move {
                    acc.extend(b);
                    Ok(acc)
                })
                .await?;
            fields.keterangan_diklat = Some(String::from_utf8_lossy(&bytes).trim().to_string());
            continue;
        }
        if name == "tahun_diklat" {
            let bytes = field
                .try_fold(Vec::new(), |mut acc, b| async move {
                    acc.extend(b);
                    Ok(acc)
                })
                .await?;
            let s = String::from_utf8_lossy(&bytes).trim().to_string();
            let v: i32 = s
                .parse()
                .map_err(|_| actix_web::error::ErrorBadRequest("tahun_diklat harus angka"))?;
            fields.tahun_diklat = Some(v);
            continue;
        }
        if name == "sertifikat_diklat" {
            // simpan file baru
            let filename = cd.as_ref().and_then(|d| d.get_filename()).unwrap_or("file");
            let sanitized = filename.replace(['\\', '/', ':', ';', '\0'], "_");
            let ext = FsPath::new(&sanitized)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_else(|| "bin".to_string());

            if !ALLOWED_EXT.contains(&ext.as_str()) {
                return Err(actix_web::error::ErrorBadRequest(
                    "Ekstensi file tidak diizinkan",
                ));
            }

            let ts = Utc::now().timestamp_millis();
            let unique = format!("{ts}_{sanitized}");
            let full_path = PathBuf::from(DIR_SERTIF).join(&unique);

            let mut size: usize = 0;
            let mut f = fs::File::create(&full_path).await.map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Gagal buat file: {e}"))
            })?;

            let mut stream = field;
            while let Some(chunk) = stream.try_next().await? {
                size += chunk.len();
                if size > MAX_FILE_BYTES {
                    let _ = fs::remove_file(&full_path).await;
                    return Err(actix_web::error::ErrorBadRequest(
                        "Ukuran file melebihi 5MB",
                    ));
                }
                f.write_all(&chunk).await.map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!("Gagal tulis file: {e}"))
                })?;
            }
            f.flush().await.ok();

            fields.new_file_rel = Some(format!("{}/{}", DIR_SERTIF, unique));
        }
    }

    Ok(fields)
}

#[put("/api/userpanel/diklat/{id}")] // id = id diklat (row)
pub async fn update_diklat(
    pool: Data<MySqlPool>,
    path: Path<i32>,
    req: HttpRequest,
    payload: Multipart, // multipart, file opsional
) -> Result<HttpResponse> {
    let diklat_id = path.into_inner();

    // AuthZ & ownership
    ensure_owner_diklat(&req, pool.get_ref(), diklat_id).await?;

    // Ambil data lama (untuk hapus file lama jika diganti)
    let old = sqlx::query("SELECT sertifikat_diklat FROM diklat WHERE id = ?")
        .bind(diklat_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {e}")))?;

    let old_path: Option<String> = old.map(|r| r.get::<String, _>(0));

    // Parse multipart
    let upd = parse_update_multipart(payload).await?;

    if let Some(tm) = upd.tahun_diklat {
        if !(1900..=2100).contains(&tm) {
            return Err(actix_web::error::ErrorBadRequest(
                "Tahun diklat tidak valid",
            ));
        }
    }

    // Lakukan update
    let res = sqlx::query(
        r#"
        UPDATE diklat
        SET
            keterangan_diklat = IFNULL(?, keterangan_diklat),
            tahun_diklat      = IFNULL(?, tahun_diklat),
            sertifikat_diklat = IFNULL(?, sertifikat_diklat),
            updated_at        = NOW()
        WHERE id = ?
        "#,
    )
    .bind(upd.keterangan_diklat.as_deref()) // Option<&str>
    .bind(upd.tahun_diklat) // Option<i32>
    .bind(upd.new_file_rel.as_deref()) // Option<&str>
    .bind(diklat_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        error!("DB update diklat error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memperbarui data diklat")
    })?;

    if res.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Data diklat tidak ditemukan",
        ));
    }

    // Jika ada file baru, hapus file lama
    if let Some(newp) = upd.new_file_rel {
        if let Some(oldp) = old_path {
            if oldp != newp {
                if FsPath::new(&oldp).exists() {
                    let _ = fs::remove_file(&oldp).await;
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "updated": res.rows_affected()
    })))
}

// ========== DELETE: hapus row & file ==========
#[delete("/api/userpanel/diklat/{id}")]
pub async fn delete_diklat(
    pool: Data<MySqlPool>,
    path: Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let diklat_id = path.into_inner();

    // AuthZ & ownership
    ensure_owner_diklat(&req, pool.get_ref(), diklat_id).await?;

    // Ambil path file sebelum delete
    let row = sqlx::query("SELECT sertifikat_diklat FROM diklat WHERE id = ?")
        .bind(diklat_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {e}")))?;

    let file_path: Option<String> = row.map(|r| r.get::<String, _>(0));

    let res = sqlx::query("DELETE FROM diklat WHERE id = ?")
        .bind(diklat_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            error!("DB delete diklat error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menghapus data diklat")
        })?;

    if res.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Data diklat tidak ditemukan",
        ));
    }

    if let Some(p) = file_path {
        if FsPath::new(&p).exists() {
            let _ = fs::remove_file(&p).await;
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, FromRow, Debug)]
struct PenghargaanResponse {
    id: i32,
    id_pdp: String,
    keterangan_penghargaan: String,
    sertifikat_penghargaan: String, // path relatif
    tahun_penghargaan: u32,
}

// ========== Helper Ownership ==========
async fn ensure_owner_penghargaan(
    req: &HttpRequest,
    pool: &MySqlPool,
    penghargaan_id: i32,
) -> Result<()> {
    let claims =
        auth::verify_jwt(req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let row = sqlx::query("SELECT id_pdp FROM penghargaan WHERE id = ?")
        .bind(penghargaan_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error!("DB select penghargaan.id_pdp error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengambil data")
        })?;

    let owner_id = row
        .ok_or_else(|| actix_web::error::ErrorNotFound("Data penghargaan tidak ditemukan"))?
        .get::<String, _>(0);

    let is_owner = claims.id_pdp.map(|pid| pid == owner_id).unwrap_or(false);
    if !is_owner {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }
    Ok(())
}

// ========== Helper Simpan Upload ==========
async fn save_file_sertifikat(
    mut payload: Multipart,
) -> Result<(Option<String>, Option<String>, Option<i32>)> {
    // Pastikan folder ada
    if !FsPath::new(DIR_SERTIF).exists() {
        fs::create_dir_all(DIR_SERTIF).await.map_err(|e| {
            error!("Gagal membuat folder {}: {:?}", DIR_SERTIF, e);
            actix_web::error::ErrorInternalServerError("Gagal menyiapkan folder upload")
        })?;
    }

    let mut rel_path: Option<String> = None;
    let mut ket: Option<String> = None;
    let mut tahun: Option<i32> = None;

    while let Some(field) = payload.try_next().await.map_err(|e| {
        error!("Multipart stream error: {:?}", e);
        actix_web::error::ErrorBadRequest("Multipart tidak valid")
    })? {
        let cd = field.content_disposition().cloned();
        let name = cd.as_ref().and_then(|d| d.get_name()).unwrap_or("");

        // Text fields
        if name == "keterangan_penghargaan" {
            let bytes = field
                .try_fold(Vec::new(), |mut acc, b| async move {
                    acc.extend(b);
                    Ok(acc)
                })
                .await
                .map_err(|e| {
                    error!("Baca field keterangan_penghargaan gagal: {:?}", e);
                    actix_web::error::ErrorBadRequest("Field keterangan_penghargaan tidak valid")
                })?;
            ket = Some(String::from_utf8_lossy(&bytes).trim().to_string());
            continue;
        }
        if name == "tahun_penghargaan" {
            let bytes = field
                .try_fold(Vec::new(), |mut acc, b| async move {
                    acc.extend(b);
                    Ok(acc)
                })
                .await
                .map_err(|e| {
                    error!("Baca field tahun_penghargaan gagal: {:?}", e);
                    actix_web::error::ErrorBadRequest("Field tahun_penghargaan tidak valid")
                })?;
            let s = String::from_utf8_lossy(&bytes).trim().to_string();
            let v: i32 = s
                .parse()
                .map_err(|_| actix_web::error::ErrorBadRequest("tahun_penghargaan harus angka"))?;
            tahun = Some(v);
            continue;
        }

        // File field
        if name == "sertifikat_penghargaan" {
            // Nama file original (opsional)
            let filename = cd.as_ref().and_then(|d| d.get_filename()).unwrap_or("file");
            // cegah path traversal
            let sanitized = filename.replace(['\\', '/', ':', ';', '\0'], "_");

            // Ambil ekstensi
            let ext = FsPath::new(&sanitized)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_else(|| "bin".to_string());

            if !ALLOWED_EXT.contains(&ext.as_str()) {
                return Err(actix_web::error::ErrorBadRequest(
                    "Ekstensi file tidak diizinkan",
                ));
            }

            // Nama unik
            let ts = Utc::now().timestamp_millis();
            let unique = format!("{ts}_{sanitized}");
            let full_path = PathBuf::from(DIR_SERTIF).join(&unique);

            // Tulis file dengan limit ukuran
            let mut size: usize = 0;
            let mut f = fs::File::create(&full_path).await.map_err(|e| {
                error!("Gagal membuat file: {:?}", e);
                actix_web::error::ErrorInternalServerError("Gagal menyimpan file")
            })?;

            let mut stream = field;
            while let Some(chunk) = stream.try_next().await.map_err(|e| {
                error!("Gagal membaca chunk upload: {:?}", e);
                actix_web::error::ErrorBadRequest("Upload gagal")
            })? {
                size += chunk.len();
                if size > MAX_FILE_BYTES {
                    // hapus partial file
                    let _ = fs::remove_file(&full_path).await;
                    return Err(actix_web::error::ErrorBadRequest(
                        "Ukuran file melebihi 5MB",
                    ));
                }
                f.write_all(&chunk).await.map_err(|e| {
                    error!("Gagal menulis file: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Gagal menyimpan file")
                })?;
            }
            f.flush().await.ok();

            let rel = format!("{}/{}", DIR_SERTIF, unique);
            rel_path = Some(rel);
        }
    }

    Ok((rel_path, ket, tahun))
}

// ========== GET: list penghargaan by id_pdp ==========
#[get("/api/userpanel/penghargaan/{id}")]
pub async fn get_penghargaan(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<HttpResponse> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Anggota",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id_pdp = path.into_inner();

    // Jika role Anggota/Pelaksana, batasi ke pemilik saja
    if ["Pelaksana", "Anggota"].contains(&claims.role.as_str()) {
        let is_owner = claims.id_pdp.map(|pid| pid == id_pdp).unwrap_or(false);
        if !is_owner {
            return Err(actix_web::error::ErrorForbidden(
                "Anda tidak memiliki akses ke data ini!",
            ));
        }
    }

    let data = sqlx::query_as::<_, PenghargaanResponse>(
        "SELECT id, id_pdp, keterangan_penghargaan, sertifikat_penghargaan, tahun_penghargaan
         FROM penghargaan WHERE id_pdp = ? ORDER BY tahun_penghargaan DESC, id DESC",
    )
    .bind(id_pdp)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    Ok(HttpResponse::Ok().json(data))
}

// ========== POST: multipart (buat baru) ==========
#[post("/api/userpanel/penghargaan/{id}")]
pub async fn add_penghargaan(
    pool: Data<MySqlPool>,
    path: Path<String>,
    req: HttpRequest,
    payload: Multipart,
) -> Result<HttpResponse> {
    let id_pdp = path.into_inner();

    // Auth: pemilik saja
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    let is_owner = claims.id_pdp.map(|pid| pid == id_pdp).unwrap_or(false);
    if !is_owner {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let (rel_path, ket, tahun) = save_file_sertifikat(payload).await?;

    let ket =
        ket.ok_or_else(|| actix_web::error::ErrorBadRequest("keterangan_penghargaan wajib diisi"))?;
    let tahun =
        tahun.ok_or_else(|| actix_web::error::ErrorBadRequest("tahun_penghargaan wajib diisi"))?;
    let sert = rel_path.ok_or_else(|| {
        actix_web::error::ErrorBadRequest("sertifikat_penghargaan (file) wajib diunggah")
    })?;

    if !(1900..=2100).contains(&tahun) {
        return Err(actix_web::error::ErrorBadRequest(
            "Tahun penghargaan tidak valid",
        ));
    }

    // Perbaikan: jumlah kolom & placeholder HARUS cocok (4 kolom -> 4 placeholder)
    let res = sqlx::query(
        r#"
        INSERT INTO penghargaan (id_pdp, keterangan_penghargaan, sertifikat_penghargaan, tahun_penghargaan)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&id_pdp)
    .bind(ket.trim())
    .bind(sert) // path relatif
    .bind(tahun)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        error!("DB insert penghargaan error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyimpan data penghargaan")
    })?;

    // ===== Mulai transaction =====
    let mut transaction = pool.begin().await.map_err(|e| {
        error!("DB transaction error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memulai transaksi database")
    })?;

    // ===== Update status PDP menjadi "Simental" =====
    let update_status_query = r#"
        UPDATE pdp
        SET status = ?
        WHERE id = ?
    "#;

    sqlx::query(update_status_query)
        .bind("Simental")
        .bind(&id_pdp)
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            error!("DB update status PDP error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengupdate status PDP")
        })?;

    // ===== Commit transaction =====
    transaction.commit().await.map_err(|e| {
        error!("DB commit error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyelesaikan transaksi")
    })?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "inserted_id": res.last_insert_id()
    })))
}

// ========== PUT: multipart (update sebagian, opsional ganti file) ==========
#[derive(Debug)]
struct PenghargaanUpdateFields {
    keterangan_penghargaan: Option<String>,
    tahun_penghargaan: Option<i32>,
    new_file_rel: Option<String>, // jika ada file baru
}

async fn parse_file_multipart(mut payload: Multipart) -> Result<PenghargaanUpdateFields> {
    let mut fields = PenghargaanUpdateFields {
        keterangan_penghargaan: None,
        tahun_penghargaan: None,
        new_file_rel: None,
    };

    if !FsPath::new(DIR_SERTIF).exists() {
        fs::create_dir_all(DIR_SERTIF).await.map_err(|e| {
            error!("Gagal membuat folder {}: {:?}", DIR_SERTIF, e);
            actix_web::error::ErrorInternalServerError("Gagal menyiapkan folder upload")
        })?;
    }

    while let Some(field) = payload.try_next().await.map_err(|e| {
        error!("Multipart stream error: {:?}", e);
        actix_web::error::ErrorBadRequest("Multipart tidak valid")
    })? {
        let cd = field.content_disposition().cloned();
        let name = cd.as_ref().and_then(|d| d.get_name()).unwrap_or("");

        if name == "keterangan_penghargaan" {
            let bytes = field
                .try_fold(Vec::new(), |mut acc, b| async move {
                    acc.extend(b);
                    Ok(acc)
                })
                .await?;
            fields.keterangan_penghargaan =
                Some(String::from_utf8_lossy(&bytes).trim().to_string());
            continue;
        }
        if name == "tahun_penghargaan" {
            let bytes = field
                .try_fold(Vec::new(), |mut acc, b| async move {
                    acc.extend(b);
                    Ok(acc)
                })
                .await?;
            let s = String::from_utf8_lossy(&bytes).trim().to_string();
            let v: i32 = s
                .parse()
                .map_err(|_| actix_web::error::ErrorBadRequest("tahun_penghargaan harus angka"))?;
            fields.tahun_penghargaan = Some(v);
            continue;
        }
        if name == "sertifikat_penghargaan" {
            // simpan file baru
            let filename = cd.as_ref().and_then(|d| d.get_filename()).unwrap_or("file");
            let sanitized = filename.replace(['\\', '/', ':', ';', '\0'], "_");
            let ext = FsPath::new(&sanitized)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_else(|| "bin".to_string());

            if !ALLOWED_EXT.contains(&ext.as_str()) {
                return Err(actix_web::error::ErrorBadRequest(
                    "Ekstensi file tidak diizinkan",
                ));
            }

            let ts = Utc::now().timestamp_millis();
            let unique = format!("{ts}_{sanitized}");
            let full_path = PathBuf::from(DIR_SERTIF).join(&unique);

            let mut size: usize = 0;
            let mut f = fs::File::create(&full_path).await.map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Gagal buat file: {e}"))
            })?;

            let mut stream = field;
            while let Some(chunk) = stream.try_next().await? {
                size += chunk.len();
                if size > MAX_FILE_BYTES {
                    let _ = fs::remove_file(&full_path).await;
                    return Err(actix_web::error::ErrorBadRequest(
                        "Ukuran file melebihi 5MB",
                    ));
                }
                f.write_all(&chunk).await.map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!("Gagal tulis file: {e}"))
                })?;
            }
            f.flush().await.ok();

            fields.new_file_rel = Some(format!("{}/{}", DIR_SERTIF, unique));
        }
    }

    Ok(fields)
}

#[put("/api/userpanel/penghargaan/{id}")] // id = id penghargaan (row)
pub async fn update_penghargaan(
    pool: Data<MySqlPool>,
    path: Path<i32>,
    req: HttpRequest,
    payload: Multipart, // multipart, file opsional
) -> Result<HttpResponse> {
    let penghargaan_id = path.into_inner();

    // AuthZ & ownership
    ensure_owner_penghargaan(&req, pool.get_ref(), penghargaan_id).await?;

    // Ambil data lama (untuk hapus file lama jika diganti)
    let old = sqlx::query("SELECT sertifikat_penghargaan FROM penghargaan WHERE id = ?")
        .bind(penghargaan_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {e}")))?;

    let old_path: Option<String> = old.map(|r| r.get::<String, _>(0));

    // Parse multipart
    let upd = parse_file_multipart(payload).await?;

    if let Some(tm) = upd.tahun_penghargaan {
        if !(1900..=2100).contains(&tm) {
            return Err(actix_web::error::ErrorBadRequest(
                "Tahun penghargaan tidak valid",
            ));
        }
    }

    // Lakukan update
    let res = sqlx::query(
        r#"
        UPDATE penghargaan
        SET
            keterangan_penghargaan = IFNULL(?, keterangan_penghargaan),
            tahun_penghargaan      = IFNULL(?, tahun_penghargaan),
            sertifikat_penghargaan = IFNULL(?, sertifikat_penghargaan),
            updated_at        = NOW()
        WHERE id = ?
        "#,
    )
    .bind(upd.keterangan_penghargaan.as_deref()) // Option<&str>
    .bind(upd.tahun_penghargaan) // Option<i32>
    .bind(upd.new_file_rel.as_deref()) // Option<&str>
    .bind(penghargaan_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        error!("DB update penghargaan error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memperbarui data penghargaan")
    })?;

    if res.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Data penghargaan tidak ditemukan",
        ));
    }

    // Jika ada file baru, hapus file lama
    if let Some(newp) = upd.new_file_rel {
        if let Some(oldp) = old_path {
            if oldp != newp {
                if FsPath::new(&oldp).exists() {
                    let _ = fs::remove_file(&oldp).await;
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "updated": res.rows_affected()
    })))
}

// ========== DELETE: hapus row & file ==========
#[delete("/api/userpanel/penghargaan/{id}")]
pub async fn delete_penghargaan(
    pool: Data<MySqlPool>,
    path: Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let penghargaan_id = path.into_inner();

    // AuthZ & ownership
    ensure_owner_penghargaan(&req, pool.get_ref(), penghargaan_id).await?;

    // Ambil path file sebelum delete
    let row = sqlx::query("SELECT sertifikat_penghargaan FROM penghargaan WHERE id = ?")
        .bind(penghargaan_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {e}")))?;

    let file_path: Option<String> = row.map(|r| r.get::<String, _>(0));

    let res = sqlx::query("DELETE FROM penghargaan WHERE id = ?")
        .bind(penghargaan_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            error!("DB delete penghargaan error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menghapus data penghargaan")
        })?;

    if res.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Data penghargaan tidak ditemukan",
        ));
    }

    if let Some(p) = file_path {
        if FsPath::new(&p).exists() {
            let _ = fs::remove_file(&p).await;
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

// =============================================== ORGANISASI ===========================================
#[derive(Serialize, FromRow, Debug)]
struct Organisasi {
    id: i32,
    id_pdp: String,
    nama_organisasi: String,
    posisi: String,
    status: Option<String>,
    tahun_masuk: u32,
    tahun_keluar: Option<u32>,
}

// DTO input
#[derive(Deserialize, Debug)]
struct OrganisasiIn {
    nama_organisasi: String,
    posisi: String,
    status: Option<String>,
    tahun_masuk: i32,
    tahun_keluar: Option<i32>,
}

// DTO update (sudah benar tri-state)
#[derive(Deserialize, Debug)]
pub struct OrganisasiUpdate {
    pub nama_organisasi: Option<String>,
    pub posisi: Option<String>,
    pub status: Option<Option<String>>,
    pub tahun_masuk: Option<i32>,
    pub tahun_keluar: Option<Option<i32>>,
}
async fn ensure_owner_organisasi(
    req: &HttpRequest,
    pool: &MySqlPool,
    organisasi_id: i32,
) -> Result<()> {
    let claims =
        auth::verify_jwt(req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    let row = sqlx::query("SELECT id_pdp FROM organisasi WHERE id = ?")
        .bind(organisasi_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error!("DB select organisasi.id_pdp error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengambil data")
        })?;
    let owner_id = row
        .ok_or_else(|| actix_web::error::ErrorNotFound("Data organisasi tidak ditemukan"))?
        .get::<String, _>(0);
    let is_owner = claims.id_pdp.map(|pid| pid == owner_id).unwrap_or(false);
    if !is_owner {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }
    Ok(())
}

#[get("/api/userpanel/organisasi/{id}")]
pub async fn get_organisasi(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>, // <- i32 saja biar konsisten dengan kolom id_pdp
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Anggota",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id_pdp = path.into_inner();

    let data_organisasi = sqlx::query_as::<_, Organisasi>(
        r#"
        SELECT id, id_pdp, nama_organisasi, posisi, status, tahun_masuk, tahun_keluar
        FROM organisasi
        WHERE id_pdp = ?
        ORDER BY tahun_masuk DESC, id DESC
        "#,
    )
    .bind(id_pdp)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    Ok(HttpResponse::Ok().json(data_organisasi))
}

#[post("/api/userpanel/organisasi/{id}")]
pub async fn add_organisasi(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    form: web::Json<OrganisasiIn>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let pdp_id = path.into_inner();

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    let is_owner = claims.id_pdp.map(|pid| pid == pdp_id).unwrap_or(false);
    if !is_owner {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    if form.nama_organisasi.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Nama organisasi wajib diisi",
        ));
    }
    if form.posisi.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Posisi/jabatan wajib diisi",
        ));
    }
    if !(1900..=2100).contains(&form.tahun_masuk) {
        return Err(actix_web::error::ErrorBadRequest("Tahun masuk tidak valid"));
    }
    if let Some(tk) = form.tahun_keluar {
        if !(1900..=2100).contains(&tk) {
            return Err(actix_web::error::ErrorBadRequest(
                "Tahun keluar tidak valid",
            ));
        }
        if tk < form.tahun_masuk {
            return Err(actix_web::error::ErrorBadRequest(
                "Tahun keluar tidak boleh < tahun masuk",
            ));
        }
    }

    let query = r#"
        INSERT INTO organisasi
        (id_pdp, nama_organisasi, posisi, status, tahun_masuk, tahun_keluar)
        VALUES (?, ?, ?, ?, ?, ?)
    "#;

    let res = sqlx::query(query)
        .bind(&pdp_id)
        .bind(form.nama_organisasi.trim())
        .bind(form.posisi.trim())
        .bind(&form.status) // Option<String> -> NULLable ok
        .bind(form.tahun_masuk) // i32
        .bind(form.tahun_keluar) // Option<i32> -> NULLable ok
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            error!("DB insert organisasi error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menyimpan data organisasi")
        })?;

    // ===== Mulai transaction =====
    let mut transaction = pool.begin().await.map_err(|e| {
        error!("DB transaction error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memulai transaksi database")
    })?;

    // ===== Update status PDP menjadi "Simental" =====
    let update_status_query = r#"
        UPDATE pdp
        SET status = ?
        WHERE id = ?
    "#;

    sqlx::query(update_status_query)
        .bind("Simental")
        .bind(&pdp_id)
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            error!("DB update status PDP error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengupdate status PDP")
        })?;

    // ===== Commit transaction =====
    transaction.commit().await.map_err(|e| {
        error!("DB commit error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menyelesaikan transaksi")
    })?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "inserted_id": res.last_insert_id()
    })))
}

#[put("/api/userpanel/organisasi/{id}")]
pub async fn update_organisasi(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    form: web::Json<OrganisasiUpdate>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let organisasi_id = path.into_inner();
    // pastikan pemilik
    ensure_owner_organisasi(&req, pool.get_ref(), organisasi_id).await?;

    let res = sqlx::query(
        r#"
        UPDATE organisasi
        SET
            nama_organisasi = ?,
            posisi = ?,
            status = ?,
            tahun_masuk = ?,
            tahun_keluar = ?,
            updated_at = NOW()
        WHERE id = ?
        "#,
    )
    .bind(&form.nama_organisasi)
    .bind(&form.posisi)
    .bind(&form.status)
    .bind(&form.tahun_masuk)
    .bind(&form.tahun_keluar)
    .bind(organisasi_id)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        error!("DB update organisasi error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal memperbarui data organisasi")
    })?;

    if res.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Data organisasi tidak ditemukan",
        ));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "updated": res.rows_affected()
    })))
}

// ===== DELETE: Hapus organisasi by id =====
#[delete("/api/userpanel/organisasi/{id}")]
pub async fn delete_organisasi(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let organisasi_id = path.into_inner();
    // AuthZ & ownership
    ensure_owner_organisasi(&req, pool.get_ref(), organisasi_id).await?;

    let res = sqlx::query("DELETE FROM organisasi WHERE id = ?")
        .bind(organisasi_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            error!("DB delete organisasi error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal menghapus data organisasi")
        })?;
    if res.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound(
            "Data organisasi tidak ditemukan",
        ));
    }
    // 204 No Content (kosong), atau 200 JSON — pilih salah satu.
    Ok(HttpResponse::NoContent().finish())
}

// =============================================== KEGIATAN ===========================================
#[derive(Serialize, FromRow, Debug)]
struct Kegiatan {
    id: i32,
    id_pdp: String,
    id_kegiatan: i32,
    kode_pendaftaran: String,
    nama_kegiatan: String,
    bukti_pembayaran: String,
    status: Option<String>,
    tanggal: u32,
    biaya: i32,
    jumlah_pembayaran: Option<i32>,
}

#[get("/api/userpanel/kegiatan/{id}")]
pub async fn get_kegiatan(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Anggota",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id_pdp = path.into_inner();

    let data_kegiatan = sqlx::query_as::<_, Kegiatan>(
        r#"
        SELECT id, id_pdp, id_kegiatan, kode_pendaftaran, nama_kegiatan, tanggal, biaya, bukti_pembayaran, jumlah_pembayaran, status
        FROM kegiatan_pdp
        WHERE id_pdp = ?
        ORDER BY tanggal DESC, id DESC
        "#,
    )
    .bind(id_pdp)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    Ok(HttpResponse::Ok().json(data_kegiatan))
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct Ketum {
    nama_lengkap: String,
    jabatan: Option<String>,
}

#[get("/api/userpanel/ketum")]
pub async fn get_ketum(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Data
    let data = sqlx::query_as::<_, Ketum>(
        r#"
        SELECT nama_lengkap, jabatan
        FROM pelaksana_pusat
        WHERE jabatan = "Ketua Umum"
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    Ok(HttpResponse::Ok().json(data))
}
