use crate::auth;
use actix_multipart::Multipart;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, get, post, put, web};
use chrono::NaiveDate;
use futures_util::TryStreamExt as _;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, MySqlPool};
use uuid::Uuid;

use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct PamongInput {
    pub id_paskibraka: i32,
    pub tanggal: String,
    pub id_pamong: Option<String>,
    pub nilai_ketaqwaan: Option<i32>,
    pub nilai_niat_kemauan: Option<i32>,
    pub nilai_keberanian: Option<i32>,
    pub nilai_komunikasi: Option<i32>,
    pub nilai_keterbukaan: Option<i32>,
    pub nilai_ketelitian: Option<i32>,
    pub nilai_kesadaran: Option<i32>,
    pub nilai_toleransi: Option<i32>,
    pub nilai_keikhlasan: Option<i32>,
    pub nilai_mempercayai: Option<i32>,
    pub nilai_jiwa_korsa: Option<i32>,
    pub nilai_kekeluargaan: Option<i32>,
    pub nilai_persatuan_kesatuan: Option<i32>,
    pub nilai_ketahanan: Option<i32>,
    pub nilai_kekompakan_keseragaman: Option<i32>,
    pub nilai_ketertiban: Option<i32>,
    pub nilai_kesopanan: Option<i32>,
    pub nilai_kesigapan: Option<i32>,
    pub nilai_kewajaran: Option<i32>,
    pub nilai_ketanggapan: Option<i32>,
    pub nilai_ketenangan: Option<i32>,
    pub nilai_menyimak: Option<i32>,
    pub nilai_kebiasaan: Option<i32>,
    pub nilai_mengelola_stres: Option<i32>,
    pub nilai_menghargai_waktu: Option<i32>,
    pub nilai_berbicara: Option<i32>,
    pub nilai_berjalan: Option<i32>,
    pub nilai_makan_minum: Option<i32>,
    pub nilai_kehadiran: Option<i32>,
    pub nilai_hubungan_interpersonal: Option<i32>,
    pub nilai_ketaatan: Option<i32>,
    pub nilai_istirahat_malam: Option<i32>,
    pub nilai_keindahan: Option<i32>,
    pub nilai_kerapihan: Option<i32>,
    pub nilai_kebersihan: Option<i32>,
    pub nilai_berpakaian: Option<i32>,
    pub nilai_penampilan_rambut: Option<i32>,
    pub nilai_bersih_rapih_wangi: Option<i32>,
    pub catatan: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PamongJournal {
    pub id: String,
    pub id_paskibraka: i32,
    pub id_pamong: String,
    pub tanggal: NaiveDate,
    pub nilai_ketaqwaan: Option<i32>,
    pub nilai_niat_kemauan: Option<i32>,
    pub nilai_keberanian: Option<i32>,
    pub nilai_komunikasi: Option<i32>,
    pub nilai_keterbukaan: Option<i32>,
    pub nilai_ketelitian: Option<i32>,
    pub nilai_kesadaran: Option<i32>,
    pub nilai_toleransi: Option<i32>,
    pub nilai_keikhlasan: Option<i32>,
    pub nilai_mempercayai: Option<i32>,
    pub nilai_jiwa_korsa: Option<i32>,
    pub nilai_kekeluargaan: Option<i32>,
    pub nilai_persatuan_kesatuan: Option<i32>,
    pub nilai_ketahanan: Option<i32>,
    pub nilai_kekompakan_keseragaman: Option<i32>,
    pub nilai_ketertiban: Option<i32>,
    pub nilai_kesopanan: Option<i32>,
    pub nilai_kesigapan: Option<i32>,
    pub nilai_kewajaran: Option<i32>,
    pub nilai_ketanggapan: Option<i32>,
    pub nilai_ketenangan: Option<i32>,
    pub nilai_menyimak: Option<i32>,
    pub nilai_kebiasaan: Option<i32>,
    pub nilai_mengelola_stres: Option<i32>,
    pub nilai_menghargai_waktu: Option<i32>,
    pub nilai_berbicara: Option<i32>,
    pub nilai_berjalan: Option<i32>,
    pub nilai_makan_minum: Option<i32>,
    pub nilai_kehadiran: Option<i32>,
    pub nilai_hubungan_interpersonal: Option<i32>,
    pub nilai_ketaatan: Option<i32>,
    pub nilai_istirahat_malam: Option<i32>,
    pub nilai_keindahan: Option<i32>,
    pub nilai_kerapihan: Option<i32>,
    pub nilai_kebersihan: Option<i32>,
    pub nilai_berpakaian: Option<i32>,
    pub nilai_penampilan_rambut: Option<i32>,
    pub nilai_bersih_rapih_wangi: Option<i32>,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PelatihInput {
    pub id_paskibraka: i32,
    pub tanggal: String,
    pub nilai_aba_aba: i32,
    pub nilai_berhimpun: i32,
    pub nilai_berkumpul: i32,
    pub nilai_keluar_masuk_barisan: i32,
    pub nilai_hormat: i32,
    pub nilai_sikap_sempurna: i32,
    pub nilai_istirahat: i32,
    pub nilai_periksa_kerapihan: i32,
    pub nilai_berhitung: i32,
    pub nilai_lepas_kenakan_topi: i32,
    pub nilai_bubar: i32,
    pub nilai_lencang_depan: i32,
    pub nilai_lencang_kanan_kiri: i32,
    pub nilai_setengah_lengan_lencang_kanan_kiri: i32,
    pub nilai_hadap_kanan_kiri: i32,
    pub nilai_hadap_serong_kanan_kiri: i32,
    pub nilai_balik_kanan: i32,
    pub nilai_langkah_bisa: i32,
    pub nilai_langkah_tegap: i32,
    pub nilai_sikap_awal_berlari: i32,
    pub nilai_jalan_di_tempat: i32,
    pub nilai_4_langkah_ke_depan: i32,
    pub nilai_4_langkah_ke_kanan: i32,
    pub nilai_4_langkah_ke_kiri: i32,
    pub nilai_4_langkah_ke_belakang: i32,
    pub nilai_lipat_bendera: i32,
    pub nilai_bentang_bendera: i32,
    pub nilai_10_tahap_penurunan: i32,
    pub nilai_jadi_kibra_pembentang: i32,
    pub nilai_jadi_kibra_pembawa: i32,
    pub nilai_jadi_kibra_pengerek: i32,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DokterInput {
    pub id_paskibraka: i32,
    pub tanggal: String,
    pub tensi: String,
    pub suhu: f64,
    pub keluhan: Option<String>,
    pub diagnosa: Option<String>,
    pub terapi_obat: Option<String>,
    pub rekomendasi_istirahat: String,
}

#[derive(Debug, Deserialize)]
pub struct FilterParams {
    pub search: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CandidateSummary {
    pub id: i32,
    pub nama_lengkap: Option<String>,
    pub jk: Option<String>,
    pub id_pamong: Option<String>,
    pub no_peserta: Option<String>,
    pub photo: Option<String>,
    pub provinsi: Option<String>,
    pub kabupaten_kota: Option<String>,
    pub status: Option<String>,
}

#[get("/api/pemusatan/candidates")]
pub async fn get_candidates(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    query: web::Query<FilterParams>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = [
        "Pamong",
        "Pelatih",
        "Dokter",
        "Admin Pemusatan",
        "Superadmin",
    ];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let mut sql = String::from(
        "SELECT id, nama_lengkap, jk, id_pamong, no_peserta, photo, provinsi, kabupaten_kota, status FROM data_capaska WHERE 1=1",
    );
    let mut params: Vec<String> = Vec::new();

    // Jika role adalah Pamong, filter berdasarkan id_pamong
    if claims.role.as_str() == "Pamong" {
        sql.push_str(" AND id_pamong = ?");
        params.push(claims.user_id.clone());
    }

    if let Some(ref search) = query.search {
        if !search.is_empty() {
            sql.push_str(" AND nama_lengkap LIKE ?");
            params.push(format!("%{}%", search));
        }
    }
    sql.push_str(" ORDER BY nama_lengkap ASC");

    let mut q = sqlx::query_as::<_, CandidateSummary>(&sql);
    for param in params {
        q = q.bind(param);
    }

    let candidates = q
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(candidates))
}

// 2. Submit Pamong Log
#[post("/api/pemusatan/pamong")]
pub async fn submit_pamong(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<PamongInput>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Pamong", "Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let pamong_id = if let Some(ref pid) = payload.id_pamong {
        if !pid.is_empty() && (claims.role == "Admin Pemusatan" || claims.role == "Superadmin") {
            pid.clone()
        } else {
            claims.user_id.clone()
        }
    } else {
        claims.user_id.clone()
    };

    let id = Uuid::new_v4().to_string();
    let query = r#"
        INSERT INTO jurnal_pemusatan_pamong (
            id, id_paskibraka, id_pamong, tanggal,
            nilai_ketaqwaan, nilai_niat_kemauan, nilai_keberanian, nilai_komunikasi,
            nilai_keterbukaan, nilai_ketelitian, nilai_kesadaran, nilai_toleransi,
            nilai_keikhlasan, nilai_mempercayai, nilai_jiwa_korsa, nilai_kekeluargaan,
            nilai_persatuan_kesatuan, nilai_ketahanan, nilai_kekompakan_keseragaman, nilai_ketertiban,
            nilai_kesopanan, nilai_kesigapan, nilai_kewajaran, nilai_ketanggapan,
            nilai_ketenangan, nilai_menyimak, nilai_kebiasaan, nilai_mengelola_stres,
            nilai_menghargai_waktu, nilai_berbicara, nilai_berjalan, nilai_makan_minum,
            nilai_kehadiran, nilai_hubungan_interpersonal, nilai_ketaatan, nilai_istirahat_malam,
            nilai_keindahan, nilai_kerapihan, nilai_kebersihan, nilai_berpakaian,
            nilai_penampilan_rambut, nilai_bersih_rapih_wangi, catatan
        )
        VALUES (
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?
        )
        ON DUPLICATE KEY UPDATE
            id_pamong = VALUES(id_pamong),
            nilai_ketaqwaan = COALESCE(VALUES(nilai_ketaqwaan), nilai_ketaqwaan),
            nilai_niat_kemauan = COALESCE(VALUES(nilai_niat_kemauan), nilai_niat_kemauan),
            nilai_keberanian = COALESCE(VALUES(nilai_keberanian), nilai_keberanian),
            nilai_komunikasi = COALESCE(VALUES(nilai_komunikasi), nilai_komunikasi),
            nilai_keterbukaan = COALESCE(VALUES(nilai_keterbukaan), nilai_keterbukaan),
            nilai_ketelitian = COALESCE(VALUES(nilai_ketelitian), nilai_ketelitian),
            nilai_kesadaran = COALESCE(VALUES(nilai_kesadaran), nilai_kesadaran),
            nilai_toleransi = COALESCE(VALUES(nilai_toleransi), nilai_toleransi),
            nilai_keikhlasan = COALESCE(VALUES(nilai_keikhlasan), nilai_keikhlasan),
            nilai_mempercayai = COALESCE(VALUES(nilai_mempercayai), nilai_mempercayai),
            nilai_jiwa_korsa = COALESCE(VALUES(nilai_jiwa_korsa), nilai_jiwa_korsa),
            nilai_kekeluargaan = COALESCE(VALUES(nilai_kekeluargaan), nilai_kekeluargaan),
            nilai_persatuan_kesatuan = COALESCE(VALUES(nilai_persatuan_kesatuan), nilai_persatuan_kesatuan),
            nilai_ketahanan = COALESCE(VALUES(nilai_ketahanan), nilai_ketahanan),
            nilai_kekompakan_keseragaman = COALESCE(VALUES(nilai_kekompakan_keseragaman), nilai_kekompakan_keseragaman),
            nilai_ketertiban = COALESCE(VALUES(nilai_ketertiban), nilai_ketertiban),
            nilai_kesopanan = COALESCE(VALUES(nilai_kesopanan), nilai_kesopanan),
            nilai_kesigapan = COALESCE(VALUES(nilai_kesigapan), nilai_kesigapan),
            nilai_kewajaran = COALESCE(VALUES(nilai_kewajaran), nilai_kewajaran),
            nilai_ketanggapan = COALESCE(VALUES(nilai_ketanggapan), nilai_ketanggapan),
            nilai_ketenangan = COALESCE(VALUES(nilai_ketenangan), nilai_ketenangan),
            nilai_menyimak = COALESCE(VALUES(nilai_menyimak), nilai_menyimak),
            nilai_kebiasaan = COALESCE(VALUES(nilai_kebiasaan), nilai_kebiasaan),
            nilai_mengelola_stres = COALESCE(VALUES(nilai_mengelola_stres), nilai_mengelola_stres),
            nilai_menghargai_waktu = COALESCE(VALUES(nilai_menghargai_waktu), nilai_menghargai_waktu),
            nilai_berbicara = COALESCE(VALUES(nilai_berbicara), nilai_berbicara),
            nilai_berjalan = COALESCE(VALUES(nilai_berjalan), nilai_berjalan),
            nilai_makan_minum = COALESCE(VALUES(nilai_makan_minum), nilai_makan_minum),
            nilai_kehadiran = COALESCE(VALUES(nilai_kehadiran), nilai_kehadiran),
            nilai_hubungan_interpersonal = COALESCE(VALUES(nilai_hubungan_interpersonal), nilai_hubungan_interpersonal),
            nilai_ketaatan = COALESCE(VALUES(nilai_ketaatan), nilai_ketaatan),
            nilai_istirahat_malam = COALESCE(VALUES(nilai_istirahat_malam), nilai_istirahat_malam),
            nilai_keindahan = COALESCE(VALUES(nilai_keindahan), nilai_keindahan),
            nilai_kerapihan = COALESCE(VALUES(nilai_kerapihan), nilai_kerapihan),
            nilai_kebersihan = COALESCE(VALUES(nilai_kebersihan), nilai_kebersihan),
            nilai_berpakaian = COALESCE(VALUES(nilai_berpakaian), nilai_berpakaian),
            nilai_penampilan_rambut = COALESCE(VALUES(nilai_penampilan_rambut), nilai_penampilan_rambut),
            nilai_bersih_rapih_wangi = COALESCE(VALUES(nilai_bersih_rapih_wangi), nilai_bersih_rapih_wangi),
            catatan = COALESCE(VALUES(catatan), catatan)
    "#;

    sqlx::query(query)
        .bind(&id)
        .bind(payload.id_paskibraka)
        .bind(&pamong_id)
        .bind(&payload.tanggal)
        .bind(payload.nilai_ketaqwaan)
        .bind(payload.nilai_niat_kemauan)
        .bind(payload.nilai_keberanian)
        .bind(payload.nilai_komunikasi)
        .bind(payload.nilai_keterbukaan)
        .bind(payload.nilai_ketelitian)
        .bind(payload.nilai_kesadaran)
        .bind(payload.nilai_toleransi)
        .bind(payload.nilai_keikhlasan)
        .bind(payload.nilai_mempercayai)
        .bind(payload.nilai_jiwa_korsa)
        .bind(payload.nilai_kekeluargaan)
        .bind(payload.nilai_persatuan_kesatuan)
        .bind(payload.nilai_ketahanan)
        .bind(payload.nilai_kekompakan_keseragaman)
        .bind(payload.nilai_ketertiban)
        .bind(payload.nilai_kesopanan)
        .bind(payload.nilai_kesigapan)
        .bind(payload.nilai_kewajaran)
        .bind(payload.nilai_ketanggapan)
        .bind(payload.nilai_ketenangan)
        .bind(payload.nilai_menyimak)
        .bind(payload.nilai_kebiasaan)
        .bind(payload.nilai_mengelola_stres)
        .bind(payload.nilai_menghargai_waktu)
        .bind(payload.nilai_berbicara)
        .bind(payload.nilai_berjalan)
        .bind(payload.nilai_makan_minum)
        .bind(payload.nilai_kehadiran)
        .bind(payload.nilai_hubungan_interpersonal)
        .bind(payload.nilai_ketaatan)
        .bind(payload.nilai_istirahat_malam)
        .bind(payload.nilai_keindahan)
        .bind(payload.nilai_kerapihan)
        .bind(payload.nilai_kebersihan)
        .bind(payload.nilai_berpakaian)
        .bind(payload.nilai_penampilan_rambut)
        .bind(payload.nilai_bersih_rapih_wangi)
        .bind(&payload.catatan)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Log this action
    crate::utils::log_activity(
        pool.get_ref(),
        Some(&claims.user_id),
        Some(&claims.nama_user),
        Some(&claims.role),
        "SUBMIT_PAMONG_JOURNAL",
        "PEMUSATAN",
        "SUCCESS",
        Some(&format!(
            "Saved Pamong journal for capaska {}",
            payload.id_paskibraka
        )),
        Some(&req),
    )
    .await;

    Ok(HttpResponse::Ok()
        .json(json!({ "status": "success", "message": "Jurnal Pamong berhasil disimpan" })))
}

#[derive(Debug, Deserialize)]
pub struct AssignPamongInput {
    pub id_capaska: i32,
    pub id_pamong: String,
}

#[post("/api/pemusatan/assign-pamong")]
pub async fn assign_pamong(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<AssignPamongInput>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // Hanya Admin/Superadmin yang bisa assign
    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak. Hanya Admin Pemusatan atau Superadmin.",
        ));
    }

    // Validasi: cek apakah pamong dengan id tersebut ada dan role-nya Pamong
    let check_pamong = sqlx::query!(
        "SELECT role FROM users WHERE id = ? AND role = 'Pamong'",
        payload.id_pamong
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if check_pamong.is_none() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "User dengan role Pamong tidak ditemukan"
        })));
    }

    // Update data_capaska
    let result = sqlx::query!(
        "UPDATE data_capaska SET id_pamong = ? WHERE id = ?",
        payload.id_pamong,
        payload.id_capaska
    )
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Capaska tidak ditemukan"
        })));
    }

    // Log activity
    crate::utils::log_activity(
        pool.get_ref(),
        Some(&claims.user_id),
        Some(&claims.nama_user),
        Some(&claims.role),
        "ASSIGN_PAMONG",
        "PEMUSATAN",
        "SUCCESS",
        Some(&format!(
            "Assigned Pamong {} to Capaska {}",
            payload.id_pamong, payload.id_capaska
        )),
        Some(&req),
    )
    .await;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Pamong berhasil diassign ke Capaska"
    })))
}

