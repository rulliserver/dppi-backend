<<<<<<< HEAD
//kesbangpol_controller.rs
use crate::{
    auth,
    controllers::pdp_controller::{
        EncryptedPdp, PaginationParams, PaginationPdpParams, Pdp, decrypt_pdp_row, filter_pdp_data,
    },
};
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, get,
    web::{self, Data, Path},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool, Row};
use std::cmp::{max, min};

//get data Pdp sesuai daerah
//Jumlah PDP berdasarkan status ================================================================================
#[derive(Serialize, FromRow, Debug)]
struct PdpStatus {
    id: i32,
    status: Option<String>,
}

#[get("/api/kesbangpol/pdp-terdaftar")]
pub async fn kesbangpol_get_pdp_terdaftar(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Admin Kesbangpol dan Pelaksana yang dapat mengakses",
        ));
    }

    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let pdp_terdaftar: Vec<PdpStatus> = if id_kabupaten.is_some() {
        // Admin Kesbangpol kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ? AND id_kabupaten = ?
             AND (status = '' OR status IS NULL)",
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // Admin Kesbangpol provinsi - filter hanya berdasarkan provinsi
        // dan pdp yang id_kabupaten-nya NULL atau empty
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ?
             AND (id_kabupaten IS NULL OR id_kabupaten = '')
             AND (status = '' OR status IS NULL)",
        )
        .bind(id_provinsi)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    Ok(HttpResponse::Ok().json(pdp_terdaftar))
}

#[get("/api/pdp-belum-diverifikasi")]
pub async fn kesbangpol_get_pdp_belum_diverifikasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Admin Kesbangpol dan Pelaksana yang dapat mengakses",
        ));
    }

    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let pdp_terdaftar: Vec<PdpStatus> = if id_kabupaten.is_some() {
        // Admin Kesbangpol kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ? AND id_kabupaten = ?
             AND status = 'Belum Diverifikasi' ",
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // Admin Kesbangpol provinsi - filter hanya berdasarkan provinsi
        // dan pdp yang id_kabupaten-nya NULL atau empty
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ?
             AND (id_kabupaten IS NULL OR id_kabupaten = '')
             AND status = 'Belum Diverifikasi' ",
        )
        .bind(id_provinsi)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    Ok(HttpResponse::Ok().json(pdp_terdaftar))
}

#[get("/api/kesbangpol/pdp-diverifikasi")]
pub async fn kesbangpol_get_pdp_diverifikasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Admin Kesbangpol dan Pelaksana yang dapat mengakses",
        ));
    }

    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let pdp_terdaftar: Vec<PdpStatus> = if id_kabupaten.is_some() {
        // Admin Kesbangpol kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ? AND id_kabupaten = ?
             AND status = 'Verified' ",
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // Admin Kesbangpol provinsi - filter hanya berdasarkan provinsi
        // dan pdp yang id_kabupaten-nya NULL atau empty
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ?
             AND (id_kabupaten IS NULL OR id_kabupaten = '')
             AND status = 'Verified' ",
        )
        .bind(id_provinsi)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    Ok(HttpResponse::Ok().json(pdp_terdaftar))
}

//PDP berdasarkan STATUS =================================================================================================================================================================

