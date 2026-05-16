//src/controller/dppi_dilantik_controller.rs
use actix_web::{Error, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPool, prelude::FromRow};

use crate::auth;

#[derive(Deserialize, Debug)]
struct DppiDilantikRequest {
    tingkat: String,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i64>,
}

#[derive(Deserialize, Serialize, FromRow, Debug)]
struct DppiDilantik {
    id: i32,
    tingkat: String,
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i64>,
    nama_kabupaten: Option<String>,
    nama_provinsi: Option<String>,
}

#[get("/api/adminpanel/dppi-dilantik")]
pub async fn get_dppi_dilantik(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // Verifikasi JWT & Role
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator", "Pelaksana"].contains(&_claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator",
        ));
    }

    let data = sqlx::query_as::<_, DppiDilantik>(
        r#"
        SELECT d.id, d.tingkat, d.id_provinsi, d.id_kabupaten, p.nama_provinsi, k.nama_kabupaten
        FROM dppi_dilantik d
        LEFT JOIN kabupaten k ON d.id_kabupaten = k.id
        LEFT JOIN provinsi p ON d.id_provinsi = p.id
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(data))
}

#[post("/api/adminpanel/dppi-dilantik")]
pub async fn create_dppi_dilantik(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    form: web::Json<DppiDilantikRequest>,
) -> Result<impl Responder, Error> {
    // Verifikasi JWT & Role
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator", "Pelaksana"].contains(&_claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator",
        ));
    }

    let _result = sqlx::query(
        r#"
        INSERT INTO dppi_dilantik (tingkat, id_kabupaten, id_provinsi)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(&form.tingkat)
    .bind(&form.id_kabupaten)
    .bind(&form.id_provinsi)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(serde_json::json!({

        "message": "DPPI yang dilantik berhasil ditambahkan"
    })))
}

#[put("/api/adminpanel/dppi-dilantik/{id}")]
pub async fn put_dppi_dilantik(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    form: web::Json<DppiDilantikRequest>,
) -> Result<impl Responder, Error> {
    // Verifikasi JWT & Role
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator", "Pelaksana"].contains(&_claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator",
        ));
    }
    let id = path.into_inner();

    sqlx::query(
        r#"
        UPDATE dppi_dilantik
        SET tingkat = ?, id_kabupaten = ?, id_provinsi =?
        WHERE id = ?
        "#,
    )
    .bind(&form.tingkat)
    .bind(&form.id_kabupaten)
    .bind(&form.id_provinsi)
    .bind(id)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "DPPI dilantik berhasil diupdate"
    })))
}

// DELETE - Hapus Perusahaan
#[delete("/api/adminpanel/dppi-dilantik/{id}")]
pub async fn delete_dppi_dilantik(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let _claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;
    if !["Superadmin", "Administrator", "Pelaksana"].contains(&_claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin/Administrator",
        ));
    }
    let id = path.into_inner();

    sqlx::query("DELETE FROM dppi_dilantik WHERE id = ? ")
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Perusahaan berhasil dihapus"
    })))
}
