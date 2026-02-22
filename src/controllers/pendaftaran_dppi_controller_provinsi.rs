// src/controllers/pendaftaran_dppi_controller_provinsi.rs
use actix_web::{HttpRequest, HttpResponse, delete, get, post, put, web};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::Local;
use rust_decimal::Decimal;
use rust_xlsxwriter::{Format, Workbook, XlsxError};
use serde_json::json;
use sqlx::MySqlPool;
use std::fs;
use std::path::Path;
use uuid::Uuid;
// Import models
use crate::utils::{SubmitConfirmationData, send_submit_confirmation_email};
use crate::{
    auth,
    models::pendaftaran_dppi_provinsi::{
        FilterParamsProvinsi, NewPendaftaranDppiProvinsi, PaginatedResponse,
        PendaftaranDppiWithProvinsi, UpdateStatusRequestProvinsi, UploadDocumentRequestProvinsi,
    },
};

#[get("/api/pendaftaran-dppi-provinsi/download")]
pub async fn download_to_excel_provinsi(
    pool: web::Data<MySqlPool>,
    query: web::Query<ExportParamsProvinsi>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };
    if !["Superadmin", "Administrator", "Admin Pendaftaran"].contains(&claims.role.as_str()) {
        return HttpResponse::Forbidden().json(json!({
            "error": "Anda tidak memiliki izin untuk mengakses API ini"
        }));
    }
    let pendaftaran_list = sqlx::query_as::<_, PendaftaranDppiWithProvinsi>(
        r#"
        SELECT * FROM pendaftaran_dppi_provinsi ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await;

    match pendaftaran_list {
        Ok(list) => {
            let use_simple = query.simple.unwrap_or(false);

            let result = if use_simple {
                create_excel_workbook_simple(&list)
            } else {
                create_excel_workbook(&list)
            };

            match result {
                Ok(bytes) => {
                    let filename = format!(
                        "pendaftaran-dppi-{}.xlsx",
                        Local::now().format("%Y%m%d_%H%M%S")
                    );

                    HttpResponse::Ok()
                        .append_header((
                            "Content-Type",
                            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                        ))
                        .append_header((
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", filename),
                        ))
                        .body(bytes)
                }
                Err(e) => {
                    log::error!("Error creating Excel: {}", e);
                    HttpResponse::InternalServerError()
                        .json(json!({"error": format!("Gagal membuat Excel: {}", e)}))
                }
            }
        }
        Err(e) => {
            log::error!("Error fetching data: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Gagal mengambil data"}))
        }
    }
}

#[get("/api/pendaftaran-dppi-provinsi/stats")]
pub async fn get_stats_provinsi(pool: web::Data<MySqlPool>, req: HttpRequest) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };
    if !["Superadmin", "Administrator", "Admin Pendaftaran"].contains(&claims.role.as_str()) {
        return HttpResponse::Forbidden().json(json!({
            "error": "Anda tidak memiliki izin untuk mengakses API ini"
        }));
    }
    let result = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total_pendaftaran,
            SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
            SUM(CASE WHEN status = 'review' THEN 1 ELSE 0 END) as review,
            SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END) as approved,
            SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END) as rejected,
            SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as completed,
            COUNT(DISTINCT id_provinsi) as total_provinsi
        FROM pendaftaran_dppi_provinsi
        "#
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(stats) => HttpResponse::Ok().json(json!({
            "total": stats.total_pendaftaran,
            "pending": stats.pending.unwrap_or_default() as Decimal,
            "review": stats.review.unwrap_or_default() as Decimal,
            "approved": stats.approved.unwrap_or_default() as Decimal,
            "rejected": stats.rejected.unwrap_or_default() as Decimal,
            "completed": stats.completed.unwrap_or_default() as Decimal,
            "total_provinsi": stats.total_provinsi,
        })),
        Err(e) => {
            log::error!("Error fetching stats: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Gagal mengambil statistik"
            }))
        }
    }
}