// =======================
// Controllers
// =======================
#[get("/api/kesbangpol/pdp-belum-registrasi")]
pub async fn kesbangpol_list_pdp_belum_registrasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 50);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    // Build WHERE clause berdasarkan level admin
    let (base_where_clause, has_kabupaten_filter) = if id_kabupaten.is_some() {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        (
            "WHERE p.id_provinsi = ? AND p.id_kabupaten = ? AND (p.status = '' OR p.status IS NULL)".to_string(),
            true
        )
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        (
            "WHERE p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '') AND (p.status = '' OR p.status IS NULL)".to_string(),
            false
        )
    };

    let mut where_clause = base_where_clause;
    let mut binds: Vec<String> = vec![];

    // Tambahkan filter keyword jika ada
    if !keyword.is_empty() {
        where_clause.push_str(
            " AND (p.no_piagam LIKE ? OR p.nama_lengkap LIKE ? OR p.no_simental LIKE ? OR p.jk LIKE ? \
             OR p.tingkat_penugasan LIKE ? OR CAST(p.thn_tugas AS CHAR) LIKE ? \
             OR p.email LIKE ? OR p.telepon LIKE ? OR p.nik LIKE ? \
             OR pd.nama_provinsi LIKE ? OR kd.nama_kabupaten LIKE ? \
             OR pp.nama_provinsi LIKE ? OR kp.nama_kabupaten LIKE ?)"
        );
        let needle = format!("%{}%", keyword);
        for _ in 0..13 {
            binds.push(needle.clone());
        }
    }

    // Query untuk total count
    let count_sql = format!(
        "SELECT COUNT(*) as cnt FROM pdp p
        LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
        LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
        LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
        LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
        {where_clause}"
    );

    let mut count_query = sqlx::query(&count_sql);

    // Bind parameter utama berdasarkan level admin
    count_query = count_query.bind(&id_provinsi);
    if has_kabupaten_filter {
        if let Some(kab_id) = &id_kabupaten {
            count_query = count_query.bind(kab_id);
        }
    }

    // Bind parameter pencarian
    for b in &binds {
        count_query = count_query.bind(b);
    }

    let total: i64 = count_query
        .fetch_one(pool.get_ref())
        .await
        .map(|row| row.get::<i64, _>("cnt"))
        .map_err(|e| {
            log::error!("Error counting PDP: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Query untuk data
    let data_sql = format!(
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
         {where_clause}
         ORDER BY p.id ASC
         LIMIT ? OFFSET ?"
    );

    let mut data_query = sqlx::query_as::<_, EncryptedPdp>(&data_sql);

    // Bind parameter utama berdasarkan level admin
    data_query = data_query.bind(&id_provinsi);
    if has_kabupaten_filter {
        if let Some(kab_id) = &id_kabupaten {
            data_query = data_query.bind(kab_id);
        }
    }

    // Bind parameter pencarian
    for b in &binds {
        data_query = data_query.bind(b);
    }

    // Bind parameter pagination
    data_query = data_query.bind(limit).bind(offset);

    let encrypted_rows: Vec<EncryptedPdp> =
        data_query.fetch_all(pool.get_ref()).await.map_err(|e| {
            log::error!("Error fetching PDP data: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Dekripsi rows
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::error!("Gagal mendekripsi data PDP: {:?}", e);
                continue;
            }
        }
    }

    // Hitung pagination info
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + limit, total as u32);

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
        data: decrypted_rows,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

//PDP BELUM DIVERIFIKASI
#[get("/api/kesbangpol/pdp-belum-diverifikasi")]
pub async fn kesbangpol_list_pdp_belum_diverifikasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol", "Superadmin"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching for: '{}', page: {}, limit: {}, provinsi: {:?}, kabupaten: {:?}",
        keyword,
        page,
        limit,
        id_provinsi,
        id_kabupaten
    );

    // **APPROACH HYBRID dengan FILTER WILAYAH**
    let kesbangpol_all_pdp_belum_diverifikasi =
        kesbangpol_fetch_all_pdp_belum_diverifikasi(pool.clone(), id_provinsi, id_kabupaten)
            .await?;
    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&kesbangpol_all_pdp_belum_diverifikasi, &keyword)
    } else {
        kesbangpol_all_pdp_belum_diverifikasi
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

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

    let from = if total == 0 {
        0
    } else {
        (start_idx + 1) as u32
    };
    let to = if total == 0 { 0 } else { end_idx as u32 };

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn kesbangpol_fetch_all_pdp_belum_diverifikasi(
    pool: web::Data<MySqlPool>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let provinsi_id =
        id_provinsi.ok_or_else(|| actix_web::error::ErrorBadRequest("ID Provinsi tidak valid"))?;

    let encrypted_rows = if let Some(kab_id) = id_kabupaten {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Belum Diverifikasi'
            AND p.id_provinsi = ? AND p.id_kabupaten = ?
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .bind(kab_id)
        .fetch_all(pool.get_ref())
        .await
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Belum Diverifikasi'
            AND p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .fetch_all(pool.get_ref())
        .await
    }
    .map_err(|e| {
        log::error!("❌ Error fetching PDP Belum Diverifikasi: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

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

//PDP VERIFIED
#[get("/api/kesbangpol/pdp-verified")]
pub async fn kesbangpol_list_pdp_verified(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching for: '{}', page: {}, limit: {}, provinsi: {:?}, kabupaten: {:?}",
        keyword,
        page,
        limit,
        id_provinsi,
        id_kabupaten
    );

    // **APPROACH HYBRID dengan FILTER WILAYAH**
    let kesbangpol_all_pdp_verified =
        kesbangpol_fetch_all_pdp_verified(pool.clone(), id_provinsi, id_kabupaten).await?;
    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&kesbangpol_all_pdp_verified, &keyword)
    } else {
        kesbangpol_all_pdp_verified
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

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

    let from = if total == 0 {
        0
    } else {
        (start_idx + 1) as u32
    };
    let to = if total == 0 { 0 } else { end_idx as u32 };

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn kesbangpol_fetch_all_pdp_verified(
    pool: web::Data<MySqlPool>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let provinsi_id =
        id_provinsi.ok_or_else(|| actix_web::error::ErrorBadRequest("ID Provinsi tidak valid"))?;

    let encrypted_rows = if let Some(kab_id) = id_kabupaten {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Verified'
            AND p.id_provinsi = ? AND p.id_kabupaten = ?
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .bind(kab_id)
        .fetch_all(pool.get_ref())
        .await
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Verified'
            AND p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .fetch_all(pool.get_ref())
        .await
    }
    .map_err(|e| {
        log::error!("❌ Error fetching PDP Verified: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

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

//PDP SIMENTAL
#[get("/api/kesbangpol/pdp-simental")]
pub async fn kesbangpol_list_pdp_simental(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching for: '{}', page: {}, limit: {}, provinsi: {:?}, kabupaten: {:?}",
        keyword,
        page,
        limit,
        id_provinsi,
        id_kabupaten
    );

    // **APPROACH HYBRID dengan FILTER WILAYAH**
    let kesbangpol_all_pdp_simental =
        kesbangpol_fetch_all_pdp_simental(pool.clone(), id_provinsi, id_kabupaten).await?;
    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&kesbangpol_all_pdp_simental, &keyword)
    } else {
        kesbangpol_all_pdp_simental
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

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

    let from = if total == 0 {
        0
    } else {
        (start_idx + 1) as u32
    };
    let to = if total == 0 { 0 } else { end_idx as u32 };

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn kesbangpol_fetch_all_pdp_simental(
    pool: web::Data<MySqlPool>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let provinsi_id =
        id_provinsi.ok_or_else(|| actix_web::error::ErrorBadRequest("ID Provinsi tidak valid"))?;

    let encrypted_rows = if let Some(kab_id) = id_kabupaten {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            AND p.id_provinsi = ? AND p.id_kabupaten = ?
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .bind(kab_id)
        .fetch_all(pool.get_ref())
        .await
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            AND p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .fetch_all(pool.get_ref())
        .await
    }
    .map_err(|e| {
        log::error!("❌ Error fetching PDP Simental: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

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
#[get("/api/kesbangpol/pdp-tidak-aktif")]
pub async fn kesbangpol_list_pdp_tidak_aktif(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching for: '{}', page: {}, limit: {}, provinsi: {:?}, kabupaten: {:?}",
        keyword,
        page,
        limit,
        id_provinsi,
        id_kabupaten
    );

    // **APPROACH HYBRID dengan FILTER WILAYAH**
    let kesbangpol_all_pdp_tidak_aktif =
        kesbangpol_fetch_all_pdp_tidak_aktif(pool.clone(), id_provinsi, id_kabupaten).await?;
    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&kesbangpol_all_pdp_tidak_aktif, &keyword)
    } else {
        kesbangpol_all_pdp_tidak_aktif
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

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

    let from = if total == 0 {
        0
    } else {
        (start_idx + 1) as u32
    };
    let to = if total == 0 { 0 } else { end_idx as u32 };

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn kesbangpol_fetch_all_pdp_tidak_aktif(
    pool: web::Data<MySqlPool>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let provinsi_id =
        id_provinsi.ok_or_else(|| actix_web::error::ErrorBadRequest("ID Provinsi tidak valid"))?;

    let encrypted_rows = if let Some(kab_id) = id_kabupaten {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Tidak Aktif'
            AND p.id_provinsi = ? AND p.id_kabupaten = ?
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .bind(kab_id)
        .fetch_all(pool.get_ref())
        .await
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Tidak Aktif'
            AND p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .fetch_all(pool.get_ref())
        .await
    }
    .map_err(|e| {
        log::error!("❌ Error fetching PDP Tidak Aktif: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

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

//pagination dan pencarian
#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    q: Option<String>,
}

#[derive(Serialize)]
struct PaginationLink {
    url: Option<String>,
    label: String,
    active: bool,
}

#[derive(Serialize)]
struct PaginatedResponse<T> {
    links: Vec<PaginationLink>,
    data: Vec<T>,
    current_page: u32,
    total_pages: u32,
    total_items: u64,
    per_page: u32,
    from: u64,
    to: u64,
    query: String,
}

fn make_url(base_path: &str, page: u32, per_page: u32, q: &Option<String>) -> String {
    match q {
        Some(s) if !s.is_empty() => format!(
            "{}?page={}&per_page={}&q={}",
            base_path,
            page,
            per_page,
            urlencoding::encode(s)
        ),
        _ => format!("{}?page={}&per_page={}", base_path, page, per_page),
    }
}

fn build_links(
    base_path: &str,
    current: u32,
    total_pages: u32,
    per_page: u32,
    q: &Option<String>,
) -> Vec<PaginationLink> {
    let mut links: Vec<PaginationLink> = Vec::new();

    // Prev
    if current > 1 {
        links.push(PaginationLink {
            url: Some(make_url(base_path, current - 1, per_page, q)),
            label: "&laquo; Previous".into(),
            active: false,
        });
    } else {
        links.push(PaginationLink {
            url: None,
            label: "&laquo; Previous".into(),
            active: false,
        });
    }

    // Pages 1..N
    for p in 1..=total_pages {
        if p == current {
            links.push(PaginationLink {
                url: None,
                label: p.to_string(),
                active: true,
            });
        } else {
            links.push(PaginationLink {
                url: Some(make_url(base_path, p, per_page, q)),
                label: p.to_string(),
                active: false,
            });
        }
    }

    // Next
    if current < total_pages {
        links.push(PaginationLink {
            url: Some(make_url(base_path, current + 1, per_page, q)),
            label: "Next &raquo;".into(),
            active: false,
        });
    } else {
        links.push(PaginationLink {
            url: None,
            label: "Next &raquo;".into(),
            active: false,
        });
    }

    links
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct PelaksanaProvinsi {
    id: i32,
    id_pdp: Option<i32>,
    id_provinsi: i32,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_provinsi: Option<String>,
}

#[get("/api/kesbangpol/pelaksana-provinsi")]
pub async fn kesbangpol_get_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Admin Kesbangpol dan Pelaksana yang dapat mengakses",
        ));
    }
    let id_provinsi = claims.id_provinsi;
    // Params
    let mut page = query.page.unwrap_or(1);
    if page == 0 {
        page = 1;
    }
    let mut per_page = query.per_page.unwrap_or(10);
    per_page = per_page.clamp(1, 100);

    let q_trimmed = query.q.as_ref().map(|s| s.trim().to_string());
    let has_q = q_trimmed.as_ref().is_some_and(|s| !s.is_empty());
    let base_path = "/api/adminpanel/pelaksana-provinsi";

    // ===== COUNT =====
    let (total_items,): (i64,) = if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE (pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?) AND pp.id_provinsi = ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.id_provinsi = ?
            "#,
        )
        .bind(id_provinsi)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let total_items = max(0, total_items) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + per_page as u64 - 1) / per_page as u64) as u32
    };
    let current = min(page, total_pages);
    let offset = ((current - 1) as u64) * (per_page as u64);

    // ===== DATA =====
    let data: Vec<PelaksanaProvinsi> = if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE (pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?) AND pp.id_provinsi = ?
            ORDER BY pp.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.id_provinsi = ?
            ORDER BY pp.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(id_provinsi)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        std::cmp::min(offset + data.len() as u64, total_items)
    };

    // links untuk komponen Pagination.tsx
    let links = build_links(base_path, current, total_pages, per_page, &q_trimmed);

    let resp = PaginatedResponse {
        links,
        data,
        current_page: current,
        total_pages,
        total_items,
        per_page,
        from,
        to,
        query: q_trimmed.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[derive(Serialize, FromRow, Debug)]
struct PelaksanaKabupaten {
    id: i32,
    id_pdp: Option<i32>,
    id_provinsi: i32,
    id_kabupaten: i32,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_provinsi: Option<String>,
    nama_kabupaten: Option<String>,
}
#[get("/api/kesbangpol/pelaksana-kabupaten")]
pub async fn kesbangpol_get_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, Error> {
    // Auth
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Administrator atau Pelaksana yang dapat mengakses",
        ));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    use std::cmp::{max, min};

    // Params
    let mut page = query.page.unwrap_or(1);
    if page == 0 {
        page = 1;
    }
    let mut per_page = query.per_page.unwrap_or(10);
    per_page = per_page.clamp(1, 100);

    let q_trimmed = query.q.as_ref().map(|s| s.trim().to_string());
    let has_q = q_trimmed.as_ref().is_some_and(|s| !s.is_empty());
    let base_path = "/api/adminpanel/pelaksana-kabupaten";

    // COUNT
    let (total_items,): (i64,) = if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ? OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?) AND p.id = ? AND k.id =?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE p.id = ? AND k.id =?
            "#,
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let total_items = max(0, total_items) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + per_page as u64 - 1) / per_page as u64) as u32
    };
    let current = min(page, total_pages);
    let offset = ((current - 1) as u64) * (per_page as u64);

    // DATA
    let data: Vec<PelaksanaKabupaten> = if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ? OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?) AND p.id = ? AND k.id =?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE p.id = ? AND k.id =?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        std::cmp::min(offset + data.len() as u64, total_items)
    };

    let links = build_links(base_path, current, total_pages, per_page, &q_trimmed);

    let resp = PaginatedResponse {
        links,
        data,
        current_page: current,
        total_pages,
        total_items,
        per_page,
        from,
        to,
        query: q_trimmed.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

//provinsi
#[derive(Serialize, FromRow, Debug)]
struct Provinsi {
    id: i32,
    nama_provinsi: String,
}
//provinsi-by-id
#[get("/api/provinsi/{id}")]
pub async fn get_provinsi_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let provinsi = sqlx::query_as::<_, Provinsi>(
        r#"
        SELECT id, nama_provinsi FROM provinsi WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(provinsi))
}