#[get("/api/pemusatan/list-pamong")]
pub async fn get_pamong_list(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak. Hanya Admin Pemusatan atau Superadmin.",
        ));
    }

    let pamong_list = sqlx::query!(
        r#"
        SELECT
            u.id,
            u.name,
            COUNT(dc.id) as count_assigned
        FROM users u
        LEFT JOIN data_capaska dc ON u.id = dc.id_pamong
        WHERE u.role = 'Pamong'
        GROUP BY u.id, u.name
        ORDER BY u.name ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<serde_json::Value> = pamong_list
        .into_iter()
        .map(|row| {
            json!({
                "id": row.id,
                "nama_user": row.name,
                "count_assigned": row.count_assigned
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/pemusatan/pamong/{id}/{tanggal}")]
pub async fn get_pamong_journal(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<(i32, String)>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = [
        "Pamong",
        "Pelatih",
        "Dokter",
        "Admin Pemusatan",
        "Superadmin",
    ];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let (id_paskibraka, tanggal) = path.into_inner();

    let query = r#"
        SELECT
            id, id_paskibraka, id_pamong, tanggal,
            nilai_ketaqwaan, nilai_niat_kemauan, nilai_keberanian, nilai_komunikasi,
            nilai_keterbukaan, nilai_ketelitian, nilai_kesadaran, nilai_toleransi,
            nilai_keikhlasan, nilai_mempercayai, nilai_jiwa_korsa, nilai_kekeluargaan,
            nilai_persatuan_kesatuan, nilai_ketahanan, nilai_kekompakan_keseragaman, nilai_ketertiban,
            nilai_kesopanan, nilai_kesigapan, nilai_kewajaran, nilai_ketanggapan,
            nilai_ketenangan, nilai_menyimak, nilai_kebiasaan, nilai_mengelola_stres,
            nilai_menghargai_waktu, nilai_berbicara, nilai_berjalan, nilai_makan_minum,
            nilai_kehadiran, nilai_hubungan_interpersonal, nilai_ketaatan, nilai_istirahat_malam,
            nilai_keindahan, nilai_kerapihan, nilai_kebersihan, nilai_berpakaian,
            nilai_penampilan_rambut, nilai_bersih_rapih_wangi, catatan
        FROM jurnal_pemusatan_pamong
        WHERE id_paskibraka = ? AND tanggal = ?
        LIMIT 1
    "#;

    let result: Option<PamongJournal> = sqlx::query_as(query)
        .bind(id_paskibraka)
        .bind(&tanggal)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    match result {
        Some(data) => Ok(HttpResponse::Ok().json(data)),
        None => Ok(HttpResponse::NotFound().json(json!({ "message": "Data tidak ditemukan" }))),
    }
}
// 3. Submit Pelatih Log
#[post("/api/pemusatan/pelatih")]
pub async fn submit_pelatih(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<PelatihInput>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Pelatih", "Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id = Uuid::new_v4().to_string();
    let query = r#"
        INSERT INTO jurnal_pemusatan_pelatih (
            id, id_paskibraka, id_petugas, tanggal,
            nilai_aba_aba, nilai_berhimpun, nilai_berkumpul, nilai_keluar_masuk_barisan,
            nilai_hormat, nilai_sikap_sempurna, nilai_istirahat, nilai_periksa_kerapihan,
            nilai_berhitung, nilai_lepas_kenakan_topi, nilai_bubar, nilai_lencang_depan,
            nilai_lencang_kanan_kiri, nilai_setengah_lengan_lencang_kanan_kiri, nilai_hadap_kanan_kiri, nilai_hadap_serong_kanan_kiri,
            nilai_balik_kanan, nilai_langkah_bisa, nilai_langkah_tegap, nilai_sikap_awal_berlari,
            nilai_jalan_di_tempat, nilai_4_langkah_ke_depan, nilai_4_langkah_ke_kanan, nilai_4_langkah_ke_kiri,
            nilai_4_langkah_ke_belakang, nilai_lipat_bendera, nilai_bentang_bendera, nilai_10_tahap_penurunan,
            nilai_jadi_kibra_pembentang, nilai_jadi_kibra_pembawa, nilai_jadi_kibra_pengerek, catatan
        )
        VALUES (
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?,
            ?, ?, ?, ?
        )
        ON DUPLICATE KEY UPDATE
            id_petugas = VALUES(id_petugas),
            nilai_aba_aba = VALUES(nilai_aba_aba),
            nilai_berhimpun = VALUES(nilai_berhimpun),
            nilai_berkumpul = VALUES(nilai_berkumpul),
            nilai_keluar_masuk_barisan = VALUES(nilai_keluar_masuk_barisan),
            nilai_hormat = VALUES(nilai_hormat),
            nilai_sikap_sempurna = VALUES(nilai_sikap_sempurna),
            nilai_istirahat = VALUES(nilai_istirahat),
            nilai_periksa_kerapihan = VALUES(nilai_periksa_kerapihan),
            nilai_berhitung = VALUES(nilai_berhitung),
            nilai_lepas_kenakan_topi = VALUES(nilai_lepas_kenakan_topi),
            nilai_bubar = VALUES(nilai_bubar),
            nilai_lencang_depan = VALUES(nilai_lencang_depan),
            nilai_lencang_kanan_kiri = VALUES(nilai_lencang_kanan_kiri),
            nilai_setengah_lengan_lencang_kanan_kiri = VALUES(nilai_setengah_lengan_lencang_kanan_kiri),
            nilai_hadap_kanan_kiri = VALUES(nilai_hadap_kanan_kiri),
            nilai_hadap_serong_kanan_kiri = VALUES(nilai_hadap_serong_kanan_kiri),
            nilai_balik_kanan = VALUES(nilai_balik_kanan),
            nilai_langkah_bisa = VALUES(nilai_langkah_bisa),
            nilai_langkah_tegap = VALUES(nilai_langkah_tegap),
            nilai_sikap_awal_berlari = VALUES(nilai_sikap_awal_berlari),
            nilai_jalan_di_tempat = VALUES(nilai_jalan_di_tempat),
            nilai_4_langkah_ke_depan = VALUES(nilai_4_langkah_ke_depan),
            nilai_4_langkah_ke_kanan = VALUES(nilai_4_langkah_ke_kanan),
            nilai_4_langkah_ke_kiri = VALUES(nilai_4_langkah_ke_kiri),
            nilai_4_langkah_ke_belakang = VALUES(nilai_4_langkah_ke_belakang),
            nilai_lipat_bendera = VALUES(nilai_lipat_bendera),
            nilai_bentang_bendera = VALUES(nilai_bentang_bendera),
            nilai_10_tahap_penurunan = VALUES(nilai_10_tahap_penurunan),
            nilai_jadi_kibra_pembentang = VALUES(nilai_jadi_kibra_pembentang),
            nilai_jadi_kibra_pembawa = VALUES(nilai_jadi_kibra_pembawa),
            nilai_jadi_kibra_pengerek = VALUES(nilai_jadi_kibra_pengerek),
            catatan = VALUES(catatan)
    "#;

    sqlx::query(query)
        .bind(&id)
        .bind(payload.id_paskibraka)
        .bind(&claims.user_id)
        .bind(&payload.tanggal)
        .bind(payload.nilai_aba_aba)
        .bind(payload.nilai_berhimpun)
        .bind(payload.nilai_berkumpul)
        .bind(payload.nilai_keluar_masuk_barisan)
        .bind(payload.nilai_hormat)
        .bind(payload.nilai_sikap_sempurna)
        .bind(payload.nilai_istirahat)
        .bind(payload.nilai_periksa_kerapihan)
        .bind(payload.nilai_berhitung)
        .bind(payload.nilai_lepas_kenakan_topi)
        .bind(payload.nilai_bubar)
        .bind(payload.nilai_lencang_depan)
        .bind(payload.nilai_lencang_kanan_kiri)
        .bind(payload.nilai_setengah_lengan_lencang_kanan_kiri)
        .bind(payload.nilai_hadap_kanan_kiri)
        .bind(payload.nilai_hadap_serong_kanan_kiri)
        .bind(payload.nilai_balik_kanan)
        .bind(payload.nilai_langkah_bisa)
        .bind(payload.nilai_langkah_tegap)
        .bind(payload.nilai_sikap_awal_berlari)
        .bind(payload.nilai_jalan_di_tempat)
        .bind(payload.nilai_4_langkah_ke_depan)
        .bind(payload.nilai_4_langkah_ke_kanan)
        .bind(payload.nilai_4_langkah_ke_kiri)
        .bind(payload.nilai_4_langkah_ke_belakang)
        .bind(payload.nilai_lipat_bendera)
        .bind(payload.nilai_bentang_bendera)
        .bind(payload.nilai_10_tahap_penurunan)
        .bind(payload.nilai_jadi_kibra_pembentang)
        .bind(payload.nilai_jadi_kibra_pembawa)
        .bind(payload.nilai_jadi_kibra_pengerek)
        .bind(&payload.catatan)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Log this action
    crate::utils::log_activity(
        pool.get_ref(),
        Some(&claims.user_id),
        Some(&claims.nama_user),
        Some(&claims.role),
        "SUBMIT_PELATIH_JOURNAL",
        "PEMUSATAN",
        "SUCCESS",
        Some(&format!(
            "Saved Pelatih journal for capaska {}",
            payload.id_paskibraka
        )),
        Some(&req),
    )
    .await;

    Ok(HttpResponse::Ok()
        .json(json!({ "status": "success", "message": "Jurnal Pelatih berhasil disimpan" })))
}

// 4. Submit Dokter Log
#[post("/api/pemusatan/dokter")]
pub async fn submit_dokter(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<DokterInput>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Dokter", "Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id = Uuid::new_v4().to_string();
    let query = r#"
        INSERT INTO jurnal_pemusatan_dokter (id, id_paskibraka, id_petugas, tanggal, tensi, suhu, keluhan, diagnosa, terapi_obat, rekomendasi_istirahat)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            id_petugas = VALUES(id_petugas),
            tensi = VALUES(tensi),
            suhu = VALUES(suhu),
            keluhan = VALUES(keluhan),
            diagnosa = VALUES(diagnosa),
            terapi_obat = VALUES(terapi_obat),
            rekomendasi_istirahat = VALUES(rekomendasi_istirahat)
    "#;

    sqlx::query(query)
        .bind(&id)
        .bind(payload.id_paskibraka)
        .bind(&claims.user_id)
        .bind(&payload.tanggal)
        .bind(&payload.tensi)
        .bind(payload.suhu)
        .bind(&payload.keluhan)
        .bind(&payload.diagnosa)
        .bind(&payload.terapi_obat)
        .bind(&payload.rekomendasi_istirahat)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Log this action
    crate::utils::log_activity(
        pool.get_ref(),
        Some(&claims.user_id),
        Some(&claims.nama_user),
        Some(&claims.role),
        "SUBMIT_DOKTER_JOURNAL",
        "PEMUSATAN",
        "SUCCESS",
        Some(&format!(
            "Saved Dokter journal for capaska {}",
            payload.id_paskibraka
        )),
        Some(&req),
    )
    .await;

    Ok(HttpResponse::Ok()
        .json(json!({ "status": "success", "message": "Jurnal Dokter berhasil disimpan" })))
}

// 5. Fetch Full Profiling Journal for a Candidate
#[get("/api/pemusatan/jurnal/{id}")]
pub async fn get_jurnal(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = [
        "Pamong",
        "Pelatih",
        "Dokter",
        "Admin Pemusatan",
        "Superadmin",
    ];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_paskibraka = path.into_inner();

    // 1. Fetch Candidate Profile details dari data_capaska
    let profile_sql = r#"
        SELECT
            id,
            nama_lengkap,
            photo,
            jk,
            no_peserta,
            no_hp,
            tanggal_lahir,
            tempat_lahir,
            provinsi,
            kabupaten_kota,
            asal_sekolah,
            status
        FROM data_capaska
        WHERE id = ?
        LIMIT 1
    "#;

    #[derive(Debug, FromRow)]
    struct ProfileData {
        pub id: i32,
        pub nama_lengkap: Option<String>,
        pub photo: Option<String>,
        pub jk: Option<String>,
        pub no_peserta: Option<String>,
        pub no_hp: Option<String>,
        pub tanggal_lahir: Option<chrono::NaiveDate>,
        pub tempat_lahir: Option<String>,
        pub provinsi: Option<String>,
        pub kabupaten_kota: Option<String>,
        pub asal_sekolah: Option<String>,
        pub status: Option<String>,
    }

    let profile: Option<ProfileData> = sqlx::query_as(profile_sql)
        .bind(id_paskibraka)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let candidate = match profile {
        Some(c) => c,
        None => {
            return Ok(
                HttpResponse::NotFound().json(json!({ "message": "Peserta tidak ditemukan" }))
            );
        }
    };

    // 2. Fetch Daily logs - PAMONG (Gunakan Option untuk semua field karena bisa NULL)
    let pamong_logs_sql = r#"
        SELECT
            tanggal,
            nilai_ketaqwaan, nilai_niat_kemauan, nilai_keberanian, nilai_komunikasi,
            nilai_keterbukaan, nilai_ketelitian, nilai_kesadaran, nilai_toleransi,
            nilai_keikhlasan, nilai_mempercayai, nilai_jiwa_korsa, nilai_kekeluargaan,
            nilai_persatuan_kesatuan, nilai_ketahanan, nilai_kekompakan_keseragaman, nilai_ketertiban,
            nilai_kesopanan, nilai_kesigapan, nilai_kewajaran, nilai_ketanggapan,
            nilai_ketenangan, nilai_menyimak, nilai_kebiasaan, nilai_mengelola_stres,
            nilai_menghargai_waktu, nilai_berbicara, nilai_berjalan, nilai_makan_minum,
            nilai_kehadiran, nilai_hubungan_interpersonal, nilai_ketaatan, nilai_istirahat_malam,
            nilai_keindahan, nilai_kerapihan, nilai_kebersihan, nilai_berpakaian,
            nilai_penampilan_rambut, nilai_bersih_rapih_wangi, catatan
        FROM jurnal_pemusatan_pamong
        WHERE id_paskibraka = ?
        ORDER BY tanggal ASC
    "#;

    let pamong_logs: Vec<serde_json::Value> = sqlx::query(pamong_logs_sql)
        .bind(id_paskibraka)
        .fetch_all(pool.get_ref())
        .await
        .map(|rows| {
            rows.into_iter().map(|row| {
                use sqlx::Row;
                let t: chrono::NaiveDate = row.get("tanggal");
                let catatan: Option<String> = row.get("catatan");

                json!({
                    "tanggal": t.to_string(),
                    "nilai_ketaqwaan": row.get::<Option<i32>, _>("nilai_ketaqwaan"),
                    "nilai_niat_kemauan": row.get::<Option<i32>, _>("nilai_niat_kemauan"),
                    "nilai_keberanian": row.get::<Option<i32>, _>("nilai_keberanian"),
                    "nilai_komunikasi": row.get::<Option<i32>, _>("nilai_komunikasi"),
                    "nilai_keterbukaan": row.get::<Option<i32>, _>("nilai_keterbukaan"),
                    "nilai_ketelitian": row.get::<Option<i32>, _>("nilai_ketelitian"),
                    "nilai_kesadaran": row.get::<Option<i32>, _>("nilai_kesadaran"),
                    "nilai_toleransi": row.get::<Option<i32>, _>("nilai_toleransi"),
                    "nilai_keikhlasan": row.get::<Option<i32>, _>("nilai_keikhlasan"),
                    "nilai_mempercayai": row.get::<Option<i32>, _>("nilai_mempercayai"),
                    "nilai_jiwa_korsa": row.get::<Option<i32>, _>("nilai_jiwa_korsa"),
                    "nilai_kekeluargaan": row.get::<Option<i32>, _>("nilai_kekeluargaan"),
                    "nilai_persatuan_kesatuan": row.get::<Option<i32>, _>("nilai_persatuan_kesatuan"),
                    "nilai_ketahanan": row.get::<Option<i32>, _>("nilai_ketahanan"),
                    "nilai_kekompakan_keseragaman": row.get::<Option<i32>, _>("nilai_kekompakan_keseragaman"),
                    "nilai_ketertiban": row.get::<Option<i32>, _>("nilai_ketertiban"),
                    "nilai_kesopanan": row.get::<Option<i32>, _>("nilai_kesopanan"),
                    "nilai_kesigapan": row.get::<Option<i32>, _>("nilai_kesigapan"),
                    "nilai_kewajaran": row.get::<Option<i32>, _>("nilai_kewajaran"),
                    "nilai_ketanggapan": row.get::<Option<i32>, _>("nilai_ketanggapan"),
                    "nilai_ketenangan": row.get::<Option<i32>, _>("nilai_ketenangan"),
                    "nilai_menyimak": row.get::<Option<i32>, _>("nilai_menyimak"),
                    "nilai_kebiasaan": row.get::<Option<i32>, _>("nilai_kebiasaan"),
                    "nilai_mengelola_stres": row.get::<Option<i32>, _>("nilai_mengelola_stres"),
                    "nilai_menghargai_waktu": row.get::<Option<i32>, _>("nilai_menghargai_waktu"),
                    "nilai_berbicara": row.get::<Option<i32>, _>("nilai_berbicara"),
                    "nilai_berjalan": row.get::<Option<i32>, _>("nilai_berjalan"),
                    "nilai_makan_minum": row.get::<Option<i32>, _>("nilai_makan_minum"),
                    "nilai_kehadiran": row.get::<Option<i32>, _>("nilai_kehadiran"),
                    "nilai_hubungan_interpersonal": row.get::<Option<i32>, _>("nilai_hubungan_interpersonal"),
                    "nilai_ketaatan": row.get::<Option<i32>, _>("nilai_ketaatan"),
                    "nilai_istirahat_malam": row.get::<Option<i32>, _>("nilai_istirahat_malam"),
                    "nilai_keindahan": row.get::<Option<i32>, _>("nilai_keindahan"),
                    "nilai_kerapihan": row.get::<Option<i32>, _>("nilai_kerapihan"),
                    "nilai_kebersihan": row.get::<Option<i32>, _>("nilai_kebersihan"),
                    "nilai_berpakaian": row.get::<Option<i32>, _>("nilai_berpakaian"),
                    "nilai_penampilan_rambut": row.get::<Option<i32>, _>("nilai_penampilan_rambut"),
                    "nilai_bersih_rapih_wangi": row.get::<Option<i32>, _>("nilai_bersih_rapih_wangi"),
                    "catatan": catatan
                })
            }).collect()
        })
        .unwrap_or_else(|_| Vec::new());

    // 3. Fetch Pelatih logs
    let pelatih_logs_sql = r#"
        SELECT
            tanggal,
            nilai_aba_aba, nilai_berhimpun, nilai_berkumpul, nilai_keluar_masuk_barisan,
            nilai_hormat, nilai_sikap_sempurna, nilai_istirahat, nilai_periksa_kerapihan,
            nilai_berhitung, nilai_lepas_kenakan_topi, nilai_bubar, nilai_lencang_depan,
            nilai_lencang_kanan_kiri, nilai_setengah_lengan_lencang_kanan_kiri, nilai_hadap_kanan_kiri, nilai_hadap_serong_kanan_kiri,
            nilai_balik_kanan, nilai_langkah_bisa, nilai_langkah_tegap, nilai_sikap_awal_berlari,
            nilai_jalan_di_tempat, nilai_4_langkah_ke_depan, nilai_4_langkah_ke_kanan, nilai_4_langkah_ke_kiri,
            nilai_4_langkah_ke_belakang, nilai_lipat_bendera, nilai_bentang_bendera, nilai_10_tahap_penurunan,
            nilai_jadi_kibra_pembentang, nilai_jadi_kibra_pembawa, nilai_jadi_kibra_pengerek, catatan
        FROM jurnal_pemusatan_pelatih
        WHERE id_paskibraka = ?
        ORDER BY tanggal ASC
    "#;

    let pelatih_logs: Vec<serde_json::Value> = sqlx::query(pelatih_logs_sql)
        .bind(id_paskibraka)
        .fetch_all(pool.get_ref())
        .await
        .map(|rows| {
            rows.into_iter().map(|row| {
                use sqlx::Row;
                let t: chrono::NaiveDate = row.get("tanggal");
                let catatan: Option<String> = row.get("catatan");
                json!({
                    "tanggal": t.to_string(),
                    "nilai_aba_aba": row.get::<Option<i32>, _>("nilai_aba_aba"),
                    "nilai_berhimpun": row.get::<Option<i32>, _>("nilai_berhimpun"),
                    "nilai_berkumpul": row.get::<Option<i32>, _>("nilai_berkumpul"),
                    "nilai_keluar_masuk_barisan": row.get::<Option<i32>, _>("nilai_keluar_masuk_barisan"),
                    "nilai_hormat": row.get::<Option<i32>, _>("nilai_hormat"),
                    "nilai_sikap_sempurna": row.get::<Option<i32>, _>("nilai_sikap_sempurna"),
                    "nilai_istirahat": row.get::<Option<i32>, _>("nilai_istirahat"),
                    "nilai_periksa_kerapihan": row.get::<Option<i32>, _>("nilai_periksa_kerapihan"),
                    "nilai_berhitung": row.get::<Option<i32>, _>("nilai_berhitung"),
                    "nilai_lepas_kenakan_topi": row.get::<Option<i32>, _>("nilai_lepas_kenakan_topi"),
                    "nilai_bubar": row.get::<Option<i32>, _>("nilai_bubar"),
                    "nilai_lencang_depan": row.get::<Option<i32>, _>("nilai_lencang_depan"),
                    "nilai_lencang_kanan_kiri": row.get::<Option<i32>, _>("nilai_lencang_kanan_kiri"),
                    "nilai_setengah_lengan_lencang_kanan_kiri": row.get::<Option<i32>, _>("nilai_setengah_lengan_lencang_kanan_kiri"),
                    "nilai_hadap_kanan_kiri": row.get::<Option<i32>, _>("nilai_hadap_kanan_kiri"),
                    "nilai_hadap_serong_kanan_kiri": row.get::<Option<i32>, _>("nilai_hadap_serong_kanan_kiri"),
                    "nilai_balik_kanan": row.get::<Option<i32>, _>("nilai_balik_kanan"),
                    "nilai_langkah_bisa": row.get::<Option<i32>, _>("nilai_langkah_bisa"),
                    "nilai_langkah_tegap": row.get::<Option<i32>, _>("nilai_langkah_tegap"),
                    "nilai_sikap_awal_berlari": row.get::<Option<i32>, _>("nilai_sikap_awal_berlari"),
                    "nilai_jalan_di_tempat": row.get::<Option<i32>, _>("nilai_jalan_di_tempat"),
                    "nilai_4_langkah_ke_depan": row.get::<Option<i32>, _>("nilai_4_langkah_ke_depan"),
                    "nilai_4_langkah_ke_kanan": row.get::<Option<i32>, _>("nilai_4_langkah_ke_kanan"),
                    "nilai_4_langkah_ke_kiri": row.get::<Option<i32>, _>("nilai_4_langkah_ke_kiri"),
                    "nilai_4_langkah_ke_belakang": row.get::<Option<i32>, _>("nilai_4_langkah_ke_belakang"),
                    "nilai_lipat_bendera": row.get::<Option<i32>, _>("nilai_lipat_bendera"),
                    "nilai_bentang_bendera": row.get::<Option<i32>, _>("nilai_bentang_bendera"),
                    "nilai_10_tahap_penurunan": row.get::<Option<i32>, _>("nilai_10_tahap_penurunan"),
                    "nilai_jadi_kibra_pembentang": row.get::<Option<i32>, _>("nilai_jadi_kibra_pembentang"),
                    "nilai_jadi_kibra_pembawa": row.get::<Option<i32>, _>("nilai_jadi_kibra_pembawa"),
                    "nilai_jadi_kibra_pengerek": row.get::<Option<i32>, _>("nilai_jadi_kibra_pengerek"),
                    "catatan": catatan
                })
            }).collect()
        })
        .unwrap_or_else(|_| Vec::new());

    // 4. Fetch Dokter logs
    let dokter_logs_sql = r#"
        SELECT
            tanggal,
            tensi,
            suhu,
            keluhan,
            diagnosa,
            terapi_obat,
            rekomendasi_istirahat
        FROM jurnal_pemusatan_dokter
        WHERE id_paskibraka = ?
        ORDER BY tanggal ASC
    "#;

    let dokter_logs: Vec<serde_json::Value> = sqlx::query(dokter_logs_sql)
        .bind(id_paskibraka)
        .fetch_all(pool.get_ref())
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| {
                    use sqlx::Row;
                    let t: chrono::NaiveDate = row.get("tanggal");
                    let tensi: Option<String> = row.get("tensi");
                    let suhu: Option<rust_decimal::Decimal> = row.get("suhu");
                    let keluhan: Option<String> = row.get("keluhan");
                    let diagnosa: Option<String> = row.get("diagnosa");
                    let obat: Option<String> = row.get("terapi_obat");
                    let rek: Option<String> = row.get("rekomendasi_istirahat");
                    json!({
                        "tanggal": t.to_string(),
                        "tensi": tensi,
                        "suhu": suhu.map(|d| d.to_string()),
                        "keluhan": keluhan,
                        "diagnosa": diagnosa,
                        "terapi_obat": obat,
                        "rekomendasi_istirahat": rek
                    })
                })
                .collect()
        })
        .unwrap_or_else(|_| Vec::new());

    // Build response dengan struktur yang benar
    let response = json!({
        "profile": {
            "id": candidate.id,
            "nama_lengkap": candidate.nama_lengkap,
            "photo": candidate.photo,
            "jk": candidate.jk,
            "no_peserta": candidate.no_peserta,
            "no_hp": candidate.no_hp,
            "tanggal_lahir": candidate.tanggal_lahir.map(|d| d.to_string()),
            "tempat_lahir": candidate.tempat_lahir,
            "provinsi": candidate.provinsi,
            "kabupaten_kota": candidate.kabupaten_kota,
            "asal_sekolah": candidate.asal_sekolah,
            "status": candidate.status
        },
        "pemusatan": {
            "pamong": pamong_logs,
            "pelatih": pelatih_logs,
            "dokter": dokter_logs
        }
    });

    Ok(HttpResponse::Ok().json(response))
}

// Update struct definition
#[derive(Debug, Serialize)]
pub struct PamongDashboardStats {
    pub tanggal: String,
    pub jumlah_sikap: rust_decimal::Decimal, // Ubah ke Decimal
    pub jumlah_penampilan: rust_decimal::Decimal, // Ubah ke Decimal
    pub total_penilaian: i64,
    pub rata_rata_sikap: f64,
    pub rata_rata_penampilan: f64,
}

#[derive(Debug, Serialize)]
pub struct PamongCandidateStats {
    pub id: i32,
    pub nama_lengkap: String,
    pub jk: String,
    pub total_sikap: i64,
    pub total_penampilan: i64,
    pub total_penilaian: i64,
    pub rata_rata_sikap: f64,
    pub rata_rata_penampilan: f64,
    pub nilai_keseluruhan: f64,
    pub nilai_rata_rata: f64,
}

#[derive(Debug, Serialize)]
pub struct PamongDashboardResponse {
    pub daily_stats: Vec<PamongDashboardStats>,
    pub candidate_stats: Vec<PamongCandidateStats>,
    pub total_candidates: i64,
    pub total_entries: i64,
}

#[get("/api/pemusatan/pamong/dashboard")]
pub async fn get_pamong_dashboard(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if claims.role.as_str() != "Pamong" {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak. Hanya untuk Pamong.",
        ));
    }

    let id_pamong = &claims.user_id;

    // Get list of Capaska IDs assigned to this Pamong
    let assigned_capaska =
        sqlx::query!("SELECT id FROM data_capaska WHERE id_pamong = ?", id_pamong)
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    let capaska_ids: Vec<i32> = assigned_capaska.iter().map(|row| row.id).collect();

    if capaska_ids.is_empty() {
        // Return empty dashboard if no assigned capaska
        let response = PamongDashboardResponse {
            daily_stats: Vec::new(),
            candidate_stats: Vec::new(),
            total_candidates: 0,
            total_entries: 0,
        };
        return Ok(HttpResponse::Ok().json(response));
    }

    // Build IN clause for capaska IDs
    let placeholders: Vec<String> = capaska_ids.iter().map(|_| "?".to_string()).collect();
    let in_clause = placeholders.join(",");

    // Daily stats dengan filter capaska yang diassign
    let daily_sql = format!(
        r#"
        SELECT
            tanggal,
            COUNT(*) as total_entries,
            CAST(SUM(
                CASE WHEN nilai_ketaqwaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_niat_kemauan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_keberanian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_komunikasi IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_keterbukaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketelitian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kesadaran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_toleransi IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_keikhlasan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_mempercayai IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_jiwa_korsa IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kekeluargaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_persatuan_kesatuan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketahanan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kekompakan_keseragaman IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketertiban IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kesopanan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kesigapan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kewajaran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketanggapan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketenangan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_menyimak IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kebiasaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_mengelola_stres IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_menghargai_waktu IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_berbicara IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_berjalan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_makan_minum IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kehadiran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_hubungan_interpersonal IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketaatan IS NOT NULL THEN 1 ELSE 0 END
            ) AS SIGNED) as jumlah_sikap,
            CAST(SUM(
                CASE WHEN nilai_istirahat_malam IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_keindahan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kerapihan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kebersihan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_berpakaian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_penampilan_rambut IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_bersih_rapih_wangi IS NOT NULL THEN 1 ELSE 0 END
            ) AS SIGNED) as jumlah_penampilan
        FROM jurnal_pemusatan_pamong
        WHERE id_pamong = ? AND id_paskibraka IN ({})
        GROUP BY tanggal
        ORDER BY tanggal DESC
        LIMIT 30
        "#,
        in_clause
    );

    // Execute daily query with dynamic params
    let mut daily_query = sqlx::query(&daily_sql);
    daily_query = daily_query.bind(id_pamong);
    for id in &capaska_ids {
        daily_query = daily_query.bind(id);
    }

    let daily_rows = daily_query
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let daily_stats_response: Vec<PamongDashboardStats> = daily_rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            let tanggal: chrono::NaiveDate = row.get("tanggal");
            let total_entries: i64 = row.get("total_entries");
            let jumlah_sikap: i64 = row.get("jumlah_sikap");
            let jumlah_penampilan: i64 = row.get("jumlah_penampilan");

            let rata_rata_sikap = if total_entries > 0 {
                jumlah_sikap as f64 / total_entries as f64
            } else {
                0.0
            };
            let rata_rata_penampilan = if total_entries > 0 {
                jumlah_penampilan as f64 / total_entries as f64
            } else {
                0.0
            };

            PamongDashboardStats {
                tanggal: tanggal.to_string(),
                jumlah_sikap: rust_decimal::Decimal::from(jumlah_sikap),
                jumlah_penampilan: rust_decimal::Decimal::from(jumlah_penampilan),
                total_penilaian: total_entries,
                rata_rata_sikap,
                rata_rata_penampilan,
            }
        })
        .collect();

    // Candidate stats dengan filter capaska yang diassign
    let candidate_sql = format!(
        r#"
        SELECT
            dc.id,
            dc.nama_lengkap,
            dc.jk,
            COUNT(jp.id) as total_entries,
            CAST(SUM(
                CASE WHEN jp.nilai_ketaqwaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_niat_kemauan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_keberanian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_komunikasi IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_keterbukaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_ketelitian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kesadaran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_toleransi IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_keikhlasan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_mempercayai IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_jiwa_korsa IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kekeluargaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_persatuan_kesatuan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_ketahanan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kekompakan_keseragaman IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_ketertiban IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kesopanan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kesigapan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kewajaran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_ketanggapan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_ketenangan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_menyimak IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kebiasaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_mengelola_stres IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_menghargai_waktu IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_berbicara IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_berjalan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_makan_minum IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kehadiran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_hubungan_interpersonal IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_ketaatan IS NOT NULL THEN 1 ELSE 0 END
            ) AS SIGNED) as total_sikap,
            CAST(SUM(
                CASE WHEN jp.nilai_istirahat_malam IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_keindahan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kerapihan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_kebersihan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_berpakaian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_penampilan_rambut IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN jp.nilai_bersih_rapih_wangi IS NOT NULL THEN 1 ELSE 0 END
            ) AS SIGNED) as total_penampilan
        FROM data_capaska dc
        LEFT JOIN jurnal_pemusatan_pamong jp
            ON dc.id = jp.id_paskibraka AND jp.id_pamong = ?
        WHERE dc.id_pamong = ?
        GROUP BY dc.id, dc.nama_lengkap, dc.jk
        ORDER BY COUNT(jp.id) DESC, dc.nama_lengkap ASC
        "#
    );

    let candidate_stats_response: Vec<PamongCandidateStats> = sqlx::query(&candidate_sql)
        .bind(id_pamong)
        .bind(id_pamong)
        .fetch_all(pool.get_ref())
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| {
                    use sqlx::Row;
                    let id: i32 = row.get("id");
                    let nama_lengkap: String = row.get("nama_lengkap");
                    let jk: String = row.get("jk");
                    let total_entries: i64 = row.get("total_entries");
                    let total_sikap: i64 = row.get("total_sikap");
                    let total_penampilan: i64 = row.get("total_penampilan");

                    let rata_rata_sikap = if total_entries > 0 {
                        total_sikap as f64 / total_entries as f64
                    } else {
                        0.0
                    };
                    let rata_rata_penampilan = if total_entries > 0 {
                        total_penampilan as f64 / total_entries as f64
                    } else {
                        0.0
                    };

                    let mut valid = Vec::new();
                    if rata_rata_sikap > 0.0 { valid.push(rata_rata_sikap); }
                    if rata_rata_penampilan > 0.0 { valid.push(rata_rata_penampilan); }
                    let nilai_keseluruhan = rata_rata_sikap + rata_rata_penampilan;
                    let nilai_rata_rata = if !valid.is_empty() {
                        valid.iter().sum::<f64>() / valid.len() as f64
                    } else {
                        0.0
                    };

                    PamongCandidateStats {
                        id,
                        nama_lengkap,
                        jk,
                        total_sikap,
                        total_penampilan,
                        total_penilaian: total_entries,
                        rata_rata_sikap,
                        rata_rata_penampilan,
                        nilai_keseluruhan,
                        nilai_rata_rata,
                    }
                })
                .collect()
        })
        .unwrap_or_else(|_| Vec::new());

    let total_candidates = candidate_stats_response.len() as i64;
    let total_entries: i64 = candidate_stats_response
        .iter()
        .map(|c| c.total_penilaian)
        .sum();

    let response = PamongDashboardResponse {
        daily_stats: daily_stats_response,
        candidate_stats: candidate_stats_response,
        total_candidates,
        total_entries,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCapaskaInput {
    pub nama_lengkap: Option<String>,
    pub jk: Option<String>,
    pub no_peserta: Option<String>,
    pub no_hp: Option<String>,
    pub tanggal_lahir: Option<String>,
    pub tempat_lahir: Option<String>,
    pub provinsi: Option<String>,
    pub kabupaten_kota: Option<String>,
    pub asal_sekolah: Option<String>,
    pub status: Option<String>,
}

#[get("/api/pemusatan/capaska/{id}")]
pub async fn get_capaska(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak. Hanya Admin Pemusatan atau Superadmin.",
        ));
    }

    let id = path.into_inner();

    let capaska = sqlx::query!(
        r#"
        SELECT
            id,
            nama_lengkap,
            photo,
            jk,
            asal_sekolah,
            no_peserta,
            no_hp,
            tanggal_lahir,
            tempat_lahir,
            provinsi,
            kabupaten_kota,
            id_pamong,
            status
        FROM data_capaska
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    match capaska {
        Some(data) => Ok(HttpResponse::Ok().json(json!({
            "id": data.id,
            "nama_lengkap": data.nama_lengkap,
            "photo": data.photo,
            "jk": data.jk,
            "asal_sekolah": data.asal_sekolah,
            "no_peserta": data.no_peserta,
            "no_hp": data.no_hp,
            "tanggal_lahir": data.tanggal_lahir.map(|d| d.to_string()),
            "tempat_lahir": data.tempat_lahir,
            "provinsi": data.provinsi,
            "kabupaten_kota": data.kabupaten_kota,
            "id_pamong": data.id_pamong,
            "status": data.status,
        }))),
        None => Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Data tidak ditemukan"
        }))),
    }
}

