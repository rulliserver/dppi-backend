use crate::auth;
use actix_web::{Error, HttpRequest, HttpResponse, Responder, get, post, web};
use serde::Deserialize;
use serde_json::json;
use sqlx::{MySqlPool, Row};

// Conversion helper from alphanumeric user ID to deterministic integer
fn user_id_to_int(user_id: &str) -> i32 {
    let clean_id = user_id.trim();
    if let Ok(val) = i32::from_str_radix(&clean_id[..std::cmp::min(7, clean_id.len())], 16) {
        val
    } else {
        let mut hash = 0;
        for c in clean_id.chars() {
            hash = (hash * 31 + c as i32) % 1_000_000;
        }
        hash
    }
}

// Structs for Input payloads
#[derive(Debug, Deserialize)]
pub struct PbbInput {
    pub id_capaska: i32,
    pub id_provinsi: i32,
    pub nilai_sikap_sempurna: i32,
    pub nilai_hormat: i32,
    pub nilai_jalan_ditempat: i32,
    pub nilai_sikap_istirahat: i32,
    pub nilai_langkah_biasa: i32,
    pub nilai_langkah_tegap: i32,
    pub nilai_meluruskan_barisan: i32,
    pub nilai_melangkah: i32,
    pub nilai_hadap_kanan_kiri: i32,
    pub nilai_serong_kanan_kiri: i32,
    pub nilai_suara_komando: i32,
    pub status: String, // "Draft", "Rated"
}

#[derive(Debug, Deserialize)]
pub struct WawancaraInput {
    pub id_capaska: i32,
    pub id_provinsi: i32,
    pub nilai1: f64, // Pancasila
    pub nilai2: f64, // Intelegensia
    pub nilai3: f64, // Minat Bakat
    pub nilai4: f64, // Penampilan
    pub status: String, // "Selesai"
}

#[derive(Debug, Deserialize)]
pub struct KesehatanInput {
    pub id_capaska: i32,
    pub score_mata: i32,
    pub score_gigi: i32,
    pub score_tht: i32,
}

#[derive(Debug, Deserialize)]
pub struct PsikotesInput {
    pub id_capaska: i32, // Matches data_paskibraka.id
    pub iq: i32,
    pub iq_kategori: String,
}

// Candidates Roster (with summaries of selection statuses)
#[get("/api/seleksi/candidates")]
pub async fn get_candidates(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Jurnalis", "Juri PBB", "Juri Minat Bakat", "Pewawancara", "Dokter Penilai"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let sql = r#"
        SELECT 
            c.id, 
            c.nama_lengkap, 
            c.nomor_dada, 
            c.jk, 
            c.nama_instansi_pendidikan,
            (SELECT COUNT(*) FROM pbb2026 WHERE id_capaska = c.id) as has_pbb,
            (SELECT COUNT(*) FROM wawancara WHERE id_capaska = c.id) as has_wawancara,
            (SELECT COUNT(*) FROM pemeriksaan_kesehatan WHERE id_capaska = c.id) as has_kesehatan,
            (SELECT COUNT(*) FROM psikotes WHERE nomor_tes = c.nomor_dada OR nama_asesi = c.nama_lengkap) as has_psikotes
        FROM data_paskibraka c
        ORDER BY c.nama_lengkap ASC
    "#;

    let rows = sqlx::query(sql)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let list: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        let id: i32 = row.get("id");
        let nama: String = row.get("nama_lengkap");
        let nomor_dada: Option<String> = row.get("nomor_dada");
        let jk: String = row.get("jk");
        let sekolah: Option<String> = row.get("nama_instansi_pendidikan");
        let has_pbb: i64 = row.get("has_pbb");
        let has_wawancara: i64 = row.get("has_wawancara");
        let has_kesehatans: i64 = row.get("has_kesehatan");
        let has_psikotes: i64 = row.get("has_psikotes");

        json!({
            "id": id,
            "nama_lengkap": nama,
            "nomor_dada": nomor_dada,
            "jk": jk,
            "nama_instansi_pendidikan": sekolah,
            "status_pbb": if has_pbb > 0 { "Sudah Dinilai" } else { "Belum" },
            "status_wawancara": if has_wawancara > 0 { "Sudah Dinilai" } else { "Belum" },
            "status_kesehatan": if has_kesehatans > 0 { "Sudah Dinilai" } else { "Belum" },
            "status_psikotes": if has_psikotes > 0 { "Sudah Dinilai" } else { "Belum" },
        })
    }).collect();

    Ok(HttpResponse::Ok().json(list))
}

