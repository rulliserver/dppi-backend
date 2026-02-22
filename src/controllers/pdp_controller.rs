<<<<<<< HEAD
use crate::{
    auth,
    controllers::pelaksana_controller::remove_file_if_exists,
    utils::{self, send_rejection_email, send_verified_email},
};
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, delete, get, put,
    web::{self, Data, Path},
};
use bcrypt::{DEFAULT_COST, hash};
use chrono::NaiveDate;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;
use sqlx::{FromRow, MySqlPool, Row, mysql::MySqlRow};
use std::{env, path::Path as Jalur};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
// =======================
// Models
// =======================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pdp {
    pub id: String,
    pub no_simental: Option<String>,
    pub no_piagam: Option<String>,
    pub nik: Option<String>,
    pub nama_lengkap: String,
    pub jk: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tgl_lahir: Option<NaiveDate>,
    pub alamat: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub jurusan: Option<String>,
    pub nama_instansi_pendidikan: Option<String>,
    pub kabupaten_domisili: Option<String>,
    pub provinsi_domisili: Option<String>,
    pub email: Option<String>,
    pub telepon: Option<String>,
    pub posisi: Option<String>,
    pub tingkat_kepengurusan: Option<String>,
    pub jabatan: Option<String>,
    pub tingkat_penugasan: Option<String>,
    pub kabupaten: Option<String>,
    pub provinsi: Option<String>,
    pub thn_tugas: Option<i32>,
    pub status: Option<String>,
    pub photo: Option<String>,
    pub id_kabupaten_domisili: Option<i32>,
    pub id_provinsi_domisili: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub id_hobi: Option<String>,
    pub id_bakat: Option<i32>,
    pub detail_bakat: Option<String>,
    pub id_minat: Option<i32>,
    pub detail_minat: Option<String>,
    pub id_minat_2: Option<i32>,
    pub detail_minat_2: Option<String>,
    pub keterangan: Option<String>,
    pub file_piagam: Option<String>,
}

// **Filter di application level
pub fn filter_pdp_data(data: &[Pdp], keyword: &str) -> Vec<Pdp> {
    let keyword_lower = keyword.to_lowercase();

    data.iter()
        .filter(|pdp| {
            // **Cari di SEMUA field termasuk yang terdekripsi**
            pdp.nama_lengkap.to_lowercase().contains(&keyword_lower)
                || pdp
                    .email
                    .as_ref()
                    .map_or(false, |e| e.to_lowercase().contains(&keyword_lower))
                || pdp.telepon.as_ref().map_or(false, |t| t.contains(&keyword))
                || pdp.nik.as_ref().map_or(false, |n| n.contains(&keyword))
                || pdp
                    .tempat_lahir
                    .as_ref()
                    .map_or(false, |tl| tl.to_lowercase().contains(&keyword_lower))
                || pdp
                    .alamat
                    .as_ref()
                    .map_or(false, |a| a.to_lowercase().contains(&keyword_lower))
                || pdp
                    .pendidikan_terakhir
                    .as_ref()
                    .map_or(false, |p| p.to_lowercase().contains(&keyword_lower))
                || pdp
                    .jurusan
                    .as_ref()
                    .map_or(false, |j| j.to_lowercase().contains(&keyword_lower))
                || pdp
                    .nama_instansi_pendidikan
                    .as_ref()
                    .map_or(false, |ni| ni.to_lowercase().contains(&keyword_lower))
                || pdp
                    .posisi
                    .as_ref()
                    .map_or(false, |pos| pos.to_lowercase().contains(&keyword_lower))
                || pdp
                    .tingkat_kepengurusan
                    .as_ref()
                    .map_or(false, |tk| tk.to_lowercase().contains(&keyword_lower))
                || pdp
                    .jabatan
                    .as_ref()
                    .map_or(false, |j| j.to_lowercase().contains(&keyword_lower))
                || pdp
                    .tingkat_penugasan
                    .as_ref()
                    .map_or(false, |tp| tp.to_lowercase().contains(&keyword_lower))
                || pdp
                    .provinsi_domisili
                    .as_ref()
                    .map_or(false, |pd| pd.to_lowercase().contains(&keyword_lower))
                || pdp
                    .kabupaten_domisili
                    .as_ref()
                    .map_or(false, |kd| kd.to_lowercase().contains(&keyword_lower))
                || pdp
                    .provinsi
                    .as_ref()
                    .map_or(false, |pp| pp.to_lowercase().contains(&keyword_lower))
                || pdp
                    .kabupaten
                    .as_ref()
                    .map_or(false, |kp| kp.to_lowercase().contains(&keyword_lower))
                || pdp
                    .thn_tugas
                    .map_or(false, |tt| tt.to_string().contains(&keyword))
                || pdp
                    .no_piagam
                    .as_ref()
                    .map_or(false, |np| np.contains(&keyword))
                || pdp
                    .no_simental
                    .as_ref()
                    .map_or(false, |ns| ns.contains(&keyword))
        })
        .cloned()
        .collect()
}

// **Fetch semua data verified (sekali query)**

impl<'r> FromRow<'r, MySqlRow> for Pdp {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            no_simental: row.try_get("no_simental").ok(),
            no_piagam: row.try_get("no_piagam").ok(),
            nik: row.try_get("nik").ok(),
            nama_lengkap: row.try_get("nama_lengkap")?,
            jk: row.try_get("jk").ok(),
            tempat_lahir: row.try_get("tempat_lahir").ok(),
            tgl_lahir: row.try_get("tgl_lahir").ok(),
            alamat: row.try_get("alamat").ok(),
            pendidikan_terakhir: row.try_get("pendidikan_terakhir").ok(),
            jurusan: row.try_get("jurusan").ok(),
            nama_instansi_pendidikan: row.try_get("nama_instansi_pendidikan").ok(),
            kabupaten_domisili: row.try_get("kabupaten_domisili").ok(),
            provinsi_domisili: row.try_get("provinsi_domisili").ok(),
            email: row.try_get("email").ok(),
            telepon: row.try_get("telepon").ok(),
            posisi: row.try_get("posisi").ok(),
            tingkat_kepengurusan: row.try_get("tingkat_kepengurusan").ok(),
            jabatan: row.try_get("jabatan").ok(),
            tingkat_penugasan: row.try_get("tingkat_penugasan").ok(),
            kabupaten: row.try_get("kabupaten").ok(),
            provinsi: row.try_get("provinsi").ok(),
            thn_tugas: row.try_get("thn_tugas").ok(),
            status: row.try_get("status").ok(),
            photo: row.try_get("photo").ok(),
            id_kabupaten_domisili: row.try_get("id_kabupaten_domisili").ok(),
            id_provinsi_domisili: row.try_get("id_provinsi_domisili").ok(),
            id_kabupaten: row.try_get("id_kabupaten").ok(),
            id_provinsi: row.try_get("id_provinsi").ok(),
            id_hobi: row.try_get("id_hobi").ok(),
            id_bakat: row.try_get("id_bakat").ok(),
            id_minat: row.try_get("id_minat").ok(),
            id_minat_2: row.try_get("id_minat_2").ok(),
            detail_bakat: row.try_get("detail_bakat").ok(),
            detail_minat: row.try_get("detail_minat").ok(),
            detail_minat_2: row.try_get("detail_minat_2").ok(),
            keterangan: row.try_get("keterangan").ok(),
            file_piagam: row.try_get("file_piagam").ok(),
        })
    }
}

#[derive(Debug, FromRow)]
pub struct EncryptedPdp {
    pub id: String,
    pub no_simental: Option<String>,
    pub no_piagam: Option<String>,
    pub nik: Option<Vec<u8>>,
    pub nama_lengkap: Vec<u8>,
    pub jk: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tgl_lahir: Option<NaiveDate>,
    pub alamat: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub jurusan: Option<String>,
    pub nama_instansi_pendidikan: Option<String>,
    pub id_kabupaten_domisili: Option<i32>,
    pub id_provinsi_domisili: Option<i32>,
    pub email: Option<Vec<u8>>,
    pub telepon: Option<Vec<u8>>,
    pub posisi: Option<String>,
    pub tingkat_kepengurusan: Option<String>,
    pub jabatan: Option<String>,
    pub tingkat_penugasan: Option<String>,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub thn_tugas: Option<String>,
    pub status: Option<String>,
    pub photo: Option<String>,
    pub nik_nonce: Option<Vec<u8>>,
    pub nama_nonce: Option<Vec<u8>>,
    pub email_nonce: Option<Vec<u8>>,
    pub telepon_nonce: Option<Vec<u8>>,
    pub id_hobi: Option<String>,
    pub id_minat: Option<i32>,
    pub detail_minat: Option<String>,
    pub id_minat_2: Option<i32>,
    pub detail_minat_2: Option<String>,
    pub id_bakat: Option<i32>,
    pub detail_bakat: Option<String>,
    pub keterangan: Option<String>,
    pub file_piagam: Option<String>,

    // Join fields - tambahkan ini
    pub provinsi_domisili_nama: Option<String>,
    pub kabupaten_domisili_nama: Option<String>,
    pub provinsi_penugasan_nama: Option<String>,
    pub kabupaten_penugasan_nama: Option<String>,
}

// Fungsi untuk mendekripsi data PDP
pub fn decrypt_pdp_row(encrypted: EncryptedPdp) -> Result<Pdp, actix_web::Error> {
    log::debug!(
        "Processing PDP ID: {}, is_encrypted: {}",
        encrypted.id,
        encrypted.nama_nonce.is_some()
    );

    let nama_lengkap = if let Some(nonce) = &encrypted.nama_nonce {
        // Data terenkripsi - lakukan dekripsi
        log::debug!("Decrypting nama_lengkap for ID: {}", encrypted.id);

        let key = crate::utils::get_encryption_key().map_err(|e| {
            log::error!(
                "Failed to get encryption key for ID {}: {}",
                encrypted.id,
                e
            );
            actix_web::error::ErrorInternalServerError("Encryption key error")
        })?;

        // Validasi panjang nonce
        if nonce.len() != 24 {
            log::error!(
                "Invalid nonce length for nama_lengkap ID {}: {} bytes",
                encrypted.id,
                nonce.len()
            );
            return Err(actix_web::error::ErrorInternalServerError(
                "Invalid nonce length",
            ));
        }

        crate::utils::decrypt_data(&encrypted.nama_lengkap, nonce, &key).map_err(|e| {
            log::error!(
                "Failed to decrypt nama_lengkap for ID {}: {}",
                encrypted.id,
                e
            );
            actix_web::error::ErrorInternalServerError("Gagal mendekripsi data")
        })?
    } else {
        // Data belum terenkripsi - langsung convert dari bytes ke string
        log::debug!("Using plaintext nama_lengkap for ID: {}", encrypted.id);
        String::from_utf8(encrypted.nama_lengkap.clone()).map_err(|e| {
            log::warn!(
                "Gagal convert nama_lengkap bytes ke string untuk ID {}: {}",
                encrypted.id,
                e
            );
            actix_web::error::ErrorInternalServerError("Gagal memproses data")
        })?
    };

    let email = if let Some(email_cipher) = &encrypted.email {
        if let Some(nonce) = &encrypted.email_nonce {
            log::debug!("Decrypting email for ID: {}", encrypted.id);
            let key = crate::utils::get_encryption_key().map_err(|e| {
                log::error!(
                    "Failed to get encryption key for email ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Encryption key error")
            })?;

            if nonce.len() != 24 {
                log::error!(
                    "Invalid email nonce length for ID {}: {} bytes",
                    encrypted.id,
                    nonce.len()
                );
                return Err(actix_web::error::ErrorInternalServerError(
                    "Invalid nonce length",
                ));
            }

            Some(
                crate::utils::decrypt_data(email_cipher, nonce, &key).map_err(|e| {
                    log::error!("Failed to decrypt email for ID {}: {}", encrypted.id, e);
                    actix_web::error::ErrorInternalServerError("Gagal mendekripsi data")
                })?,
            )
        } else {
            log::debug!("Using plaintext email for ID: {}", encrypted.id);
            Some(String::from_utf8(email_cipher.clone()).map_err(|e| {
                log::warn!(
                    "Gagal convert email bytes ke string untuk ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Gagal memproses data")
            })?)
        }
    } else {
        None
    };

    let nik = if let Some(nik_cipher) = &encrypted.nik {
        if let Some(nonce) = &encrypted.nik_nonce {
            log::debug!("Decrypting nik for ID: {}", encrypted.id);
            let key = crate::utils::get_encryption_key().map_err(|e| {
                log::error!(
                    "Failed to get encryption key for nik ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Encryption key error")
            })?;

            if nonce.len() != 24 {
                log::error!(
                    "Invalid nik nonce length for ID {}: {} bytes",
                    encrypted.id,
                    nonce.len()
                );
                return Err(actix_web::error::ErrorInternalServerError(
                    "Invalid nonce length",
                ));
            }

            Some(
                crate::utils::decrypt_data(nik_cipher, nonce, &key).map_err(|e| {
                    log::error!("Failed to decrypt nik for ID {}: {}", encrypted.id, e);
                    actix_web::error::ErrorInternalServerError("Gagal mendekripsi data")
                })?,
            )
        } else {
            log::debug!("Using plaintext nik for ID: {}", encrypted.id);
            Some(String::from_utf8(nik_cipher.clone()).map_err(|e| {
                log::warn!(
                    "Gagal convert nik bytes ke string untuk ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Gagal memproses data")
            })?)
        }
    } else {
        None
    };
    let telepon = if let Some(telepon_cipher) = &encrypted.telepon {
        if let Some(nonce) = &encrypted.telepon_nonce {
            log::debug!("Decrypting telepon for ID: {}", encrypted.id);
            let key = crate::utils::get_encryption_key().map_err(|e| {
                log::error!(
                    "Failed to get encryption key for telepon ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Encryption key error")
            })?;

            if nonce.len() != 24 {
                log::error!(
                    "Invalid telepon nonce length for ID {}: {} bytes",
                    encrypted.id,
                    nonce.len()
                );
                return Err(actix_web::error::ErrorInternalServerError(
                    "Invalid nonce length",
                ));
            }

            Some(
                crate::utils::decrypt_data(telepon_cipher, nonce, &key).map_err(|e| {
                    log::error!("Failed to decrypt telepon for ID {}: {}", encrypted.id, e);
                    actix_web::error::ErrorInternalServerError("Gagal mendekripsi data")
                })?,
            )
        } else {
            log::debug!("Using plaintext telepon for ID: {}", encrypted.id);
            Some(String::from_utf8(telepon_cipher.clone()).map_err(|e| {
                log::warn!(
                    "Gagal convert telepon bytes ke string untuk ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Gagal memproses data")
            })?)
        }
    } else {
        None
    };

    // Convert tahun tugas dari String ke i32
    let thn_tugas = convert_year_to_i32(encrypted.thn_tugas);

    Ok(Pdp {
        id: encrypted.id,
        no_simental: encrypted.no_simental,
        no_piagam: encrypted.no_piagam,
        nik,
        nama_lengkap,
        jk: encrypted.jk,
        tempat_lahir: encrypted.tempat_lahir,
        tgl_lahir: encrypted.tgl_lahir,
        alamat: encrypted.alamat,
        pendidikan_terakhir: encrypted.pendidikan_terakhir,
        jurusan: encrypted.jurusan,
        nama_instansi_pendidikan: encrypted.nama_instansi_pendidikan,
        // Simpan ID dan nama untuk domisili
        id_kabupaten_domisili: encrypted.id_kabupaten_domisili,
        id_provinsi_domisili: encrypted.id_provinsi_domisili,
        kabupaten_domisili: encrypted.kabupaten_domisili_nama,
        provinsi_domisili: encrypted.provinsi_domisili_nama,
        email,
        telepon,
        posisi: encrypted.posisi,
        tingkat_kepengurusan: encrypted.tingkat_kepengurusan,
        jabatan: encrypted.jabatan,
        tingkat_penugasan: encrypted.tingkat_penugasan,
        // Simpan ID dan nama untuk penugasan
        id_kabupaten: encrypted.id_kabupaten,
        id_provinsi: encrypted.id_provinsi,
        kabupaten: encrypted.kabupaten_penugasan_nama,
        provinsi: encrypted.provinsi_penugasan_nama,
        thn_tugas: thn_tugas,
        status: encrypted.status,
        photo: encrypted.photo,
        id_hobi: encrypted.id_hobi,
        id_bakat: encrypted.id_bakat,
        detail_bakat: encrypted.detail_bakat,
        id_minat: encrypted.id_minat,
        detail_minat: encrypted.detail_minat,
        id_minat_2: encrypted.id_minat_2,
        detail_minat_2: encrypted.detail_minat_2,
        keterangan: encrypted.keterangan,
        file_piagam: encrypted.file_piagam,
    })
}