#[put("/api/pemusatan/capaska/{id}")]
pub async fn update_capaska(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    payload: web::Json<UpdateCapaskaInput>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak. Hanya Admin Pemusatan atau Superadmin.",
        ));
    }

    let id = path.into_inner();

    // Build dynamic query untuk hanya update field yang dikirim
    let mut updates = Vec::new();
    let mut bind_params: Vec<String> = Vec::new();

    if let Some(ref val) = payload.nama_lengkap {
        updates.push("nama_lengkap = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.jk {
        updates.push("jk = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.no_peserta {
        updates.push("no_peserta = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.no_hp {
        updates.push("no_hp = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.tanggal_lahir {
        updates.push("tanggal_lahir = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.tempat_lahir {
        updates.push("tempat_lahir = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.provinsi {
        updates.push("provinsi = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.kabupaten_kota {
        updates.push("kabupaten_kota = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.asal_sekolah {
        updates.push("asal_sekolah = ?");
        bind_params.push(val.clone());
    }
    if let Some(ref val) = payload.status {
        updates.push("status = ?");
        bind_params.push(val.clone());
    }

    // Jika tidak ada field yang diupdate
    if updates.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Tidak ada data yang akan diupdate"
        })));
    }

    // Build query string
    let query_str = format!(
        "UPDATE data_capaska SET {} WHERE id = ?",
        updates.join(", ")
    );

    // Execute dengan dynamic params
    let mut query = sqlx::query(&query_str);
    for param in bind_params {
        query = query.bind(param);
    }
    query = query.bind(id);

    let result = query.execute(pool.get_ref()).await.map_err(|e| {
        eprintln!("Query error: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Data tidak ditemukan"
        })));
    }

    // Log activity
    crate::utils::log_activity(
        pool.get_ref(),
        Some(&claims.user_id),
        Some(&claims.nama_user),
        Some(&claims.role),
        "UPDATE_CAPASKA",
        "PEMUSATAN",
        "SUCCESS",
        Some(&format!("Updated Capaska data for id {}", id)),
        Some(&req),
    )
    .await;

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Data Capaska berhasil diperbarui"
    })))
}