// PBB Endpoints
#[get("/api/seleksi/pbb/{id_capaska}")]
pub async fn get_pbb(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Juri PBB"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_capaska = path.into_inner();
    let sql = "SELECT * FROM pbb2026 WHERE id_capaska = ? LIMIT 1";
    let row_opt = sqlx::query(sql)
        .bind(id_capaska)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    match row_opt {
        Some(row) => {
            Ok(HttpResponse::Ok().json(json!({
                "id_capaska": id_capaska,
                "id_provinsi": row.get::<i32, _>("id_provinsi"),
                "nilai_sikap_sempurna": row.get::<i32, _>("nilai_sikap_sempurna"),
                "nilai_hormat": row.get::<i32, _>("nilai_hormat"),
                "nilai_jalan_ditempat": row.get::<i32, _>("nilai_jalan_ditempat"),
                "nilai_sikap_istirahat": row.get::<i32, _>("nilai_sikap_istirahat"),
                "nilai_langkah_biasa": row.get::<i32, _>("nilai_langkah_biasa"),
                "nilai_langkah_tegap": row.get::<i32, _>("nilai_langkah_tegap"),
                "nilai_meluruskan_barisan": row.get::<i32, _>("nilai_meluruskan_barisan"),
                "nilai_melangkah": row.get::<i32, _>("nilai_melangkah"),
                "nilai_hadap_kanan_kiri": row.get::<i32, _>("nilai_hadap_kanan_kiri"),
                "nilai_serong_kanan_kiri": row.get::<i32, _>("nilai_serong_kanan_kiri"),
                "nilai_suara_komando": row.get::<i32, _>("nilai_suara_komando"),
                "status": row.get::<Option<String>, _>("status").unwrap_or_else(|| "Draft".to_string())
            })))
        },
        None => Ok(HttpResponse::Ok().json(json!({ "id_capaska": id_capaska, "nilai_sikap_sempurna": 0 })))
    }
}

#[post("/api/seleksi/pbb")]
pub async fn submit_pbb(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<PbbInput>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Juri PBB"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let juri_id_int = user_id_to_int(&claims.user_id);

    let query = r#"
        INSERT INTO pbb2026 (
            id_capaska, id_provinsi, id_juri, 
            nilai_sikap_sempurna, nilai_hormat, nilai_jalan_ditempat, nilai_sikap_istirahat, 
            nilai_langkah_biasa, nilai_langkah_tegap, nilai_meluruskan_barisan, nilai_melangkah, 
            nilai_hadap_kanan_kiri, nilai_serong_kanan_kiri, nilai_suara_komando, status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            nilai_sikap_sempurna = VALUES(nilai_sikap_sempurna),
            nilai_hormat = VALUES(nilai_hormat),
            nilai_jalan_ditempat = VALUES(nilai_jalan_ditempat),
            nilai_sikap_istirahat = VALUES(nilai_sikap_istirahat),
            nilai_langkah_biasa = VALUES(nilai_langkah_biasa),
            nilai_langkah_tegap = VALUES(nilai_langkah_tegap),
            nilai_meluruskan_barisan = VALUES(nilai_meluruskan_barisan),
            nilai_melangkah = VALUES(nilai_melangkah),
            nilai_hadap_kanan_kiri = VALUES(nilai_hadap_kanan_kiri),
            nilai_serong_kanan_kiri = VALUES(nilai_serong_kanan_kiri),
            nilai_suara_komando = VALUES(nilai_suara_komando),
            status = VALUES(status)
    "#;

    sqlx::query(query)
        .bind(payload.id_capaska)
        .bind(payload.id_provinsi)
        .bind(juri_id_int)
        .bind(payload.nilai_sikap_sempurna)
        .bind(payload.nilai_hormat)
        .bind(payload.nilai_jalan_ditempat)
        .bind(payload.nilai_sikap_istirahat)
        .bind(payload.nilai_langkah_biasa)
        .bind(payload.nilai_langkah_tegap)
        .bind(payload.nilai_meluruskan_barisan)
        .bind(payload.nilai_melangkah)
        .bind(payload.nilai_hadap_kanan_kiri)
        .bind(payload.nilai_serong_kanan_kiri)
        .bind(payload.nilai_suara_komando)
        .bind(&payload.status)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({ "status": "success", "message": "Penilaian PBB berhasil disimpan" })))
}