fn convert_year_to_i32(year_str: Option<String>) -> Option<i32> {
    year_str.and_then(|s| s.parse::<i32>().ok())
}

// =======================
// DTOs
// =======================

#[derive(Debug, Serialize)]
struct ApiMessage {
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationPdpParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub q: Option<String>,
    pub provinsi_id: Option<i32>,
    pub kabupaten_id: Option<i32>,
}
// =======================
// Controllers
// =======================
#[get("/api/adminpanel/pdp-belum-registrasi")]
pub async fn list_pdp_belum_registrasi(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP belum registrasi: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_belum_registrasi = fetch_all_pdp_belum_registrasi(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_belum_registrasi, &keyword)
    } else {
        all_pdp_belum_registrasi
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_belum_registrasi(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE (p.status IS NULL OR p.status = '')",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Belum Registrasi: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "Fetched {} PDP Belum Registrasi records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    Ok(decrypted_rows)
}

//PDP BELUM DIVERIFIKASI
#[get("/api/adminpanel/pdp-belum-diverifikasi")]
pub async fn list_pdp_belum_diverifikasi(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP belum diverifikasi: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_belum_diverifikasi = fetch_all_pdp_belum_diverifikasi(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_belum_diverifikasi, &keyword)
    } else {
        all_pdp_belum_diverifikasi
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_belum_diverifikasi(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Belum Diverifikasi'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Belum Diverifikasi: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "Fetched {} PDP Belum Diverifikasi records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-verified")]
pub async fn list_pdp_verified(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP verified: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_verified = fetch_all_pdp_verified(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_verified, &keyword)
    } else {
        all_pdp_verified
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_verified(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    log::debug!("🔍 Starting fetch_all_pdp_verified");
    log::debug!(
        "📊 Filter params - provinsi_id: {:?}, kabupaten_id: {:?}",
        provinsi_id,
        kabupaten_id
    );

    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Verified'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah - CARA SAMA
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Verified: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Verified records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

//PDP SIMENTAL
#[get("/api/adminpanel/pdp-simental")]
pub async fn list_pdp_simental(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP simental: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_simental = fetch_all_pdp_simental(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_simental, &keyword)
    } else {
        all_pdp_simental
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_simental(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Simental'
         ",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Simental: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "Fetched {} PDP Simental records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    Ok(decrypted_rows)
}

//PDP TIDAK AKTIF
#[get("/api/adminpanel/pdp-tidak-aktif")]
pub async fn list_pdp_tidak_aktif(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP tidak aktif: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_tidak_aktif = fetch_all_pdp_tidak_aktif(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_tidak_aktif, &keyword)
    } else {
        all_pdp_tidak_aktif
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_tidak_aktif(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Tidak Aktif'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Tidak Aktif: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "Fetched {} PDP Tidak Aktif records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    Ok(decrypted_rows)
}

#[put("/api/adminpanel/pdp-update-status/{id}")]
pub async fn update_status(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    mut payload: Multipart, // Hapus .clone() di sini
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id = path.into_inner();

    // Proses multipart payload sekali saja untuk mengambil semua field
    let mut status_value = String::new();
    let mut keterangan_value = String::new();

    while let Some(mut field) = payload.try_next().await.map_err(|e| {
        log::error!("Error processing multipart field: {}", e);
        actix_web::error::ErrorBadRequest("Invalid multipart data")
    })? {
        let field_name = field.name().unwrap_or_default().to_string();

        if field_name == "status" {
            let mut bytes = web::BytesMut::new();
            while let Some(chunk) = field.try_next().await.map_err(|e| {
                log::error!("Error reading status field: {}", e);
                actix_web::error::ErrorBadRequest("Error reading status field")
            })? {
                bytes.extend_from_slice(&chunk);
            }
            status_value = String::from_utf8(bytes.to_vec()).map_err(|_| {
                actix_web::error::ErrorBadRequest("Status field is not valid UTF-8")
            })?;
        } else if field_name == "keterangan" {
            let mut bytes = web::BytesMut::new();
            while let Some(chunk) = field.try_next().await.map_err(|e| {
                log::error!("Error reading keterangan field: {}", e);
                actix_web::error::ErrorBadRequest("Error reading keterangan field")
            })? {
                bytes.extend_from_slice(&chunk);
            }
            keterangan_value = String::from_utf8(bytes.to_vec()).map_err(|_| {
                actix_web::error::ErrorBadRequest("Keterangan field is not valid UTF-8")
            })?;
        } else {
            // Skip unknown fields
            while let Some(_) = field.try_next().await.map_err(|e| {
                log::warn!("Error skipping unknown field {}: {}", field_name, e);
                actix_web::error::ErrorBadRequest("Error processing multipart data")
            })? {}
        }
    }

    let new_status = status_value.trim().to_string();
    let keterangan = keterangan_value.trim().to_string();

    if new_status.is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Field 'status' wajib diisi",
        ));
    }

    // Jika status Ditolak dan keterangan kosong
    if new_status.eq_ignore_ascii_case("Ditolak") && keterangan.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Alasan penolakan wajib diisi",
        ));
    }

    // PERBAIKAN: Baca thn_tugas sebagai i32 langsung dari database
    let (email_cipher, nama_cipher, telepon_cipher, posisi, photo, alamat, id_provinsi, id_kabupaten, email_nonce, nama_nonce, telepon_nonce, jk, tingkat_kepengurusan, id_provinsi_domisili, id_kabupaten_domisili, thn_tugas, no_simental, jabatan, file_piagam): (
        Option<Vec<u8>>,
        Vec<u8>,
        Option<Vec<u8>>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<i32>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = sqlx::query("SELECT email, nama_lengkap, telepon, posisi, photo, alamat, id_provinsi, id_kabupaten, email_nonce, nama_nonce, telepon_nonce, jk, tingkat_kepengurusan, id_provinsi_domisili, id_kabupaten_domisili, thn_tugas, no_simental, jabatan, file_piagam FROM pdp WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.get_ref())
        .await
        .map(|row| {
            (
                row.get::<Option<Vec<u8>>, _>(0),   // email (dienkripsi)
                row.get::<Vec<u8>, _>(1),           // nama_lengkap (dienkripsi)
                row.get::<Option<Vec<u8>>, _>(2),   // telepon (dienkripsi)
                row.get::<Option<String>, _>(3),    // posisi (tidak dienkripsi)
                row.get::<Option<String>, _>(4),    // avatar (tidak dienkripsi)
                row.get::<Option<String>, _>(5),    // alamat (tidak dienkripsi)
                row.get::<Option<i32>, _>(6),       // id_provinsi (tidak dienkripsi)
                row.get::<Option<i32>, _>(7),       // id_kabupaten (tidak dienkripsi)
                row.get::<Option<Vec<u8>>, _>(8),   // email_nonce
                row.get::<Option<Vec<u8>>, _>(9),   // nama_nonce
                row.get::<Option<Vec<u8>>, _>(10),  // telepon_nonce
                row.get::<Option<String>, _>(11),   // jk
                row.get::<Option<String>, _>(12),   // tingkat_kepengurusan
                row.get::<Option<i32>, _>(13),      // id_provinsi_domisili
                row.get::<Option<i32>, _>(14),      // id_kabupaten_domisili
                row.get::<Option<i32>, _>(15),      // thn_tugas sebagai i32 langsung
                row.get::<Option<String>, _>(16),   // no_simental
                row.get::<Option<String>, _>(17),   // jabatan
                row.get::<Option<String>, _>(18),   // file_piagam
            )
        })
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Dekripsi email, nama, dan telepon
    let key = crate::utils::get_encryption_key()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let nama = if let Some(nonce) = nama_nonce {
        crate::utils::decrypt_data(&nama_cipher, &nonce, &key)
            .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        return Err(actix_web::error::ErrorInternalServerError(
            "Missing nama_nonce",
        ));
    };

    let email = if let (Some(email_cipher), Some(nonce)) = (email_cipher, email_nonce) {
        Some(
            crate::utils::decrypt_data(&email_cipher, &nonce, &key)
                .map_err(actix_web::error::ErrorInternalServerError)?,
        )
    } else {
        None
    };

    let telepon = if let (Some(telepon_cipher), Some(nonce)) = (telepon_cipher, telepon_nonce) {
        Some(
            crate::utils::decrypt_data(&telepon_cipher, &nonce, &key)
                .map_err(actix_web::error::ErrorInternalServerError)?,
        )
    } else {
        None
    };

    let mut final_no_simental = no_simental.clone();
    let should_generate_simental = (new_status.eq_ignore_ascii_case("Simental")
        || new_status.eq_ignore_ascii_case("Verified"))
        && (final_no_simental.is_none()
            || final_no_simental
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true));

    if should_generate_simental {
        final_no_simental = Some(
            generate_no_simental(
                pool.get_ref(),
                &tingkat_kepengurusan,
                &jk,
                id_provinsi_domisili,
                id_kabupaten_domisili,
                thn_tugas,
            )
            .await
            .map_err(|e| {
                log::error!("Gagal generate no_simental: {}", e);
                actix_web::error::ErrorInternalServerError("Gagal generate nomor simental")
            })?,
        );
    }

    // Update status pdp, no_simental, dan keterangan jika ada
    let mut update_query = "UPDATE pdp SET status = ?, updated_at = NOW()".to_string();
    if final_no_simental.is_some() {
        update_query.push_str(", no_simental = ?");
    }
    if !keterangan.trim().is_empty() {
        update_query.push_str(", keterangan = ?");
    }
    update_query.push_str(" WHERE id = ?");

    let mut query = sqlx::query(&update_query).bind(&new_status);

    if let Some(no_simental_value) = &final_no_simental {
        query = query.bind(no_simental_value);
    }

    if !keterangan.trim().is_empty() {
        query = query.bind(&keterangan);
    }

    query = query.bind(&id);

    query
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // ===== TAMBAHAN: Insert ke tabel pelaksana jika posisi = "Pelaksana" =====
    let mut pelaksana_inserted = false;
    if new_status.eq_ignore_ascii_case("Verified") && posisi.as_deref() == Some("Pelaksana") {
        log::debug!(
            "Insert pelaksana - tingkat: {:?}, provinsi: {:?}, kabupaten: {:?}",
            tingkat_kepengurusan,
            id_provinsi,
            id_kabupaten
        );

        pelaksana_inserted = insert_into_pelaksana_table(
            pool.get_ref(),
            id.clone(),
            &nama,
            &photo,
            &tingkat_kepengurusan,
            &jabatan,
            &id_provinsi,
            &id_kabupaten,
        )
        .await
        .map_err(|e| {
            log::error!("Gagal insert ke tabel pelaksana: {}", e);
            actix_web::error::ErrorInternalServerError("Gagal menambahkan data pelaksana")
        })?;
    }

    // Jika Verified => insert ke tabel users dan kirim email
    if new_status.eq_ignore_ascii_case("Verified") {
        if let Some(email) = email.as_deref() {
            if !email.trim().is_empty() {
                // Generate random password
                let plain_password = crate::utils::generate_random_password(8);

                // Hash password untuk disimpan di database
                let hashed_password = hash(&plain_password, DEFAULT_COST)
                    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

                // Ambil role dari posisi
                let role = if let Some(posisi) = &posisi {
                    if posisi.trim().is_empty() {
                        "User".to_string()
                    } else {
                        posisi.clone()
                    }
                } else {
                    "User".to_string() // Default role jika posisi null
                };

                // Cek apakah user sudah ada berdasarkan email
                let existing_user: Option<(String,)> =
                    sqlx::query_as("SELECT id FROM users WHERE email = ?")
                        .bind(email) // email: &str
                        .fetch_optional(pool.get_ref())
                        .await
                        .map_err(actix_web::error::ErrorInternalServerError)?;

                if existing_user.is_none() {
                    let user_id = generate_short_uuid_10_upper();

                    sqlx::query(
                        "INSERT INTO users (id, name, email, role, password, avatar, phone, id_pdp, address, id_provinsi, id_kabupaten, created_at, updated_at)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())",
                    )
                    .bind(&user_id)     // ✅ users.id (UUID10)
                    .bind(&nama)        // name
                    .bind(email)        // email (&str)
                    .bind(&role)        // role
                    .bind(&hashed_password)
                    .bind(&photo)
                    .bind(&telepon)
                    .bind(&id)          // id_pdp (String)
                    .bind(&alamat)
                    .bind(&id_provinsi)
                    .bind(&id_kabupaten)
                    .execute(pool.get_ref())
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
                } else {
                    // Update user yang sudah ada (tidak perlu ubah users.id)
                    sqlx::query(
                        "UPDATE users
                         SET name = ?, role = ?, password = ?, avatar = ?, phone = ?, id_pdp = ?, address = ?, id_provinsi = ?, id_kabupaten = ?, updated_at = NOW()
                         WHERE email = ?",
                    )
                    .bind(&nama)
                    .bind(&role)
                    .bind(&hashed_password)
                    .bind(&photo)
                    .bind(&telepon)
                    .bind(&id)
                    .bind(&alamat)
                    .bind(&id_provinsi)
                    .bind(&id_kabupaten)
                    .bind(email) // ✅ &str
                    .execute(pool.get_ref())
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
                }
                // Kirim email dengan password
                let mail_res = send_verified_email(email, &nama, &plain_password).await;
                return match mail_res {
                    Ok(_) => Ok(HttpResponse::Ok().json(ApiMessage {
                        message: format!("Status berhasil diupdate! User telah dibuat dengan role '{}'{}{}",
                            role,
                            if should_generate_simental { " dan NRA telah digenerate" } else { "" },
                            if pelaksana_inserted { " serta data telah ditambahkan ke tabel pelaksana" } else { "" }),
                    })),
                    Err(e) => Ok(HttpResponse::Ok().json(ApiMessage {
                        message: format!("Status berhasil diupdate, user dibuat dengan role '{}', namun gagal mengirim email: {}{}{}",
                            role, e,
                            if should_generate_simental { " dan NRA telah digenerate" } else { "" },
                            if pelaksana_inserted { " serta data telah ditambahkan ke tabel pelaksana" } else { "" }),
                    })),
                };
            }
        }
        return Ok(HttpResponse::Ok().json(ApiMessage {
            message: format!("Status berhasil diupdate! (Catatan: user tidak dibuat karena alamat email kosong){}{}",
                if should_generate_simental { " dan NRA telah digenerate" } else { "" },
                if pelaksana_inserted { " serta data telah ditambahkan ke tabel pelaksana" } else { "" }),
        }));
    }

    // Jika Ditolak => hapus data PDP dan kirim email pemberitahuan penolakan
    if new_status.eq_ignore_ascii_case("Ditolak") {
        // Hapus data PDP dari database
        let delete_result = sqlx::query("DELETE FROM pdp WHERE id = ?")
            .bind(&id)
            .execute(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        if delete_result.rows_affected() == 0 {
            return Err(actix_web::error::ErrorInternalServerError(
                "Gagal menghapus data PDP",
            ));
        }

        // Juga hapus dari users table jika ada
        let _ = sqlx::query("DELETE FROM users WHERE id_pdp = ?")
            .bind(&id)
            .execute(pool.get_ref())
            .await;

        // Hapus file photo jika ada
        if let Some(photo_path) = photo {
            let full_path = format!(".{}", photo_path.trim_start_matches('/'));
            let _ = tokio::fs::remove_file(full_path).await;
        }

        // Hapus file piagam jika ada
        if let Some(file_piagam) = &file_piagam {
            let full_path = format!(".{}", file_piagam.trim_start_matches('/'));
            let _ = tokio::fs::remove_file(full_path).await;
        }

        // Kirim email pemberitahuan penolakan
        if let Some(email) = email.as_deref() {
            if !email.trim().is_empty() {
                let mail_res = send_rejection_email(email, &nama, &keterangan).await;
                return match mail_res {
                Ok(_) => Ok(HttpResponse::Ok().json(ApiMessage {
                    message: "Data PDP berhasil ditolak dan dihapus! Email pemberitahuan telah dikirim. User dapat mendaftar ulang dengan email yang sama.".to_string(),
                })),
                Err(e) => Ok(HttpResponse::Ok().json(ApiMessage {
                    message: format!("Data PDP berhasil ditolak dan dihapus, namun gagal mengirim email: {}. User dapat mendaftar ulang dengan email yang sama.", e),
                })),
            };
            }
        }

        return Ok(HttpResponse::Ok().json(ApiMessage {
        message: "Data PDP berhasil ditolak dan dihapus! (Catatan: email pemberitahuan tidak dikirim karena alamat email kosong). User dapat mendaftar ulang.".to_string(),
    }));
    }

    Ok(HttpResponse::Ok().json(ApiMessage {
        message: format!(
            "Status berhasil diupdate!{}{}",
            if should_generate_simental {
                " dan NRA telah digenerate"
            } else {
                ""
            },
            if pelaksana_inserted {
                " serta data telah ditambahkan ke tabel pelaksana"
            } else {
                ""
            }
        ),
    }))
}
// Fungsi helper untuk insert ke tabel pelaksana - VERSI MACRO
async fn insert_into_pelaksana_table(
    pool: &MySqlPool,
    id_pdp: String,
    nama_lengkap: &str,
    photo: &Option<String>,
    tingkat_kepengurusan: &Option<String>,
    jabatan: &Option<String>,
    id_provinsi: &Option<i32>,
    id_kabupaten: &Option<i32>,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Tentukan query berdasarkan tingkat kepengurusan
    match tingkat_kepengurusan.as_deref() {
        Some("Pelaksana Tingkat Kabupaten/Kota") => {
            // Cek duplikasi
            let existing_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM pelaksana_kabupaten WHERE id_pdp = ?")
                    .bind(&id_pdp)
                    .fetch_one(pool)
                    .await?;

            if existing_count > 0 {
                log::info!("Data pelaksana kabupaten sudah ada untuk id_pdp {}", id_pdp);
                return Ok(false);
            }

            // Insert ke kabupaten
            let result = sqlx::query!(
                "INSERT INTO pelaksana_kabupaten (id_pdp, nama_lengkap, photo, jabatan, id_provinsi, id_kabupaten, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, NOW(), NOW())",
                id_pdp, nama_lengkap, photo, jabatan, id_provinsi, id_kabupaten
            )
            .execute(pool)
            .await?;

            Ok(result.rows_affected() > 0)
        }
        Some("Pelaksana Tingkat Provinsi") => {
            // Cek duplikasi
            let existing_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM pelaksana_provinsi WHERE id_pdp = ?")
                    .bind(&id_pdp)
                    .fetch_one(pool)
                    .await?;

            if existing_count > 0 {
                log::info!("Data pelaksana provinsi sudah ada untuk id_pdp {}", id_pdp);
                return Ok(false);
            }

            // Insert ke provinsi
            let result = sqlx::query!(
                "INSERT INTO pelaksana_provinsi (id_pdp, nama_lengkap, photo, jabatan, id_provinsi, created_at, updated_at) VALUES (?, ?, ?, ?, ?, NOW(), NOW())",
                id_pdp, nama_lengkap, photo, jabatan, id_provinsi
            )
            .execute(pool)
            .await?;

            Ok(result.rows_affected() > 0)
        }
        Some("Pelaksana Tingkat Pusat") => {
            // Cek duplikasi
            let existing_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM pelaksana_pusat WHERE id_pdp = ?")
                    .bind(&id_pdp)
                    .fetch_one(pool)
                    .await?;

            if existing_count > 0 {
                log::info!("Data pelaksana pusat sudah ada untuk id_pdp {}", id_pdp);
                return Ok(false);
            }

            // Insert ke pusat
            let result = sqlx::query!(
                "INSERT INTO pelaksana_pusat (id_pdp, nama_lengkap, photo, jabatan, created_at, updated_at) VALUES (?, ?, ?, ?, NOW(), NOW())",
                id_pdp, nama_lengkap, photo, jabatan
            )
            .execute(pool)
            .await?;

            Ok(result.rows_affected() > 0)
        }
        _ => {
            log::warn!(
                "Tingkat kepengurusan tidak valid untuk insert pelaksana: {:?}",
                tingkat_kepengurusan
            );
            Ok(false)
        }
    }
}

async fn generate_no_simental(
    pool: &MySqlPool,
    tingkat_kepengurusan: &Option<String>,
    jk: &Option<String>,
    id_provinsi_domisili: Option<i32>,
    id_kabupaten_domisili: Option<i32>,
    thn_tugas: Option<i32>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Kode Tingkat Kepengurusan
    let kode_tk_kepengurusan = match tingkat_kepengurusan.as_deref() {
        Some("Pelaksana Tingkat Kabupaten/Kota") => 3,
        Some("Pelaksana Tingkat Provinsi") => 2,
        _ => 1, // default untuk tingkat pusat atau lainnya
    };

    let kode_jk = match jk.as_deref() {
        Some("Laki-Laki") => 1,
        Some("Perempuan") => 2,
        _ => 0,
    };

    // ✅ Langsung pakai i32, tidak perlu parse dari String
    let last_two_digits = thn_tugas
        .map(|year| format!("{:02}", year % 100))
        .unwrap_or_else(|| "00".to_string());

    let last_nomor: Option<String> = sqlx::query_scalar(
        "SELECT RIGHT(no_simental, 5) AS digit_terbesar FROM pdp WHERE no_simental IS NOT NULL AND no_simental != '' ORDER BY digit_terbesar DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    let next_nomor_urut = if let Some(last_nomor) = last_nomor {
        let last_five = &last_nomor[last_nomor.len().saturating_sub(5)..];
        let next_num = last_five.parse::<u32>().unwrap_or(0) + 1;
        format!("{:05}", next_num)
    } else {
        "00001".to_string()
    };

    let nomor_register = format!(
        "{}{}{:02}{:02}{}{}",
        kode_tk_kepengurusan,
        kode_jk,
        id_provinsi_domisili.unwrap_or(0),
        id_kabupaten_domisili.unwrap_or(0),
        last_two_digits,
        next_nomor_urut
    );

    Ok(nomor_register)
}

#[get("/api/adminpanel/pdp/{id}")]
pub async fn get_pdp(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id = path.into_inner();

    let encrypted_row = sqlx::query_as::<_, EncryptedPdp>(
        "SELECT
        p.id,
        p.no_simental,
        p.no_piagam,
        p.nik,
        p.nama_lengkap,
        p.jk,
        p.tempat_lahir,
        p.tgl_lahir,
        p.alamat,
        p.pendidikan_terakhir,
        p.jurusan,
        p.nama_instansi_pendidikan,
        p.id_kabupaten_domisili,
        p.id_provinsi_domisili,
        p.email,
        p.telepon,
        p.posisi,
        p.jabatan,
        p.tingkat_kepengurusan,
        p.tingkat_penugasan,
        p.id_kabupaten,
        p.id_provinsi,
        CAST(p.thn_tugas AS CHAR) AS thn_tugas,
        p.`status`,
        p.photo,
        p.nik_nonce,
        p.nama_nonce,
        p.email_nonce,
        p.telepon_nonce,
        p.id_hobi,
        p.id_bakat,
        p.detail_bakat,
        p.id_minat,
        p.detail_minat,
        p.id_minat_2,
        p.detail_minat_2,
        p.keterangan,
        p.file_piagam,
        pd.nama_provinsi AS provinsi_domisili_nama,
        kd.nama_kabupaten AS kabupaten_domisili_nama,
        pp.nama_provinsi AS provinsi_penugasan_nama,
        kp.nama_kabupaten AS kabupaten_penugasan_nama
     FROM pdp AS p
     LEFT JOIN provinsi  AS pd ON p.id_provinsi_domisili = pd.id
     LEFT JOIN kabupaten AS kd ON p.id_kabupaten_domisili = kd.id
     LEFT JOIN provinsi  AS pp ON p.id_provinsi = pp.id
     LEFT JOIN kabupaten AS kp ON p.id_kabupaten = kp.id
WHERE p.id = ?
",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    // Dekripsi data
    let decrypted_row = decrypt_pdp_row(encrypted_row)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(decrypted_row))
}

#[derive(Serialize, FromRow, Debug)]
struct Pendidikan {
    id: i32,
    id_pdp: i32,
    jenjang_pendidikan: String,
    nama_instansi_pendidikan: String,
    jurusan: Option<String>,
    tahun_masuk: u32,
    tahun_lulus: u32,
}
#[get("/api/adminpanel/pendidikan/{id}")]
pub async fn get_pendidikan_pdp(
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

#[derive(Serialize, FromRow, Debug)]
struct Organisasi {
    id: i32,
    id_pdp: i32,
    nama_organisasi: String,
    status: String,
    tahun_masuk: u32,
    tahun_keluar: Option<u32>,
}
#[get("/api/adminpanel/organisasi/{id}")]
pub async fn get_organiasi_pdp(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id = path.into_inner();

    let data_organisasi=  sqlx::query_as::<_, Organisasi>(
        "SELECT id, id_pdp, nama_organisasi, posisi, tahun_masuk, tahun_keluar, status FROM organisasi WHERE id_pdp = ? ",
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    Ok(HttpResponse::Ok().json(data_organisasi))
}

#[get("/api/userpanel/pdp/{id}")]
pub async fn get_user_pdp(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // pastikan tipe sama (sesuaikan i32/i64 sesuai skema Anda)
    let is_owner = claims.id_pdp.map(|pid| pid == id).unwrap_or(false);

    let is_admin = matches!(
        claims.role.as_str(),
        "Superadmin" | "Administrator" | "Pelaksana" | "Admin Kesbangpol"
    );

    // Tolak hanya jika BUKAN admin DAN BUKAN owner
    if !(is_admin || is_owner) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }
    let encrypted_row = sqlx::query_as::<_, EncryptedPdp>(
        r#"
        SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.photo,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.id = ?
         "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    // Dekripsi data
    let decrypted_row = decrypt_pdp_row(encrypted_row)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(decrypted_row))
}

#[derive(Debug, Deserialize)]
pub struct PdpUpdatePayload {
    pub nik: Option<String>,
    pub nama_lengkap: Option<String>,
    pub email: Option<String>,
    pub telepon: Option<String>,

    // Field lain (opsional semua)
    pub no_simental: Option<String>,
    pub no_piagam: Option<String>,
    pub jk: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tgl_lahir: Option<NaiveDate>,
    pub alamat: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub jurusan: Option<String>,
    pub nama_instansi_pendidikan: Option<String>,
    pub posisi: Option<String>,
    pub tingkat_kepengurusan: Option<String>,
    pub jabatan: Option<String>,
    pub tingkat_penugasan: Option<String>,
    pub thn_tugas: Option<i32>,
    pub status: Option<String>,
    pub id_kabupaten_domisili: Option<i32>,
    pub id_provinsi_domisili: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub id_hobi: Option<Vec<String>>,
    pub id_bakat: Option<i32>,
    pub detail_bakat: Option<String>,
    pub id_minat: Option<i32>,
    pub detail_minat: Option<String>,
    pub id_minat_2: Option<i32>,
    pub detail_minat_2: Option<String>,
    pub keterangan: Option<String>,
}

async fn save_photo_file(
    mut field: actix_multipart::Field,
    dir: &Jalur,
    original_filename: Option<String>,
) -> Result<String, Error> {
    if !dir.exists() {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    let ext = original_filename
        .as_deref()
        .and_then(|n| Jalur::new(n).extension().and_then(|s| s.to_str()))
        .map(|s| format!(".{}", s))
        .unwrap_or_else(|| ".jpg".to_string());

    let filename = format!("pdp_{}{}", Uuid::new_v4(), ext);
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

    Ok(format!("uploads/assets/images/photos/{}", filename))
}

#[put("/api/userpanel/pdp/{id}")]
pub async fn update_pdp(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    mut multipart: Multipart,
) -> Result<impl Responder, Error> {
    let pdp_id = path.into_inner();

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // pastikan tipe sama (sesuaikan i32/i64 sesuai skema Anda)
    let is_owner = claims.id_pdp.map(|pid| pid == pdp_id).unwrap_or(false);

    let is_admin = matches!(
        claims.role.as_str(),
        "Superadmin" | "Administrator" | "Pelaksana" | "Admin Kesbangpol"
    );

    // Tolak hanya jika BUKAN admin DAN BUKAN owner
    if !(is_admin || is_owner) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    // Ambil data lama termasuk photo
    let row_opt = sqlx::query("SELECT photo FROM pdp WHERE id = ?")
        .bind(pdp_id.clone())
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let old_photo: Option<String> = if let Some(row) = row_opt {
        row.get::<Option<String>, _>(0)
    } else {
        return Err(actix_web::error::ErrorNotFound("Data PDP tidak ditemukan"));
    };

    // ===== Baca multipart: payload(JSON) + photo(file opsional) =====
    let mut payload_json: Option<String> = None;
    let mut new_photo_rel: Option<String> = None;
    let mut has_new_photo = false;

    while let Some(mut field) = multipart
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().cloned();
        let name = cd
            .as_ref()
            .and_then(|c| c.get_name())
            .unwrap_or_default()
            .to_string();

        if name == "payload" {
            let mut bytes = web::BytesMut::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                bytes.extend_from_slice(&chunk);
            }
            payload_json = Some(
                String::from_utf8(bytes.to_vec())
                    .map_err(|_| actix_web::error::ErrorBadRequest("payload bukan UTF-8 valid"))?,
            );
        } else if name == "photo" {
            // Cek apakah benar-benar ada file yang diupload
            if let Some(filename) = cd.and_then(|c| c.get_filename().map(|s| s.to_string())) {
                if !filename.trim().is_empty() {
                    let orig = Some(filename);
                    let rel =
                        save_photo_file(field, Jalur::new("uploads/assets/images/photos"), orig)
                            .await?;
                    new_photo_rel = Some(rel);
                    has_new_photo = true;
                }
            }
        } else {
            // drain unknown field
            while let Some(_chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {}
        }
    }

    let upd: PdpUpdatePayload = if let Some(j) = payload_json {
        serde_json::from_str(&j).map_err(|e| {
            actix_web::error::ErrorBadRequest(format!("payload JSON invalid: {}", e))
        })?
    } else {
        return Err(actix_web::error::ErrorBadRequest(
            "Payload JSON tidak ditemukan",
        ));
    };

    // ===== Kunci & blind index keys =====
    let encryption_key_hex = env::var("ENCRYPTION_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("ENCRYPTION_KEY missing"))?;
    let blind_index_key_hex = env::var("BLIND_INDEX_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("BLIND_INDEX_KEY missing"))?;

    let encryption_key_bytes = hex::decode(&encryption_key_hex)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid hex encryption key"))?;
    let blind_index_key_bytes = hex::decode(&blind_index_key_hex)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid hex blind index key"))?;

    let key = secretbox::Key::from_slice(&encryption_key_bytes)
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Invalid encryption key size"))?;

    // ===== Siapkan nilai terenkripsi BILA disediakan di payload =====
    // NIK - hanya update jika ada nilai baru
    let (nik_cipher_opt, nik_nonce_opt, nik_bi_opt) = if let Some(nik_plain) = upd.nik.as_ref() {
        if !nik_plain.trim().is_empty() {
            let (nonce, cipher) = utils::encrypt_data(nik_plain.as_bytes(), &key);
            let bi = utils::generate_blind_index(nik_plain.as_bytes(), &blind_index_key_bytes);
            (Some(cipher), Some(nonce.as_ref().to_vec()), Some(bi))
        } else {
            (None, None, None)
        }
    } else {
        (None, None, None)
    };

    // NAMA - hanya update jika ada nilai baru
    let (nama_cipher_opt, nama_nonce_opt) = if let Some(nama_plain) = upd.nama_lengkap.as_ref() {
        if !nama_plain.trim().is_empty() {
            let (nonce, cipher) = utils::encrypt_data(nama_plain.as_bytes(), &key);
            (Some(cipher), Some(nonce.as_ref().to_vec()))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // EMAIL - hanya update jika ada nilai baru
    let (email_cipher_opt, email_nonce_opt, email_bi_opt) =
        if let Some(email_plain) = upd.email.as_ref() {
            if !email_plain.trim().is_empty() {
                let norm = email_plain.trim().to_ascii_lowercase();
                let (nonce, cipher) = utils::encrypt_data(norm.as_bytes(), &key);
                let bi = utils::generate_blind_index(norm.as_bytes(), &blind_index_key_bytes);
                (Some(cipher), Some(nonce.as_ref().to_vec()), Some(bi))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

    // TELEPON - hanya update jika ada nilai baru
    let (telp_cipher_opt, telp_nonce_opt, telp_bi_opt) =
        if let Some(telp_plain) = upd.telepon.as_ref() {
            if !telp_plain.trim().is_empty() {
                let norm = utils::normalize_phone(telp_plain);
                let (nonce, cipher) = utils::encrypt_data(norm.as_bytes(), &key);
                let bi = utils::generate_blind_index(norm.as_bytes(), &blind_index_key_bytes);
                (Some(cipher), Some(nonce.as_ref().to_vec()), Some(bi))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

    // ===== Transaction =====
    let mut tx = pool
        .begin()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Query: hanya update photo jika ada file baru yang diupload
    let sql = r#"
    UPDATE pdp SET
        -- Encrypted fields (hanya update jika param NOT NULL dan tidak kosong)
        nik                  = COALESCE(?, nik),
        nik_nonce            = COALESCE(?, nik_nonce),
        nik_blind_index      = COALESCE(?, nik_blind_index),

        nama_lengkap         = COALESCE(?, nama_lengkap),
        nama_nonce           = COALESCE(?, nama_nonce),

        email                = COALESCE(?, email),
        email_nonce          = COALESCE(?, email_nonce),
        email_blind_index    = COALESCE(?, email_blind_index),

        telepon              = COALESCE(?, telepon),
        telepon_nonce        = COALESCE(?, telepon_nonce),
        telepon_blind_index  = COALESCE(?, telepon_blind_index),


        -- Plain fields
        no_simental              = ?,
        no_piagam                = ?,
        jk                       = ?,
        tempat_lahir             = ?,
        tgl_lahir                = ?,
        alamat                   = ?,
        pendidikan_terakhir      = ?,
        jurusan                  = ?,
        nama_instansi_pendidikan = ?,
        posisi                   = ?,
        tingkat_kepengurusan     = ?,
        jabatan                  = ?,
        tingkat_penugasan        = ?,
        thn_tugas                = ?,
        status                   = ?,
        id_kabupaten_domisili    = ?,
        id_provinsi_domisili     = ?,
        id_kabupaten             = ?,
        id_provinsi              = ?,
        id_hobi                  = ?,
        id_bakat                 = ?,
        detail_bakat             = ?,
        id_minat                 = ?,
        detail_minat             = ?,
        id_minat_2               = ?,
        detail_minat_2           = ?,
        keterangan               = ?,
        photo                = COALESCE(?, photo),
        updated_at               = NOW()
    WHERE id = ?
    LIMIT 1
    "#;

    let mut q = sqlx::query(sql);

    // bind encrypted fields (gunakan nilai dari payload atau NULL untuk skip update)
    q = q
        .bind(nik_cipher_opt.as_ref())
        .bind(nik_nonce_opt.as_ref())
        .bind(nik_bi_opt.as_ref())
        .bind(nama_cipher_opt.as_ref())
        .bind(nama_nonce_opt.as_ref())
        .bind(email_cipher_opt.as_ref())
        .bind(email_nonce_opt.as_ref())
        .bind(email_bi_opt.as_ref())
        .bind(telp_cipher_opt.as_ref())
        .bind(telp_nonce_opt.as_ref())
        .bind(telp_bi_opt.as_ref());

    // bind plain fields - selalu gunakan nilai dari payload
    q = q
        .bind(upd.no_simental.as_ref())
        .bind(upd.no_piagam.as_ref())
        .bind(upd.jk.as_ref())
        .bind(upd.tempat_lahir.as_ref())
        .bind(upd.tgl_lahir)
        .bind(upd.alamat.as_ref())
        .bind(upd.pendidikan_terakhir.as_ref())
        .bind(upd.jurusan.as_ref())
        .bind(upd.nama_instansi_pendidikan.as_ref())
        .bind(upd.posisi.as_ref())
        .bind(upd.tingkat_kepengurusan.as_ref())
        .bind(upd.jabatan.as_ref())
        .bind(upd.tingkat_penugasan.as_ref())
        .bind(upd.thn_tugas)
        .bind(upd.status.as_ref())
        .bind(upd.id_kabupaten_domisili)
        .bind(upd.id_provinsi_domisili)
        .bind(upd.id_kabupaten)
        .bind(upd.id_provinsi)
        .bind(upd.id_hobi.as_ref().and_then(|hobi| {
            if hobi.is_empty() {
                None
            } else {
                Some(serde_json::to_string(hobi).unwrap_or_else(|_| "[]".to_string()))
            }
        }))
        .bind(upd.id_bakat)
        .bind(upd.detail_bakat.as_ref())
        .bind(upd.id_minat)
        .bind(upd.detail_minat.as_ref())
        .bind(upd.id_minat_2)
        .bind(upd.detail_minat_2.as_ref())
        .bind(upd.keterangan.as_ref())
        // Photo: hanya update jika ada file baru, otherwise tetap pakai yang lama
        .bind(if has_new_photo {
            new_photo_rel.as_ref()
        } else {
            None
        })
        .bind(&pdp_id);

    let result = q
        .execute(&mut *tx)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("Data PDP tidak ditemukan"));
    }

    tx.commit()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Hapus foto lama hanya jika ada foto baru & path berbeda
    if has_new_photo {
        if let (Some(new_rel), Some(old_rel)) = (new_photo_rel.as_ref(), old_photo.as_ref()) {
            if new_rel != old_rel {
                let old_abs = Jalur::new(".").join(old_rel.trim_start_matches('/'));
                if old_abs.exists() {
                    let _ = tokio::fs::remove_file(old_abs).await;
                }
            }
        }
    }

    #[derive(serde::Serialize)]
    struct Resp {
        message: String,
        photo: Option<String>,
        updated: bool,
    }

    Ok(HttpResponse::Ok().json(Resp {
        message: "PDP berhasil diperbarui".into(),
        photo: if has_new_photo {
            new_photo_rel
        } else {
            old_photo
        },
        updated: true,
    }))
}

#[delete("/api/adminpanel/pdp/{id}")]
pub async fn delete_pdp(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Administrator atau Pelaksana yang dapat mengakses",
        ));
    }

    let id = path.into_inner();

    // Ambil foto lama sebelum delete - gunakan fetch_optional untuk menangani kasus tidak ada data
    let old_photo_opt: Option<String> = match sqlx::query_as("SELECT photo FROM pdp WHERE id = ?")
        .bind(&id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        Some((photo,)) => photo,
        None => None,
    };

    let old_photo_user: Option<String> =
        match sqlx::query_as("SELECT avatar FROM users WHERE id_pdp = ?")
            .bind(&id)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
        {
            Some((avatar,)) => avatar,
            None => None,
        };

    // Hapus file fisik jika ada
    if let Some(oldp) = old_photo_opt {
        remove_file_if_exists(&oldp);
    }
    if let Some(oldpu) = old_photo_user {
        remove_file_if_exists(&oldpu);
    }

    // Hapus row dari pdp
    let result = sqlx::query("DELETE FROM pdp WHERE id = ?")
        .bind(&id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().body("Data PDP tidak ditemukan"));
    }

    // Hapus user yang terkait (jika ada)
    let hapus_user = sqlx::query("DELETE FROM users WHERE id_pdp = ?")
        .bind(&id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Tidak perlu error jika user tidak ditemukan, karena mungkin tidak semua PDP punya user

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Berhasil dihapus",
        "id": id,
        "pdp_deleted": result.rows_affected(),
        "user_deleted": hapus_user.rows_affected()
    })))
}

#[get("/api/adminpanel/pdp-belum-registrasi-all")]
pub async fn list_pdp_belum_registrasi_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!(
        "Downloading all PDP Belum Registrasi with filter: '{}'",
        keyword
    );

    // Fetch semua data tanpa pagination
    let all_pdp_belum_registrasi =
        fetch_all_pdp_belum_registrasi_all(pool.clone(), params.provinsi_id, params.kabupaten_id)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_belum_registrasi, &keyword)
    } else {
        all_pdp_belum_registrasi
    };

    log::debug!(
        "Downloading {} PDP Belum Registrasi records",
        filtered_data.len()
    );

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_all_pdp_belum_registrasi_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE (p.status IS NULL OR p.status = '')",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Belum Registrasi: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Belum Registrasi records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-belum-diverifikasi-all")]
pub async fn list_pdp_belum_diverifikasi_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!(
        "Downloading all PDP Belum Diverifikasi with filter: '{}'",
        keyword
    );

    // Fetch semua data tanpa pagination
    let all_pdp_belum_diverifikasi =
        fetch_all_pdp_belum_diverifikasi_all(pool.clone(), params.provinsi_id, params.kabupaten_id)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_belum_diverifikasi, &keyword)
    } else {
        all_pdp_belum_diverifikasi
    };

    log::debug!(
        "Downloading {} PDP Belum Diverifikasi records",
        filtered_data.len()
    );

    Ok(HttpResponse::Ok().json(filtered_data))
}