// Create new pendaftaran
#[post("/api/pendaftaran-dppi-provinsi")]
pub async fn create_pendaftaran_provinsi(
    pool: web::Data<MySqlPool>,
    form: web::Json<NewPendaftaranDppiProvinsi>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match req.headers().get("user-id") {
        Some(header) => header.to_str().unwrap_or("0").parse::<i32>().unwrap_or(0),
        None => 0,
    };

    let result = sqlx::query!(
        r#"
        INSERT INTO pendaftaran_dppi_provinsi (
            id_provinsi,
            nama_provinsi,
            nama_pic,
            jabatan_pic,
            nip_pic,
            no_telp_pic,
            email_pic,
            ketua_1,
            ketua_2,
            wakil_ketua_1,
            wakil_ketua_2,
            sekretaris_1,
            sekretaris_2,
            kepala_divisi_dukungan_1,
            kepala_divisi_dukungan_2,
            kepala_divisi_kompetensi_1,
            kepala_divisi_kompetensi_2,
            kepala_divisi_aktualisasi_1,
            kepala_divisi_aktualisasi_2,
            kepala_divisi_kominfo_1,
            kepala_divisi_kominfo_2,
            status,
            created_by,
            updated_by
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?)
        "#,
        form.id_provinsi,
        form.nama_provinsi,
        form.nama_pic,
        form.jabatan_pic,
        form.nip_pic,
        form.no_telp_pic,
        form.email_pic,
        form.ketua_1,
        form.ketua_2,
        form.wakil_ketua_1,
        form.wakil_ketua_2,
        form.sekretaris_1,
        form.sekretaris_2,
        form.kepala_divisi_dukungan_1,
        form.kepala_divisi_dukungan_2,
        form.kepala_divisi_kompetensi_1,
        form.kepala_divisi_kompetensi_2,
        form.kepala_divisi_aktualisasi_1,
        form.kepala_divisi_aktualisasi_2,
        form.kepala_divisi_kominfo_1,
        form.kepala_divisi_kominfo_2,
        user_id,
        user_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            let id = result.last_insert_id() as i32;

            // Ambil data yang baru saja diinsert untuk dikirim email
            let new_data_result = sqlx::query_as::<_, PendaftaranDppiWithProvinsi>(
                r#"SELECT * FROM pendaftaran_dppi_provinsi WHERE id = ?"#,
            )
            .bind(id)
            .fetch_one(pool.get_ref())
            .await;

            match new_data_result {
                Ok(new_data) => {
                    // Format ID Registrasi (bisa disesuaikan)
                    let id_registrasi = format!("DPPI-PROV-{:06}", id);

                    // Kirim email secara async (jangan blocking)
                    let email_to = new_data.email_pic.clone();
                    let email_data = SubmitConfirmationData::new(
                    format!("Provinsi {}", new_data.nama_provinsi),
                    new_data.nama_pic.clone(),
                    id_registrasi.clone(),
                    Local::now().format("%d/%m/%Y").to_string(),
                )
                .add_participant(
                    "Ketua",
                    vec![new_data.ketua_1.clone(), new_data.ketua_2.clone()]
                )
                .add_participant(
                    "Wakil Ketua",
                    vec![new_data.wakil_ketua_1.clone(), new_data.wakil_ketua_2.clone()]
                )
                .add_participant(
                    "Sekretaris",
                    vec![new_data.sekretaris_1.clone(), new_data.sekretaris_2.clone()]
                )
                .add_participant(
                    "Kepala Divisi Dukungan Pembentukan Paskibraka dan Purnapaskibraka Duta Pancasila",
                    vec![
                        new_data.kepala_divisi_dukungan_1.clone(),
                        new_data.kepala_divisi_dukungan_2.clone()
                    ]
                )
                .add_participant(
                    "Kepala Divisi Peningkatan Kompetensi",
                    vec![
                        new_data.kepala_divisi_kompetensi_1.clone(),
                        new_data.kepala_divisi_kompetensi_2.clone()
                    ]
                )
                .add_participant(
                    "Kepala Divisi Aktualisasi Nilai-Nilai Pancasila",
                    vec![
                        new_data.kepala_divisi_aktualisasi_1.clone(),
                        new_data.kepala_divisi_aktualisasi_2.clone()
                    ]
                )
                .add_participant(
                    "Kepala Divisi Komunikasi, Teknologi dan Informasi",
                    vec![
                        new_data.kepala_divisi_kominfo_1.clone(),
                        new_data.kepala_divisi_kominfo_2.clone()
                    ]
                );

                    // Kirim email di background tanpa blocking response
                    let _pool_clone = pool.clone();
                    let email_to_clone = email_to.clone();
                    let email_data_clone = email_data.clone();
                    actix_web::rt::spawn(async move {
                        match send_submit_confirmation_email(&email_to_clone, &email_data_clone)
                            .await
                        {
                            Ok(_) => {
                                log::info!(
                                    "Email konfirmasi berhasil dikirim ke: {}",
                                    email_to_clone
                                );
                            }
                            Err(e) => {
                                log::error!("Gagal mengirim email ke {}: {}", email_to_clone, e);
                            }
                        }
                    });

                    HttpResponse::Created().json(json!({
                    "message": "Pendaftaran berhasil dibuat dan email konfirmasi sedang dikirim",
                    "id": id,
                    "id_registrasi": id_registrasi
                }))
                }
                Err(e) => {
                    log::error!("Error fetching inserted data for email: {}", e);
                    HttpResponse::Created().json(json!({
                        "message": "Pendaftaran berhasil dibuat (email konfirmasi gagal)",
                        "id": id
                    }))
                }
            }
        }
        Err(e) => {
            log::error!("Error creating pendaftaran: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Gagal membuat pendaftaran"}))
        }
    }
}