#[derive(Serialize, FromRow, Debug)]
struct Kabupaten {
    id: i32,
    id_provinsi: i32,
    nama_kabupaten: String,
}
//kabupaten-by-id
#[get("/api/kabupaten/{id}")]
pub async fn get_kabupaten_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let provinsi = sqlx::query_as::<_, Kabupaten>(
        r#"
        SELECT id, id_provinsi, nama_kabupaten FROM kabupaten WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(provinsi))
}
#[get("/api/wilayah/kabupaten/{provinsi_id}")]
pub async fn get_kabupaten_by_provinsi(
    pool: Data<MySqlPool>,
    path: Path<i32>,
) -> Result<impl Responder, Error> {
    let provinsi_id = path.into_inner();

    let kabupaten: Vec<(i32, String)> = sqlx::query_as(
        "SELECT id, nama_kabupaten FROM kabupaten WHERE id_provinsi = ? ORDER BY nama_kabupaten",
    )
    .bind(provinsi_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Error fetching kabupaten: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    #[derive(Serialize)]
    struct Wilayah {
        id: i32,
        nama_kabupaten: String,
    }

    let result: Vec<Wilayah> = kabupaten
        .into_iter()
        .map(|(id, nama_kabupaten)| Wilayah { id, nama_kabupaten })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[get("/api/kesbangpol/pdp-belum-registrasi-all")]
pub async fn pdp_kesbangpol_belum_registrasi_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!(
        "Downloading all PDP Belum Registrasi with filter: '{}'",
        keyword
    );

    // Fetch semua data tanpa pagination
    let all_pdp_belum_registrasi = fetch_pdp_kesbangpol_belum_registrasi_all(
        pool.clone(),
        claims.id_provinsi,
        claims.id_kabupaten,
    )
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

async fn fetch_pdp_kesbangpol_belum_registrasi_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
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

#[get("/api/kesbangpol/pdp-belum-diverifikasi-all")]
pub async fn pdp_kesbangpol_belum_diverifikasi_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!(
        "Downloading all PDP Belum Diverifikasi with filter: '{}'",
        keyword
    );

    // Fetch semua data tanpa pagination
    let all_pdp_belum_diverifikasi = fetch_pdp_kesbangpol_belum_diverifikasi_all(
        pool.clone(),
        claims.id_provinsi,
        claims.id_kabupaten,
    )
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

async fn fetch_pdp_kesbangpol_belum_diverifikasi_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
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

#[get("/api/kesbangpol/pdp-verified-all")]
pub async fn pdp_kesbangpol_verified_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP verified with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_verified =
        fetch_pdp_kesbangpol_verified_all(pool.clone(), claims.id_provinsi, claims.id_kabupaten)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_verified, &keyword)
    } else {
        all_pdp_verified
    };

    log::debug!("Downloading {} PDP Verified records", filtered_data.len());

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_pdp_kesbangpol_verified_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
    }

    sql.push_str(" ORDER BY p.id DESC");

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