// Fungsi baru tanpa pagination
async fn fetch_all_pdp_belum_diverifikasi_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Belum Diverifikasi'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Belum Diverifikasi: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Belum Diverifikasi records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-verified-all")]
pub async fn list_pdp_verified_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP verified with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_verified =
        fetch_all_pdp_verified_all(pool.clone(), params.provinsi_id, params.kabupaten_id).await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_verified, &keyword)
    } else {
        all_pdp_verified
    };

    log::debug!("Downloading {} PDP Verified records", filtered_data.len());

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_all_pdp_verified_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Verified'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Verified: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Verified records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-simental-all")]
pub async fn list_pdp_simental_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP simental with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_simental =
        fetch_all_pdp_simental_all(pool.clone(), params.provinsi_id, params.kabupaten_id).await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_simental, &keyword)
    } else {
        all_pdp_simental
    };

    log::debug!("Downloading {} PDP Simental records", filtered_data.len());

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_all_pdp_simental_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Simental'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Simental: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Simental records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-tidak-aktif-all")]
pub async fn list_pdp_tidak_aktif_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP Tidak Aktif with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_tidak_aktif =
        fetch_all_pdp_tidak_aktif_all(pool.clone(), params.provinsi_id, params.kabupaten_id)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_tidak_aktif, &keyword)
    } else {
        all_pdp_tidak_aktif
    };

    log::debug!(
        "Downloading {} PDP Tidak Aktif records",
        filtered_data.len()
    );

    Ok(HttpResponse::Ok().json(filtered_data))
}

