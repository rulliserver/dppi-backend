use crate::auth;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, get, post, web};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, MySqlPool};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PamongInput {
    pub id_paskibraka: i32,
    pub tanggal: String,
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
    pub nama_lengkap: String,
    pub jk: String,
    pub id_pamong: Option<String>,
}

// 1. Fetch roster of candidates
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

    let mut sql =
        String::from("SELECT id, nama_lengkap, jk, id_pamong FROM data_capaska WHERE 1=1");
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
            nilai_ketaqwaan = VALUES(nilai_ketaqwaan),
            nilai_niat_kemauan = VALUES(nilai_niat_kemauan),
            nilai_keberanian = VALUES(nilai_keberanian),
            nilai_komunikasi = VALUES(nilai_komunikasi),
            nilai_keterbukaan = VALUES(nilai_keterbukaan),
            nilai_ketelitian = VALUES(nilai_ketelitian),
            nilai_kesadaran = VALUES(nilai_kesadaran),
            nilai_toleransi = VALUES(nilai_toleransi),
            nilai_keikhlasan = VALUES(nilai_keikhlasan),
            nilai_mempercayai = VALUES(nilai_mempercayai),
            nilai_jiwa_korsa = VALUES(nilai_jiwa_korsa),
            nilai_kekeluargaan = VALUES(nilai_kekeluargaan),
            nilai_persatuan_kesatuan = VALUES(nilai_persatuan_kesatuan),
            nilai_ketahanan = VALUES(nilai_ketahanan),
            nilai_kekompakan_keseragaman = VALUES(nilai_kekompakan_keseragaman),
            nilai_ketertiban = VALUES(nilai_ketertiban),
            nilai_kesopanan = VALUES(nilai_kesopanan),
            nilai_kesigapan = VALUES(nilai_kesigapan),
            nilai_kewajaran = VALUES(nilai_kewajaran),
            nilai_ketanggapan = VALUES(nilai_ketanggapan),
            nilai_ketenangan = VALUES(nilai_ketenangan),
            nilai_menyimak = VALUES(nilai_menyimak),
            nilai_kebiasaan = VALUES(nilai_kebiasaan),
            nilai_mengelola_stres = VALUES(nilai_mengelola_stres),
            nilai_menghargai_waktu = VALUES(nilai_menghargai_waktu),
            nilai_berbicara = VALUES(nilai_berbicara),
            nilai_berjalan = VALUES(nilai_berjalan),
            nilai_makan_minum = VALUES(nilai_makan_minum),
            nilai_kehadiran = VALUES(nilai_kehadiran),
            nilai_hubungan_interpersonal = VALUES(nilai_hubungan_interpersonal),
            nilai_ketaatan = VALUES(nilai_ketaatan),
            nilai_istirahat_malam = VALUES(nilai_istirahat_malam),
            nilai_keindahan = VALUES(nilai_keindahan),
            nilai_kerapihan = VALUES(nilai_kerapihan),
            nilai_kebersihan = VALUES(nilai_kebersihan),
            nilai_berpakaian = VALUES(nilai_berpakaian),
            nilai_penampilan_rambut = VALUES(nilai_penampilan_rambut),
            nilai_bersih_rapih_wangi = VALUES(nilai_bersih_rapih_wangi),
            catatan = VALUES(catatan)
    "#;

    sqlx::query(query)
        .bind(&id)
        .bind(payload.id_paskibraka)
        .bind(&claims.user_id)
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
            id, id_paskibraka, id_pamong, tanggal,
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
            id_pamong = VALUES(id_pamong),
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
        INSERT INTO jurnal_pemusatan_dokter (id, id_paskibraka, id_pamong, tanggal, tensi, suhu, keluhan, diagnosa, terapi_obat, rekomendasi_istirahat)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            id_pamong = VALUES(id_pamong),
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

    let allowed_roles = ["Admin Pemusatan", "Superadmin"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_paskibraka = path.into_inner();

    // 1. Fetch Candidate Profile details
    let profile_sql = "SELECT id, nama_lengkap, photo, jk, nama_instansi_pendidikan, nomor_dada, status FROM data_paskibraka WHERE id = ? LIMIT 1";
    let profile: Option<CandidateSummary> = sqlx::query_as(profile_sql)
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

    // 2. Fetch Selection scores
    // a. Wawancara
    let w_sql =
        "SELECT nilai1, nilai2, nilai3, nilai4, status FROM wawancara WHERE id_capaska = ? LIMIT 1";
    let wawancara: serde_json::Value = sqlx::query(w_sql)
        .bind(id_paskibraka)
        .fetch_optional(pool.get_ref())
        .await
        .map(|opt| {
            opt.map(|row| {
                use sqlx::Row;
                let n1: Option<rust_decimal::Decimal> = row.try_get("nilai1").ok();
                let n2: Option<rust_decimal::Decimal> = row.try_get("nilai2").ok();
                let n3: Option<rust_decimal::Decimal> = row.try_get("nilai3").ok();
                let n4: Option<rust_decimal::Decimal> = row.try_get("nilai4").ok();
                let status: Option<String> = row.try_get("status").ok();
                json!({
                    "nilai_pancasila_kebangsaan": n1.map(|d| d.to_string()),
                    "nilai_intelegensia_umum": n2.map(|d| d.to_string()),
                    "nilai_minat_bakat": n3.map(|d| d.to_string()),
                    "nilai_penampilan": n4.map(|d| d.to_string()),
                    "status": status
                })
            })
            .unwrap_or(serde_json::Value::Null)
        })
        .unwrap_or(serde_json::Value::Null);

    // b. Psikotes
    let psi_sql = "SELECT iq, iq_kategori, sub1, sub2, sub3 FROM psikotes WHERE nomor_tes = ? OR nama_asesi = ? LIMIT 1";
    let psikotes: serde_json::Value = sqlx::query(psi_sql)
        .bind(id_paskibraka.to_string())
        .bind(&candidate.nama_lengkap)
        .fetch_optional(pool.get_ref())
        .await
        .map(|opt| {
            opt.map(|row| {
                use sqlx::Row;
                let iq: Option<i32> = row.try_get("iq").ok();
                let iq_kat: Option<String> = row.try_get("iq_kategori").ok();
                json!({ "iq": iq, "kategori": iq_kat })
            })
            .unwrap_or(serde_json::Value::Null)
        })
        .unwrap_or(serde_json::Value::Null);

    // c. Kesehatan (pemeriksaan_kesehatan)
    let kes_sql = "SELECT jenis_pemeriksaan, score_mata, score_gigi, score_tht FROM pemeriksaan_kesehatan WHERE id_capaska = ? LIMIT 1";
    let kesehatan: serde_json::Value = sqlx::query(kes_sql)
        .bind(id_paskibraka)
        .fetch_optional(pool.get_ref())
        .await
        .map(|opt| {
            opt.map(|row| {
                use sqlx::Row;
                let sm: Option<i32> = row.try_get("score_mata").ok();
                let sg: Option<i32> = row.try_get("score_gigi").ok();
                let st: Option<i32> = row.try_get("score_tht").ok();
                json!({ "score_mata": sm, "score_gigi": sg, "score_tht": st })
            })
            .unwrap_or(serde_json::Value::Null)
        })
        .unwrap_or(serde_json::Value::Null);

    // d. PBB (Average of judges)
    let pbb_sql = r#"
        SELECT
            AVG(nilai_sikap_sempurna) as ss,
            AVG(nilai_hormat) as h,
            AVG(nilai_jalan_ditempat) as jd,
            AVG(nilai_sikap_istirahat) as i,
            AVG(nilai_langkah_tegap) as lt
        FROM pbb2026
        WHERE id_capaska = ?
    "#;
    let pbb: serde_json::Value = sqlx::query(pbb_sql)
        .bind(id_paskibraka)
        .fetch_one(pool.get_ref())
        .await
        .map(|row| {
            use sqlx::Row;
            let ss: Option<rust_decimal::Decimal> = row.try_get("ss").ok();
            let h: Option<rust_decimal::Decimal> = row.try_get("h").ok();
            let jd: Option<rust_decimal::Decimal> = row.try_get("jd").ok();
            let i: Option<rust_decimal::Decimal> = row.try_get("i").ok();
            let lt: Option<rust_decimal::Decimal> = row.try_get("lt").ok();
            json!({
                "sikap_sempurna": ss.map(|d| d.to_string()),
                "hormat": h.map(|d| d.to_string()),
                "jalan_ditempat": jd.map(|d| d.to_string()),
                "istirahat": i.map(|d| d.to_string()),
                "langkah_tegap": lt.map(|d| d.to_string())
            })
        })
        .unwrap_or(serde_json::Value::Null);

    // 3. Fetch Daily logs
    // a. Pamong logs
    let pamong_logs_sql = r#"
        SELECT
            tanggal, nilai_ketaqwaan, nilai_niat_kemauan, nilai_keberanian, nilai_komunikasi,
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
                    "nilai_ketaqwaan": row.get::<i32, _>("nilai_ketaqwaan"),
                    "nilai_niat_kemauan": row.get::<i32, _>("nilai_niat_kemauan"),
                    "nilai_keberanian": row.get::<i32, _>("nilai_keberanian"),
                    "nilai_komunikasi": row.get::<i32, _>("nilai_komunikasi"),
                    "nilai_keterbukaan": row.get::<i32, _>("nilai_keterbukaan"),
                    "nilai_ketelitian": row.get::<i32, _>("nilai_ketelitian"),
                    "nilai_kesadaran": row.get::<i32, _>("nilai_kesadaran"),
                    "nilai_toleransi": row.get::<i32, _>("nilai_toleransi"),
                    "nilai_keikhlasan": row.get::<i32, _>("nilai_keikhlasan"),
                    "nilai_mempercayai": row.get::<i32, _>("nilai_mempercayai"),
                    "nilai_jiwa_korsa": row.get::<i32, _>("nilai_jiwa_korsa"),
                    "nilai_kekeluargaan": row.get::<i32, _>("nilai_kekeluargaan"),
                    "nilai_persatuan_kesatuan": row.get::<i32, _>("nilai_persatuan_kesatuan"),
                    "nilai_ketahanan": row.get::<i32, _>("nilai_ketahanan"),
                    "nilai_kekompakan_keseragaman": row.get::<i32, _>("nilai_kekompakan_keseragaman"),
                    "nilai_ketertiban": row.get::<i32, _>("nilai_ketertiban"),
                    "nilai_kesopanan": row.get::<i32, _>("nilai_kesopanan"),
                    "nilai_kesigapan": row.get::<i32, _>("nilai_kesigapan"),
                    "nilai_kewajaran": row.get::<i32, _>("nilai_kewajaran"),
                    "nilai_ketanggapan": row.get::<i32, _>("nilai_ketanggapan"),
                    "nilai_ketenangan": row.get::<i32, _>("nilai_ketenangan"),
                    "nilai_menyimak": row.get::<i32, _>("nilai_menyimak"),
                    "nilai_kebiasaan": row.get::<i32, _>("nilai_kebiasaan"),
                    "nilai_mengelola_stres": row.get::<i32, _>("nilai_mengelola_stres"),
                    "nilai_menghargai_waktu": row.get::<i32, _>("nilai_menghargai_waktu"),
                    "nilai_berbicara": row.get::<i32, _>("nilai_berbicara"),
                    "nilai_berjalan": row.get::<i32, _>("nilai_berjalan"),
                    "nilai_makan_minum": row.get::<i32, _>("nilai_makan_minum"),
                    "nilai_kehadiran": row.get::<i32, _>("nilai_kehadiran"),
                    "nilai_hubungan_interpersonal": row.get::<i32, _>("nilai_hubungan_interpersonal"),
                    "nilai_ketaatan": row.get::<i32, _>("nilai_ketaatan"),
                    "nilai_istirahat_malam": row.get::<i32, _>("nilai_istirahat_malam"),
                    "nilai_keindahan": row.get::<i32, _>("nilai_keindahan"),
                    "nilai_kerapihan": row.get::<i32, _>("nilai_kerapihan"),
                    "nilai_kebersihan": row.get::<i32, _>("nilai_kebersihan"),
                    "nilai_berpakaian": row.get::<i32, _>("nilai_berpakaian"),
                    "nilai_penampilan_rambut": row.get::<i32, _>("nilai_penampilan_rambut"),
                    "nilai_bersih_rapih_wangi": row.get::<i32, _>("nilai_bersih_rapih_wangi"),
                    "catatan": catatan
                })
            }).collect()
        })
        .unwrap_or_else(|_| Vec::new());

    // b. Pelatih logs
    let pelatih_logs_sql = r#"
        SELECT
            tanggal, nilai_aba_aba, nilai_berhimpun, nilai_berkumpul, nilai_keluar_masuk_barisan,
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
                    "nilai_aba_aba": row.get::<i32, _>("nilai_aba_aba"),
                    "nilai_berhimpun": row.get::<i32, _>("nilai_berhimpun"),
                    "nilai_berkumpul": row.get::<i32, _>("nilai_berkumpul"),
                    "nilai_keluar_masuk_barisan": row.get::<i32, _>("nilai_keluar_masuk_barisan"),
                    "nilai_hormat": row.get::<i32, _>("nilai_hormat"),
                    "nilai_sikap_sempurna": row.get::<i32, _>("nilai_sikap_sempurna"),
                    "nilai_istirahat": row.get::<i32, _>("nilai_istirahat"),
                    "nilai_periksa_kerapihan": row.get::<i32, _>("nilai_periksa_kerapihan"),
                    "nilai_berhitung": row.get::<i32, _>("nilai_berhitung"),
                    "nilai_lepas_kenakan_topi": row.get::<i32, _>("nilai_lepas_kenakan_topi"),
                    "nilai_bubar": row.get::<i32, _>("nilai_bubar"),
                    "nilai_lencang_depan": row.get::<i32, _>("nilai_lencang_depan"),
                    "nilai_lencang_kanan_kiri": row.get::<i32, _>("nilai_lencang_kanan_kiri"),
                    "nilai_setengah_lengan_lencang_kanan_kiri": row.get::<i32, _>("nilai_setengah_lengan_lencang_kanan_kiri"),
                    "nilai_hadap_kanan_kiri": row.get::<i32, _>("nilai_hadap_kanan_kiri"),
                    "nilai_hadap_serong_kanan_kiri": row.get::<i32, _>("nilai_hadap_serong_kanan_kiri"),
                    "nilai_balik_kanan": row.get::<i32, _>("nilai_balik_kanan"),
                    "nilai_langkah_bisa": row.get::<i32, _>("nilai_langkah_bisa"),
                    "nilai_langkah_tegap": row.get::<i32, _>("nilai_langkah_tegap"),
                    "nilai_sikap_awal_berlari": row.get::<i32, _>("nilai_sikap_awal_berlari"),
                    "nilai_jalan_di_tempat": row.get::<i32, _>("nilai_jalan_di_tempat"),
                    "nilai_4_langkah_ke_depan": row.get::<i32, _>("nilai_4_langkah_ke_depan"),
                    "nilai_4_langkah_ke_kanan": row.get::<i32, _>("nilai_4_langkah_ke_kanan"),
                    "nilai_4_langkah_ke_kiri": row.get::<i32, _>("nilai_4_langkah_ke_kiri"),
                    "nilai_4_langkah_ke_belakang": row.get::<i32, _>("nilai_4_langkah_ke_belakang"),
                    "nilai_lipat_bendera": row.get::<i32, _>("nilai_lipat_bendera"),
                    "nilai_bentang_bendera": row.get::<i32, _>("nilai_bentang_bendera"),
                    "nilai_10_tahap_penurunan": row.get::<i32, _>("nilai_10_tahap_penurunan"),
                    "nilai_jadi_kibra_pembentang": row.get::<i32, _>("nilai_jadi_kibra_pembentang"),
                    "nilai_jadi_kibra_pembawa": row.get::<i32, _>("nilai_jadi_kibra_pembawa"),
                    "nilai_jadi_kibra_pengerek": row.get::<i32, _>("nilai_jadi_kibra_pengerek"),
                    "catatan": catatan
                })
            }).collect()
        })
        .unwrap_or_else(|_| Vec::new());

    // c. Dokter logs
    let dokter_logs_sql = "SELECT tanggal, tensi, suhu, keluhan, diagnosa, terapi_obat, rekomendasi_istirahat FROM jurnal_pemusatan_dokter WHERE id_paskibraka = ? ORDER BY tanggal ASC";
    let dokter_logs: Vec<serde_json::Value> = sqlx::query(dokter_logs_sql)
        .bind(id_paskibraka)
        .fetch_all(pool.get_ref())
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| {
                    use sqlx::Row;
                    let t: chrono::NaiveDate = row.get("tanggal");
                    let tensi: String = row.get("tensi");
                    let suhu: rust_decimal::Decimal = row.get("suhu");
                    let keluhan: Option<String> = row.get("keluhan");
                    let diagnosa: Option<String> = row.get("diagnosa");
                    let obat: Option<String> = row.get("terapi_obat");
                    let rek: String = row.get("rekomendasi_istirahat");
                    json!({
                        "tanggal": t.to_string(),
                        "tensi": tensi,
                        "suhu": suhu.to_string(),
                        "keluhan": keluhan,
                        "diagnosa": diagnosa,
                        "terapi_obat": obat,
                        "rekomendasi_istirahat": rek
                    })
                })
                .collect()
        })
        .unwrap_or_else(|_| Vec::new());

    Ok(HttpResponse::Ok().json(json!({
        "profile": candidate,
        "seleksi": {
            "wawancara": wawancara,
            "psikotes": psikotes,
            "kesehatan": kesehatan,
            "pbb": pbb
        },
        "pemusatan": {
            "pamong": pamong_logs,
            "pelatih": pelatih_logs,
            "dokter": dokter_logs
        }
    })))
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

                    PamongCandidateStats {
                        id,
                        nama_lengkap,
                        jk,
                        total_sikap,
                        total_penampilan,
                        total_penilaian: total_entries,
                        rata_rata_sikap,
                        rata_rata_penampilan,
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
