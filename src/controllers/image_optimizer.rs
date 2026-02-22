<<<<<<< HEAD
use actix_web::{HttpResponse, Responder, get, web};
use image::{GenericImageView, imageops::FilterType};
use serde::Deserialize;
use std::{fs, path::PathBuf};
use webp::Encoder;

#[derive(Deserialize)]
pub struct ImageQuery {
    src: String,
    w: Option<u32>,
    q: Option<u8>, // kualitas 1–100
}

#[get("/image")]
pub async fn optimize_image(query: web::Query<ImageQuery>) -> impl Responder {
    let src_path = PathBuf::from(&query.src);
    if !src_path.exists() {
        return HttpResponse::NotFound().body("Source image not found");
    }

    let width = query.w.unwrap_or(0);
    let quality = query.q.unwrap_or(80).clamp(1, 100); // batasi 1–100

    // path file hasil webp
    let mut webp_path = src_path.clone();
    webp_path.set_extension("webp");

    // kalau sudah ada, kirim langsung
    if webp_path.exists() {
        if let Ok(bytes) = fs::read(&webp_path) {
            return HttpResponse::Ok()
                .content_type("image/webp")
                .insert_header(("Cache-Control", "max-age=31536000"))
                .body(bytes);
        }
    }

    // buka gambar asli
    let bytes = fs::read(&src_path).unwrap();
    let mut img = image::load_from_memory(&bytes).unwrap();

    // resize kalau ada w
    if width > 0 {
        let (w, h) = img.dimensions();
        let new_height = (h * width) / w;
        img = img.resize(width, new_height, FilterType::Lanczos3);
    }

    // convert ke RGB dan encode WebP dengan kualitas
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let encoder = Encoder::from_rgb(&rgb, w, h);
    let webp_bytes = encoder.encode(quality as f32).to_vec();

    // simpan hasilnya
    if let Some(parent) = webp_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(err) = fs::write(&webp_path, &webp_bytes) {
        eprintln!("Error saving WebP file: {}", err);
    }

    HttpResponse::Ok()
        .content_type("image/webp")
        .insert_header(("Cache-Control", "max-age=31536000"))
        .body(webp_bytes)
}
=======
use actix_web::{HttpResponse, Responder, get, web};
use image::{GenericImageView, imageops::FilterType};
use serde::Deserialize;
use std::{fs, path::PathBuf};
use webp::Encoder;

#[derive(Deserialize)]
pub struct ImageQuery {
    src: String,
    w: Option<u32>,
    q: Option<u8>, // kualitas 1–100
}

#[get("/image")]
pub async fn optimize_image(query: web::Query<ImageQuery>) -> impl Responder {
    let src_path = PathBuf::from(&query.src);
    if !src_path.exists() {
        return HttpResponse::NotFound().body("Source image not found");
    }

    let width = query.w.unwrap_or(0);
    let quality = query.q.unwrap_or(80).clamp(1, 100); // batasi 1–100

    // path file hasil webp
    let mut webp_path = src_path.clone();
    webp_path.set_extension("webp");

    // kalau sudah ada, kirim langsung
    if webp_path.exists() {
        if let Ok(bytes) = fs::read(&webp_path) {
            return HttpResponse::Ok()
                .content_type("image/webp")
                .insert_header(("Cache-Control", "max-age=31536000"))
                .body(bytes);
        }
    }

    // buka gambar asli
    let bytes = fs::read(&src_path).unwrap();
    let mut img = image::load_from_memory(&bytes).unwrap();

    // resize kalau ada w
    if width > 0 {
        let (w, h) = img.dimensions();
        let new_height = (h * width) / w;
        img = img.resize(width, new_height, FilterType::Lanczos3);
    }

    // convert ke RGB dan encode WebP dengan kualitas
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let encoder = Encoder::from_rgb(&rgb, w, h);
    let webp_bytes = encoder.encode(quality as f32).to_vec();

    // simpan hasilnya
    if let Some(parent) = webp_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(err) = fs::write(&webp_path, &webp_bytes) {
        eprintln!("Error saving WebP file: {}", err);
    }

    HttpResponse::Ok()
        .content_type("image/webp")
        .insert_header(("Cache-Control", "max-age=31536000"))
        .body(webp_bytes)
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
