<<<<<<< HEAD
// src/controllers/post_controller.rs

use actix_web::{
    Error, HttpResponse, Responder, Result, get,
    web::{self, Data, Path},
};

use serde::Serialize;
use sqlx::{MySqlPool, prelude::FromRow};

use crate::controllers::pdp_controller::{EncryptedPdp, decrypt_pdp_row};

// =============================================== pdp ===========================================

#[get("/api/pdp/{id}")]
pub async fn get_pdp_cv(
    pool: Data<MySqlPool>,
    path: Path<i64>,
) -> Result<impl Responder, Error> {
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

#[get("/api/pendidikan/{id}")]
pub async fn get_pendidikan_cv(
    pool: Data<MySqlPool>,
    path: Path<i64>,
) -> Result<impl Responder, Error> {
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

// =============================================== ORGANISASI ===========================================
#[derive(Serialize, FromRow, Debug)]
struct Organisasi {
    id: i32,
    id_pdp: i32,
    nama_organisasi: String,
    posisi: String,
    status: Option<String>,
    tahun_masuk: u32,
    tahun_keluar: Option<u32>,
}

#[get("/api/organisasi/{id}")]
pub async fn get_organisasi_cv(
    pool: Data<MySqlPool>,
    path: Path<i32>, // <- i32 saja biar konsisten dengan kolom id_pdp
) -> Result<impl Responder, Error> {
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

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct Ketum {
    nama_lengkap: String,
    jabatan: Option<String>,
}

#[get("/api/ketum")]
pub async fn get_ketum_id_card(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
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
=======
// src/controllers/post_controller.rs

use actix_web::{
    Error, HttpResponse, Responder, Result, get,
    web::{self, Data, Path},
};

use serde::Serialize;
use sqlx::{MySqlPool, prelude::FromRow};

use crate::controllers::pdp_controller::{EncryptedPdp, decrypt_pdp_row};

// =============================================== pdp ===========================================

#[get("/api/pdp/{id}")]
pub async fn get_pdp_cv(
    pool: Data<MySqlPool>,
    path: Path<i64>,
) -> Result<impl Responder, Error> {
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

#[get("/api/pendidikan/{id}")]
pub async fn get_pendidikan_cv(
    pool: Data<MySqlPool>,
    path: Path<i64>,
) -> Result<impl Responder, Error> {
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

// =============================================== ORGANISASI ===========================================
#[derive(Serialize, FromRow, Debug)]
struct Organisasi {
    id: i32,
    id_pdp: i32,
    nama_organisasi: String,
    posisi: String,
    status: Option<String>,
    tahun_masuk: u32,
    tahun_keluar: Option<u32>,
}

#[get("/api/organisasi/{id}")]
pub async fn get_organisasi_cv(
    pool: Data<MySqlPool>,
    path: Path<i32>, // <- i32 saja biar konsisten dengan kolom id_pdp
) -> Result<impl Responder, Error> {
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

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct Ketum {
    nama_lengkap: String,
    jabatan: Option<String>,
}

#[get("/api/ketum")]
pub async fn get_ketum_id_card(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
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
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