#[get("/api/pendaftaran-dppi-provinsi")]
pub async fn get_pendaftaran_list_provinsi(
    pool: web::Data<MySqlPool>,
    query: web::Query<FilterParamsProvinsi>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };
    if !["Superadmin", "Administrator", "Admin Pendaftaran"].contains(&claims.role.as_str()) {
        return HttpResponse::Forbidden().json(json!({
            "error": "Anda tidak memiliki izin untuk mengakses API ini"
        }));
    }
    let page = query.page.unwrap_or(1) as i64;
    let per_page = query.per_page.unwrap_or(10) as i64;
    let offset = (page - 1) * per_page;

    // Build query dynamically
    let mut base_query = String::from(
        "SELECT * FROM pendaftaran_dppi_provinsi
         WHERE 1=1",
    );

    let mut conditions = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(status) = &query.status {
        conditions.push("pd.status = ?".to_string());
        params.push(status.clone());
    }

    if let Some(id_provinsi) = query.id_provinsi {
        conditions.push("pd.id_provinsi = ?".to_string());
        params.push(id_provinsi.to_string());
    }

    if let Some(id_provinsi) = query.id_provinsi {
        conditions.push("k.id_provinsi = ?".to_string());
        params.push(id_provinsi.to_string());
    }

    if let Some(search) = &query.search {
        conditions.push(
            "(pd.nama_provinsi LIKE ? OR pd.nama_pic LIKE ? OR pd.nip_pic LIKE ?)".to_string(),
        );
        params.push(format!("%{}%", search));
        params.push(format!("%{}%", search));
        params.push(format!("%{}%", search));
    }

    if !conditions.is_empty() {
        base_query.push_str(" AND ");
        base_query.push_str(&conditions.join(" AND "));
    }

    // Count query - cara yang lebih sederhana
    let count_query_str = format!(
        "SELECT COUNT(*) as total FROM pendaftaran_dppi_provinsi pd
         LEFT JOIN provinsi k ON pd.id_provinsi = k.id
         WHERE 1=1 {}",
        if conditions.is_empty() {
            "".to_string()
        } else {
            format!(" AND {}", conditions.join(" AND "))
        }
    );

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str);

    // Bind parameters untuk count query
    for param in &params {
        count_query = count_query.bind(param);
    }

    let total_result = count_query.fetch_one(pool.get_ref()).await;

    let total = match total_result {
        Ok(total) => total,
        Err(e) => {
            log::error!("Error counting pendaftaran: {}", e);
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Gagal menghitung data"}));
        }
    };

    // Data query with pagination
    let data_query_str = format!(
        "SELECT * FROM pendaftaran_dppi_provinsi
         WHERE 1=1 {}
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
        if conditions.is_empty() {
            "".to_string()
        } else {
            format!(" AND {}", conditions.join(" AND "))
        }
    );

    let mut data_query = sqlx::query_as::<_, PendaftaranDppiWithProvinsi>(&data_query_str);

    // Bind semua parameter filter
    for param in &params {
        data_query = data_query.bind(param);
    }

    // Bind parameter pagination
    data_query = data_query.bind(per_page as i32).bind(offset as i32);

    let pendaftaran_list = data_query.fetch_all(pool.get_ref()).await;

    match pendaftaran_list {
        Ok(list) => {
            let total_pages = if per_page > 0 {
                (total as f64 / per_page as f64).ceil() as i64
            } else {
                1
            };

            HttpResponse::Ok().json(PaginatedResponse {
                data: list,
                total,
                page,
                per_page,
                total_pages,
            })
        }
        Err(e) => {
            log::error!("Error fetching pendaftaran: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": format!("Gagal mengambil data: {}", e)}))
        }
    }
}

// Get single pendaftaran by ID
#[get("/api/pendaftaran-dppi-provinsi/{id}")]
pub async fn get_pendaftaran_by_id_provinsi(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };
    if !["Superadmin", "Administrator", "Admin Pendaftaran"].contains(&claims.role.as_str()) {
        return HttpResponse::Forbidden().json(json!({
            "error": "Anda tidak memiliki izin untuk mengakses API ini"
        }));
    }
    let pendaftaran_id = id.into_inner();

    // Gunakan query_as biasa dengan select eksplisit
    let result = sqlx::query_as::<_, PendaftaranDppiWithProvinsi>(
        r#"
        SELECT
            id,
            id_provinsi,
            nama_provinsi,
            nama_pic,
            jabatan_pic,
            nip_pic,
            no_telp_pic,
            email_pic,
            ketua,
            wakil_ketua,
            sekretaris,
            kepala_divisi_dukungan,
            kepala_divisi_kompetensi,
            kepala_divisi_aktualisasi,
            kepala_divisi_kominfo,
            path_surat_sekda,
            path_daftar_riwayat_hidup,
            path_portofolio,
            path_kartu_keluarga,
            path_sertifikat_pdp,
            path_sertifikat_diktat_pip,
            status,
            created_at,
            updated_at,
            created_by,
            updated_by
        FROM pendaftaran_dppi_provinsi

        WHERE id = ?
        "#,
    )
    .bind(pendaftaran_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(pendaftaran)) => HttpResponse::Ok().json(pendaftaran),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Data tidak ditemukan"})),
        Err(e) => {
            log::error!("Error fetching pendaftaran by ID: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": format!("Gagal mengambil data: {}", e)}))
        }
    }
}