// Wawancara Endpoints
#[get("/api/seleksi/wawancara/{id_capaska}")]
pub async fn get_wawancara(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Pewawancara"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_capaska = path.into_inner();
    let sql = "SELECT * FROM wawancara WHERE id_capaska = ? LIMIT 1";
    let row_opt = sqlx::query(sql)
        .bind(id_capaska)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    match row_opt {
        Some(row) => {
            let n1: Option<rust_decimal::Decimal> = row.try_get("nilai1").ok();
            let n2: Option<rust_decimal::Decimal> = row.try_get("nilai2").ok();
            let n3: Option<rust_decimal::Decimal> = row.try_get("nilai3").ok();
            let n4: Option<rust_decimal::Decimal> = row.try_get("nilai4").ok();
            Ok(HttpResponse::Ok().json(json!({
                "id_capaska": id_capaska,
                "id_provinsi": row.get::<i32, _>("id_provinsi"),
                "nilai1": n1.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                "nilai2": n2.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                "nilai3": n3.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                "nilai4": n4.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                "status": row.get::<Option<String>, _>("status").unwrap_or_else(|| "Selesai".to_string())
            })))
        },
        None => Ok(HttpResponse::Ok().json(json!({ "id_capaska": id_capaska, "nilai1": 0.0 })))
    }
}

#[post("/api/seleksi/wawancara")]
pub async fn submit_wawancara(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<WawancaraInput>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Pewawancara"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let pewawancara_id_int = user_id_to_int(&claims.user_id);

    let query = r#"
        INSERT INTO wawancara (id_capaska, id_provinsi, id_pewawancara, nilai1, nilai2, nilai3, nilai4, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            nilai1 = VALUES(nilai1),
            nilai2 = VALUES(nilai2),
            nilai3 = VALUES(nilai3),
            nilai4 = VALUES(nilai4),
            status = VALUES(status)
    "#;

    sqlx::query(query)
        .bind(payload.id_capaska)
        .bind(payload.id_provinsi)
        .bind(pewawancara_id_int)
        .bind(payload.nilai1)
        .bind(payload.nilai2)
        .bind(payload.nilai3)
        .bind(payload.nilai4)
        .bind(&payload.status)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({ "status": "success", "message": "Penilaian Wawancara berhasil disimpan" })))
}

// Kesehatan Endpoints
#[get("/api/seleksi/kesehatan/{id_capaska}")]
pub async fn get_kesehatan(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Dokter Penilai"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_capaska = path.into_inner();
    let sql = "SELECT score_mata, score_gigi, score_tht FROM pemeriksaan_kesehatan WHERE id_capaska = ? LIMIT 1";
    let row_opt = sqlx::query(sql)
        .bind(id_capaska)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    match row_opt {
        Some(row) => {
            Ok(HttpResponse::Ok().json(json!({
                "id_capaska": id_capaska,
                "score_mata": row.get::<Option<i32>, _>("score_mata").unwrap_or(0),
                "score_gigi": row.get::<Option<i32>, _>("score_gigi").unwrap_or(0),
                "score_tht": row.get::<Option<i32>, _>("score_tht").unwrap_or(0),
            })))
        },
        None => Ok(HttpResponse::Ok().json(json!({ "id_capaska": id_capaska, "score_mata": 0, "score_gigi": 0, "score_tht": 0 })))
    }
}

#[post("/api/seleksi/kesehatan")]
pub async fn submit_kesehatan(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<KesehatanInput>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Dokter Penilai"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let petugas_id_int = user_id_to_int(&claims.user_id);

    let query = r#"
        INSERT INTO pemeriksaan_kesehatan (id_capaska, id_petugas, score_mata, score_gigi, score_tht, tanggal_pemeriksaan, jenis_pemeriksaan)
        VALUES (?, ?, ?, ?, ?, CURRENT_DATE, 'Awal')
        ON DUPLICATE KEY UPDATE
            score_mata = VALUES(score_mata),
            score_gigi = VALUES(score_gigi),
            score_tht = VALUES(score_tht),
            tanggal_pemeriksaan = CURRENT_DATE
    "#;

    sqlx::query(query)
        .bind(payload.id_capaska)
        .bind(petugas_id_int)
        .bind(payload.score_mata)
        .bind(payload.score_gigi)
        .bind(payload.score_tht)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({ "status": "success", "message": "Pemeriksaan Kesehatan berhasil disimpan" })))
}