#[post("/api/pemusatan/capaska/{id}/photo")]
pub async fn upload_capaska_photo(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak. Hanya Admin Pemusatan atau Superadmin.",
        ));
    }

    let id = path.into_inner();
    let mut photo_filename: Option<String> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().unwrap();
        let field_name = content_disposition.get_name().unwrap_or("");

        if field_name == "photo" {
            // Get filename
            let filename = content_disposition
                .get_filename()
                .map(|f| {
                    let ext = f.split('.').last().unwrap_or("jpg");
                    format!("capaska_{}_{}.{}", id, chrono::Utc::now().timestamp(), ext)
                })
                .unwrap_or_else(|| {
                    format!("capaska_{}_{}.jpg", id, chrono::Utc::now().timestamp())
                });

            // Read file data
            let mut data = Vec::new();
            while let Some(chunk) = field.try_next().await? {
                data.extend_from_slice(&chunk);
            }

            // Validate file size (max 2MB)
            if data.len() > 2 * 1024 * 1024 {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "Ukuran file maksimal 2MB"
                })));
            }

            // Create upload directory if not exists
            let upload_dir = std::path::Path::new("uploads/capaska");
            if !upload_dir.exists() {
                std::fs::create_dir_all(upload_dir).map_err(|e| {
                    actix_web::error::ErrorInternalServerError(format!("Create dir error: {}", e))
                })?;
            }

            // Save file
            let file_path = upload_dir.join(&filename);
            std::fs::write(&file_path, data).map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Write file error: {}", e))
            })?;

            photo_filename = Some(filename);
        }
    }

    if let Some(photo) = photo_filename {
        let result = sqlx::query!("UPDATE data_capaska SET photo = ? WHERE id = ?", photo, id)
            .execute(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        if result.rows_affected() == 0 {
            return Ok(HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "Data tidak ditemukan"
            })));
        }

        // Log activity
        crate::utils::log_activity(
            pool.get_ref(),
            Some(&claims.user_id),
            Some(&claims.nama_user),
            Some(&claims.role),
            "UPLOAD_CAPASKA_PHOTO",
            "PEMUSATAN",
            "SUCCESS",
            Some(&format!("Uploaded photo for Capaska id {}", id)),
            Some(&req),
        )
        .await;

        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Foto berhasil diupload",
            "photo": photo
        })))
    } else {
        Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Tidak ada file photo yang diupload"
        })))
    }
}