// Upload document for pendaftaran
#[post("/api/pendaftaran-dppi-provinsi/{id}/upload/{field_name}")]
pub async fn upload_document_provinsi(
    pool: web::Data<MySqlPool>,
    path: web::Path<(i32, String)>,
    form: web::Json<UploadDocumentRequestProvinsi>,
) -> HttpResponse {
    let (id, field_name) = path.into_inner();

    // Ensure base upload directories exist
    if let Err(e) = ensure_upload_dirs() {
        log::error!("Failed to ensure upload directories: {}", e);
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Gagal menyiapkan direktori upload: {}", e)
        }));
    }

    log::info!(
        "Upload request: id={}, field={}, file={}",
        id,
        field_name,
        form.file_name
    );

    // Validate file type
    if !form.file_name.to_lowercase().ends_with(".pdf") {
        log::warn!("File bukan PDF: {}", form.file_name);
        return HttpResponse::BadRequest().json(json!({
            "error": "Hanya file PDF yang diperbolehkan"
        }));
    }

    // Decode base64 content
    let file_content = match BASE64_STANDARD.decode(&form.base64_content) {
        Ok(content) => {
            log::info!("Base64 decoded successfully, size: {} bytes", content.len());
            content
        }
        Err(e) => {
            log::error!("Error decoding base64: {}", e);
            return HttpResponse::BadRequest().json(json!({
                "error": "Format file tidak valid"
            }));
        }
    };

    // Check file size (20MB limit)
    if file_content.len() > 20 * 1024 * 1024 {
        log::warn!("File terlalu besar: {} bytes", file_content.len());
        return HttpResponse::BadRequest().json(json!({
            "error": "Ukuran file tidak boleh lebih dari 10MB"
        }));
    }

    // Create directory if not exists
    let upload_dir = format!("./uploads/assets/pendaftaran-dppi-provinsi/{}", id);
    log::info!("Creating upload directory: {}", upload_dir);

    if let Err(e) = fs::create_dir_all(&upload_dir) {
        log::error!("Error creating directory {}: {}", upload_dir, e);

        // Coba buat parent directory dulu
        let parent_dir = format!("./uploads/assets/pendaftaran-dppi-provinsi");
        if let Err(e2) = fs::create_dir_all(&parent_dir) {
            log::error!("Error creating parent directory {}: {}", parent_dir, e2);
        }

        // Coba lagi
        if let Err(e3) = fs::create_dir_all(&upload_dir) {
            log::error!("Still error creating directory: {}", e3);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Gagal membuat direktori upload: {}", e3)
            }));
        }
    }

    log::info!("Directory created successfully: {}", upload_dir);

    // Generate unique filename
    let unique_id = Uuid::new_v4();
    let file_name = format!("{}_{}_{}", field_name, unique_id, form.file_name);
    let file_path = format!("{}/{}", upload_dir, file_name);

    log::info!("Saving file to: {}", file_path);

    // Save file
    match fs::write(&file_path, &file_content) {
        Ok(_) => {
            log::info!("File saved successfully: {} bytes", file_content.len());
        }
        Err(e) => {
            log::error!("Error saving file to {}: {}", file_path, e);

            // Check if directory is writable
            match fs::metadata(&upload_dir) {
                Ok(metadata) => {
                    log::info!("Directory metadata: {:?}", metadata.permissions());
                }
                Err(e2) => {
                    log::error!("Cannot access directory metadata: {}", e2);
                }
            }

            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Gagal menyimpan file: {}", e)
            }));
        }
    }

    // Update database
    let column_name = match field_name.as_str() {
        "surat_sekda" => "path_surat_sekda",
        "daftar_riwayat_hidup" => "path_daftar_riwayat_hidup",
        "portofolio" => "path_portofolio",
        "kartu_keluarga" => "path_kartu_keluarga",
        "sertifikat_pdp" => "path_sertifikat_pdp",
        "sertifikat_diktat_pip" => "path_sertifikat_diktat_pip",
        _ => {
            log::warn!("Invalid field name: {}", field_name);
            // Hapus file yang sudah tersimpan
            let _ = fs::remove_file(&file_path);
            return HttpResponse::BadRequest().json(json!({
                "error": "Nama field tidak valid"
            }));
        }
    };

    let update_query = format!(
        "UPDATE pendaftaran_dppi_provinsi
         SET {} = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
        column_name
    );

    log::info!("Updating database column: {}", column_name);

    match sqlx::query(&update_query)
        .bind(&file_path)
        .bind(id)
        .execute(pool.get_ref())
        .await
    {
        Ok(result) => {
            log::info!(
                "Database updated successfully, rows affected: {}",
                result.rows_affected()
            );
            HttpResponse::Ok().json(json!({
                "message": "Dokumen berhasil diupload",
                "file_path": file_path,
                "file_name": file_name
            }))
        }
        Err(e) => {
            log::error!("Error updating document path: {}", e);

            // Clean up uploaded file if database update fails
            match fs::remove_file(&file_path) {
                Ok(_) => log::info!("Cleaned up file after DB error"),
                Err(rm_err) => log::error!("Failed to clean up file: {}", rm_err),
            }

            HttpResponse::InternalServerError().json(json!({
                "error": format!("Gagal menyimpan informasi dokumen: {}", e)
            }))
        }
    }
}