// Fungsi baru tanpa pagination
async fn fetch_all_pdp_tidak_aktif_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Tidak Aktif'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Tidak Aktif: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Tidak Aktif records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

fn generate_short_uuid_10_upper() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    raw[..10].to_uppercase()
}

// =======================
// Router config helper
// =======================
pub fn scope() -> actix_web::Scope {
    web::scope("")
        .service(list_pdp_belum_registrasi)
        .service(list_pdp_belum_diverifikasi)
        .service(list_pdp_verified)
        .service(list_pdp_simental)
        .service(list_pdp_tidak_aktif)
        .service(get_user_pdp)
        .service(get_pdp)
        .service(update_status)
        .service(get_pendidikan_pdp)
        .service(get_organiasi_pdp)
        .service(update_pdp)
        .service(delete_pdp)
        .service(list_pdp_belum_registrasi_all)
        .service(list_pdp_belum_diverifikasi_all)
        .service(list_pdp_verified_all)
        .service(list_pdp_simental_all)
        .service(list_pdp_tidak_aktif_all)
}
=======
use crate::{
    auth,
    controllers::pelaksana_controller::remove_file_if_exists,
    utils::{self, send_rejection_email, send_verified_email},
};
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, delete, get, put,
    web::{self, Data, Path},
};
use bcrypt::{DEFAULT_COST, hash};
use chrono::NaiveDate;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;
use sqlx::{FromRow, MySqlPool, Row, mysql::MySqlRow};
use std::{env, path::Path as Jalur};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
// =======================
// Models
// =======================
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pdp {
    pub id: String,
    pub no_simental: Option<String>,
    pub no_piagam: Option<String>,
    pub nik: Option<String>,
    pub nama_lengkap: String,
    pub jk: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tgl_lahir: Option<NaiveDate>,
    pub alamat: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub jurusan: Option<String>,
    pub nama_instansi_pendidikan: Option<String>,
    pub kabupaten_domisili: Option<String>,
    pub provinsi_domisili: Option<String>,
    pub email: Option<String>,
    pub telepon: Option<String>,
    pub posisi: Option<String>,
    pub tingkat_kepengurusan: Option<String>,
    pub jabatan: Option<String>,
    pub tingkat_penugasan: Option<String>,
    pub kabupaten: Option<String>,
    pub provinsi: Option<String>,
    pub thn_tugas: Option<i32>,
    pub status: Option<String>,
    pub photo: Option<String>,
    pub id_kabupaten_domisili: Option<i32>,
    pub id_provinsi_domisili: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub id_hobi: Option<String>,
    pub id_bakat: Option<i32>,
    pub detail_bakat: Option<String>,
    pub id_minat: Option<i32>,
    pub detail_minat: Option<String>,
    pub id_minat_2: Option<i32>,
    pub detail_minat_2: Option<String>,
    pub keterangan: Option<String>,
    pub file_piagam: Option<String>,
}