#[derive(Debug, Serialize)]
pub struct DashboardOverview {
    pub total_capaska: i64,
    pub total_pamong: i64,
    pub total_pelatih: i64,
    pub total_dokter: i64,
    pub total_journal_today: i64,
    pub total_unassigned_capaska: i64,
    pub avg_sikap: f64,
    pub avg_penampilan: f64,
}

#[derive(Debug, Serialize)]
pub struct PamongStat {
    pub id: String,
    pub nama_user: String,
    pub total_capaska: i64,
    pub total_journal: i64,
    pub avg_sikap: f64,
    pub avg_penampilan: f64,
    pub last_active: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DailyStat {
    pub tanggal: String,
    pub total_sikap: i64,
    pub total_penampilan: i64,
    pub total_journal: i64,
}

#[derive(Debug, Serialize)]
pub struct CapaskaProgress {
    pub id: i32,
    pub nama_lengkap: String,
    pub no_peserta: String,
    pub pamong_name: Option<String>,
    pub total_journal_pamong: i64,
    pub total_journal_pelatih: i64,
    pub total_journal_dokter: i64,
    pub status: Option<String>,
    pub pamong_sikap_avg: f64,
    pub pamong_penampilan_avg: f64,
    pub pelatih_pbb_avg: f64,
    pub pelatih_bendera_avg: f64,
    pub nilai_keseluruhan: f64,
    pub nilai_rata_rata: f64,
}

#[derive(Debug, Serialize)]
pub struct RecentActivity {
    pub id: String,
    pub user_id: String,
    pub nama_user: String,
    pub role: String,
    pub action: String,
    pub module: String,
    pub status: String,
    pub created_at: String,

}

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse::<f64>().unwrap_or(0.0)
}

// 3. Helper function untuk convert Option<Decimal> ke f64
fn option_decimal_to_f64(d: Option<Decimal>) -> f64 {
    d.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0)
}

#[derive(Debug, Serialize)]
pub struct RoleActivityStat {
    pub role: String,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub overview: DashboardOverview,
    pub daily_stats: Vec<DailyStat>,
    pub pamong_stats: Vec<PamongStat>,
    pub capaska_progress: Vec<CapaskaProgress>,
    pub recent_activities: Vec<RecentActivity>,
    pub role_activity_stats: Vec<RoleActivityStat>,
}