// Update status pendaftaran
#[put("/api/pendaftaran-dppi-provinsi/{id}/status")]
pub async fn update_status_provinsi(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    form: web::Json<UpdateStatusRequestProvinsi>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };
    let id = path.into_inner();
    let user_id = claims.user_id;
    // Validate status
    let valid_statuses = vec!["pending", "review", "approved", "rejected", "completed"];
    if !valid_statuses.contains(&form.status.as_str()) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Status tidak valid"
        }));
    }

    let result = sqlx::query!(
        r#"
        UPDATE pendaftaran_dppi_provinsi
        SET status = ?, updated_at = CURRENT_TIMESTAMP, updated_by = ?
        WHERE id = ?
        "#,
        form.status,
        user_id,
        id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(json!({
                    "message": "Status berhasil diupdate"
                }))
            } else {
                HttpResponse::NotFound().json(json!({
                    "error": "Data tidak ditemukan"
                }))
            }
        }
        Err(e) => {
            log::error!("Error updating status: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Gagal mengupdate status"
            }))
        }
    }
}

// Delete pendaftaran
#[delete("/api/pendaftaran-dppi-provinsi/{id}")]
pub async fn delete_pendaftaran_provinsi(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };
    if !["Superadmin", "Administrator", "Admin Pendaftaran"].contains(&claims.role.as_str()) {
        return HttpResponse::Forbidden().json(json!({
            "error": "Anda tidak memiliki izin untuk mengakses API ini"
        }));
    }
    let pendaftaran_id = id.into_inner();

    log::info!("Deleting pendaftaran ID: {}", pendaftaran_id);

    // Check if pendaftaran exists
    let pendaftaran = sqlx::query!(
        "SELECT * FROM pendaftaran_dppi_provinsi WHERE id = ?",
        pendaftaran_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match pendaftaran {
        Ok(Some(_)) => {
            // Delete uploaded files - GUNAKAN PATH YANG SAMA
            let upload_dir = format!(
                "./uploads/assets/pendaftaran-dppi-provinsi/{}",
                pendaftaran_id
            );
            log::info!("Checking upload directory: {}", upload_dir);

            if Path::new(&upload_dir).exists() {
                match fs::remove_dir_all(&upload_dir) {
                    Ok(_) => log::info!("Successfully deleted upload directory"),
                    Err(e) => log::error!("Error deleting upload directory: {}", e),
                }
            } else {
                log::warn!("Upload directory does not exist: {}", upload_dir);
            }

            // Delete from database
            match sqlx::query!(
                "DELETE FROM pendaftaran_dppi_provinsi WHERE id = ?",
                pendaftaran_id
            )
            .execute(pool.get_ref())
            .await
            {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        log::info!("Successfully deleted pendaftaran from database");
                        HttpResponse::Ok().json(json!({
                            "message": "Pendaftaran berhasil dihapus"
                        }))
                    } else {
                        log::warn!("No rows affected when deleting pendaftaran");
                        HttpResponse::NotFound().json(json!({
                            "error": "Data tidak ditemukan"
                        }))
                    }
                }
                Err(e) => {
                    log::error!("Database error deleting pendaftaran: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": format!("Gagal menghapus data: {}", e)
                    }))
                }
            }
        }
        Ok(None) => {
            log::warn!("Pendaftaran not found: {}", pendaftaran_id);
            HttpResponse::NotFound().json(json!({"error": "Data tidak ditemukan"}))
        }
        Err(e) => {
            log::error!("Error fetching pendaftaran: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Gagal mengambil data: {}", e)
            }))
        }
    }
}