#[get("/api/kesbangpol/pdp-simental-all")]
pub async fn pdp_kesbangpol_simental_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP simental with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_simental =
        fetch_pdp_kesbangpol_simental_all(pool.clone(), claims.id_provinsi, claims.id_kabupaten)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_simental, &keyword)
    } else {
        all_pdp_simental
    };

    log::debug!("Downloading {} PDP Simental records", filtered_data.len());

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_pdp_kesbangpol_simental_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
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

#[get("/api/kesbangpol/pdp-tidak-aktif-all")]
pub async fn pdp_kesbangpol_tidak_aktif_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP Tidak Aktif with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_tidak_aktif =
        fetch_pdp_kesbangpol_tidak_aktif_all(pool.clone(), claims.id_provinsi, claims.id_kabupaten)
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

async fn fetch_pdp_kesbangpol_tidak_aktif_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
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
=======
//kesbangpol_controller.rs
use crate::{
    auth,
    controllers::pdp_controller::{
        EncryptedPdp, PaginationParams, PaginationPdpParams, Pdp, decrypt_pdp_row, filter_pdp_data,
    },
};
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, get,
    web::{self, Data, Path},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool, Row};
use std::cmp::{max, min};

//get data Pdp sesuai daerah
//Jumlah PDP berdasarkan status ================================================================================
#[derive(Serialize, FromRow, Debug)]
struct PdpStatus {
    id: i32,
    status: Option<String>,
}

#[get("/api/kesbangpol/pdp-terdaftar")]
pub async fn kesbangpol_get_pdp_terdaftar(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Admin Kesbangpol dan Pelaksana yang dapat mengakses",
        ));
    }

    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let pdp_terdaftar: Vec<PdpStatus> = if id_kabupaten.is_some() {
        // Admin Kesbangpol kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ? AND id_kabupaten = ?
             AND (status = '' OR status IS NULL)",
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // Admin Kesbangpol provinsi - filter hanya berdasarkan provinsi
        // dan pdp yang id_kabupaten-nya NULL atau empty
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ?
             AND (id_kabupaten IS NULL OR id_kabupaten = '')
             AND (status = '' OR status IS NULL)",
        )
        .bind(id_provinsi)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    Ok(HttpResponse::Ok().json(pdp_terdaftar))
}

#[get("/api/pdp-belum-diverifikasi")]
pub async fn kesbangpol_get_pdp_belum_diverifikasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Admin Kesbangpol dan Pelaksana yang dapat mengakses",
        ));
    }

    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let pdp_terdaftar: Vec<PdpStatus> = if id_kabupaten.is_some() {
        // Admin Kesbangpol kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ? AND id_kabupaten = ?
             AND status = 'Belum Diverifikasi' ",
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // Admin Kesbangpol provinsi - filter hanya berdasarkan provinsi
        // dan pdp yang id_kabupaten-nya NULL atau empty
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ?
             AND (id_kabupaten IS NULL OR id_kabupaten = '')
             AND status = 'Belum Diverifikasi' ",
        )
        .bind(id_provinsi)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    Ok(HttpResponse::Ok().json(pdp_terdaftar))
}