// Psikotes Endpoints
#[get("/api/seleksi/psikotes/{id_capaska}")]
pub async fn get_psikotes(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_capaska = path.into_inner();
    
    // Fetch Candidate Profile to get name & dad number
    let c_sql = "SELECT nama_lengkap, nomor_dada FROM data_paskibraka WHERE id = ? LIMIT 1";
    let c_row = sqlx::query(c_sql)
        .bind(id_capaska)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let (nama, nomor_dada) = match c_row {
        Some(r) => (r.get::<String, _>("nama_lengkap"), r.get::<Option<String>, _>("nomor_dada")),
        None => return Ok(HttpResponse::NotFound().json(json!({ "message": "Peserta tidak ditemukan" })))
    };

    let sql = "SELECT iq, iq_kategori FROM psikotes WHERE nomor_tes = ? OR nama_asesi = ? LIMIT 1";
    let row_opt = sqlx::query(sql)
        .bind(nomor_dada.unwrap_or_default())
        .bind(nama)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    match row_opt {
        Some(row) => {
            Ok(HttpResponse::Ok().json(json!({
                "id_capaska": id_capaska,
                "iq": row.get::<Option<i32>, _>("iq").unwrap_or(0),
                "iq_kategori": row.get::<Option<String>, _>("iq_kategori").unwrap_or_default(),
            })))
        },
        None => Ok(HttpResponse::Ok().json(json!({ "id_capaska": id_capaska, "iq": 0, "iq_kategori": "" })))
    }
}

#[post("/api/seleksi/psikotes")]
pub async fn submit_psikotes(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<PsikotesInput>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    // Fetch Candidate Profile to get details
    let c_sql = "SELECT nama_lengkap, nomor_dada, jk, nama_instansi_pendidikan FROM data_paskibraka WHERE id = ? LIMIT 1";
    let c_row = sqlx::query(c_sql)
        .bind(payload.id_capaska)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let (nama, nomor_dada, jk, sekolah) = match c_row {
        Some(r) => (
            r.get::<String, _>("nama_lengkap"), 
            r.get::<Option<String>, _>("nomor_dada").unwrap_or_default(),
            r.get::<String, _>("jk"),
            r.get::<Option<String>, _>("nama_instansi_pendidikan").unwrap_or_default()
        ),
        None => return Ok(HttpResponse::NotFound().json(json!({ "message": "Peserta tidak ditemukan" })))
    };

    let query = r#"
        INSERT INTO psikotes (nomor_tes, nama_asesi, jenis_kelamin, asal_sekolah, iq, iq_kategori)
        VALUES (?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            iq = VALUES(iq),
            iq_kategori = VALUES(iq_kategori)
    "#;

    sqlx::query(query)
        .bind(nomor_dada)
        .bind(nama)
        .bind(jk)
        .bind(sekolah)
        .bind(payload.iq)
        .bind(&payload.iq_kategori)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({ "status": "success", "message": "Psikotes berhasil disimpan" })))
}

#[derive(Debug, Deserialize)]
pub struct MinatBakatInput {
    pub id_capaska: i32,
    pub id_provinsi: i32,
    pub skor: i32,
    pub kategori: String,
    pub status: String,
}

#[get("/api/seleksi/minat-bakat/{id_capaska}")]
pub async fn get_minat_bakat(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Juri Minat Bakat"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let id_capaska = path.into_inner();
    let sql = "SELECT skor, kategori, status FROM minat_bakat WHERE id_capaska = ? LIMIT 1";
    let row_opt = sqlx::query(sql)
        .bind(id_capaska)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    match row_opt {
        Some(row) => {
            Ok(HttpResponse::Ok().json(json!({
                "id_capaska": id_capaska,
                "skor": row.get::<i32, _>("skor"),
                "kategori": row.get::<Option<String>, _>("kategori").unwrap_or_default(),
                "status": row.get::<Option<String>, _>("status").unwrap_or_else(|| "Rated".to_string()),
            })))
        },
        None => Ok(HttpResponse::Ok().json(json!({ "id_capaska": id_capaska, "skor": 0, "kategori": "", "status": "Rated" })))
    }
}

#[post("/api/seleksi/minat-bakat")]
pub async fn submit_minat_bakat(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: web::Json<MinatBakatInput>,
) -> Result<impl Responder, Error> {
    let claims = auth::verify_jwt(&req)
        .map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let allowed_roles = ["Superadmin", "Admin Penilaian", "Juri Minat Bakat"];
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden("Akses ditolak"));
    }

    let penilai_id_int = user_id_to_int(&claims.user_id);

    let query = r#"
        INSERT INTO minat_bakat (id_provinsi, id_capaska, id_penilai, skor, kategori, status)
        VALUES (?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            skor = VALUES(skor),
            kategori = VALUES(kategori),
            status = VALUES(status)
    "#;

    sqlx::query(query)
        .bind(payload.id_provinsi)
        .bind(payload.id_capaska)
        .bind(penilai_id_int)
        .bind(payload.skor)
        .bind(&payload.kategori)
        .bind(&payload.status)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(json!({ "status": "success", "message": "Penilaian Minat Bakat berhasil disimpan" })))
}