// **Filter di application level
pub fn filter_pdp_data(data: &[Pdp], keyword: &str) -> Vec<Pdp> {
    let keyword_lower = keyword.to_lowercase();

    data.iter()
        .filter(|pdp| {
            // **Cari di SEMUA field termasuk yang terdekripsi**
            pdp.nama_lengkap.to_lowercase().contains(&keyword_lower)
                || pdp
                    .email
                    .as_ref()
                    .map_or(false, |e| e.to_lowercase().contains(&keyword_lower))
                || pdp.telepon.as_ref().map_or(false, |t| t.contains(&keyword))
                || pdp.nik.as_ref().map_or(false, |n| n.contains(&keyword))
                || pdp
                    .tempat_lahir
                    .as_ref()
                    .map_or(false, |tl| tl.to_lowercase().contains(&keyword_lower))
                || pdp
                    .alamat
                    .as_ref()
                    .map_or(false, |a| a.to_lowercase().contains(&keyword_lower))
                || pdp
                    .pendidikan_terakhir
                    .as_ref()
                    .map_or(false, |p| p.to_lowercase().contains(&keyword_lower))
                || pdp
                    .jurusan
                    .as_ref()
                    .map_or(false, |j| j.to_lowercase().contains(&keyword_lower))
                || pdp
                    .nama_instansi_pendidikan
                    .as_ref()
                    .map_or(false, |ni| ni.to_lowercase().contains(&keyword_lower))
                || pdp
                    .posisi
                    .as_ref()
                    .map_or(false, |pos| pos.to_lowercase().contains(&keyword_lower))
                || pdp
                    .tingkat_kepengurusan
                    .as_ref()
                    .map_or(false, |tk| tk.to_lowercase().contains(&keyword_lower))
                || pdp
                    .jabatan
                    .as_ref()
                    .map_or(false, |j| j.to_lowercase().contains(&keyword_lower))
                || pdp
                    .tingkat_penugasan
                    .as_ref()
                    .map_or(false, |tp| tp.to_lowercase().contains(&keyword_lower))
                || pdp
                    .provinsi_domisili
                    .as_ref()
                    .map_or(false, |pd| pd.to_lowercase().contains(&keyword_lower))
                || pdp
                    .kabupaten_domisili
                    .as_ref()
                    .map_or(false, |kd| kd.to_lowercase().contains(&keyword_lower))
                || pdp
                    .provinsi
                    .as_ref()
                    .map_or(false, |pp| pp.to_lowercase().contains(&keyword_lower))
                || pdp
                    .kabupaten
                    .as_ref()
                    .map_or(false, |kp| kp.to_lowercase().contains(&keyword_lower))
                || pdp
                    .thn_tugas
                    .map_or(false, |tt| tt.to_string().contains(&keyword))
                || pdp
                    .no_piagam
                    .as_ref()
                    .map_or(false, |np| np.contains(&keyword))
                || pdp
                    .no_simental
                    .as_ref()
                    .map_or(false, |ns| ns.contains(&keyword))
        })
        .cloned()
        .collect()
}

// **Fetch semua data verified (sekali query)**

impl<'r> FromRow<'r, MySqlRow> for Pdp {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            no_simental: row.try_get("no_simental").ok(),
            no_piagam: row.try_get("no_piagam").ok(),
            nik: row.try_get("nik").ok(),
            nama_lengkap: row.try_get("nama_lengkap")?,
            jk: row.try_get("jk").ok(),
            tempat_lahir: row.try_get("tempat_lahir").ok(),
            tgl_lahir: row.try_get("tgl_lahir").ok(),
            alamat: row.try_get("alamat").ok(),
            pendidikan_terakhir: row.try_get("pendidikan_terakhir").ok(),
            jurusan: row.try_get("jurusan").ok(),
            nama_instansi_pendidikan: row.try_get("nama_instansi_pendidikan").ok(),
            kabupaten_domisili: row.try_get("kabupaten_domisili").ok(),
            provinsi_domisili: row.try_get("provinsi_domisili").ok(),
            email: row.try_get("email").ok(),
            telepon: row.try_get("telepon").ok(),
            posisi: row.try_get("posisi").ok(),
            tingkat_kepengurusan: row.try_get("tingkat_kepengurusan").ok(),
            jabatan: row.try_get("jabatan").ok(),
            tingkat_penugasan: row.try_get("tingkat_penugasan").ok(),
            kabupaten: row.try_get("kabupaten").ok(),
            provinsi: row.try_get("provinsi").ok(),
            thn_tugas: row.try_get("thn_tugas").ok(),
            status: row.try_get("status").ok(),
            photo: row.try_get("photo").ok(),
            id_kabupaten_domisili: row.try_get("id_kabupaten_domisili").ok(),
            id_provinsi_domisili: row.try_get("id_provinsi_domisili").ok(),
            id_kabupaten: row.try_get("id_kabupaten").ok(),
            id_provinsi: row.try_get("id_provinsi").ok(),
            id_hobi: row.try_get("id_hobi").ok(),
            id_bakat: row.try_get("id_bakat").ok(),
            id_minat: row.try_get("id_minat").ok(),
            id_minat_2: row.try_get("id_minat_2").ok(),
            detail_bakat: row.try_get("detail_bakat").ok(),
            detail_minat: row.try_get("detail_minat").ok(),
            detail_minat_2: row.try_get("detail_minat_2").ok(),
            keterangan: row.try_get("keterangan").ok(),
            file_piagam: row.try_get("file_piagam").ok(),
        })
    }
}

#[derive(Debug, FromRow)]
pub struct EncryptedPdp {
    pub id: String,
    pub no_simental: Option<String>,
    pub no_piagam: Option<String>,
    pub nik: Option<Vec<u8>>,
    pub nama_lengkap: Vec<u8>,
    pub jk: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tgl_lahir: Option<NaiveDate>,
    pub alamat: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub jurusan: Option<String>,
    pub nama_instansi_pendidikan: Option<String>,
    pub id_kabupaten_domisili: Option<i32>,
    pub id_provinsi_domisili: Option<i32>,
    pub email: Option<Vec<u8>>,
    pub telepon: Option<Vec<u8>>,
    pub posisi: Option<String>,
    pub tingkat_kepengurusan: Option<String>,
    pub jabatan: Option<String>,
    pub tingkat_penugasan: Option<String>,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub thn_tugas: Option<String>,
    pub status: Option<String>,
    pub photo: Option<String>,
    pub nik_nonce: Option<Vec<u8>>,
    pub nama_nonce: Option<Vec<u8>>,
    pub email_nonce: Option<Vec<u8>>,
    pub telepon_nonce: Option<Vec<u8>>,
    pub id_hobi: Option<String>,
    pub id_minat: Option<i32>,
    pub detail_minat: Option<String>,
    pub id_minat_2: Option<i32>,
    pub detail_minat_2: Option<String>,
    pub id_bakat: Option<i32>,
    pub detail_bakat: Option<String>,
    pub keterangan: Option<String>,
    pub file_piagam: Option<String>,

    // Join fields - tambahkan ini
    pub provinsi_domisili_nama: Option<String>,
    pub kabupaten_domisili_nama: Option<String>,
    pub provinsi_penugasan_nama: Option<String>,
    pub kabupaten_penugasan_nama: Option<String>,
}

// Fungsi untuk mendekripsi data PDP
pub fn decrypt_pdp_row(encrypted: EncryptedPdp) -> Result<Pdp, actix_web::Error> {
    log::debug!(
        "Processing PDP ID: {}, is_encrypted: {}",
        encrypted.id,
        encrypted.nama_nonce.is_some()
    );

    let nama_lengkap = if let Some(nonce) = &encrypted.nama_nonce {
        // Data terenkripsi - lakukan dekripsi
        log::debug!("Decrypting nama_lengkap for ID: {}", encrypted.id);

        let key = crate::utils::get_encryption_key().map_err(|e| {
            log::error!(
                "Failed to get encryption key for ID {}: {}",
                encrypted.id,
                e
            );
            actix_web::error::ErrorInternalServerError("Encryption key error")
        })?;

        // Validasi panjang nonce
        if nonce.len() != 24 {
            log::error!(
                "Invalid nonce length for nama_lengkap ID {}: {} bytes",
                encrypted.id,
                nonce.len()
            );
            return Err(actix_web::error::ErrorInternalServerError(
                "Invalid nonce length",
            ));
        }

        crate::utils::decrypt_data(&encrypted.nama_lengkap, nonce, &key).map_err(|e| {
            log::error!(
                "Failed to decrypt nama_lengkap for ID {}: {}",
                encrypted.id,
                e
            );
            actix_web::error::ErrorInternalServerError("Gagal mendekripsi data")
        })?
    } else {
        // Data belum terenkripsi - langsung convert dari bytes ke string
        log::debug!("Using plaintext nama_lengkap for ID: {}", encrypted.id);
        String::from_utf8(encrypted.nama_lengkap.clone()).map_err(|e| {
            log::warn!(
                "Gagal convert nama_lengkap bytes ke string untuk ID {}: {}",
                encrypted.id,
                e
            );
            actix_web::error::ErrorInternalServerError("Gagal memproses data")
        })?
    };

    let email = if let Some(email_cipher) = &encrypted.email {
        if let Some(nonce) = &encrypted.email_nonce {
            log::debug!("Decrypting email for ID: {}", encrypted.id);
            let key = crate::utils::get_encryption_key().map_err(|e| {
                log::error!(
                    "Failed to get encryption key for email ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Encryption key error")
            })?;

            if nonce.len() != 24 {
                log::error!(
                    "Invalid email nonce length for ID {}: {} bytes",
                    encrypted.id,
                    nonce.len()
                );
                return Err(actix_web::error::ErrorInternalServerError(
                    "Invalid nonce length",
                ));
            }

            Some(
                crate::utils::decrypt_data(email_cipher, nonce, &key).map_err(|e| {
                    log::error!("Failed to decrypt email for ID {}: {}", encrypted.id, e);
                    actix_web::error::ErrorInternalServerError("Gagal mendekripsi data")
                })?,
            )
        } else {
            log::debug!("Using plaintext email for ID: {}", encrypted.id);
            Some(String::from_utf8(email_cipher.clone()).map_err(|e| {
                log::warn!(
                    "Gagal convert email bytes ke string untuk ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Gagal memproses data")
            })?)
        }
    } else {
        None
    };

    let nik = if let Some(nik_cipher) = &encrypted.nik {
        if let Some(nonce) = &encrypted.nik_nonce {
            log::debug!("Decrypting nik for ID: {}", encrypted.id);
            let key = crate::utils::get_encryption_key().map_err(|e| {
                log::error!(
                    "Failed to get encryption key for nik ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Encryption key error")
            })?;

            if nonce.len() != 24 {
                log::error!(
                    "Invalid nik nonce length for ID {}: {} bytes",
                    encrypted.id,
                    nonce.len()
                );
                return Err(actix_web::error::ErrorInternalServerError(
                    "Invalid nonce length",
                ));
            }

            Some(
                crate::utils::decrypt_data(nik_cipher, nonce, &key).map_err(|e| {
                    log::error!("Failed to decrypt nik for ID {}: {}", encrypted.id, e);
                    actix_web::error::ErrorInternalServerError("Gagal mendekripsi data")
                })?,
            )
        } else {
            log::debug!("Using plaintext nik for ID: {}", encrypted.id);
            Some(String::from_utf8(nik_cipher.clone()).map_err(|e| {
                log::warn!(
                    "Gagal convert nik bytes ke string untuk ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Gagal memproses data")
            })?)
        }
    } else {
        None
    };
    let telepon = if let Some(telepon_cipher) = &encrypted.telepon {
        if let Some(nonce) = &encrypted.telepon_nonce {
            log::debug!("Decrypting telepon for ID: {}", encrypted.id);
            let key = crate::utils::get_encryption_key().map_err(|e| {
                log::error!(
                    "Failed to get encryption key for telepon ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Encryption key error")
            })?;

            if nonce.len() != 24 {
                log::error!(
                    "Invalid telepon nonce length for ID {}: {} bytes",
                    encrypted.id,
                    nonce.len()
                );
                return Err(actix_web::error::ErrorInternalServerError(
                    "Invalid nonce length",
                ));
            }

            Some(
                crate::utils::decrypt_data(telepon_cipher, nonce, &key).map_err(|e| {
                    log::error!("Failed to decrypt telepon for ID {}: {}", encrypted.id, e);
                    actix_web::error::ErrorInternalServerError("Gagal mendekripsi data")
                })?,
            )
        } else {
            log::debug!("Using plaintext telepon for ID: {}", encrypted.id);
            Some(String::from_utf8(telepon_cipher.clone()).map_err(|e| {
                log::warn!(
                    "Gagal convert telepon bytes ke string untuk ID {}: {}",
                    encrypted.id,
                    e
                );
                actix_web::error::ErrorInternalServerError("Gagal memproses data")
            })?)
        }
    } else {
        None
    };

    // Convert tahun tugas dari String ke i32
    let thn_tugas = convert_year_to_i32(encrypted.thn_tugas);

    Ok(Pdp {
        id: encrypted.id,
        no_simental: encrypted.no_simental,
        no_piagam: encrypted.no_piagam,
        nik,
        nama_lengkap,
        jk: encrypted.jk,
        tempat_lahir: encrypted.tempat_lahir,
        tgl_lahir: encrypted.tgl_lahir,
        alamat: encrypted.alamat,
        pendidikan_terakhir: encrypted.pendidikan_terakhir,
        jurusan: encrypted.jurusan,
        nama_instansi_pendidikan: encrypted.nama_instansi_pendidikan,
        // Simpan ID dan nama untuk domisili
        id_kabupaten_domisili: encrypted.id_kabupaten_domisili,
        id_provinsi_domisili: encrypted.id_provinsi_domisili,
        kabupaten_domisili: encrypted.kabupaten_domisili_nama,
        provinsi_domisili: encrypted.provinsi_domisili_nama,
        email,
        telepon,
        posisi: encrypted.posisi,
        tingkat_kepengurusan: encrypted.tingkat_kepengurusan,
        jabatan: encrypted.jabatan,
        tingkat_penugasan: encrypted.tingkat_penugasan,
        // Simpan ID dan nama untuk penugasan
        id_kabupaten: encrypted.id_kabupaten,
        id_provinsi: encrypted.id_provinsi,
        kabupaten: encrypted.kabupaten_penugasan_nama,
        provinsi: encrypted.provinsi_penugasan_nama,
        thn_tugas: thn_tugas,
        status: encrypted.status,
        photo: encrypted.photo,
        id_hobi: encrypted.id_hobi,
        id_bakat: encrypted.id_bakat,
        detail_bakat: encrypted.detail_bakat,
        id_minat: encrypted.id_minat,
        detail_minat: encrypted.detail_minat,
        id_minat_2: encrypted.id_minat_2,
        detail_minat_2: encrypted.detail_minat_2,
        keterangan: encrypted.keterangan,
        file_piagam: encrypted.file_piagam,
    })
}

fn convert_year_to_i32(year_str: Option<String>) -> Option<i32> {
    year_str.and_then(|s| s.parse::<i32>().ok())
}

// =======================
// DTOs
// =======================