// Download document
#[get("/api/pendaftaran-dppi-provinsi/{id}/download/{document_type}")]
pub async fn download_document_provinsi(
    pool: web::Data<MySqlPool>,
    path: web::Path<(i32, String)>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };
    if !["Superadmin", "Administrator", "Admin Pendaftaran"].contains(&claims.role.as_str()) {
        return HttpResponse::Forbidden().json(json!({
            "error": "Anda tidak memiliki izin untuk mengakses API ini"
        }));
    }
    let (id, document_type) = path.into_inner();

    // Get file path from database
    let column_name = match document_type.as_str() {
        "surat_sekda" => "path_surat_sekda",
        "daftar_riwayat_hidup" => "path_daftar_riwayat_hidup",
        "portofolio" => "path_portofolio",
        "kartu_keluarga" => "path_kartu_keluarga",
        "sertifikat_pdp" => "path_sertifikat_pdp",
        "sertifikat_diktat_pip" => "path_sertifikat_diktat_pip",
        _ => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Tipe dokumen tidak valid"
            }));
        }
    };

    let query = format!(
        "SELECT {} FROM pendaftaran_dppi_provinsi WHERE id = ?",
        column_name
    );

    let result = sqlx::query_scalar::<_, String>(&query)
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(file_path)) => {
            if file_path.is_empty() {
                return HttpResponse::NotFound().json(json!({
                    "error": "Dokumen tidak ditemukan"
                }));
            }

            match fs::read(&file_path) {
                Ok(file_content) => {
                    let file_name = Path::new(&file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("document.pdf");

                    HttpResponse::Ok()
                        .content_type("application/pdf")
                        .append_header((
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", file_name),
                        ))
                        .body(file_content)
                }
                Err(e) => {
                    log::error!("Error reading file: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": "Gagal membaca file"
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Data tidak ditemukan"})),
        Err(e) => {
            log::error!("Error fetching file path: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Gagal mengambil informasi dokumen"
            }))
        }
    }
}
fn ensure_upload_dirs() -> std::io::Result<()> {
    let base_dirs = vec![
        "./uploads",
        "./uploads/assets",
        "./uploads/assets/pendaftaran-dppi-provinsi",
    ];

    for dir in base_dirs {
        if !Path::new(dir).exists() {
            log::info!("Creating directory: {}", dir);
            fs::create_dir_all(dir)?;
            log::info!("Created directory: {}", dir);
        } else {
            log::info!("Directory already exists: {}", dir);

            // Check permissions
            let metadata = fs::metadata(dir)?;
            log::info!("Permissions for {}: {:?}", dir, metadata.permissions());
        }
    }

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct ExportParamsProvinsi {
    pub simple: Option<bool>,
}

fn create_excel_workbook(data: &[PendaftaranDppiWithProvinsi]) -> Result<Vec<u8>, XlsxError> {
    // Create a new Excel workbook
    let mut workbook = Workbook::new();

    // Add a worksheet
    let worksheet = workbook.add_worksheet();

    // Create formats
    let header_format = Format::new()
        .set_bold()
        .set_border(rust_xlsxwriter::FormatBorder::Thin)
        .set_background_color(rust_xlsxwriter::Color::RGB(0x4472C4))
        .set_font_color(rust_xlsxwriter::Color::RGB(0xFFFFFF));

    let date_format = Format::new().set_num_format("yyyy-mm-dd hh:mm:ss");

    // Write headers
    let headers = [
        "No.",
        "",
        "Provinsi",
        "Nama PIC",
        "Jabatan PIC",
        "NIP PIC",
        "No. Telepon",
        "Email",
        "Status",
        "Tanggal Pendaftaran",
        "Ketua 1",
        "Ketua 2",
        "Wakil Ketua 1",
        "Wakil Ketua 2",
        "Sekretaris 1",
        "Sekretaris 2",
        "Kepala Divisi Dukungan Pembentukan Paskibraka dan Duta Pancasila 1",
        "Kepala Divisi Dukungan Pembentukan Paskibraka dan Duta Pancasila 2",
        "Kepala Divisi Peningkatan Kompetensi 1",
        "Kepala Divisi Peningkatan Kompetensi 2",
        "Kepala Divisi Aktualisasi Nilai-nilai Pancasila 1",
        "Kepala Divisi Aktualisasi Nilai-nilai Pancasila 2",
        "Kepala Divisi Komunikasi, Teknologi dan Informasi 1",
        "Kepala Divisi Komunikasi, Teknologi dan Informasi 2",
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet.write_with_format(0, col as u16, *header, &header_format)?;
        worksheet.set_column_width(col as u16, 20.0)?;
    }

    // Set specific column widths
    worksheet.set_column_width(1, 25.0)?; //
    worksheet.set_column_width(2, 20.0)?; // Provinsi
    worksheet.set_column_width(3, 25.0)?; // Nama PIC
    worksheet.set_column_width(4, 25.0)?; // Jabatan PIC
    worksheet.set_column_width(7, 30.0)?; // Email
    worksheet.set_column_width(9, 25.0)?; // Tanggal Pendaftaran

    // Write data rows
    for (row_idx, item) in data.iter().enumerate() {
        let row = (row_idx + 1) as u32;

        // No.
        worksheet.write_number(row, 0, row)?;

        // Provinsi
        worksheet.write_string(row, 1, &item.nama_provinsi)?;

        // Nama PIC
        worksheet.write_string(row, 2, &item.nama_pic)?;

        // Jabatan PIC
        worksheet.write_string(row, 3, &item.jabatan_pic)?;

        // NIP PIC
        worksheet.write_string(row, 4, &item.nip_pic)?;

        // No. Telepon
        worksheet.write_string(row, 5, &item.no_telp_pic)?;

        // Email
        worksheet.write_string(row, 6, &item.email_pic)?;

        // Status dengan warna berdasarkan status
        let status_format = match item.status.as_str() {
            "pending" => Format::new()
                .set_font_color(rust_xlsxwriter::Color::RGB(0xFFA500)) // Orange
                .set_bold(),
            "review" => Format::new()
                .set_font_color(rust_xlsxwriter::Color::RGB(0x0000FF)) // Blue
                .set_bold(),
            "approved" => Format::new()
                .set_font_color(rust_xlsxwriter::Color::RGB(0x008000)) // Green
                .set_bold(),
            "rejected" => Format::new()
                .set_font_color(rust_xlsxwriter::Color::RGB(0xFF0000)) // Red
                .set_bold(),
            "completed" => Format::new()
                .set_font_color(rust_xlsxwriter::Color::RGB(0x800080)) // Purple
                .set_bold(),
            _ => Format::new(),
        };
        worksheet.write_string_with_format(row, 7, &item.status, &status_format)?;

        // Tanggal Pendaftaran
        let datetime_str = item.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        worksheet.write_string_with_format(row, 8, &datetime_str, &date_format)?;

        // Struktur Organisasi
        worksheet.write_string(row, 9, &item.ketua_1)?;
        worksheet.write_string(row, 10, &item.ketua_2)?;
        worksheet.write_string(row, 11, &item.wakil_ketua_1)?;
        worksheet.write_string(row, 12, &item.wakil_ketua_2)?;
        worksheet.write_string(row, 13, &item.sekretaris_1)?;
        worksheet.write_string(row, 14, &item.sekretaris_2)?;
        worksheet.write_string(row, 15, &item.kepala_divisi_dukungan_1)?;
        worksheet.write_string(row, 16, &item.kepala_divisi_dukungan_2)?;
        worksheet.write_string(row, 17, &item.kepala_divisi_kompetensi_1)?;
        worksheet.write_string(row, 18, &item.kepala_divisi_kompetensi_2)?;
        worksheet.write_string(row, 19, &item.kepala_divisi_aktualisasi_1)?;
        worksheet.write_string(row, 20, &item.kepala_divisi_aktualisasi_2)?;
        worksheet.write_string(row, 21, &item.kepala_divisi_kominfo_1)?;
        worksheet.write_string(row, 22, &item.kepala_divisi_kominfo_2)?;
        // Created By & Updated By
        worksheet.write_string(row, 23, item.created_by.as_deref().unwrap_or(""))?;
        worksheet.write_string(row, 24, item.updated_by.as_deref().unwrap_or(""))?;
    }

    // Add summary sheet
    let summary_sheet = workbook.add_worksheet();
    summary_sheet.set_name("Ringkasan")?;

    // Write summary headers
    let summary_headers = ["Status", "Jumlah", "Persentase"];
    for (col, header) in summary_headers.iter().enumerate() {
        summary_sheet.write_with_format(0, col as u16, *header, &header_format)?;
    }

    // Calculate summary
    let total = data.len() as f64;
    let status_counts = data
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, item| {
            *acc.entry(item.status.clone()).or_insert(0) += 1;
            acc
        });

    let mut row = 1;
    for (status, count) in status_counts {
        summary_sheet.write_string(row, 0, &status)?;
        summary_sheet.write_number(row, 1, count as f64)?;

        let percentage = if total > 0.0 {
            (count as f64 / total) * 100.0
        } else {
            0.0
        };
        let percentage_format = Format::new().set_num_format("0.00%");
        summary_sheet.write_number_with_format(row, 2, percentage / 100.0, &percentage_format)?;

        row += 1;
    }

    // Total row
    let total_format = Format::new()
        .set_bold()
        .set_border_top(rust_xlsxwriter::FormatBorder::Double);
    summary_sheet.write_string_with_format(row, 0, "TOTAL", &total_format)?;
    summary_sheet.write_number_with_format(row, 1, total, &total_format)?;
    summary_sheet.write_string_with_format(row, 2, "100.00%", &total_format)?;

    // Auto-fit columns in summary
    summary_sheet.autofit();

    // Save to bytes
    workbook.save_to_buffer()
}

fn create_excel_workbook_simple(
    data: &[PendaftaranDppiWithProvinsi],
) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Simple header format
    let header_format = Format::new().set_bold();

    // Write headers
    worksheet.write_with_format(0, 0, "ID", &header_format)?;
    worksheet.write_with_format(0, 1, "Provinsi", &header_format)?;
    worksheet.write_with_format(0, 2, "Nama PIC", &header_format)?;
    worksheet.write_with_format(0, 3, "Jabatan", &header_format)?;
    worksheet.write_with_format(0, 4, "NIP", &header_format)?;
    worksheet.write_with_format(0, 5, "Status", &header_format)?;
    worksheet.write_with_format(0, 6, "Tanggal", &header_format)?;

    // Write data
    for (row_idx, item) in data.iter().enumerate() {
        let row = (row_idx + 1) as u32;

        worksheet.write_number(row, 0, item.id as f64)?;
        worksheet.write_string(row, 1, &item.nama_provinsi)?;
        worksheet.write_string(row, 2, &item.nama_pic)?;
        worksheet.write_string(row, 3, &item.jabatan_pic)?;
        worksheet.write_string(row, 4, &item.nip_pic)?;
        worksheet.write_string(row, 5, &item.status)?;

        let date_str = item.created_at.format("%Y-%m-%d").to_string();
        worksheet.write_string(row, 6, &date_str)?;
    }

    // Auto-fit columns
    for col in 0..8 {
        worksheet.set_column_width(col, 20.0)?;
    }

    workbook.save_to_buffer()
}

