use crate::{auth, utils::send_generic_email_mail_send};
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, Result, delete, error::ErrorBadRequest,
    error::ErrorInternalServerError, get, post, web,
};
use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use mime_guess::MimeGuess;
use serde::Serialize;
use sqlx::{MySqlPool, prelude::FromRow, query, query_as};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Serialize, FromRow, Debug)]
struct Contact {
    id: i32,
    nama: String,
    telepon: Option<String>,
    email: String,
    jenis_pesan: String,
    evidance: Option<String>,
    pesan: String,
    keterangan: Option<String>,
    created_at: DateTime<Utc>,
}
#[get("/api/contact")]
pub async fn get_contact(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    let contacts: Vec<Contact> = sqlx::query_as::<_, Contact>(
        "SELECT id, nama, telepon, email, jenis_pesan, evidance, pesan, keterangan, created_at FROM contacts ORDER BY id DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(contacts))
}

// ================== DELETE CONTACT ==================
#[derive(Serialize, FromRow, Debug)]
struct ContactDelete {
    id: i32,
    evidance: Option<String>,
}
#[delete("/api/contact/{id}")]
pub async fn delete_contact(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let id_to_delete = path.into_inner();
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    // Langkah 1: Ambil path file dari database
    let contact_to_delete: Option<ContactDelete> = query_as!(
        ContactDelete,
        "SELECT id, evidance FROM contacts WHERE id = ?",
        id_to_delete
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    if contact_to_delete.is_none() {
        return Ok(
            HttpResponse::NotFound().body(format!("Contact with id {} not found", id_to_delete))
        );
    }

    let evidance_path: Option<PathBuf> =
        contact_to_delete.and_then(|c| c.evidance).map(|filename| {
            // Gabungkan nama file dengan direktori upload
            Path::new("./").join(filename)
        });

    // Langkah 2: Hapus entri dari database
    let result = query!("DELETE FROM contacts WHERE id = ?", id_to_delete)
        .execute(pool.get_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(
            HttpResponse::NotFound().body(format!("Contact with id {} not found", id_to_delete))
        );
    }

    // Langkah 3: Hapus file dari sistem file (jika ada)
    if let Some(path_to_delete) = evidance_path {
        if path_to_delete.exists() {
            if let Err(e) = fs::remove_file(&path_to_delete).await {
                eprintln!("Failed to delete file {}: {}", path_to_delete.display(), e);
            }
        }
    }

    Ok(HttpResponse::Ok().body(format!(
        "Contact with id {} and its evidence deleted successfully",
        id_to_delete
    )))
}

//Jumlah PDP berdasarkan status ================================================================================
#[derive(Serialize, FromRow, Debug)]
struct PdpStatus {
    id: String,
    status: Option<String>,
}
#[get("/api/pdp-terdaftar")]
pub async fn get_pdp_terdaftar(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin, Administrator, atau Admin Kesbangpol yang dapat mengakses",
        ));
    }
    let pdp_terdaftar: Vec<PdpStatus> = sqlx::query_as::<_, PdpStatus>(
        "SELECT id, status FROM pdp WHERE status = '' OR status IS NULL;",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(pdp_terdaftar))
}

#[get("/api/pdp-belum-diverifikasi")]
pub async fn get_pdp_belum_diverifikasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin, Administrator, atau Admin Kesbangpol yang dapat mengakses",
        ));
    }
    let pdp_belum_diverifikasi: Vec<PdpStatus> = sqlx::query_as::<_, PdpStatus>(
        "SELECT id, status FROM pdp WHERE status = 'Belum Diverifikasi'",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(pdp_belum_diverifikasi))
}

#[get("/api/pdp-diverifikasi")]
pub async fn get_pdp_diverifikasi(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin, Administrator, atau Admin Kesbangpol yang dapat mengakses",
        ));
    }
    let pdp_diverifikasi: Vec<PdpStatus> =
        sqlx::query_as::<_, PdpStatus>("SELECT id, status FROM pdp WHERE status = 'Verified'")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(pdp_diverifikasi))
}