#[derive(Debug, Serialize)]
struct ApiMessage {
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationPdpParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub q: Option<String>,
    pub provinsi_id: Option<i32>,
    pub kabupaten_id: Option<i32>,
}
// =======================
// Controllers
// =======================
#[get("/api/adminpanel/pdp-belum-registrasi")]
pub async fn list_pdp_belum_registrasi(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP belum registrasi: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_belum_registrasi = fetch_all_pdp_belum_registrasi(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_belum_registrasi, &keyword)
    } else {
        all_pdp_belum_registrasi
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_belum_registrasi(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE (p.status IS NULL OR p.status = '')",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Belum Registrasi: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "Fetched {} PDP Belum Registrasi records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    Ok(decrypted_rows)
}

//PDP BELUM DIVERIFIKASI
#[get("/api/adminpanel/pdp-belum-diverifikasi")]
pub async fn list_pdp_belum_diverifikasi(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP belum diverifikasi: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_belum_diverifikasi = fetch_all_pdp_belum_diverifikasi(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_belum_diverifikasi, &keyword)
    } else {
        all_pdp_belum_diverifikasi
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_belum_diverifikasi(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Belum Diverifikasi'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Belum Diverifikasi: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "Fetched {} PDP Belum Diverifikasi records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-verified")]
pub async fn list_pdp_verified(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP verified: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_verified = fetch_all_pdp_verified(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_verified, &keyword)
    } else {
        all_pdp_verified
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_verified(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    log::debug!("🔍 Starting fetch_all_pdp_verified");
    log::debug!(
        "📊 Filter params - provinsi_id: {:?}, kabupaten_id: {:?}",
        provinsi_id,
        kabupaten_id
    );

    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Verified'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah - CARA SAMA
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Verified: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Verified records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

//PDP SIMENTAL
#[get("/api/adminpanel/pdp-simental")]
pub async fn list_pdp_simental(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP simental: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_simental = fetch_all_pdp_simental(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_simental, &keyword)
    } else {
        all_pdp_simental
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_simental(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Simental'
         ",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Simental: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "Fetched {} PDP Simental records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    Ok(decrypted_rows)
}

//PDP TIDAK AKTIF
#[get("/api/adminpanel/pdp-tidak-aktif")]
pub async fn list_pdp_tidak_aktif(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching PDP tidak aktif: '{}', page: {}, limit: {}",
        keyword,
        page,
        limit
    );

    // **APPROACH HYBRID: Database query + Application filtering**
    let all_pdp_tidak_aktif = fetch_all_pdp_tidak_aktif(
        pool.clone(),
        pagination.provinsi_id,
        pagination.kabupaten_id,
    )
    .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_tidak_aktif, &keyword)
    } else {
        all_pdp_tidak_aktif
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let start_idx = offset as usize;
    let end_idx = std::cmp::min(start_idx + limit as usize, filtered_data.len());
    let paginated_data = if start_idx < filtered_data.len() {
        filtered_data[start_idx..end_idx].to_vec()
    } else {
        Vec::new()
    };

    log::debug!(
        "Results: {}/{} records, page {}/{}",
        paginated_data.len(),
        total,
        page,
        total_pages
    );

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        limit: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from: (start_idx + 1) as u32,
        to: end_idx as u32,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_all_pdp_tidak_aktif(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Tidak Aktif'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Tidak Aktif: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "Fetched {} PDP Tidak Aktif records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    Ok(decrypted_rows)
}

#[put("/api/adminpanel/pdp-update-status/{id}")]
pub async fn update_status(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    mut payload: Multipart, // Hapus .clone() di sini
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id = path.into_inner();

    // Proses multipart payload sekali saja untuk mengambil semua field
    let mut status_value = String::new();
    let mut keterangan_value = String::new();

    while let Some(mut field) = payload.try_next().await.map_err(|e| {
        log::error!("Error processing multipart field: {}", e);
        actix_web::error::ErrorBadRequest("Invalid multipart data")
    })? {
        let field_name = field.name().unwrap_or_default().to_string();

        if field_name == "status" {
            let mut bytes = web::BytesMut::new();
            while let Some(chunk) = field.try_next().await.map_err(|e| {
                log::error!("Error reading status field: {}", e);
                actix_web::error::ErrorBadRequest("Error reading status field")
            })? {
                bytes.extend_from_slice(&chunk);
            }
            status_value = String::from_utf8(bytes.to_vec()).map_err(|_| {
                actix_web::error::ErrorBadRequest("Status field is not valid UTF-8")
            })?;
        } else if field_name == "keterangan" {
            let mut bytes = web::BytesMut::new();
            while let Some(chunk) = field.try_next().await.map_err(|e| {
                log::error!("Error reading keterangan field: {}", e);
                actix_web::error::ErrorBadRequest("Error reading keterangan field")
            })? {
                bytes.extend_from_slice(&chunk);
            }
            keterangan_value = String::from_utf8(bytes.to_vec()).map_err(|_| {
                actix_web::error::ErrorBadRequest("Keterangan field is not valid UTF-8")
            })?;
        } else {
            // Skip unknown fields
            while let Some(_) = field.try_next().await.map_err(|e| {
                log::warn!("Error skipping unknown field {}: {}", field_name, e);
                actix_web::error::ErrorBadRequest("Error processing multipart data")
            })? {}
        }
    }

    let new_status = status_value.trim().to_string();
    let keterangan = keterangan_value.trim().to_string();

    if new_status.is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Field 'status' wajib diisi",
        ));
    }

    // Jika status Ditolak dan keterangan kosong
    if new_status.eq_ignore_ascii_case("Ditolak") && keterangan.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Alasan penolakan wajib diisi",
        ));
    }

    // PERBAIKAN: Baca thn_tugas sebagai i32 langsung dari database
    let (email_cipher, nama_cipher, telepon_cipher, posisi, photo, alamat, id_provinsi, id_kabupaten, email_nonce, nama_nonce, telepon_nonce, jk, tingkat_kepengurusan, id_provinsi_domisili, id_kabupaten_domisili, thn_tugas, no_simental, jabatan, file_piagam): (
        Option<Vec<u8>>,
        Vec<u8>,
        Option<Vec<u8>>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<i32>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = sqlx::query("SELECT email, nama_lengkap, telepon, posisi, photo, alamat, id_provinsi, id_kabupaten, email_nonce, nama_nonce, telepon_nonce, jk, tingkat_kepengurusan, id_provinsi_domisili, id_kabupaten_domisili, thn_tugas, no_simental, jabatan, file_piagam FROM pdp WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.get_ref())
        .await
        .map(|row| {
            (
                row.get::<Option<Vec<u8>>, _>(0),   // email (dienkripsi)
                row.get::<Vec<u8>, _>(1),           // nama_lengkap (dienkripsi)
                row.get::<Option<Vec<u8>>, _>(2),   // telepon (dienkripsi)
                row.get::<Option<String>, _>(3),    // posisi (tidak dienkripsi)
                row.get::<Option<String>, _>(4),    // avatar (tidak dienkripsi)
                row.get::<Option<String>, _>(5),    // alamat (tidak dienkripsi)
                row.get::<Option<i32>, _>(6),       // id_provinsi (tidak dienkripsi)
                row.get::<Option<i32>, _>(7),       // id_kabupaten (tidak dienkripsi)
                row.get::<Option<Vec<u8>>, _>(8),   // email_nonce
                row.get::<Option<Vec<u8>>, _>(9),   // nama_nonce
                row.get::<Option<Vec<u8>>, _>(10),  // telepon_nonce
                row.get::<Option<String>, _>(11),   // jk
                row.get::<Option<String>, _>(12),   // tingkat_kepengurusan
                row.get::<Option<i32>, _>(13),      // id_provinsi_domisili
                row.get::<Option<i32>, _>(14),      // id_kabupaten_domisili
                row.get::<Option<i32>, _>(15),      // thn_tugas sebagai i32 langsung
                row.get::<Option<String>, _>(16),   // no_simental
                row.get::<Option<String>, _>(17),   // jabatan
                row.get::<Option<String>, _>(18),   // file_piagam
            )
        })
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Dekripsi email, nama, dan telepon
    let key = crate::utils::get_encryption_key()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let nama = if let Some(nonce) = nama_nonce {
        crate::utils::decrypt_data(&nama_cipher, &nonce, &key)
            .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        return Err(actix_web::error::ErrorInternalServerError(
            "Missing nama_nonce",
        ));
    };

    let email = if let (Some(email_cipher), Some(nonce)) = (email_cipher, email_nonce) {
        Some(
            crate::utils::decrypt_data(&email_cipher, &nonce, &key)
                .map_err(actix_web::error::ErrorInternalServerError)?,
        )
    } else {
        None
    };

    let telepon = if let (Some(telepon_cipher), Some(nonce)) = (telepon_cipher, telepon_nonce) {
        Some(
            crate::utils::decrypt_data(&telepon_cipher, &nonce, &key)
                .map_err(actix_web::error::ErrorInternalServerError)?,
        )
    } else {
        None
    };

    let mut final_no_simental = no_simental.clone();
    let should_generate_simental = (new_status.eq_ignore_ascii_case("Simental")
        || new_status.eq_ignore_ascii_case("Verified"))
        && (final_no_simental.is_none()
            || final_no_simental
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true));

    if should_generate_simental {
        final_no_simental = Some(
            generate_no_simental(
                pool.get_ref(),
                &tingkat_kepengurusan,
                &jk,
                id_provinsi_domisili,
                id_kabupaten_domisili,
                thn_tugas,
            )
            .await
            .map_err(|e| {
                log::error!("Gagal generate no_simental: {}", e);
                actix_web::error::ErrorInternalServerError("Gagal generate nomor simental")
            })?,
        );
    }

    // Update status pdp, no_simental, dan keterangan jika ada
    let mut update_query = "UPDATE pdp SET status = ?, updated_at = NOW()".to_string();
    if final_no_simental.is_some() {
        update_query.push_str(", no_simental = ?");
    }
    if !keterangan.trim().is_empty() {
        update_query.push_str(", keterangan = ?");
    }
    update_query.push_str(" WHERE id = ?");

    let mut query = sqlx::query(&update_query).bind(&new_status);

    if let Some(no_simental_value) = &final_no_simental {
        query = query.bind(no_simental_value);
    }

    if !keterangan.trim().is_empty() {
        query = query.bind(&keterangan);
    }

    query = query.bind(&id);

    query
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // ===== TAMBAHAN: Insert ke tabel pelaksana jika posisi = "Pelaksana" =====
    let mut pelaksana_inserted = false;
    if new_status.eq_ignore_ascii_case("Verified") && posisi.as_deref() == Some("Pelaksana") {
        log::debug!(
            "Insert pelaksana - tingkat: {:?}, provinsi: {:?}, kabupaten: {:?}",
            tingkat_kepengurusan,
            id_provinsi,
            id_kabupaten
        );

        pelaksana_inserted = insert_into_pelaksana_table(
            pool.get_ref(),
            id.clone(),
            &nama,
            &photo,
            &tingkat_kepengurusan,
            &jabatan,
            &id_provinsi,
            &id_kabupaten,
        )
        .await
        .map_err(|e| {
            log::error!("Gagal insert ke tabel pelaksana: {}", e);
            actix_web::error::ErrorInternalServerError("Gagal menambahkan data pelaksana")
        })?;
    }

    // Jika Verified => insert ke tabel users dan kirim email
    if new_status.eq_ignore_ascii_case("Verified") {
        if let Some(email) = email.as_deref() {
            if !email.trim().is_empty() {
                // Generate random password
                let plain_password = crate::utils::generate_random_password(8);

                // Hash password untuk disimpan di database
                let hashed_password = hash(&plain_password, DEFAULT_COST)
                    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

                // Ambil role dari posisi
                let role = if let Some(posisi) = &posisi {
                    if posisi.trim().is_empty() {
                        "User".to_string()
                    } else {
                        posisi.clone()
                    }
                } else {
                    "User".to_string() // Default role jika posisi null
                };

                // Cek apakah user sudah ada berdasarkan email
                let existing_user: Option<(String,)> =
                    sqlx::query_as("SELECT id FROM users WHERE email = ?")
                        .bind(email) // email: &str
                        .fetch_optional(pool.get_ref())
                        .await
                        .map_err(actix_web::error::ErrorInternalServerError)?;

                if existing_user.is_none() {
                    let user_id = generate_short_uuid_10_upper();

                    sqlx::query(
                        "INSERT INTO users (id, name, email, role, password, avatar, phone, id_pdp, address, id_provinsi, id_kabupaten, created_at, updated_at)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())",
                    )
                    .bind(&user_id)     // ✅ users.id (UUID10)
                    .bind(&nama)        // name
                    .bind(email)        // email (&str)
                    .bind(&role)        // role
                    .bind(&hashed_password)
                    .bind(&photo)
                    .bind(&telepon)
                    .bind(&id)          // id_pdp (String)
                    .bind(&alamat)
                    .bind(&id_provinsi)
                    .bind(&id_kabupaten)
                    .execute(pool.get_ref())
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
                } else {
                    // Update user yang sudah ada (tidak perlu ubah users.id)
                    sqlx::query(
                        "UPDATE users
                         SET name = ?, role = ?, password = ?, avatar = ?, phone = ?, id_pdp = ?, address = ?, id_provinsi = ?, id_kabupaten = ?, updated_at = NOW()
                         WHERE email = ?",
                    )
                    .bind(&nama)
                    .bind(&role)
                    .bind(&hashed_password)
                    .bind(&photo)
                    .bind(&telepon)
                    .bind(&id)
                    .bind(&alamat)
                    .bind(&id_provinsi)
                    .bind(&id_kabupaten)
                    .bind(email) // ✅ &str
                    .execute(pool.get_ref())
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
                }
                // Kirim email dengan password
                let mail_res = send_verified_email(email, &nama, &plain_password).await;
                return match mail_res {
                    Ok(_) => Ok(HttpResponse::Ok().json(ApiMessage {
                        message: format!("Status berhasil diupdate! User telah dibuat dengan role '{}'{}{}",
                            role,
                            if should_generate_simental { " dan NRA telah digenerate" } else { "" },
                            if pelaksana_inserted { " serta data telah ditambahkan ke tabel pelaksana" } else { "" }),
                    })),
                    Err(e) => Ok(HttpResponse::Ok().json(ApiMessage {
                        message: format!("Status berhasil diupdate, user dibuat dengan role '{}', namun gagal mengirim email: {}{}{}",
                            role, e,
                            if should_generate_simental { " dan NRA telah digenerate" } else { "" },
                            if pelaksana_inserted { " serta data telah ditambahkan ke tabel pelaksana" } else { "" }),
                    })),
                };
            }
        }
        return Ok(HttpResponse::Ok().json(ApiMessage {
            message: format!("Status berhasil diupdate! (Catatan: user tidak dibuat karena alamat email kosong){}{}",
                if should_generate_simental { " dan NRA telah digenerate" } else { "" },
                if pelaksana_inserted { " serta data telah ditambahkan ke tabel pelaksana" } else { "" }),
        }));
    }

    // Jika Ditolak => hapus data PDP dan kirim email pemberitahuan penolakan
    if new_status.eq_ignore_ascii_case("Ditolak") {
        // Hapus data PDP dari database
        let delete_result = sqlx::query("DELETE FROM pdp WHERE id = ?")
            .bind(&id)
            .execute(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        if delete_result.rows_affected() == 0 {
            return Err(actix_web::error::ErrorInternalServerError(
                "Gagal menghapus data PDP",
            ));
        }

        // Juga hapus dari users table jika ada
        let _ = sqlx::query("DELETE FROM users WHERE id_pdp = ?")
            .bind(&id)
            .execute(pool.get_ref())
            .await;

        // Hapus file photo jika ada
        if let Some(photo_path) = photo {
            let full_path = format!(".{}", photo_path.trim_start_matches('/'));
            let _ = tokio::fs::remove_file(full_path).await;
        }

        // Hapus file piagam jika ada
        if let Some(file_piagam) = &file_piagam {
            let full_path = format!(".{}", file_piagam.trim_start_matches('/'));
            let _ = tokio::fs::remove_file(full_path).await;
        }

        // Kirim email pemberitahuan penolakan
        if let Some(email) = email.as_deref() {
            if !email.trim().is_empty() {
                let mail_res = send_rejection_email(email, &nama, &keterangan).await;
                return match mail_res {
                Ok(_) => Ok(HttpResponse::Ok().json(ApiMessage {
                    message: "Data PDP berhasil ditolak dan dihapus! Email pemberitahuan telah dikirim. User dapat mendaftar ulang dengan email yang sama.".to_string(),
                })),
                Err(e) => Ok(HttpResponse::Ok().json(ApiMessage {
                    message: format!("Data PDP berhasil ditolak dan dihapus, namun gagal mengirim email: {}. User dapat mendaftar ulang dengan email yang sama.", e),
                })),
            };
            }
        }

        return Ok(HttpResponse::Ok().json(ApiMessage {
        message: "Data PDP berhasil ditolak dan dihapus! (Catatan: email pemberitahuan tidak dikirim karena alamat email kosong). User dapat mendaftar ulang.".to_string(),
    }));
    }

    Ok(HttpResponse::Ok().json(ApiMessage {
        message: format!(
            "Status berhasil diupdate!{}{}",
            if should_generate_simental {
                " dan NRA telah digenerate"
            } else {
                ""
            },
            if pelaksana_inserted {
                " serta data telah ditambahkan ke tabel pelaksana"
            } else {
                ""
            }
        ),
    }))
}
// Fungsi helper untuk insert ke tabel pelaksana - VERSI MACRO
async fn insert_into_pelaksana_table(
    pool: &MySqlPool,
    id_pdp: String,
    nama_lengkap: &str,
    photo: &Option<String>,
    tingkat_kepengurusan: &Option<String>,
    jabatan: &Option<String>,
    id_provinsi: &Option<i32>,
    id_kabupaten: &Option<i32>,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Tentukan query berdasarkan tingkat kepengurusan
    match tingkat_kepengurusan.as_deref() {
        Some("Pelaksana Tingkat Kabupaten/Kota") => {
            // Cek duplikasi
            let existing_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM pelaksana_kabupaten WHERE id_pdp = ?")
                    .bind(&id_pdp)
                    .fetch_one(pool)
                    .await?;

            if existing_count > 0 {
                log::info!("Data pelaksana kabupaten sudah ada untuk id_pdp {}", id_pdp);
                return Ok(false);
            }

            // Insert ke kabupaten
            let result = sqlx::query!(
                "INSERT INTO pelaksana_kabupaten (id_pdp, nama_lengkap, photo, jabatan, id_provinsi, id_kabupaten, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, NOW(), NOW())",
                id_pdp, nama_lengkap, photo, jabatan, id_provinsi, id_kabupaten
            )
            .execute(pool)
            .await?;

            Ok(result.rows_affected() > 0)
        }
        Some("Pelaksana Tingkat Provinsi") => {
            // Cek duplikasi
            let existing_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM pelaksana_provinsi WHERE id_pdp = ?")
                    .bind(&id_pdp)
                    .fetch_one(pool)
                    .await?;

            if existing_count > 0 {
                log::info!("Data pelaksana provinsi sudah ada untuk id_pdp {}", id_pdp);
                return Ok(false);
            }

            // Insert ke provinsi
            let result = sqlx::query!(
                "INSERT INTO pelaksana_provinsi (id_pdp, nama_lengkap, photo, jabatan, id_provinsi, created_at, updated_at) VALUES (?, ?, ?, ?, ?, NOW(), NOW())",
                id_pdp, nama_lengkap, photo, jabatan, id_provinsi
            )
            .execute(pool)
            .await?;

            Ok(result.rows_affected() > 0)
        }
        Some("Pelaksana Tingkat Pusat") => {
            // Cek duplikasi
            let existing_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM pelaksana_pusat WHERE id_pdp = ?")
                    .bind(&id_pdp)
                    .fetch_one(pool)
                    .await?;

            if existing_count > 0 {
                log::info!("Data pelaksana pusat sudah ada untuk id_pdp {}", id_pdp);
                return Ok(false);
            }

            // Insert ke pusat
            let result = sqlx::query!(
                "INSERT INTO pelaksana_pusat (id_pdp, nama_lengkap, photo, jabatan, created_at, updated_at) VALUES (?, ?, ?, ?, NOW(), NOW())",
                id_pdp, nama_lengkap, photo, jabatan
            )
            .execute(pool)
            .await?;

            Ok(result.rows_affected() > 0)
        }
        _ => {
            log::warn!(
                "Tingkat kepengurusan tidak valid untuk insert pelaksana: {:?}",
                tingkat_kepengurusan
            );
            Ok(false)
        }
    }
}

async fn generate_no_simental(
    pool: &MySqlPool,
    tingkat_kepengurusan: &Option<String>,
    jk: &Option<String>,
    id_provinsi_domisili: Option<i32>,
    id_kabupaten_domisili: Option<i32>,
    thn_tugas: Option<i32>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Kode Tingkat Kepengurusan
    let kode_tk_kepengurusan = match tingkat_kepengurusan.as_deref() {
        Some("Pelaksana Tingkat Kabupaten/Kota") => 3,
        Some("Pelaksana Tingkat Provinsi") => 2,
        _ => 1, // default untuk tingkat pusat atau lainnya
    };

    let kode_jk = match jk.as_deref() {
        Some("Laki-Laki") => 1,
        Some("Perempuan") => 2,
        _ => 0,
    };

    // ✅ Langsung pakai i32, tidak perlu parse dari String
    let last_two_digits = thn_tugas
        .map(|year| format!("{:02}", year % 100))
        .unwrap_or_else(|| "00".to_string());

    let last_nomor: Option<String> = sqlx::query_scalar(
        "SELECT RIGHT(no_simental, 5) AS digit_terbesar FROM pdp WHERE no_simental IS NOT NULL AND no_simental != '' ORDER BY digit_terbesar DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    let next_nomor_urut = if let Some(last_nomor) = last_nomor {
        let last_five = &last_nomor[last_nomor.len().saturating_sub(5)..];
        let next_num = last_five.parse::<u32>().unwrap_or(0) + 1;
        format!("{:05}", next_num)
    } else {
        "00001".to_string()
    };

    let nomor_register = format!(
        "{}{}{:02}{:02}{}{}",
        kode_tk_kepengurusan,
        kode_jk,
        id_provinsi_domisili.unwrap_or(0),
        id_kabupaten_domisili.unwrap_or(0),
        last_two_digits,
        next_nomor_urut
    );

    Ok(nomor_register)
}

#[get("/api/adminpanel/pdp/{id}")]
pub async fn get_pdp(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id = path.into_inner();

    let encrypted_row = sqlx::query_as::<_, EncryptedPdp>(
        "SELECT
        p.id,
        p.no_simental,
        p.no_piagam,
        p.nik,
        p.nama_lengkap,
        p.jk,
        p.tempat_lahir,
        p.tgl_lahir,
        p.alamat,
        p.pendidikan_terakhir,
        p.jurusan,
        p.nama_instansi_pendidikan,
        p.id_kabupaten_domisili,
        p.id_provinsi_domisili,
        p.email,
        p.telepon,
        p.posisi,
        p.jabatan,
        p.tingkat_kepengurusan,
        p.tingkat_penugasan,
        p.id_kabupaten,
        p.id_provinsi,
        CAST(p.thn_tugas AS CHAR) AS thn_tugas,
        p.`status`,
        p.photo,
        p.nik_nonce,
        p.nama_nonce,
        p.email_nonce,
        p.telepon_nonce,
        p.id_hobi,
        p.id_bakat,
        p.detail_bakat,
        p.id_minat,
        p.detail_minat,
        p.id_minat_2,
        p.detail_minat_2,
        p.keterangan,
        p.file_piagam,
        pd.nama_provinsi AS provinsi_domisili_nama,
        kd.nama_kabupaten AS kabupaten_domisili_nama,
        pp.nama_provinsi AS provinsi_penugasan_nama,
        kp.nama_kabupaten AS kabupaten_penugasan_nama
     FROM pdp AS p
     LEFT JOIN provinsi  AS pd ON p.id_provinsi_domisili = pd.id
     LEFT JOIN kabupaten AS kd ON p.id_kabupaten_domisili = kd.id
     LEFT JOIN provinsi  AS pp ON p.id_provinsi = pp.id
     LEFT JOIN kabupaten AS kp ON p.id_kabupaten = kp.id
WHERE p.id = ?
",
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    // Dekripsi data
    let decrypted_row = decrypt_pdp_row(encrypted_row)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(decrypted_row))
}

#[derive(Serialize, FromRow, Debug)]
struct Pendidikan {
    id: i32,
    id_pdp: i32,
    jenjang_pendidikan: String,
    nama_instansi_pendidikan: String,
    jurusan: Option<String>,
    tahun_masuk: u32,
    tahun_lulus: u32,
}
#[get("/api/adminpanel/pendidikan/{id}")]
pub async fn get_pendidikan_pdp(
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

#[derive(Serialize, FromRow, Debug)]
struct Organisasi {
    id: i32,
    id_pdp: i32,
    nama_organisasi: String,
    status: String,
    tahun_masuk: u32,
    tahun_keluar: Option<u32>,
}
#[get("/api/adminpanel/organisasi/{id}")]
pub async fn get_organiasi_pdp(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Admin Kesbangpol",
        "Pelaksana",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    let id = path.into_inner();

    let data_organisasi=  sqlx::query_as::<_, Organisasi>(
        "SELECT id, id_pdp, nama_organisasi, posisi, tahun_masuk, tahun_keluar, status FROM organisasi WHERE id_pdp = ? ",
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    Ok(HttpResponse::Ok().json(data_organisasi))
}

#[get("/api/userpanel/pdp/{id}")]
pub async fn get_user_pdp(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    path: Path<String>,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // pastikan tipe sama (sesuaikan i32/i64 sesuai skema Anda)
    let is_owner = claims.id_pdp.map(|pid| pid == id).unwrap_or(false);

    let is_admin = matches!(
        claims.role.as_str(),
        "Superadmin" | "Administrator" | "Pelaksana" | "Admin Kesbangpol"
    );

    // Tolak hanya jika BUKAN admin DAN BUKAN owner
    if !(is_admin || is_owner) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }
    let encrypted_row = sqlx::query_as::<_, EncryptedPdp>(
        r#"
        SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.photo,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.id = ?
         "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;

    // Dekripsi data
    let decrypted_row = decrypt_pdp_row(encrypted_row)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(decrypted_row))
}

#[derive(Debug, Deserialize)]
pub struct PdpUpdatePayload {
    pub nik: Option<String>,
    pub nama_lengkap: Option<String>,
    pub email: Option<String>,
    pub telepon: Option<String>,

    // Field lain (opsional semua)
    pub no_simental: Option<String>,
    pub no_piagam: Option<String>,
    pub jk: Option<String>,
    pub tempat_lahir: Option<String>,
    pub tgl_lahir: Option<NaiveDate>,
    pub alamat: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub jurusan: Option<String>,
    pub nama_instansi_pendidikan: Option<String>,
    pub posisi: Option<String>,
    pub tingkat_kepengurusan: Option<String>,
    pub jabatan: Option<String>,
    pub tingkat_penugasan: Option<String>,
    pub thn_tugas: Option<i32>,
    pub status: Option<String>,
    pub id_kabupaten_domisili: Option<i32>,
    pub id_provinsi_domisili: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub id_hobi: Option<Vec<String>>,
    pub id_bakat: Option<i32>,
    pub detail_bakat: Option<String>,
    pub id_minat: Option<i32>,
    pub detail_minat: Option<String>,
    pub id_minat_2: Option<i32>,
    pub detail_minat_2: Option<String>,
    pub keterangan: Option<String>,
}

async fn save_photo_file(
    mut field: actix_multipart::Field,
    dir: &Jalur,
    original_filename: Option<String>,
) -> Result<String, Error> {
    if !dir.exists() {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    let ext = original_filename
        .as_deref()
        .and_then(|n| Jalur::new(n).extension().and_then(|s| s.to_str()))
        .map(|s| format!(".{}", s))
        .unwrap_or_else(|| ".jpg".to_string());

    let filename = format!("pdp_{}{}", Uuid::new_v4(), ext);
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

    Ok(format!("uploads/assets/images/photos/{}", filename))
}

#[put("/api/userpanel/pdp/{id}")]
pub async fn update_pdp(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    mut multipart: Multipart,
) -> Result<impl Responder, Error> {
    let pdp_id = path.into_inner();

    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // pastikan tipe sama (sesuaikan i32/i64 sesuai skema Anda)
    let is_owner = claims.id_pdp.map(|pid| pid == pdp_id).unwrap_or(false);

    let is_admin = matches!(
        claims.role.as_str(),
        "Superadmin" | "Administrator" | "Pelaksana" | "Admin Kesbangpol"
    );

    // Tolak hanya jika BUKAN admin DAN BUKAN owner
    if !(is_admin || is_owner) {
        return Err(actix_web::error::ErrorForbidden(
            "Anda tidak memiliki akses ke API ini!",
        ));
    }

    // Ambil data lama termasuk photo
    let row_opt = sqlx::query("SELECT photo FROM pdp WHERE id = ?")
        .bind(pdp_id.clone())
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let old_photo: Option<String> = if let Some(row) = row_opt {
        row.get::<Option<String>, _>(0)
    } else {
        return Err(actix_web::error::ErrorNotFound("Data PDP tidak ditemukan"));
    };

    // ===== Baca multipart: payload(JSON) + photo(file opsional) =====
    let mut payload_json: Option<String> = None;
    let mut new_photo_rel: Option<String> = None;
    let mut has_new_photo = false;

    while let Some(mut field) = multipart
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().cloned();
        let name = cd
            .as_ref()
            .and_then(|c| c.get_name())
            .unwrap_or_default()
            .to_string();

        if name == "payload" {
            let mut bytes = web::BytesMut::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                bytes.extend_from_slice(&chunk);
            }
            payload_json = Some(
                String::from_utf8(bytes.to_vec())
                    .map_err(|_| actix_web::error::ErrorBadRequest("payload bukan UTF-8 valid"))?,
            );
        } else if name == "photo" {
            // Cek apakah benar-benar ada file yang diupload
            if let Some(filename) = cd.and_then(|c| c.get_filename().map(|s| s.to_string())) {
                if !filename.trim().is_empty() {
                    let orig = Some(filename);
                    let rel =
                        save_photo_file(field, Jalur::new("uploads/assets/images/photos"), orig)
                            .await?;
                    new_photo_rel = Some(rel);
                    has_new_photo = true;
                }
            }
        } else {
            // drain unknown field
            while let Some(_chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {}
        }
    }

    let upd: PdpUpdatePayload = if let Some(j) = payload_json {
        serde_json::from_str(&j).map_err(|e| {
            actix_web::error::ErrorBadRequest(format!("payload JSON invalid: {}", e))
        })?
    } else {
        return Err(actix_web::error::ErrorBadRequest(
            "Payload JSON tidak ditemukan",
        ));
    };

    // ===== Kunci & blind index keys =====
    let encryption_key_hex = env::var("ENCRYPTION_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("ENCRYPTION_KEY missing"))?;
    let blind_index_key_hex = env::var("BLIND_INDEX_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("BLIND_INDEX_KEY missing"))?;

    let encryption_key_bytes = hex::decode(&encryption_key_hex)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid hex encryption key"))?;
    let blind_index_key_bytes = hex::decode(&blind_index_key_hex)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid hex blind index key"))?;

    let key = secretbox::Key::from_slice(&encryption_key_bytes)
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Invalid encryption key size"))?;

    // ===== Siapkan nilai terenkripsi BILA disediakan di payload =====
    // NIK - hanya update jika ada nilai baru
    let (nik_cipher_opt, nik_nonce_opt, nik_bi_opt) = if let Some(nik_plain) = upd.nik.as_ref() {
        if !nik_plain.trim().is_empty() {
            let (nonce, cipher) = utils::encrypt_data(nik_plain.as_bytes(), &key);
            let bi = utils::generate_blind_index(nik_plain.as_bytes(), &blind_index_key_bytes);
            (Some(cipher), Some(nonce.as_ref().to_vec()), Some(bi))
        } else {
            (None, None, None)
        }
    } else {
        (None, None, None)
    };

    // NAMA - hanya update jika ada nilai baru
    let (nama_cipher_opt, nama_nonce_opt) = if let Some(nama_plain) = upd.nama_lengkap.as_ref() {
        if !nama_plain.trim().is_empty() {
            let (nonce, cipher) = utils::encrypt_data(nama_plain.as_bytes(), &key);
            (Some(cipher), Some(nonce.as_ref().to_vec()))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // EMAIL - hanya update jika ada nilai baru
    let (email_cipher_opt, email_nonce_opt, email_bi_opt) =
        if let Some(email_plain) = upd.email.as_ref() {
            if !email_plain.trim().is_empty() {
                let norm = email_plain.trim().to_ascii_lowercase();
                let (nonce, cipher) = utils::encrypt_data(norm.as_bytes(), &key);
                let bi = utils::generate_blind_index(norm.as_bytes(), &blind_index_key_bytes);
                (Some(cipher), Some(nonce.as_ref().to_vec()), Some(bi))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

    // TELEPON - hanya update jika ada nilai baru
    let (telp_cipher_opt, telp_nonce_opt, telp_bi_opt) =
        if let Some(telp_plain) = upd.telepon.as_ref() {
            if !telp_plain.trim().is_empty() {
                let norm = utils::normalize_phone(telp_plain);
                let (nonce, cipher) = utils::encrypt_data(norm.as_bytes(), &key);
                let bi = utils::generate_blind_index(norm.as_bytes(), &blind_index_key_bytes);
                (Some(cipher), Some(nonce.as_ref().to_vec()), Some(bi))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

    // ===== Transaction =====
    let mut tx = pool
        .begin()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Query: hanya update photo jika ada file baru yang diupload
    let sql = r#"
    UPDATE pdp SET
        -- Encrypted fields (hanya update jika param NOT NULL dan tidak kosong)
        nik                  = COALESCE(?, nik),
        nik_nonce            = COALESCE(?, nik_nonce),
        nik_blind_index      = COALESCE(?, nik_blind_index),

        nama_lengkap         = COALESCE(?, nama_lengkap),
        nama_nonce           = COALESCE(?, nama_nonce),

        email                = COALESCE(?, email),
        email_nonce          = COALESCE(?, email_nonce),
        email_blind_index    = COALESCE(?, email_blind_index),

        telepon              = COALESCE(?, telepon),
        telepon_nonce        = COALESCE(?, telepon_nonce),
        telepon_blind_index  = COALESCE(?, telepon_blind_index),


        -- Plain fields
        no_simental              = ?,
        no_piagam                = ?,
        jk                       = ?,
        tempat_lahir             = ?,
        tgl_lahir                = ?,
        alamat                   = ?,
        pendidikan_terakhir      = ?,
        jurusan                  = ?,
        nama_instansi_pendidikan = ?,
        posisi                   = ?,
        tingkat_kepengurusan     = ?,
        jabatan                  = ?,
        tingkat_penugasan        = ?,
        thn_tugas                = ?,
        status                   = ?,
        id_kabupaten_domisili    = ?,
        id_provinsi_domisili     = ?,
        id_kabupaten             = ?,
        id_provinsi              = ?,
        id_hobi                  = ?,
        id_bakat                 = ?,
        detail_bakat             = ?,
        id_minat                 = ?,
        detail_minat             = ?,
        id_minat_2               = ?,
        detail_minat_2           = ?,
        keterangan               = ?,
        photo                = COALESCE(?, photo),
        updated_at               = NOW()
    WHERE id = ?
    LIMIT 1
    "#;

    let mut q = sqlx::query(sql);

    // bind encrypted fields (gunakan nilai dari payload atau NULL untuk skip update)
    q = q
        .bind(nik_cipher_opt.as_ref())
        .bind(nik_nonce_opt.as_ref())
        .bind(nik_bi_opt.as_ref())
        .bind(nama_cipher_opt.as_ref())
        .bind(nama_nonce_opt.as_ref())
        .bind(email_cipher_opt.as_ref())
        .bind(email_nonce_opt.as_ref())
        .bind(email_bi_opt.as_ref())
        .bind(telp_cipher_opt.as_ref())
        .bind(telp_nonce_opt.as_ref())
        .bind(telp_bi_opt.as_ref());

    // bind plain fields - selalu gunakan nilai dari payload
    q = q
        .bind(upd.no_simental.as_ref())
        .bind(upd.no_piagam.as_ref())
        .bind(upd.jk.as_ref())
        .bind(upd.tempat_lahir.as_ref())
        .bind(upd.tgl_lahir)
        .bind(upd.alamat.as_ref())
        .bind(upd.pendidikan_terakhir.as_ref())
        .bind(upd.jurusan.as_ref())
        .bind(upd.nama_instansi_pendidikan.as_ref())
        .bind(upd.posisi.as_ref())
        .bind(upd.tingkat_kepengurusan.as_ref())
        .bind(upd.jabatan.as_ref())
        .bind(upd.tingkat_penugasan.as_ref())
        .bind(upd.thn_tugas)
        .bind(upd.status.as_ref())
        .bind(upd.id_kabupaten_domisili)
        .bind(upd.id_provinsi_domisili)
        .bind(upd.id_kabupaten)
        .bind(upd.id_provinsi)
        .bind(upd.id_hobi.as_ref().and_then(|hobi| {
            if hobi.is_empty() {
                None
            } else {
                Some(serde_json::to_string(hobi).unwrap_or_else(|_| "[]".to_string()))
            }
        }))
        .bind(upd.id_bakat)
        .bind(upd.detail_bakat.as_ref())
        .bind(upd.id_minat)
        .bind(upd.detail_minat.as_ref())
        .bind(upd.id_minat_2)
        .bind(upd.detail_minat_2.as_ref())
        .bind(upd.keterangan.as_ref())
        // Photo: hanya update jika ada file baru, otherwise tetap pakai yang lama
        .bind(if has_new_photo {
            new_photo_rel.as_ref()
        } else {
            None
        })
        .bind(&pdp_id);

    let result = q
        .execute(&mut *tx)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("Data PDP tidak ditemukan"));
    }

    tx.commit()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Hapus foto lama hanya jika ada foto baru & path berbeda
    if has_new_photo {
        if let (Some(new_rel), Some(old_rel)) = (new_photo_rel.as_ref(), old_photo.as_ref()) {
            if new_rel != old_rel {
                let old_abs = Jalur::new(".").join(old_rel.trim_start_matches('/'));
                if old_abs.exists() {
                    let _ = tokio::fs::remove_file(old_abs).await;
                }
            }
        }
    }

    #[derive(serde::Serialize)]
    struct Resp {
        message: String,
        photo: Option<String>,
        updated: bool,
    }

    Ok(HttpResponse::Ok().json(Resp {
        message: "PDP berhasil diperbarui".into(),
        photo: if has_new_photo {
            new_photo_rel
        } else {
            old_photo
        },
        updated: true,
    }))
}

#[delete("/api/adminpanel/pdp/{id}")]
pub async fn delete_pdp(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if ![
        "Superadmin",
        "Administrator",
        "Pelaksana",
        "Admin Kesbangpol",
    ]
    .contains(&claims.role.as_str())
    {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Administrator atau Pelaksana yang dapat mengakses",
        ));
    }

    let id = path.into_inner();

    // Ambil foto lama sebelum delete - gunakan fetch_optional untuk menangani kasus tidak ada data
    let old_photo_opt: Option<String> = match sqlx::query_as("SELECT photo FROM pdp WHERE id = ?")
        .bind(&id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    {
        Some((photo,)) => photo,
        None => None,
    };

    let old_photo_user: Option<String> =
        match sqlx::query_as("SELECT avatar FROM users WHERE id_pdp = ?")
            .bind(&id)
            .fetch_optional(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
        {
            Some((avatar,)) => avatar,
            None => None,
        };

    // Hapus file fisik jika ada
    if let Some(oldp) = old_photo_opt {
        remove_file_if_exists(&oldp);
    }
    if let Some(oldpu) = old_photo_user {
        remove_file_if_exists(&oldpu);
    }

    // Hapus row dari pdp
    let result = sqlx::query("DELETE FROM pdp WHERE id = ?")
        .bind(&id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().body("Data PDP tidak ditemukan"));
    }

    // Hapus user yang terkait (jika ada)
    let hapus_user = sqlx::query("DELETE FROM users WHERE id_pdp = ?")
        .bind(&id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Tidak perlu error jika user tidak ditemukan, karena mungkin tidak semua PDP punya user

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Berhasil dihapus",
        "id": id,
        "pdp_deleted": result.rows_affected(),
        "user_deleted": hapus_user.rows_affected()
    })))
}

#[get("/api/adminpanel/pdp-belum-registrasi-all")]
pub async fn list_pdp_belum_registrasi_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!(
        "Downloading all PDP Belum Registrasi with filter: '{}'",
        keyword
    );

    // Fetch semua data tanpa pagination
    let all_pdp_belum_registrasi =
        fetch_all_pdp_belum_registrasi_all(pool.clone(), params.provinsi_id, params.kabupaten_id)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_belum_registrasi, &keyword)
    } else {
        all_pdp_belum_registrasi
    };

    log::debug!(
        "Downloading {} PDP Belum Registrasi records",
        filtered_data.len()
    );

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_all_pdp_belum_registrasi_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE (p.status IS NULL OR p.status = '')",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Belum Registrasi: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Belum Registrasi records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-belum-diverifikasi-all")]
pub async fn list_pdp_belum_diverifikasi_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!(
        "Downloading all PDP Belum Diverifikasi with filter: '{}'",
        keyword
    );

    // Fetch semua data tanpa pagination
    let all_pdp_belum_diverifikasi =
        fetch_all_pdp_belum_diverifikasi_all(pool.clone(), params.provinsi_id, params.kabupaten_id)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_belum_diverifikasi, &keyword)
    } else {
        all_pdp_belum_diverifikasi
    };

    log::debug!(
        "Downloading {} PDP Belum Diverifikasi records",
        filtered_data.len()
    );

    Ok(HttpResponse::Ok().json(filtered_data))
}