#[get("/api/pemusatan/admin/dashboard")]
pub async fn get_admin_dashboard(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    query: web::Query<DashboardFilter>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Akses ditolak. Hanya Admin Pemusatan atau Superadmin.",
        ));
    }

    let days = query.days.unwrap_or(30);

    // 1. Overview Statistics
    let overview = sqlx::query!(
        r#"
        SELECT
            (SELECT COUNT(*) FROM data_capaska) as total_capaska,
            (SELECT COUNT(*) FROM users WHERE role = 'Pamong') as total_pamong,
            (SELECT COUNT(*) FROM users WHERE role = 'Pelatih') as total_pelatih,
            (SELECT COUNT(*) FROM users WHERE role = 'Dokter') as total_dokter,
            (SELECT COUNT(*) FROM data_capaska WHERE id_pamong IS NULL) as total_unassigned,
            (SELECT COUNT(*) FROM jurnal_pemusatan_pamong WHERE DATE(tanggal) = CURDATE()) as total_journal_today,
            (
                SELECT AVG(
                    (COALESCE(nilai_ketaqwaan, 0) + COALESCE(nilai_niat_kemauan, 0) +
                     COALESCE(nilai_keberanian, 0) + COALESCE(nilai_komunikasi, 0) +
                     COALESCE(nilai_keterbukaan, 0) + COALESCE(nilai_ketelitian, 0) +
                     COALESCE(nilai_kesadaran, 0) + COALESCE(nilai_toleransi, 0) +
                     COALESCE(nilai_keikhlasan, 0) + COALESCE(nilai_mempercayai, 0) +
                     COALESCE(nilai_jiwa_korsa, 0) + COALESCE(nilai_kekeluargaan, 0) +
                     COALESCE(nilai_persatuan_kesatuan, 0) + COALESCE(nilai_ketahanan, 0) +
                     COALESCE(nilai_kekompakan_keseragaman, 0) + COALESCE(nilai_ketertiban, 0) +
                     COALESCE(nilai_kesopanan, 0) + COALESCE(nilai_kesigapan, 0) +
                     COALESCE(nilai_kewajaran, 0) + COALESCE(nilai_ketanggapan, 0) +
                     COALESCE(nilai_ketenangan, 0) + COALESCE(nilai_menyimak, 0) +
                     COALESCE(nilai_kebiasaan, 0) + COALESCE(nilai_mengelola_stres, 0) +
                     COALESCE(nilai_menghargai_waktu, 0) + COALESCE(nilai_berbicara, 0) +
                     COALESCE(nilai_berjalan, 0) + COALESCE(nilai_makan_minum, 0) +
                     COALESCE(nilai_kehadiran, 0) + COALESCE(nilai_hubungan_interpersonal, 0) +
                     COALESCE(nilai_ketaatan, 0)
                    ) / 31
                ) FROM jurnal_pemusatan_pamong
            ) as avg_sikap,
            (
                SELECT AVG(
                    (COALESCE(nilai_istirahat_malam, 0) + COALESCE(nilai_keindahan, 0) +
                     COALESCE(nilai_kerapihan, 0) + COALESCE(nilai_kebersihan, 0) +
                     COALESCE(nilai_berpakaian, 0) + COALESCE(nilai_penampilan_rambut, 0) +
                     COALESCE(nilai_bersih_rapih_wangi, 0)
                    ) / 7
                ) FROM jurnal_pemusatan_pamong
            ) as avg_penampilan
        "#
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let overview_response = DashboardOverview {
        total_capaska: overview.total_capaska.unwrap_or(0),
        total_pamong: overview.total_pamong.unwrap_or(0),
        total_pelatih: overview.total_pelatih.unwrap_or(0),
        total_dokter: overview.total_dokter.unwrap_or(0),
        total_journal_today: overview.total_journal_today.unwrap_or(0),
        total_unassigned_capaska: overview.total_unassigned.unwrap_or(0),
        avg_sikap: option_decimal_to_f64(overview.avg_sikap),
        avg_penampilan: option_decimal_to_f64(overview.avg_penampilan),
    };

    // 2. Daily Stats - Last N days
    let daily_stats = sqlx::query!(
        r#"
        SELECT
            tanggal,
            COUNT(*) as total_journal,
            CAST(SUM(
                CASE WHEN nilai_ketaqwaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_niat_kemauan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_keberanian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_komunikasi IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_keterbukaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketelitian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kesadaran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_toleransi IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_keikhlasan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_mempercayai IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_jiwa_korsa IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kekeluargaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_persatuan_kesatuan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketahanan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kekompakan_keseragaman IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketertiban IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kesopanan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kesigapan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kewajaran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketanggapan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketenangan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_menyimak IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kebiasaan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_mengelola_stres IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_menghargai_waktu IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_berbicara IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_berjalan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_makan_minum IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kehadiran IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_hubungan_interpersonal IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_ketaatan IS NOT NULL THEN 1 ELSE 0 END
            ) AS SIGNED) as total_sikap,
            CAST(SUM(
                CASE WHEN nilai_istirahat_malam IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_keindahan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kerapihan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_kebersihan IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_berpakaian IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_penampilan_rambut IS NOT NULL THEN 1 ELSE 0 END +
                CASE WHEN nilai_bersih_rapih_wangi IS NOT NULL THEN 1 ELSE 0 END
            ) AS SIGNED) as total_penampilan
        FROM jurnal_pemusatan_pamong
        WHERE tanggal >= DATE_SUB(CURDATE(), INTERVAL ? DAY)
        GROUP BY tanggal
        ORDER BY tanggal ASC
        "#,
        days
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let daily_stats_response: Vec<DailyStat> = daily_stats
        .into_iter()
        .map(|row| {
            DailyStat {
                tanggal: row.tanggal.to_string(),
                total_sikap: row.total_sikap.unwrap_or(0),
                total_penampilan: row.total_penampilan.unwrap_or(0),
                total_journal: row.total_journal,
            }
        })
        .collect();

    // 3. Pamong Stats
    let pamong_stats = sqlx::query!(
        r#"
        SELECT
            u.id,
            u.name as nama_user,
            COUNT(DISTINCT dc.id) as total_capaska,
            COUNT(jp.id) as total_journal,
            COALESCE(AVG(
                (COALESCE(jp.nilai_ketaqwaan, 0) + COALESCE(jp.nilai_niat_kemauan, 0) +
                 COALESCE(jp.nilai_keberanian, 0) + COALESCE(jp.nilai_komunikasi, 0) +
                 COALESCE(jp.nilai_keterbukaan, 0) + COALESCE(jp.nilai_ketelitian, 0) +
                 COALESCE(jp.nilai_kesadaran, 0) + COALESCE(jp.nilai_toleransi, 0) +
                 COALESCE(jp.nilai_keikhlasan, 0) + COALESCE(jp.nilai_mempercayai, 0) +
                 COALESCE(jp.nilai_jiwa_korsa, 0) + COALESCE(jp.nilai_kekeluargaan, 0) +
                 COALESCE(jp.nilai_persatuan_kesatuan, 0) + COALESCE(jp.nilai_ketahanan, 0) +
                 COALESCE(jp.nilai_kekompakan_keseragaman, 0) + COALESCE(jp.nilai_ketertiban, 0) +
                 COALESCE(jp.nilai_kesopanan, 0) + COALESCE(jp.nilai_kesigapan, 0) +
                 COALESCE(jp.nilai_kewajaran, 0) + COALESCE(jp.nilai_ketanggapan, 0) +
                 COALESCE(jp.nilai_ketenangan, 0) + COALESCE(jp.nilai_menyimak, 0) +
                 COALESCE(jp.nilai_kebiasaan, 0) + COALESCE(jp.nilai_mengelola_stres, 0) +
                 COALESCE(jp.nilai_menghargai_waktu, 0) + COALESCE(jp.nilai_berbicara, 0) +
                 COALESCE(jp.nilai_berjalan, 0) + COALESCE(jp.nilai_makan_minum, 0) +
                 COALESCE(jp.nilai_kehadiran, 0) + COALESCE(jp.nilai_hubungan_interpersonal, 0) +
                 COALESCE(jp.nilai_ketaatan, 0)
                ) / 31
            ), 0) as avg_sikap,
            COALESCE(AVG(
                (COALESCE(jp.nilai_istirahat_malam, 0) + COALESCE(jp.nilai_keindahan, 0) +
                 COALESCE(jp.nilai_kerapihan, 0) + COALESCE(jp.nilai_kebersihan, 0) +
                 COALESCE(jp.nilai_berpakaian, 0) + COALESCE(jp.nilai_penampilan_rambut, 0) +
                 COALESCE(jp.nilai_bersih_rapih_wangi, 0)
                ) / 7
            ), 0) as avg_penampilan,
            MAX(jp.created_at) as last_active
        FROM users u
        LEFT JOIN data_capaska dc ON u.id = dc.id_pamong COLLATE utf8mb4_general_ci
        LEFT JOIN jurnal_pemusatan_pamong jp ON u.id = jp.id_pamong COLLATE utf8mb4_general_ci
        WHERE u.role = 'Pamong'
        GROUP BY u.id, u.name
        ORDER BY total_journal DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let pamong_stats_response: Vec<PamongStat> = pamong_stats
        .into_iter()
        .map(|row| {
            PamongStat {
                id: row.id,
                nama_user: row.nama_user,
                total_capaska: row.total_capaska,
                total_journal: row.total_journal,
                avg_sikap: decimal_to_f64(row.avg_sikap),
                avg_penampilan: decimal_to_f64(row.avg_penampilan),
                last_active: row.last_active.map(|d| d.to_string()),
            }
        })
        .collect();

    // 4. Capaska Progress
    let capaska_progress = sqlx::query!(
        r#"
        SELECT
            dc.id,
            dc.nama_lengkap,
            dc.no_peserta,
            u.name as pamong_name,
            COUNT(DISTINCT jp.id) as total_journal_pamong,
            COUNT(DISTINCT jpel.id) as total_journal_pelatih,
            COUNT(DISTINCT jd.id) as total_journal_dokter,
            dc.status,
            COALESCE(AVG(
                (COALESCE(jp.nilai_ketaqwaan, 0) + COALESCE(jp.nilai_niat_kemauan, 0) +
                 COALESCE(jp.nilai_keberanian, 0) + COALESCE(jp.nilai_komunikasi, 0) +
                 COALESCE(jp.nilai_keterbukaan, 0) + COALESCE(jp.nilai_ketelitian, 0) +
                 COALESCE(jp.nilai_kesadaran, 0) + COALESCE(jp.nilai_toleransi, 0) +
                 COALESCE(jp.nilai_keikhlasan, 0) + COALESCE(jp.nilai_mempercayai, 0) +
                 COALESCE(jp.nilai_jiwa_korsa, 0) + COALESCE(jp.nilai_kekeluargaan, 0) +
                 COALESCE(jp.nilai_persatuan_kesatuan, 0) + COALESCE(jp.nilai_ketahanan, 0) +
                 COALESCE(jp.nilai_kekompakan_keseragaman, 0) + COALESCE(jp.nilai_ketertiban, 0) +
                 COALESCE(jp.nilai_kesopanan, 0) + COALESCE(jp.nilai_kesigapan, 0) +
                 COALESCE(jp.nilai_kewajaran, 0) + COALESCE(jp.nilai_ketanggapan, 0) +
                 COALESCE(jp.nilai_ketenangan, 0) + COALESCE(jp.nilai_menyimak, 0) +
                 COALESCE(jp.nilai_kebiasaan, 0) + COALESCE(jp.nilai_mengelola_stres, 0) +
                 COALESCE(jp.nilai_menghargai_waktu, 0) + COALESCE(jp.nilai_berbicara, 0) +
                 COALESCE(jp.nilai_berjalan, 0) + COALESCE(jp.nilai_makan_minum, 0) +
                 COALESCE(jp.nilai_kehadiran, 0) + COALESCE(jp.nilai_hubungan_interpersonal, 0) +
                 COALESCE(jp.nilai_ketaatan, 0)
                ) / 31
            ), 0) as pamong_sikap_avg,
            COALESCE(AVG(
                (COALESCE(jp.nilai_istirahat_malam, 0) + COALESCE(jp.nilai_keindahan, 0) +
                 COALESCE(jp.nilai_kerapihan, 0) + COALESCE(jp.nilai_kebersihan, 0) +
                 COALESCE(jp.nilai_berpakaian, 0) + COALESCE(jp.nilai_penampilan_rambut, 0) +
                 COALESCE(jp.nilai_bersih_rapih_wangi, 0)
                ) / 7
            ), 0) as pamong_penampilan_avg,
            COALESCE((
                SELECT AVG(
                    (COALESCE(nilai_aba_aba, 0) + COALESCE(nilai_berhimpun, 0) +
                     COALESCE(nilai_berkumpul, 0) + COALESCE(nilai_keluar_masuk_barisan, 0) +
                     COALESCE(nilai_hormat, 0) + COALESCE(nilai_sikap_sempurna, 0) +
                     COALESCE(nilai_istirahat, 0) + COALESCE(nilai_periksa_kerapihan, 0) +
                     COALESCE(nilai_berhitung, 0) + COALESCE(nilai_lepas_kenakan_topi, 0) +
                     COALESCE(nilai_bubar, 0) + COALESCE(nilai_lencang_depan, 0) +
                     COALESCE(nilai_lencang_kanan_kiri, 0) + COALESCE(nilai_setengah_lengan_lencang_kanan_kiri, 0) +
                     COALESCE(nilai_hadap_kanan_kiri, 0) + COALESCE(nilai_hadap_serong_kanan_kiri, 0) +
                     COALESCE(nilai_balik_kanan, 0) + COALESCE(nilai_langkah_bisa, 0) +
                     COALESCE(nilai_langkah_tegap, 0) + COALESCE(nilai_sikap_awal_berlari, 0) +
                     COALESCE(nilai_jalan_di_tempat, 0) + COALESCE(nilai_4_langkah_ke_depan, 0) +
                     COALESCE(nilai_4_langkah_ke_kanan, 0) + COALESCE(nilai_4_langkah_ke_kiri, 0) +
                     COALESCE(nilai_4_langkah_ke_belakang, 0)
                    ) / 25
                ) FROM jurnal_pemusatan_pelatih WHERE id_paskibraka = dc.id
            ), 0) as pelatih_pbb_avg,
            COALESCE((
                SELECT AVG(
                    (COALESCE(nilai_lipat_bendera, 0) + COALESCE(nilai_bentang_bendera, 0) +
                     COALESCE(nilai_10_tahap_penurunan, 0) + COALESCE(nilai_jadi_kibra_pembentang, 0) +
                     COALESCE(nilai_jadi_kibra_pembawa, 0) + COALESCE(nilai_jadi_kibra_pengerek, 0)
                    ) / 6
                ) FROM jurnal_pemusatan_pelatih WHERE id_paskibraka = dc.id
            ), 0) as pelatih_bendera_avg
        FROM data_capaska dc
        LEFT JOIN users u ON dc.id_pamong = u.id COLLATE utf8mb4_general_ci
        LEFT JOIN jurnal_pemusatan_pamong jp ON dc.id = jp.id_paskibraka
        LEFT JOIN jurnal_pemusatan_pelatih jpel ON dc.id = jpel.id_paskibraka
        LEFT JOIN jurnal_pemusatan_dokter jd ON dc.id = jd.id_paskibraka
        GROUP BY dc.id, dc.nama_lengkap, dc.no_peserta, u.name, dc.status
        ORDER BY dc.nama_lengkap ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let capaska_progress_response: Vec<CapaskaProgress> = capaska_progress
        .into_iter()
        .map(|row| {
            let pamong_sikap_avg = decimal_to_f64(row.pamong_sikap_avg);
            let pamong_penampilan_avg = decimal_to_f64(row.pamong_penampilan_avg);
            let pelatih_pbb_avg = decimal_to_f64(row.pelatih_pbb_avg);
            let pelatih_bendera_avg = decimal_to_f64(row.pelatih_bendera_avg);

            let mut valid_scores = Vec::new();
            if pamong_sikap_avg > 0.0 { valid_scores.push(pamong_sikap_avg); }
            if pamong_penampilan_avg > 0.0 { valid_scores.push(pamong_penampilan_avg); }
            if pelatih_pbb_avg > 0.0 { valid_scores.push(pelatih_pbb_avg); }
            if pelatih_bendera_avg > 0.0 { valid_scores.push(pelatih_bendera_avg); }

            let nilai_keseluruhan = pamong_sikap_avg + pamong_penampilan_avg + pelatih_pbb_avg + pelatih_bendera_avg;
            let nilai_rata_rata = if !valid_scores.is_empty() {
                valid_scores.iter().sum::<f64>() / valid_scores.len() as f64
            } else {
                0.0
            };

            CapaskaProgress {
                id: row.id,
                nama_lengkap: row.nama_lengkap.unwrap_or_default(),
                no_peserta: row.no_peserta.unwrap_or_default(),
                pamong_name: row.pamong_name,
                total_journal_pamong: row.total_journal_pamong,
                total_journal_pelatih: row.total_journal_pelatih,
                total_journal_dokter: row.total_journal_dokter,
                status: row.status,
                pamong_sikap_avg,
                pamong_penampilan_avg,
                pelatih_pbb_avg,
                pelatih_bendera_avg,
                nilai_keseluruhan,
                nilai_rata_rata,
            }
        })
        .collect();

    // 5. Recent Activities - tanpa description
    let recent_activities = sqlx::query!(
        r#"
        SELECT
            al.id,
            al.user_id,
            u.name as nama_user,
            u.role,
            al.action,
            al.module,
            al.status,
            al.created_at
        FROM activity_logs al
        LEFT JOIN users u ON al.user_id = u.id COLLATE utf8mb4_general_ci
        WHERE al.module = 'PEMUSATAN'
        AND (u.role IS NULL OR u.role != 'Superadmin')
        ORDER BY al.created_at DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let recent_activities_response: Vec<RecentActivity> = recent_activities
        .into_iter()
        .map(|row| {
            RecentActivity {
                id: row.id,
                user_id: row.user_id.unwrap_or_default(),
                nama_user: row.nama_user.unwrap_or_default(),
                role: row.role.unwrap_or_default(),
                action: row.action,
                module: row.module,
                status: row.status,
                created_at: row.created_at.map(|dt| dt.to_string()).unwrap_or_default(),
            }
        })
        .collect();

    // 6. Role Activity Stats
    let role_activity_stats = sqlx::query!(
        r#"
        SELECT
            u.role,
            COUNT(*) as total
        FROM activity_logs al
        LEFT JOIN users u ON al.user_id = u.id COLLATE utf8mb4_general_ci
        WHERE al.module = 'PEMUSATAN'
        AND al.created_at >= DATE_SUB(CURDATE(), INTERVAL 30 DAY)
        AND u.role != 'Superadmin'
        GROUP BY u.role
        ORDER BY total DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let role_activity_stats_response: Vec<RoleActivityStat> = role_activity_stats
        .into_iter()
        .filter(|row| row.role.is_some())
        .map(|row| {
            RoleActivityStat {
                role: row.role.unwrap_or_default(),
                total: row.total,
            }
        })
        .collect();

    let response = DashboardResponse {
        overview: overview_response,
        daily_stats: daily_stats_response,
        pamong_stats: pamong_stats_response,
        capaska_progress: capaska_progress_response,
        recent_activities: recent_activities_response,
        role_activity_stats: role_activity_stats_response,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct DashboardFilter {
    pub days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PerkembanganHarian {
    pub tanggal: String,
    pub pamong_sikap: Option<f64>,
    pub pamong_penampilan: Option<f64>,
    pub pelatih_pbb: Option<f64>,
    pub pelatih_bendera: Option<f64>,
}

#[get("/api/pemusatan/existing/{role}/{id}/{tanggal}")]
pub async fn get_existing_score(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<(String, i32, String)>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin", "Pamong", "Pelatih", "Dokter"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let (role, id_paskibraka, tanggal_str) = path.into_inner();
    let tanggal = chrono::NaiveDate::parse_from_str(&tanggal_str, "%Y-%m-%d")
        .map_err(|_| actix_web::error::ErrorBadRequest("Format tanggal tidak valid"))?;

    match role.as_str() {
        "pamong" => {
            let row = sqlx::query("SELECT * FROM jurnal_pemusatan_pamong WHERE id_paskibraka = ? AND tanggal = ? ORDER BY id DESC LIMIT 1")
                .bind(id_paskibraka)
                .bind(tanggal)
                .fetch_optional(pool.get_ref())
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;

            if let Some(r) = row {
                use sqlx::Row;
                let mut map = serde_json::Map::new();
                map.insert("id".to_string(), json!(r.get::<String, _>("id")));
                map.insert("id_paskibraka".to_string(), json!(id_paskibraka));
                map.insert("tanggal".to_string(), json!(tanggal_str));
                map.insert("catatan".to_string(), json!(r.get::<Option<String>, _>("catatan")));
                
                let cols = vec![
                    "nilai_ketaqwaan", "nilai_niat_kemauan", "nilai_keberanian", "nilai_komunikasi",
                    "nilai_keterbukaan", "nilai_ketelitian", "nilai_kesadaran", "nilai_toleransi",
                    "nilai_keikhlasan", "nilai_mempercayai", "nilai_jiwa_korsa", "nilai_kekeluargaan",
                    "nilai_persatuan_kesatuan", "nilai_ketahanan", "nilai_kekompakan_keseragaman", "nilai_ketertiban",
                    "nilai_kesopanan", "nilai_kesigapan", "nilai_kewajaran", "nilai_ketanggapan",
                    "nilai_ketenangan", "nilai_menyimak", "nilai_kebiasaan", "nilai_mengelola_stres",
                    "nilai_menghargai_waktu", "nilai_berbicara", "nilai_berjalan", "nilai_makan_minum",
                    "nilai_kehadiran", "nilai_hubungan_interpersonal", "nilai_ketaatan",
                    "nilai_istirahat_malam", "nilai_keindahan", "nilai_kerapihan", "nilai_kebersihan",
                    "nilai_berpakaian", "nilai_penampilan_rambut", "nilai_bersih_rapih_wangi"
                ];

                for c in cols {
                    map.insert(c.to_string(), json!(r.get::<Option<i32>, _>(c)));
                }

                return Ok(HttpResponse::Ok().json(serde_json::Value::Object(map)));
            }
        }
        "pelatih" => {
            let row = sqlx::query("SELECT * FROM jurnal_pemusatan_pelatih WHERE id_paskibraka = ? AND tanggal = ? ORDER BY id DESC LIMIT 1")
                .bind(id_paskibraka)
                .bind(tanggal)
                .fetch_optional(pool.get_ref())
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;

            if let Some(r) = row {
                use sqlx::Row;
                let mut map = serde_json::Map::new();
                map.insert("id".to_string(), json!(r.get::<String, _>("id")));
                map.insert("id_paskibraka".to_string(), json!(id_paskibraka));
                map.insert("tanggal".to_string(), json!(tanggal_str));
                map.insert("catatan".to_string(), json!(r.get::<Option<String>, _>("catatan")));
                
                let cols = vec![
                    "nilai_aba_aba", "nilai_berhimpun", "nilai_berkumpul", "nilai_keluar_masuk_barisan",
                    "nilai_hormat", "nilai_sikap_sempurna", "nilai_istirahat", "nilai_periksa_kerapihan",
                    "nilai_berhitung", "nilai_lepas_kenakan_topi", "nilai_bubar", "nilai_lencang_depan",
                    "nilai_lencang_kanan_kiri", "nilai_setengah_lengan_lencang_kanan_kiri", "nilai_hadap_kanan_kiri", "nilai_hadap_serong_kanan_kiri",
                    "nilai_balik_kanan", "nilai_langkah_bisa", "nilai_langkah_tegap", "nilai_sikap_awal_berlari",
                    "nilai_jalan_di_tempat", "nilai_4_langkah_ke_depan", "nilai_4_langkah_ke_kanan", "nilai_4_langkah_ke_kiri",
                    "nilai_4_langkah_ke_belakang", "nilai_lipat_bendera", "nilai_bentang_bendera", "nilai_10_tahap_penurunan",
                    "nilai_jadi_kibra_pembentang", "nilai_jadi_kibra_pembawa", "nilai_jadi_kibra_pengerek"
                ];

                for c in cols {
                    map.insert(c.to_string(), json!(r.get::<Option<i32>, _>(c)));
                }

                return Ok(HttpResponse::Ok().json(serde_json::Value::Object(map)));
            }
        }
        "dokter" => {
            let row = sqlx::query("SELECT * FROM jurnal_pemusatan_dokter WHERE id_paskibraka = ? AND tanggal = ? ORDER BY id DESC LIMIT 1")
                .bind(id_paskibraka)
                .bind(tanggal)
                .fetch_optional(pool.get_ref())
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;

            if let Some(r) = row {
                use sqlx::Row;
                let mut map = serde_json::Map::new();
                map.insert("id".to_string(), json!(r.get::<String, _>("id")));
                map.insert("id_paskibraka".to_string(), json!(id_paskibraka));
                map.insert("tanggal".to_string(), json!(tanggal_str));
                map.insert("tensi".to_string(), json!(r.get::<Option<String>, _>("tensi")));
                
                let suhu: Option<Decimal> = r.get("suhu");
                map.insert("suhu".to_string(), json!(suhu.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0))));
                
                map.insert("keluhan".to_string(), json!(r.get::<Option<String>, _>("keluhan")));
                map.insert("diagnosa".to_string(), json!(r.get::<Option<String>, _>("diagnosa")));
                map.insert("terapi_obat".to_string(), json!(r.get::<Option<String>, _>("terapi_obat")));
                map.insert("rekomendasi_istirahat".to_string(), json!(r.get::<Option<String>, _>("rekomendasi_istirahat")));

                return Ok(HttpResponse::Ok().json(serde_json::Value::Object(map)));
            }
        }
        _ => {}
    }

    Ok(HttpResponse::Ok().json(serde_json::Value::Null))
}

#[get("/api/pemusatan/admin/grafik-perkembangan/{id}")]
pub async fn get_grafik_perkembangan(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin", "Pamong", "Pelatih", "Dokter"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_paskibraka = id.into_inner();

    let pamong_rows = sqlx::query!(
        r#"
        SELECT
            tanggal,
            (
                (COALESCE(nilai_ketaqwaan, 0) + COALESCE(nilai_niat_kemauan, 0) +
                 COALESCE(nilai_keberanian, 0) + COALESCE(nilai_komunikasi, 0) +
                 COALESCE(nilai_keterbukaan, 0) + COALESCE(nilai_ketelitian, 0) +
                 COALESCE(nilai_kesadaran, 0) + COALESCE(nilai_toleransi, 0) +
                 COALESCE(nilai_keikhlasan, 0) + COALESCE(nilai_mempercayai, 0) +
                 COALESCE(nilai_jiwa_korsa, 0) + COALESCE(nilai_kekeluargaan, 0) +
                 COALESCE(nilai_persatuan_kesatuan, 0) + COALESCE(nilai_ketahanan, 0) +
                 COALESCE(nilai_kekompakan_keseragaman, 0) + COALESCE(nilai_ketertiban, 0) +
                 COALESCE(nilai_kesopanan, 0) + COALESCE(nilai_kesigapan, 0) +
                 COALESCE(nilai_kewajaran, 0) + COALESCE(nilai_ketanggapan, 0) +
                 COALESCE(nilai_ketenangan, 0) + COALESCE(nilai_menyimak, 0) +
                 COALESCE(nilai_kebiasaan, 0) + COALESCE(nilai_mengelola_stres, 0) +
                 COALESCE(nilai_menghargai_waktu, 0) + COALESCE(nilai_berbicara, 0) +
                 COALESCE(nilai_berjalan, 0) + COALESCE(nilai_makan_minum, 0) +
                 COALESCE(nilai_kehadiran, 0) + COALESCE(nilai_hubungan_interpersonal, 0) +
                 COALESCE(nilai_ketaatan, 0)
                ) / 31
            ) as avg_sikap,
            (
                (COALESCE(nilai_istirahat_malam, 0) + COALESCE(nilai_keindahan, 0) +
                 COALESCE(nilai_kerapihan, 0) + COALESCE(nilai_kebersihan, 0) +
                 COALESCE(nilai_berpakaian, 0) + COALESCE(nilai_penampilan_rambut, 0) +
                 COALESCE(nilai_bersih_rapih_wangi, 0)
                ) / 7
            ) as avg_penampilan
        FROM jurnal_pemusatan_pamong
        WHERE id_paskibraka = ?
        ORDER BY tanggal ASC
        "#,
        id_paskibraka
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let pelatih_rows = sqlx::query!(
        r#"
        SELECT
            tanggal,
            (
                (COALESCE(nilai_aba_aba, 0) + COALESCE(nilai_berhimpun, 0) +
                 COALESCE(nilai_berkumpul, 0) + COALESCE(nilai_keluar_masuk_barisan, 0) +
                 COALESCE(nilai_hormat, 0) + COALESCE(nilai_sikap_sempurna, 0) +
                 COALESCE(nilai_istirahat, 0) + COALESCE(nilai_periksa_kerapihan, 0) +
                 COALESCE(nilai_berhitung, 0) + COALESCE(nilai_lepas_kenakan_topi, 0) +
                 COALESCE(nilai_bubar, 0) + COALESCE(nilai_lencang_depan, 0) +
                 COALESCE(nilai_lencang_kanan_kiri, 0) + COALESCE(nilai_setengah_lengan_lencang_kanan_kiri, 0) +
                 COALESCE(nilai_hadap_kanan_kiri, 0) + COALESCE(nilai_hadap_serong_kanan_kiri, 0) +
                 COALESCE(nilai_balik_kanan, 0) + COALESCE(nilai_langkah_bisa, 0) +
                 COALESCE(nilai_langkah_tegap, 0) + COALESCE(nilai_sikap_awal_berlari, 0) +
                 COALESCE(nilai_jalan_di_tempat, 0) + COALESCE(nilai_4_langkah_ke_depan, 0) +
                 COALESCE(nilai_4_langkah_ke_kanan, 0) + COALESCE(nilai_4_langkah_ke_kiri, 0) +
                 COALESCE(nilai_4_langkah_ke_belakang, 0)
                ) / 25
            ) as avg_pbb,
            (
                (COALESCE(nilai_lipat_bendera, 0) + COALESCE(nilai_bentang_bendera, 0) +
                 COALESCE(nilai_10_tahap_penurunan, 0) + COALESCE(nilai_jadi_kibra_pembentang, 0) +
                 COALESCE(nilai_jadi_kibra_pembawa, 0) + COALESCE(nilai_jadi_kibra_pengerek, 0)
                ) / 6
            ) as avg_bendera
        FROM jurnal_pemusatan_pelatih
        WHERE id_paskibraka = ?
        ORDER BY tanggal ASC
        "#,
        id_paskibraka
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    use std::collections::BTreeMap;
    let mut combined: BTreeMap<String, PerkembanganHarian> = BTreeMap::new();

    for r in pamong_rows {
        let t_str = r.tanggal.to_string();
        combined.insert(
            t_str.clone(),
            PerkembanganHarian {
                tanggal: t_str,
                pamong_sikap: Some(option_decimal_to_f64(r.avg_sikap)),
                pamong_penampilan: Some(option_decimal_to_f64(r.avg_penampilan)),
                pelatih_pbb: None,
                pelatih_bendera: None,
            },
        );
    }

    for r in pelatih_rows {
        let t_str = r.tanggal.to_string();
        if let Some(entry) = combined.get_mut(&t_str) {
            entry.pelatih_pbb = Some(option_decimal_to_f64(r.avg_pbb));
            entry.pelatih_bendera = Some(option_decimal_to_f64(r.avg_bendera));
        } else {
            combined.insert(
                t_str.clone(),
                PerkembanganHarian {
                    tanggal: t_str,
                    pamong_sikap: None,
                    pamong_penampilan: None,
                    pelatih_pbb: Some(option_decimal_to_f64(r.avg_pbb)),
                    pelatih_bendera: Some(option_decimal_to_f64(r.avg_bendera)),
                },
            );
        }
    }

    let result: Vec<PerkembanganHarian> = combined.into_values().collect();
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Debug, Serialize)]
pub struct ExportPerTanggal {
    pub no_peserta: Option<String>,
    pub nama_lengkap: Option<String>,
    pub provinsi: Option<String>,
    pub tanggal: String,
    pub pamong_sikap: Option<f64>,
    pub pamong_penampilan: Option<f64>,
    pub pelatih_pbb: Option<f64>,
    pub pelatih_bendera: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ExportRataRata {
    pub no_peserta: Option<String>,
    pub nama_lengkap: Option<String>,
    pub provinsi: Option<String>,
    pub pamong_sikap_avg: f64,
    pub pamong_penampilan_avg: f64,
    pub pelatih_pbb_avg: f64,
    pub pelatih_bendera_avg: f64,
}

#[get("/api/pemusatan/admin/export/pertanggal")]
pub async fn export_pertanggal(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            dc.no_peserta,
            dc.nama_lengkap,
            dc.provinsi,
            t.tanggal,
            (
                SELECT (
                    COALESCE(nilai_ketaqwaan, 0) + COALESCE(nilai_niat_kemauan, 0) +
                    COALESCE(nilai_keberanian, 0) + COALESCE(nilai_komunikasi, 0) +
                    COALESCE(nilai_keterbukaan, 0) + COALESCE(nilai_ketelitian, 0) +
                    COALESCE(nilai_kesadaran, 0) + COALESCE(nilai_toleransi, 0) +
                    COALESCE(nilai_keikhlasan, 0) + COALESCE(nilai_mempercayai, 0) +
                    COALESCE(nilai_jiwa_korsa, 0) + COALESCE(nilai_kekeluargaan, 0) +
                    COALESCE(nilai_persatuan_kesatuan, 0) + COALESCE(nilai_ketahanan, 0) +
                    COALESCE(nilai_kekompakan_keseragaman, 0) + COALESCE(nilai_ketertiban, 0) +
                    COALESCE(nilai_kesopanan, 0) + COALESCE(nilai_kesigapan, 0) +
                    COALESCE(nilai_kewajaran, 0) + COALESCE(nilai_ketanggapan, 0) +
                    COALESCE(nilai_ketenangan, 0) + COALESCE(nilai_menyimak, 0) +
                    COALESCE(nilai_kebiasaan, 0) + COALESCE(nilai_mengelola_stres, 0) +
                    COALESCE(nilai_menghargai_waktu, 0) + COALESCE(nilai_berbicara, 0) +
                    COALESCE(nilai_berjalan, 0) + COALESCE(nilai_makan_minum, 0) +
                    COALESCE(nilai_kehadiran, 0) + COALESCE(nilai_hubungan_interpersonal, 0) +
                    COALESCE(nilai_ketaatan, 0)
                ) / 31 FROM jurnal_pemusatan_pamong WHERE id_paskibraka = dc.id AND tanggal = t.tanggal
            ) as pamong_sikap,
            (
                SELECT (
                    COALESCE(nilai_istirahat_malam, 0) + COALESCE(nilai_keindahan, 0) +
                    COALESCE(nilai_kerapihan, 0) + COALESCE(nilai_kebersihan, 0) +
                    COALESCE(nilai_berpakaian, 0) + COALESCE(nilai_penampilan_rambut, 0) +
                    COALESCE(nilai_bersih_rapih_wangi, 0)
                ) / 7 FROM jurnal_pemusatan_pamong WHERE id_paskibraka = dc.id AND tanggal = t.tanggal
            ) as pamong_penampilan,
            (
                SELECT (
                    COALESCE(nilai_aba_aba, 0) + COALESCE(nilai_berhimpun, 0) +
                    COALESCE(nilai_berkumpul, 0) + COALESCE(nilai_keluar_masuk_barisan, 0) +
                    COALESCE(nilai_hormat, 0) + COALESCE(nilai_sikap_sempurna, 0) +
                    COALESCE(nilai_istirahat, 0) + COALESCE(nilai_periksa_kerapihan, 0) +
                    COALESCE(nilai_berhitung, 0) + COALESCE(nilai_lepas_kenakan_topi, 0) +
                    COALESCE(nilai_bubar, 0) + COALESCE(nilai_lencang_depan, 0) +
                    COALESCE(nilai_lencang_kanan_kiri, 0) + COALESCE(nilai_setengah_lengan_lencang_kanan_kiri, 0) +
                    COALESCE(nilai_hadap_kanan_kiri, 0) + COALESCE(nilai_hadap_serong_kanan_kiri, 0) +
                    COALESCE(nilai_balik_kanan, 0) + COALESCE(nilai_langkah_bisa, 0) +
                    COALESCE(nilai_langkah_tegap, 0) + COALESCE(nilai_sikap_awal_berlari, 0) +
                    COALESCE(nilai_jalan_di_tempat, 0) + COALESCE(nilai_4_langkah_ke_depan, 0) +
                    COALESCE(nilai_4_langkah_ke_kanan, 0) + COALESCE(nilai_4_langkah_ke_kiri, 0) +
                    COALESCE(nilai_4_langkah_ke_belakang, 0)
                ) / 25 FROM jurnal_pemusatan_pelatih WHERE id_paskibraka = dc.id AND tanggal = t.tanggal
            ) as pelatih_pbb,
            (
                SELECT (
                    COALESCE(nilai_lipat_bendera, 0) + COALESCE(nilai_bentang_bendera, 0) +
                    COALESCE(nilai_10_tahap_penurunan, 0) + COALESCE(nilai_jadi_kibra_pembentang, 0) +
                    COALESCE(nilai_jadi_kibra_pembawa, 0) + COALESCE(nilai_jadi_kibra_pengerek, 0)
                ) / 6 FROM jurnal_pemusatan_pelatih WHERE id_paskibraka = dc.id AND tanggal = t.tanggal
            ) as pelatih_bendera
        FROM (
            SELECT id_paskibraka, tanggal FROM jurnal_pemusatan_pamong
            UNION
            SELECT id_paskibraka, tanggal FROM jurnal_pemusatan_pelatih
        ) t
        JOIN data_capaska dc ON t.id_paskibraka = dc.id
        ORDER BY t.tanggal DESC, dc.nama_lengkap ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<ExportPerTanggal> = rows
        .into_iter()
        .map(|row| ExportPerTanggal {
            no_peserta: row.no_peserta,
            nama_lengkap: row.nama_lengkap,
            provinsi: row.provinsi,
            tanggal: row.tanggal.to_string(),
            pamong_sikap: row.pamong_sikap.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)),
            pamong_penampilan: row.pamong_penampilan.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)),
            pelatih_pbb: row.pelatih_pbb.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)),
            pelatih_bendera: row.pelatih_bendera.map(|v| v.to_string().parse::<f64>().unwrap_or(0.0)),
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/pemusatan/admin/export/ratarata")]
pub async fn export_ratarata(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            dc.no_peserta,
            dc.nama_lengkap,
            dc.provinsi,
            COALESCE(AVG(
                (COALESCE(jp.nilai_ketaqwaan, 0) + COALESCE(jp.nilai_niat_kemauan, 0) +
                 COALESCE(jp.nilai_keberanian, 0) + COALESCE(jp.nilai_komunikasi, 0) +
                 COALESCE(jp.nilai_keterbukaan, 0) + COALESCE(jp.nilai_ketelitian, 0) +
                 COALESCE(jp.nilai_kesadaran, 0) + COALESCE(jp.nilai_toleransi, 0) +
                 COALESCE(jp.nilai_keikhlasan, 0) + COALESCE(jp.nilai_mempercayai, 0) +
                 COALESCE(jp.nilai_jiwa_korsa, 0) + COALESCE(jp.nilai_kekeluargaan, 0) +
                 COALESCE(jp.nilai_persatuan_kesatuan, 0) + COALESCE(jp.nilai_ketahanan, 0) +
                 COALESCE(jp.nilai_kekompakan_keseragaman, 0) + COALESCE(jp.nilai_ketertiban, 0) +
                 COALESCE(jp.nilai_kesopanan, 0) + COALESCE(jp.nilai_kesigapan, 0) +
                 COALESCE(jp.nilai_kewajaran, 0) + COALESCE(jp.nilai_ketanggapan, 0) +
                 COALESCE(jp.nilai_ketenangan, 0) + COALESCE(jp.nilai_menyimak, 0) +
                 COALESCE(jp.nilai_kebiasaan, 0) + COALESCE(jp.nilai_mengelola_stres, 0) +
                 COALESCE(jp.nilai_menghargai_waktu, 0) + COALESCE(jp.nilai_berbicara, 0) +
                 COALESCE(jp.nilai_berjalan, 0) + COALESCE(jp.nilai_makan_minum, 0) +
                 COALESCE(jp.nilai_kehadiran, 0) + COALESCE(jp.nilai_hubungan_interpersonal, 0) +
                 COALESCE(jp.nilai_ketaatan, 0)
                ) / 31
            ), 0) as pamong_sikap_avg,
            COALESCE(AVG(
                (COALESCE(jp.nilai_istirahat_malam, 0) + COALESCE(jp.nilai_keindahan, 0) +
                 COALESCE(jp.nilai_kerapihan, 0) + COALESCE(jp.nilai_kebersihan, 0) +
                 COALESCE(jp.nilai_berpakaian, 0) + COALESCE(jp.nilai_penampilan_rambut, 0) +
                 COALESCE(jp.nilai_bersih_rapih_wangi, 0)
                ) / 7
            ), 0) as pamong_penampilan_avg,
            COALESCE((
                SELECT AVG(
                    (COALESCE(nilai_aba_aba, 0) + COALESCE(nilai_berhimpun, 0) +
                     COALESCE(nilai_berkumpul, 0) + COALESCE(nilai_keluar_masuk_barisan, 0) +
                     COALESCE(nilai_hormat, 0) + COALESCE(nilai_sikap_sempurna, 0) +
                     COALESCE(nilai_istirahat, 0) + COALESCE(nilai_periksa_kerapihan, 0) +
                     COALESCE(nilai_berhitung, 0) + COALESCE(nilai_lepas_kenakan_topi, 0) +
                     COALESCE(nilai_bubar, 0) + COALESCE(nilai_lencang_depan, 0) +
                     COALESCE(nilai_lencang_kanan_kiri, 0) + COALESCE(nilai_setengah_lengan_lencang_kanan_kiri, 0) +
                     COALESCE(nilai_hadap_kanan_kiri, 0) + COALESCE(nilai_hadap_serong_kanan_kiri, 0) +
                     COALESCE(nilai_balik_kanan, 0) + COALESCE(nilai_langkah_bisa, 0) +
                     COALESCE(nilai_langkah_tegap, 0) + COALESCE(nilai_sikap_awal_berlari, 0) +
                     COALESCE(nilai_jalan_di_tempat, 0) + COALESCE(nilai_4_langkah_ke_depan, 0) +
                     COALESCE(nilai_4_langkah_ke_kanan, 0) + COALESCE(nilai_4_langkah_ke_kiri, 0) +
                     COALESCE(nilai_4_langkah_ke_belakang, 0)
                    ) / 25
                ) FROM jurnal_pemusatan_pelatih WHERE id_paskibraka = dc.id
            ), 0) as pelatih_pbb_avg,
            COALESCE((
                SELECT AVG(
                    (COALESCE(nilai_lipat_bendera, 0) + COALESCE(nilai_bentang_bendera, 0) +
                     COALESCE(nilai_10_tahap_penurunan, 0) + COALESCE(nilai_jadi_kibra_pembentang, 0) +
                     COALESCE(nilai_jadi_kibra_pembawa, 0) + COALESCE(nilai_jadi_kibra_pengerek, 0)
                    ) / 6
                ) FROM jurnal_pemusatan_pelatih WHERE id_paskibraka = dc.id
            ), 0) as pelatih_bendera_avg
        FROM data_capaska dc
        LEFT JOIN jurnal_pemusatan_pamong jp ON dc.id = jp.id_paskibraka
        GROUP BY dc.id, dc.no_peserta, dc.nama_lengkap, dc.provinsi
        ORDER BY dc.nama_lengkap ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<ExportRataRata> = rows
        .into_iter()
        .map(|row| ExportRataRata {
            no_peserta: row.no_peserta,
            nama_lengkap: row.nama_lengkap,
            provinsi: row.provinsi,
            pamong_sikap_avg: decimal_to_f64(row.pamong_sikap_avg),
            pamong_penampilan_avg: decimal_to_f64(row.pamong_penampilan_avg),
            pelatih_pbb_avg: decimal_to_f64(row.pelatih_pbb_avg),
            pelatih_bendera_avg: decimal_to_f64(row.pelatih_bendera_avg),
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}