#[get("/api/pdp-simental")]
pub async fn get_pdp_simental(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Admin Kesbangpol"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin, Administrator, atau Admin Kesbangpol yang dapat mengakses",
        ));
    }
    let pdp_simental: Vec<PdpStatus> =
        sqlx::query_as::<_, PdpStatus>("SELECT id, status FROM pdp WHERE status = 'Simental'")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(pdp_simental))
}

// ----- Helper kecil baca field text dari Multipart -----
async fn read_field_to_string(mut field: actix_multipart::Field) -> Result<String, Error> {
    let mut buf = Vec::new();
    while let Some(chunk) = field.next().await {
        buf.extend_from_slice(&chunk?);
    }
    Ok(String::from_utf8_lossy(&buf).trim().to_string())
}

#[post("/api/contact/reply")]
pub async fn reply_contact(
    mut payload: Multipart,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, Error> {
    let mut contact_id: Option<i32> = None;
    let mut to = String::new();
    let mut cc = String::new();
    let mut bcc = String::new();
    let mut subject = String::new();
    let mut message = String::new();
    let mut attachment: Option<(String, Vec<u8>, String)> = None;

    while let Some(mut field) = payload.try_next().await? {
        let name = field.name().unwrap_or_default().to_string();

        if name == "attachment" {
            let content_disposition = field.content_disposition();
            let filename = content_disposition
                .and_then(|cd| cd.get_filename())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    // Log untuk debugging
                    println!("Content-Disposition: {:?}", content_disposition);
                    "lampiran.bin".to_string()
                });

            println!("Received attachment with filename: {}", filename); // Debug log

            let mut bytes = Vec::new();
            while let Some(chunk) = field.next().await {
                bytes.extend_from_slice(&chunk?);
            }

            let mime = MimeGuess::from_path(&filename)
                .first_or_octet_stream()
                .essence_str()
                .to_string();

            attachment = Some((filename, bytes, mime));
            continue;
        }

        let value = read_field_to_string(field).await?;
        match name.as_str() {
            "id" => contact_id = value.parse::<i32>().ok(),
            "to" => to = value,
            "cc" => cc = value,
            "bcc" => bcc = value,
            "subject" => subject = value,
            "message" => message = value,
            _ => {}
        }
    }

    if to.is_empty() {
        return Err(ErrorBadRequest("Field 'to' wajib diisi"));
    }
    if subject.is_empty() {
        return Err(ErrorBadRequest("Field 'subject' wajib diisi"));
    }
    if message.is_empty() {
        return Err(ErrorBadRequest("Field 'message' wajib diisi"));
    }

    // let html = format!(
    //     r#"<div style="font-family:Arial,sans-serif;white-space:pre-wrap">{}</div>"#,
    //     html_escape::encode_text_minimal(&message),

    // );
    let escaped = html_escape::encode_text_minimal(&message);
    let html = format!(
        r#"<div style="font-family:Arial,sans-serif;white-space:pre-wrap">
{escaped}
<hr style="border:none;border-top:1px solid #e5e7eb;margin:16px 0;" />
<p style="font-size:12px;color:#6b7280;line-height:1.5;margin:0;">
  <strong>Pemberitahuan:</strong> Surat elektronik ini dikirimkan secara otomatis dari alamat
  <em>dppi@bpip.go.id</em> yang tidak memonitor balasan. Mohon untuk <strong>tidak membalas</strong> email ini.
  Apabila Anda perlu memberikan tanggapan lebih lanjut, silakan gunakan formulir pada situs resmi DPPI di
  <a href="https://dppi.bpip.go.id/kontak" target="_blank" rel="noopener">https://dppi.bpip.go.id/kontak</a>.
</p>
</div>"#
    );

    // kirim email (pakai mail_send seperti fungsi kamu sebelumnya)
    send_generic_email_mail_send(
        &to,
        Some(&cc),
        Some(&bcc),
        &subject,
        &message,
        Some(&html),
        attachment.as_ref(),
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // ==== UPDATE contacts.keterangan setelah terkirim ====
    if let Some(id) = contact_id {
        // sesuaikan schema jika perlu (mis. sismart.contacts)
        sqlx::query(
            r#"
            UPDATE contacts
            SET keterangan = 'Pesan sudah dibalas', updated_at = NOW()
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    }

    Ok(HttpResponse::Ok().body("OK"))
}