// Fungsi baru tanpa pagination
async fn fetch_all_pdp_belum_diverifikasi_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Belum Diverifikasi'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Belum Diverifikasi: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Belum Diverifikasi records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-verified-all")]
pub async fn list_pdp_verified_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP verified with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_verified =
        fetch_all_pdp_verified_all(pool.clone(), params.provinsi_id, params.kabupaten_id).await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_verified, &keyword)
    } else {
        all_pdp_verified
    };

    log::debug!("Downloading {} PDP Verified records", filtered_data.len());

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_all_pdp_verified_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Verified'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Verified: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Verified records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-simental-all")]
pub async fn list_pdp_simental_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP simental with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_simental =
        fetch_all_pdp_simental_all(pool.clone(), params.provinsi_id, params.kabupaten_id).await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_simental, &keyword)
    } else {
        all_pdp_simental
    };

    log::debug!("Downloading {} PDP Simental records", filtered_data.len());

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_all_pdp_simental_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Simental'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Simental: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Simental records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

#[get("/api/adminpanel/pdp-tidak-aktif-all")]
pub async fn list_pdp_tidak_aktif_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP Tidak Aktif with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_tidak_aktif =
        fetch_all_pdp_tidak_aktif_all(pool.clone(), params.provinsi_id, params.kabupaten_id)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_tidak_aktif, &keyword)
    } else {
        all_pdp_tidak_aktif
    };

    log::debug!(
        "Downloading {} PDP Tidak Aktif records",
        filtered_data.len()
    );

    Ok(HttpResponse::Ok().json(filtered_data))
}

