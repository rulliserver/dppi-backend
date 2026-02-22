// src/controllers/preview.rs
use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::{HttpRequest, Result, get, web};
use mime_guess;
use percent_encoding::percent_decode_str;
use std::{ffi::OsStr, path::Path};

const UPLOAD_ROOT: &str = "uploads"; // root uploads

#[get("preview/{tail:.*}")]
pub async fn preview(_req: HttpRequest, tail: web::Path<String>) -> Result<NamedFile> {
    // Decode URL path agar spasi dll aman
    let decoded = percent_decode_str(&tail).decode_utf8_lossy().to_string();

    // Cegah path traversal
    if decoded.contains("..") || decoded.starts_with('/') || decoded.starts_with('\\') {
        return Err(actix_web::error::ErrorBadRequest("Path tidak valid"));
    }

    // Pastikan file masih di bawah folder uploads
    let full_path = Path::new(UPLOAD_ROOT).join(&decoded);
    if !full_path.exists() {
        return Err(actix_web::error::ErrorNotFound("File tidak ditemukan"));
    }

    let mut file = NamedFile::open(&full_path)?;

    // Deteksi MIME
    let ct = mime_guess::from_path(&full_path).first_or_octet_stream();
    file = file.set_content_type(ct);

    // Set Content-Disposition: inline; filename="<asli>"
    let filename = full_path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("file");
    let cd = ContentDisposition {
        disposition: DispositionType::Inline,
        parameters: vec![DispositionParam::Filename(filename.to_string())],
    };
    file = file.set_content_disposition(cd);

    Ok(file)
}