// Upload rekomendasi
#[post("/api/pendaftaran-dppi-provinsi/{id}/upload-rekomendasi")]
pub async fn upload_rekomendasi_provinsi(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
    form: web::Json<UploadDocumentRequestProvinsi>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };

    // Hanya admin yang bisa upload rekomendasi
    if !["Superadmin", "Administrator", "Admin Pendaftaran"].contains(&claims.role.as_str()) {
        return HttpResponse::Forbidden().json(json!({
            "error": "Anda tidak memiliki izin untuk upload rekomendasi"
        }));
    }

    let pendaftaran_id = id.into_inner();
    let user_id = claims.user_id;

    // Cek apakah pendaftaran ada dan statusnya approved
    let pendaftaran_result = sqlx::query!(
        "SELECT status FROM pendaftaran_dppi_provinsi WHERE id = ?",
        pendaftaran_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match pendaftaran_result {
        Ok(Some(pendaftaran)) => {
            // Hanya bisa upload rekomendasi jika status approved
            if pendaftaran.status.as_deref() != Some("approved") {
                return HttpResponse::BadRequest().json(json!({
                    "error": "Hanya bisa upload rekomendasi untuk pendaftaran dengan status 'approved'"
                }));
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Data pendaftaran tidak ditemukan"
            }));
        }
        Err(e) => {
            log::error!("Error checking pendaftaran: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Gagal memeriksa data pendaftaran"
            }));
        }
    }

    // Validate file type - hanya PDF yang diperbolehkan
    if !form.file_name.to_lowercase().ends_with(".pdf") {
        return HttpResponse::BadRequest().json(json!({
            "error": "Hanya file PDF yang diperbolehkan untuk rekomendasi"
        }));
    }

    // Decode base64 content
    let file_content = match BASE64_STANDARD.decode(&form.base64_content) {
        Ok(content) => content,
        Err(e) => {
            log::error!("Error decoding base64: {}", e);
            return HttpResponse::BadRequest().json(json!({
                "error": "Format file tidak valid"
            }));
        }
    };

    // Check file size (10MB limit)
    if file_content.len() > 10 * 1024 * 1024 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Ukuran file tidak boleh lebih dari 10MB"
        }));
    }

    // Create upload directory
    let upload_dir = format!(
        "./uploads/assets/pendaftaran-dppi-provinsi/{}/rekomendasi",
        pendaftaran_id
    );

    if let Err(e) = fs::create_dir_all(&upload_dir) {
        log::error!("Error creating directory {}: {}", upload_dir, e);
        return HttpResponse::InternalServerError().json(json!({
            "error": format!("Gagal membuat direktori upload: {}", e)
        }));
    }

    // Generate unique filename
    let unique_id = Uuid::new_v4();
    let file_name = format!("rekomendasi_{}_{}", unique_id, form.file_name);
    let file_path = format!("{}/{}", upload_dir, file_name);

    // Save file
    match fs::write(&file_path, &file_content) {
        Ok(_) => {
            log::info!("Rekomendasi file saved: {}", file_path);
        }
        Err(e) => {
            log::error!("Error saving rekomendasi file: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Gagal menyimpan file: {}", e)
            }));
        }
    }

    // Update database dengan rekomendasi path
    match sqlx::query!(
        r#"
        UPDATE pendaftaran_dppi_provinsi
        SET rekomendasi = ?, updated_at = CURRENT_TIMESTAMP, updated_by = ?
        WHERE id = ?
        "#,
        file_path,
        user_id,
        pendaftaran_id
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(json!({
                    "message": "Rekomendasi berhasil diupload",
                    "file_path": file_path,
                    "file_name": file_name
                }))
            } else {
                // Clean up file jika update gagal
                let _ = fs::remove_file(&file_path);
                HttpResponse::NotFound().json(json!({
                    "error": "Data tidak ditemukan"
                }))
            }
        }
        Err(e) => {
            log::error!("Error updating rekomendasi in database: {}", e);
            // Clean up file
            let _ = fs::remove_file(&file_path);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Gagal menyimpan informasi rekomendasi: {}", e)
            }))
        }
    }
}