// Fungsi baru tanpa pagination
async fn fetch_all_pdp_tidak_aktif_all(
    pool: Data<MySqlPool>,
    provinsi_id: Option<i32>,
    kabupaten_id: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let mut sql = String::from(
        "SELECT
            p.id,
            p.no_simental,
            p.no_piagam,
            p.nik,
            p.nama_lengkap,
            p.jk,
            p.tempat_lahir,
            p.tgl_lahir,
            p.alamat,
            p.pendidikan_terakhir,
            p.jurusan,
            p.nama_instansi_pendidikan,
            p.id_kabupaten_domisili,
            p.id_provinsi_domisili,
            p.email,
            p.telepon,
            p.posisi,
            p.tingkat_kepengurusan,
            p.jabatan,
            p.tingkat_penugasan,
            p.id_kabupaten,
            p.id_provinsi,
            CAST(p.thn_tugas AS CHAR) as thn_tugas,
            p.status,
            p.photo,
            p.nik_nonce,
            p.nama_nonce,
            p.email_nonce,
            p.telepon_nonce,
            p.id_hobi,
            p.id_bakat,
            p.detail_bakat,
            p.id_minat,
            p.detail_minat,
            p.id_minat_2,
            p.detail_minat_2,
            p.keterangan,
            p.file_piagam,
            pd.nama_provinsi as provinsi_domisili_nama,
            kd.nama_kabupaten as kabupaten_domisili_nama,
            pp.nama_provinsi as provinsi_penugasan_nama,
            kp.nama_kabupaten as kabupaten_penugasan_nama
         FROM pdp p
         LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
         LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
         LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
         LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
         WHERE p.status = 'Tidak Aktif'",
    );

    // Tambahkan kondisi WHERE untuk filter wilayah
    if provinsi_id.is_some() || kabupaten_id.is_some() {
        sql.push_str(" AND ");

        if let Some(prov_id) = provinsi_id {
            sql.push_str(&format!("p.id_provinsi = {}", prov_id));

            if let Some(kab_id) = kabupaten_id {
                sql.push_str(&format!(" AND p.id_kabupaten = {}", kab_id));
            }
        } else if let Some(kab_id) = kabupaten_id {
            sql.push_str(&format!("p.id_kabupaten = {}", kab_id));
        }
    }

    sql.push_str(" ORDER BY p.id DESC");

    log::debug!("📝 Final SQL: {}", sql);

    let encrypted_rows: Vec<EncryptedPdp> = sqlx::query_as(&sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("❌ Error fetching all PDP Tidak Aktif: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    log::debug!(
        "✅ Fetched {} PDP Tidak Aktif records from database",
        encrypted_rows.len()
    );

    // Dekripsi semua data
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        let row_id = encrypted_row.id.clone();
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::warn!("⚠️ Gagal mendekripsi data PDP ID {}: {}", row_id, e);
                continue;
            }
        }
    }

    log::debug!("🔓 Successfully decrypted {} records", decrypted_rows.len());
    Ok(decrypted_rows)
}

fn generate_short_uuid_10_upper() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    raw[..10].to_uppercase()
}

// =======================
// Router config helper
// =======================
pub fn scope() -> actix_web::Scope {
    web::scope("")
        .service(list_pdp_belum_registrasi)
        .service(list_pdp_belum_diverifikasi)
        .service(list_pdp_verified)
        .service(list_pdp_simental)
        .service(list_pdp_tidak_aktif)
        .service(get_user_pdp)
        .service(get_pdp)
        .service(update_status)
        .service(get_pendidikan_pdp)
        .service(get_organiasi_pdp)
        .service(update_pdp)
        .service(delete_pdp)
        .service(list_pdp_belum_registrasi_all)
        .service(list_pdp_belum_diverifikasi_all)
        .service(list_pdp_verified_all)
        .service(list_pdp_simental_all)
        .service(list_pdp_tidak_aktif_all)
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