#[get("/api/kesbangpol/pdp-diverifikasi")]
pub async fn kesbangpol_get_pdp_diverifikasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Admin Kesbangpol dan Pelaksana yang dapat mengakses",
        ));
    }

    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let pdp_terdaftar: Vec<PdpStatus> = if id_kabupaten.is_some() {
        // Admin Kesbangpol kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ? AND id_kabupaten = ?
             AND status = 'Verified' ",
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        // Admin Kesbangpol provinsi - filter hanya berdasarkan provinsi
        // dan pdp yang id_kabupaten-nya NULL atau empty
        sqlx::query_as::<_, PdpStatus>(
            "SELECT id, status FROM pdp
             WHERE id_provinsi = ?
             AND (id_kabupaten IS NULL OR id_kabupaten = '')
             AND status = 'Verified' ",
        )
        .bind(id_provinsi)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    Ok(HttpResponse::Ok().json(pdp_terdaftar))
}

//PDP berdasarkan STATUS =================================================================================================================================================================

// =======================
// Controllers
// =======================
#[get("/api/kesbangpol/pdp-belum-registrasi")]
pub async fn kesbangpol_list_pdp_belum_registrasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 50);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    // Build WHERE clause berdasarkan level admin
    let (base_where_clause, has_kabupaten_filter) = if id_kabupaten.is_some() {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        (
            "WHERE p.id_provinsi = ? AND p.id_kabupaten = ? AND (p.status = '' OR p.status IS NULL)".to_string(),
            true
        )
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        (
            "WHERE p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '') AND (p.status = '' OR p.status IS NULL)".to_string(),
            false
        )
    };

    let mut where_clause = base_where_clause;
    let mut binds: Vec<String> = vec![];

    // Tambahkan filter keyword jika ada
    if !keyword.is_empty() {
        where_clause.push_str(
            " AND (p.no_piagam LIKE ? OR p.nama_lengkap LIKE ? OR p.no_simental LIKE ? OR p.jk LIKE ? \
             OR p.tingkat_penugasan LIKE ? OR CAST(p.thn_tugas AS CHAR) LIKE ? \
             OR p.email LIKE ? OR p.telepon LIKE ? OR p.nik LIKE ? \
             OR pd.nama_provinsi LIKE ? OR kd.nama_kabupaten LIKE ? \
             OR pp.nama_provinsi LIKE ? OR kp.nama_kabupaten LIKE ?)"
        );
        let needle = format!("%{}%", keyword);
        for _ in 0..13 {
            binds.push(needle.clone());
        }
    }

    // Query untuk total count
    let count_sql = format!(
        "SELECT COUNT(*) as cnt FROM pdp p
        LEFT JOIN provinsi pd ON p.id_provinsi_domisili = pd.id
        LEFT JOIN kabupaten kd ON p.id_kabupaten_domisili = kd.id
        LEFT JOIN provinsi pp ON p.id_provinsi = pp.id
        LEFT JOIN kabupaten kp ON p.id_kabupaten = kp.id
        {where_clause}"
    );

    let mut count_query = sqlx::query(&count_sql);

    // Bind parameter utama berdasarkan level admin
    count_query = count_query.bind(&id_provinsi);
    if has_kabupaten_filter {
        if let Some(kab_id) = &id_kabupaten {
            count_query = count_query.bind(kab_id);
        }
    }

    // Bind parameter pencarian
    for b in &binds {
        count_query = count_query.bind(b);
    }

    let total: i64 = count_query
        .fetch_one(pool.get_ref())
        .await
        .map(|row| row.get::<i64, _>("cnt"))
        .map_err(|e| {
            log::error!("Error counting PDP: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Query untuk data
    let data_sql = format!(
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
         {where_clause}
         ORDER BY p.id ASC
         LIMIT ? OFFSET ?"
    );

    let mut data_query = sqlx::query_as::<_, EncryptedPdp>(&data_sql);

    // Bind parameter utama berdasarkan level admin
    data_query = data_query.bind(&id_provinsi);
    if has_kabupaten_filter {
        if let Some(kab_id) = &id_kabupaten {
            data_query = data_query.bind(kab_id);
        }
    }

    // Bind parameter pencarian
    for b in &binds {
        data_query = data_query.bind(b);
    }

    // Bind parameter pagination
    data_query = data_query.bind(limit).bind(offset);

    let encrypted_rows: Vec<EncryptedPdp> =
        data_query.fetch_all(pool.get_ref()).await.map_err(|e| {
            log::error!("Error fetching PDP data: {:?}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    // Dekripsi rows
    let mut decrypted_rows = Vec::new();
    for encrypted_row in encrypted_rows {
        match decrypt_pdp_row(encrypted_row) {
            Ok(decrypted) => decrypted_rows.push(decrypted),
            Err(e) => {
                log::error!("Gagal mendekripsi data PDP: {:?}", e);
                continue;
            }
        }
    }

    // Hitung pagination info
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + limit, total as u32);

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
        data: decrypted_rows,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

//PDP BELUM DIVERIFIKASI
#[get("/api/kesbangpol/pdp-belum-diverifikasi")]
pub async fn kesbangpol_list_pdp_belum_diverifikasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol", "Superadmin"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching for: '{}', page: {}, limit: {}, provinsi: {:?}, kabupaten: {:?}",
        keyword,
        page,
        limit,
        id_provinsi,
        id_kabupaten
    );

    // **APPROACH HYBRID dengan FILTER WILAYAH**
    let kesbangpol_all_pdp_belum_diverifikasi =
        kesbangpol_fetch_all_pdp_belum_diverifikasi(pool.clone(), id_provinsi, id_kabupaten)
            .await?;
    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&kesbangpol_all_pdp_belum_diverifikasi, &keyword)
    } else {
        kesbangpol_all_pdp_belum_diverifikasi
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

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

    let from = if total == 0 {
        0
    } else {
        (start_idx + 1) as u32
    };
    let to = if total == 0 { 0 } else { end_idx as u32 };

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn kesbangpol_fetch_all_pdp_belum_diverifikasi(
    pool: web::Data<MySqlPool>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let provinsi_id =
        id_provinsi.ok_or_else(|| actix_web::error::ErrorBadRequest("ID Provinsi tidak valid"))?;

    let encrypted_rows = if let Some(kab_id) = id_kabupaten {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Belum Diverifikasi'
            AND p.id_provinsi = ? AND p.id_kabupaten = ?
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .bind(kab_id)
        .fetch_all(pool.get_ref())
        .await
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Belum Diverifikasi'
            AND p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .fetch_all(pool.get_ref())
        .await
    }
    .map_err(|e| {
        log::error!("❌ Error fetching PDP Belum Diverifikasi: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

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

//PDP VERIFIED
#[get("/api/kesbangpol/pdp-verified")]
pub async fn kesbangpol_list_pdp_verified(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching for: '{}', page: {}, limit: {}, provinsi: {:?}, kabupaten: {:?}",
        keyword,
        page,
        limit,
        id_provinsi,
        id_kabupaten
    );

    // **APPROACH HYBRID dengan FILTER WILAYAH**
    let kesbangpol_all_pdp_verified =
        kesbangpol_fetch_all_pdp_verified(pool.clone(), id_provinsi, id_kabupaten).await?;
    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&kesbangpol_all_pdp_verified, &keyword)
    } else {
        kesbangpol_all_pdp_verified
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

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

    let from = if total == 0 {
        0
    } else {
        (start_idx + 1) as u32
    };
    let to = if total == 0 { 0 } else { end_idx as u32 };

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn kesbangpol_fetch_all_pdp_verified(
    pool: web::Data<MySqlPool>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let provinsi_id =
        id_provinsi.ok_or_else(|| actix_web::error::ErrorBadRequest("ID Provinsi tidak valid"))?;

    let encrypted_rows = if let Some(kab_id) = id_kabupaten {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Verified'
            AND p.id_provinsi = ? AND p.id_kabupaten = ?
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .bind(kab_id)
        .fetch_all(pool.get_ref())
        .await
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Verified'
            AND p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .fetch_all(pool.get_ref())
        .await
    }
    .map_err(|e| {
        log::error!("❌ Error fetching PDP Verified: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

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

//PDP SIMENTAL
#[get("/api/kesbangpol/pdp-simental")]
pub async fn kesbangpol_list_pdp_simental(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching for: '{}', page: {}, limit: {}, provinsi: {:?}, kabupaten: {:?}",
        keyword,
        page,
        limit,
        id_provinsi,
        id_kabupaten
    );

    // **APPROACH HYBRID dengan FILTER WILAYAH**
    let kesbangpol_all_pdp_simental =
        kesbangpol_fetch_all_pdp_simental(pool.clone(), id_provinsi, id_kabupaten).await?;
    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&kesbangpol_all_pdp_simental, &keyword)
    } else {
        kesbangpol_all_pdp_simental
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

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

    let from = if total == 0 {
        0
    } else {
        (start_idx + 1) as u32
    };
    let to = if total == 0 { 0 } else { end_idx as u32 };

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn kesbangpol_fetch_all_pdp_simental(
    pool: web::Data<MySqlPool>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let provinsi_id =
        id_provinsi.ok_or_else(|| actix_web::error::ErrorBadRequest("ID Provinsi tidak valid"))?;

    let encrypted_rows = if let Some(kab_id) = id_kabupaten {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            AND p.id_provinsi = ? AND p.id_kabupaten = ?
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .bind(kab_id)
        .fetch_all(pool.get_ref())
        .await
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            AND p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .fetch_all(pool.get_ref())
        .await
    }
    .map_err(|e| {
        log::error!("❌ Error fetching PDP Simental: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

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
#[get("/api/kesbangpol/pdp-tidak-aktif")]
pub async fn kesbangpol_list_pdp_tidak_aktif(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    log::debug!(
        "Searching for: '{}', page: {}, limit: {}, provinsi: {:?}, kabupaten: {:?}",
        keyword,
        page,
        limit,
        id_provinsi,
        id_kabupaten
    );

    // **APPROACH HYBRID dengan FILTER WILAYAH**
    let kesbangpol_all_pdp_tidak_aktif =
        kesbangpol_fetch_all_pdp_tidak_aktif(pool.clone(), id_provinsi, id_kabupaten).await?;
    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&kesbangpol_all_pdp_tidak_aktif, &keyword)
    } else {
        kesbangpol_all_pdp_tidak_aktif
    };

    // **Pagination di application level**
    let total = filtered_data.len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        ((total as f64) / (limit as f64)).ceil() as u32
    };

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

    let from = if total == 0 {
        0
    } else {
        (start_idx + 1) as u32
    };
    let to = if total == 0 { 0 } else { end_idx as u32 };

    let response = PaginatedResponse {
        data: paginated_data,
        current_page: page,
        total_pages,
        total_items: total,
        limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn kesbangpol_fetch_all_pdp_tidak_aktif(
    pool: web::Data<MySqlPool>,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
) -> Result<Vec<Pdp>, Error> {
    let provinsi_id =
        id_provinsi.ok_or_else(|| actix_web::error::ErrorBadRequest("ID Provinsi tidak valid"))?;

    let encrypted_rows = if let Some(kab_id) = id_kabupaten {
        // Admin kabupaten - filter berdasarkan provinsi dan kabupaten
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Tidak Aktif'
            AND p.id_provinsi = ? AND p.id_kabupaten = ?
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .bind(kab_id)
        .fetch_all(pool.get_ref())
        .await
    } else {
        // Admin provinsi - filter hanya berdasarkan provinsi, kabupaten NULL/kosong
        sqlx::query_as::<_, EncryptedPdp>(
            "
            SELECT
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
            WHERE p.status = 'Tidak Aktif'
            AND p.id_provinsi = ? AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')
            ORDER BY p.id DESC
            ",
        )
        .bind(provinsi_id)
        .fetch_all(pool.get_ref())
        .await
    }
    .map_err(|e| {
        log::error!("❌ Error fetching PDP Tidak Aktif: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

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

//pagination dan pencarian
#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    q: Option<String>,
}

#[derive(Serialize)]
struct PaginationLink {
    url: Option<String>,
    label: String,
    active: bool,
}

#[derive(Serialize)]
struct PaginatedResponse<T> {
    links: Vec<PaginationLink>,
    data: Vec<T>,
    current_page: u32,
    total_pages: u32,
    total_items: u64,
    per_page: u32,
    from: u64,
    to: u64,
    query: String,
}

fn make_url(base_path: &str, page: u32, per_page: u32, q: &Option<String>) -> String {
    match q {
        Some(s) if !s.is_empty() => format!(
            "{}?page={}&per_page={}&q={}",
            base_path,
            page,
            per_page,
            urlencoding::encode(s)
        ),
        _ => format!("{}?page={}&per_page={}", base_path, page, per_page),
    }
}

fn build_links(
    base_path: &str,
    current: u32,
    total_pages: u32,
    per_page: u32,
    q: &Option<String>,
) -> Vec<PaginationLink> {
    let mut links: Vec<PaginationLink> = Vec::new();

    // Prev
    if current > 1 {
        links.push(PaginationLink {
            url: Some(make_url(base_path, current - 1, per_page, q)),
            label: "&laquo; Previous".into(),
            active: false,
        });
    } else {
        links.push(PaginationLink {
            url: None,
            label: "&laquo; Previous".into(),
            active: false,
        });
    }

    // Pages 1..N
    for p in 1..=total_pages {
        if p == current {
            links.push(PaginationLink {
                url: None,
                label: p.to_string(),
                active: true,
            });
        } else {
            links.push(PaginationLink {
                url: Some(make_url(base_path, p, per_page, q)),
                label: p.to_string(),
                active: false,
            });
        }
    }

    // Next
    if current < total_pages {
        links.push(PaginationLink {
            url: Some(make_url(base_path, current + 1, per_page, q)),
            label: "Next &raquo;".into(),
            active: false,
        });
    } else {
        links.push(PaginationLink {
            url: None,
            label: "Next &raquo;".into(),
            active: false,
        });
    }

    links
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct PelaksanaProvinsi {
    id: i32,
    id_pdp: Option<i32>,
    id_provinsi: i32,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_provinsi: Option<String>,
}

#[get("/api/kesbangpol/pelaksana-provinsi")]
pub async fn kesbangpol_get_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, Error> {
    // Auth & Role
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Admin Kesbangpol dan Pelaksana yang dapat mengakses",
        ));
    }
    let id_provinsi = claims.id_provinsi;
    // Params
    let mut page = query.page.unwrap_or(1);
    if page == 0 {
        page = 1;
    }
    let mut per_page = query.per_page.unwrap_or(10);
    per_page = per_page.clamp(1, 100);

    let q_trimmed = query.q.as_ref().map(|s| s.trim().to_string());
    let has_q = q_trimmed.as_ref().is_some_and(|s| !s.is_empty());
    let base_path = "/api/adminpanel/pelaksana-provinsi";

    // ===== COUNT =====
    let (total_items,): (i64,) = if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE (pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?) AND pp.id_provinsi = ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.id_provinsi = ?
            "#,
        )
        .bind(id_provinsi)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let total_items = max(0, total_items) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + per_page as u64 - 1) / per_page as u64) as u32
    };
    let current = min(page, total_pages);
    let offset = ((current - 1) as u64) * (per_page as u64);

    // ===== DATA =====
    let data: Vec<PelaksanaProvinsi> = if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE (pp.nama_lengkap LIKE ? OR pp.jabatan LIKE ? OR p.nama_provinsi LIKE ?) AND pp.id_provinsi = ?
            ORDER BY pp.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, PelaksanaProvinsi>(
            r#"
            SELECT
                pp.id,
                pp.id_pdp,
                pp.id_provinsi,
                pp.nama_lengkap,
                pp.photo,
                pp.jabatan,
                p.nama_provinsi
            FROM pelaksana_provinsi pp
            LEFT JOIN provinsi p ON pp.id_provinsi = p.id
            WHERE pp.id_provinsi = ?
            ORDER BY pp.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(id_provinsi)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        std::cmp::min(offset + data.len() as u64, total_items)
    };

    // links untuk komponen Pagination.tsx
    let links = build_links(base_path, current, total_pages, per_page, &q_trimmed);

    let resp = PaginatedResponse {
        links,
        data,
        current_page: current,
        total_pages,
        total_items,
        per_page,
        from,
        to,
        query: q_trimmed.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[derive(Serialize, FromRow, Debug)]
struct PelaksanaKabupaten {
    id: i32,
    id_pdp: Option<i32>,
    id_provinsi: i32,
    id_kabupaten: i32,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_provinsi: Option<String>,
    nama_kabupaten: Option<String>,
}
#[get("/api/kesbangpol/pelaksana-kabupaten")]
pub async fn kesbangpol_get_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, Error> {
    // Auth
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Pelaksana", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Administrator atau Pelaksana yang dapat mengakses",
        ));
    }
    let id_provinsi = claims.id_provinsi;
    let id_kabupaten = claims.id_kabupaten;

    use std::cmp::{max, min};

    // Params
    let mut page = query.page.unwrap_or(1);
    if page == 0 {
        page = 1;
    }
    let mut per_page = query.per_page.unwrap_or(10);
    per_page = per_page.clamp(1, 100);

    let q_trimmed = query.q.as_ref().map(|s| s.trim().to_string());
    let has_q = q_trimmed.as_ref().is_some_and(|s| !s.is_empty());
    let base_path = "/api/adminpanel/pelaksana-kabupaten";

    // COUNT
    let (total_items,): (i64,) = if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ? OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?) AND p.id = ? AND k.id =?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE p.id = ? AND k.id =?
            "#,
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let total_items = max(0, total_items) as u64;
    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items + per_page as u64 - 1) / per_page as u64) as u32
    };
    let current = min(page, total_pages);
    let offset = ((current - 1) as u64) * (per_page as u64);

    // DATA
    let data: Vec<PelaksanaKabupaten> = if has_q {
        let like = format!("%{}%", q_trimmed.as_ref().unwrap());
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE (pk.nama_lengkap LIKE ? OR pk.jabatan LIKE ? OR k.nama_kabupaten LIKE ? OR p.nama_provinsi LIKE ?) AND p.id = ? AND k.id =?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(&like)
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        sqlx::query_as::<_, PelaksanaKabupaten>(
            r#"
            SELECT
                pk.id,
                pk.id_pdp,
                pk.id_provinsi,
                pk.id_kabupaten,
                pk.nama_lengkap,
                pk.photo,
                pk.jabatan,
                p.nama_provinsi,
                k.nama_kabupaten
            FROM pelaksana_kabupaten pk
            LEFT JOIN kabupaten k ON pk.id_kabupaten = k.id
            LEFT JOIN provinsi  p ON pk.id_provinsi  = p.id
            WHERE p.id = ? AND k.id =?
            ORDER BY pk.id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(id_provinsi)
        .bind(id_kabupaten)
        .bind(per_page as u64)
        .bind(offset)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    };

    let from = if total_items == 0 { 0 } else { offset + 1 };
    let to = if total_items == 0 {
        0
    } else {
        std::cmp::min(offset + data.len() as u64, total_items)
    };

    let links = build_links(base_path, current, total_pages, per_page, &q_trimmed);

    let resp = PaginatedResponse {
        links,
        data,
        current_page: current,
        total_pages,
        total_items,
        per_page,
        from,
        to,
        query: q_trimmed.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

//provinsi
#[derive(Serialize, FromRow, Debug)]
struct Provinsi {
    id: i32,
    nama_provinsi: String,
}
//provinsi-by-id
#[get("/api/provinsi/{id}")]
pub async fn get_provinsi_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let provinsi = sqlx::query_as::<_, Provinsi>(
        r#"
        SELECT id, nama_provinsi FROM provinsi WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(provinsi))
}

#[derive(Serialize, FromRow, Debug)]
struct Kabupaten {
    id: i32,
    id_provinsi: i32,
    nama_kabupaten: String,
}
//kabupaten-by-id
#[get("/api/kabupaten/{id}")]
pub async fn get_kabupaten_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let provinsi = sqlx::query_as::<_, Kabupaten>(
        r#"
        SELECT id, id_provinsi, nama_kabupaten FROM kabupaten WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(provinsi))
}
#[get("/api/wilayah/kabupaten/{provinsi_id}")]
pub async fn get_kabupaten_by_provinsi(
    pool: Data<MySqlPool>,
    path: Path<i32>,
) -> Result<impl Responder, Error> {
    let provinsi_id = path.into_inner();

    let kabupaten: Vec<(i32, String)> = sqlx::query_as(
        "SELECT id, nama_kabupaten FROM kabupaten WHERE id_provinsi = ? ORDER BY nama_kabupaten",
    )
    .bind(provinsi_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Error fetching kabupaten: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    #[derive(Serialize)]
    struct Wilayah {
        id: i32,
        nama_kabupaten: String,
    }

    let result: Vec<Wilayah> = kabupaten
        .into_iter()
        .map(|(id, nama_kabupaten)| Wilayah { id, nama_kabupaten })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

#[get("/api/kesbangpol/pdp-belum-registrasi-all")]
pub async fn pdp_kesbangpol_belum_registrasi_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!(
        "Downloading all PDP Belum Registrasi with filter: '{}'",
        keyword
    );

    // Fetch semua data tanpa pagination
    let all_pdp_belum_registrasi = fetch_pdp_kesbangpol_belum_registrasi_all(
        pool.clone(),
        claims.id_provinsi,
        claims.id_kabupaten,
    )
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

async fn fetch_pdp_kesbangpol_belum_registrasi_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
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

#[get("/api/kesbangpol/pdp-belum-diverifikasi-all")]
pub async fn pdp_kesbangpol_belum_diverifikasi_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!(
        "Downloading all PDP Belum Diverifikasi with filter: '{}'",
        keyword
    );

    // Fetch semua data tanpa pagination
    let all_pdp_belum_diverifikasi = fetch_pdp_kesbangpol_belum_diverifikasi_all(
        pool.clone(),
        claims.id_provinsi,
        claims.id_kabupaten,
    )
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

async fn fetch_pdp_kesbangpol_belum_diverifikasi_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
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

#[get("/api/kesbangpol/pdp-verified-all")]
pub async fn pdp_kesbangpol_verified_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP verified with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_verified =
        fetch_pdp_kesbangpol_verified_all(pool.clone(), claims.id_provinsi, claims.id_kabupaten)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_verified, &keyword)
    } else {
        all_pdp_verified
    };

    log::debug!("Downloading {} PDP Verified records", filtered_data.len());

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_pdp_kesbangpol_verified_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
    }

    sql.push_str(" ORDER BY p.id DESC");

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

#[get("/api/kesbangpol/pdp-simental-all")]
pub async fn pdp_kesbangpol_simental_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP simental with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_simental =
        fetch_pdp_kesbangpol_simental_all(pool.clone(), claims.id_provinsi, claims.id_kabupaten)
            .await?;

    let filtered_data = if !keyword.is_empty() {
        filter_pdp_data(&all_pdp_simental, &keyword)
    } else {
        all_pdp_simental
    };

    log::debug!("Downloading {} PDP Simental records", filtered_data.len());

    Ok(HttpResponse::Ok().json(filtered_data))
}

async fn fetch_pdp_kesbangpol_simental_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
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

#[get("/api/kesbangpol/pdp-tidak-aktif-all")]
pub async fn pdp_kesbangpol_tidak_aktif_all(
    req: HttpRequest,
    pool: Data<MySqlPool>,
    web::Query(params): web::Query<PaginationPdpParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Admin Kesbangpol", "Pelaksana"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let keyword = params.q.unwrap_or_default();

    log::debug!("Downloading all PDP Tidak Aktif with filter: '{}'", keyword);

    // Fetch semua data tanpa pagination
    let all_pdp_tidak_aktif =
        fetch_pdp_kesbangpol_tidak_aktif_all(pool.clone(), claims.id_provinsi, claims.id_kabupaten)
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

async fn fetch_pdp_kesbangpol_tidak_aktif_all(
    pool: Data<MySqlPool>,
    user_provinsi_id: Option<i32>,
    user_kabupaten_id: Option<i32>,
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

    if let Some(prov_id) = user_provinsi_id {
        sql.push_str(" AND p.id_provinsi = ");
        sql.push_str(&prov_id.to_string());
    }

    if let Some(kab_id) = user_kabupaten_id {
        sql.push_str(" AND p.id_kabupaten = ");
        sql.push_str(&kab_id.to_string());
    } else {
        sql.push_str(" AND (p.id_kabupaten IS NULL OR p.id_kabupaten = '')");
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
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