// Download rekomendasi
#[get("/api/pendaftaran-dppi-provinsi/{id}/download-rekomendasi")]
pub async fn download_rekomendasi_provinsi(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
    req: HttpRequest,
) -> HttpResponse {
    let claims = match auth::verify_jwt(&req) {
        Ok(claims) => claims,
        Err(e) => {
            log::error!("JWT verification failed: {}", e);
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized"
            }));
        }
    };

    if !["Superadmin", "Administrator", "Admin Pendaftaran"].contains(&claims.role.as_str()) {
        return HttpResponse::Forbidden().json(json!({
            "error": "Anda tidak memiliki izin untuk mengakses API ini"
        }));
    }

    let pendaftaran_id = id.into_inner();

    // Get rekomendasi file path from database
    let result = sqlx::query!(
        "SELECT rekomendasi FROM pendaftaran_dppi_provinsi WHERE id = ?",
        pendaftaran_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(record)) => match record.rekomendasi {
            Some(file_path) if !file_path.is_empty() => match fs::read(&file_path) {
                Ok(file_content) => {
                    let file_name = Path::new(&file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("rekomendasi.pdf");

                    HttpResponse::Ok()
                        .content_type("application/pdf")
                        .append_header((
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", file_name),
                        ))
                        .body(file_content)
                }
                Err(e) => {
                    log::error!("Error reading rekomendasi file: {}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "error": "Gagal membaca file rekomendasi"
                    }))
                }
            },
            _ => HttpResponse::NotFound().json(json!({
                "error": "File rekomendasi tidak ditemukan"
            })),
        },
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "Data tidak ditemukan"
        })),
        Err(e) => {
            log::error!("Error fetching rekomendasi: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Gagal mengambil informasi rekomendasi"
            }))
        }
    }
}
