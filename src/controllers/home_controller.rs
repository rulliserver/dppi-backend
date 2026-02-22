<<<<<<< HEAD
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, Responder, get, post, web};
use ammonia::clean;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use futures_util::TryStreamExt as _;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, mysql::MySqlPool};
use std::{env, io::Write, path::Path};
use uuid::Uuid;

use crate::models::pengumuman::FetchPengumuman;

// setting, provinsi, kabupaten
#[derive(Serialize, FromRow, Debug)]
struct Setting {
    id: i32,
    nama: Option<String>,
    deskripsi: Option<String>,
    alamat: Option<String>,
    telepon: Option<String>,
    email: Option<String>,
}

#[derive(Serialize, FromRow, Debug)]
struct Provinsi {
    id: i32,
    nama_provinsi: String,
}

#[derive(Serialize, FromRow, Debug)]
struct Kabupaten {
    id: i32,
    nama_kabupaten: String,
    id_provinsi: i32,
}

#[derive(Serialize)]
struct DataSettingResponse {
    setting: Option<Setting>,
    provinsi: Vec<Provinsi>,
    kabupaten: Vec<Kabupaten>,
}

#[get("/api/data-setting")]
pub async fn get_data_setting(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // ambil setting pertama
    let setting: Option<Setting> = sqlx::query_as::<_, Setting>(
        "SELECT id, nama, deskripsi, alamat, telepon, email FROM settings LIMIT 1",
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    // ambil semua provinsi
    let provinsi: Vec<Provinsi> =
        sqlx::query_as::<_, Provinsi>("SELECT id, nama_provinsi FROM provinsi")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // ambil semua kabupaten
    let kabupaten: Vec<Kabupaten> =
        sqlx::query_as::<_, Kabupaten>("SELECT id, nama_kabupaten, id_provinsi FROM kabupaten")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(DataSettingResponse {
        setting,
        provinsi,
        kabupaten,
    }))
}

//gallery
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    page: Option<usize>,
    per_page: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub current_page: usize,
    pub per_page: usize,
    pub total: usize,
    pub last_page: usize,
    pub from: usize,
    pub to: usize,
}

#[derive(Serialize, FromRow, Debug)]
struct Gallery {
    id: i32,
    kegiatan: String,
    foto: String,
    keterangan: Option<String>,
    tanggal: NaiveDate,
}

#[get("/api/gallery")]
pub async fn get_gallery(
    pool: web::Data<MySqlPool>,
    query: web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    // Default values
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(8);

    // Calculate offset
    let offset = (page - 1) * per_page;

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM galleries")
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let total = total as usize;

    // Calculate pagination metadata
    let last_page = (total as f64 / per_page as f64).ceil() as usize;
    let from = offset + 1;
    let to = std::cmp::min(offset + per_page, total);

    // Fetch paginated data
    let galleries: Vec<Gallery> = sqlx::query_as::<_, Gallery>(
        "SELECT id, kegiatan, foto, keterangan, tanggal FROM galleries ORDER BY tanggal DESC LIMIT ? OFFSET ?",
    )
    .bind(per_page as i32)
    .bind(offset as i32)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    // Create paginated response
    let response = PaginatedResponse {
        data: galleries,
        current_page: page,
        per_page,
        total,
        last_page,
        from,
        to,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/all-gallery")]
pub async fn get_all_gallery(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let galleries: Vec<Gallery> = sqlx::query_as::<_, Gallery>(
        "SELECT id, kegiatan, foto, keterangan, tanggal FROM galleries ORDER BY tanggal DESC LIMIT 8",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(galleries))
}

#[get("/api/gallery/{id}")]
pub async fn get_gallery_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();
    let galleries: Vec<Gallery> = sqlx::query_as::<_, Gallery>(
        "SELECT id, kegiatan, foto, keterangan, tanggal FROM galleries WHERE id=? ORDER BY tanggal DESC",
    )
     .bind(&id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(galleries))
}

//video
#[derive(Serialize, FromRow, Debug)]
struct Video {
    id: i32,
    file_video: String,
}
#[get("/api/video")]
pub async fn get_video(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let videos: Vec<Video> =
        sqlx::query_as::<_, Video>("SELECT id, file_video FROM videos ORDER BY id ASC LIMIT 1")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(videos))
}

//berita
#[derive(Serialize, FromRow, Debug)]
struct Post {
    id: i32,
    category_id: i32,
    title: String,
    slug: String,
    news_category: i32,
    tanggal: NaiveDate,
    view: i32,
    photo: String,
    caption: Option<String>,
    body: String,
    author: String,
    sumber: Option<String>,
    approval: i32,
    status: i32,
}
#[get("/api/berita")]
pub async fn get_berita(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let posts: Vec<Post> = sqlx::query_as::<_, Post>(
        "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status FROM posts WHERE status = 1 ORDER BY tanggal DESC LIMIT 3",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(posts))
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
    q: Option<String>,
}

#[get("/api/post")]

pub async fn get_all_berita(
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<Pagination>,
) -> Result<impl Responder, Error> {
    let page = pagination.page.unwrap_or(1).max(1); // Minimal page 1
    let limit = pagination.limit.unwrap_or(8).clamp(1, 50); // Batasi max 50 per page
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    let keyword_like = format!("%{}%", keyword);

    // Query dinamis dengan LIKE jika ada keyword
    let (posts, total): (Vec<Post>, i64) = if keyword.is_empty() {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE status = 1
             ORDER BY tanggal DESC
             LIMIT ? OFFSET ?",
        )
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE status = 1")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    } else {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE status = 1 AND (title LIKE ? OR body LIKE ?)
             ORDER BY tanggal DESC
             LIMIT ? OFFSET ?",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM posts WHERE status = 1 AND (title LIKE ? OR body LIKE ?)",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    };

    let total_pages = if total == 0 {
        0
    } else {
        (total as f64 / limit as f64).ceil() as u32
    };

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        per_page: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + limit, total as u32);

    let response = PaginatedResponse {
        data: posts,
        current_page: page,
        total_pages,
        total_items: total,
        per_page: limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize)]
pub struct Pagination2 {
    pub page: Option<u32>,
    pub per_page: Option<u32>, // Ganti dari 'limit' jadi 'per_page'
    pub q: Option<String>,
}

#[get("/api/post-random")]
pub async fn get_random_berita(
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<Pagination2>,
) -> Result<impl Responder, Error> {
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = pagination.per_page.unwrap_or(8).clamp(1, 50); // Batasi max 50 per page
    let offset = (page - 1) * per_page;
    let keyword = pagination.q.unwrap_or_default();

    let keyword_like = format!("%{}%", keyword);

    // Query dinamis dengan LIKE jika ada keyword
    let (posts, total): (Vec<Post>, i64) = if keyword.is_empty() {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE status = 1
           ORDER BY RAND()
             LIMIT ? OFFSET ?",
        )
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE status = 1")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    } else {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE status = 1 AND (title LIKE ? OR body LIKE ?)
             ORDER BY RAND()
             LIMIT ? OFFSET ?",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM posts WHERE status = 1 AND (title LIKE ? OR body LIKE ?)",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    };

    let total_pages = if total == 0 {
        0
    } else {
        (total as f64 / per_page as f64).ceil() as u32
    };

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        per_page: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + per_page, total as u32);

    let response = PaginatedResponse {
        data: posts,
        current_page: page,
        total_pages,
        total_items: total,
        per_page,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/berita/{slug}")]
pub async fn get_berita_by_slug(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let slug: String = path.into_inner();

    // 1. Ambil data post terlebih dahulu (tanpa update view dulu)
    let post = sqlx::query_as::<_, Post>(
        "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
         FROM posts
         WHERE status = 1 AND slug = ?",
    )
    .bind(&slug)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let post = match post {
        Some(post) => post,
        None => return Err(actix_web::error::ErrorNotFound("Post tidak ditemukan")),
    };

    // 2. Update view count secara asynchronous (fire and forget)
    let pool_clone = pool.clone();
    let slug_clone = slug.clone();

    // Spawn task untuk update view tanpa blocking response
    actix_web::rt::spawn(async move {
        if let Err(e) =
            sqlx::query("UPDATE posts SET view = view + 1 WHERE status = 1 AND slug = ?")
                .bind(&slug_clone)
                .execute(pool_clone.get_ref())
                .await
        {
            log::error!("Gagal update view untuk post {}: {:?}", slug_clone, e);
        } else {
            log::info!("View increased for post: {}", slug_clone);
        }
    });

    Ok(HttpResponse::Ok().json(post))
}

//kegiatan

#[derive(Serialize, FromRow, Debug)]
struct Kegiatan {
    id: i32,
    kategori: String,
    nama_kegiatan: String,
    slug: String,
    photo: String,
    biaya: i32,
    lokasi: String,
    tanggal: NaiveDate,
    jam: Option<NaiveTime>,
    batas_pendaftaran: Option<NaiveDate>,
    map: Option<String>,
    link_pendaftaran: Option<String>,
    status: String,
}

#[get("/api/kegiatan")]
pub async fn get_kegiatan(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let kegiatan: Vec<Kegiatan> = sqlx::query_as::<_, Kegiatan>(
        "SELECT id, kategori, nama_kegiatan, slug, photo, biaya, lokasi, tanggal, jam, batas_pendaftaran, map, link_pendaftaran, status FROM kegiatan ORDER BY tanggal DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(kegiatan))
}

#[get("/api/kegiatan/{slug}")]
pub async fn get_kegiatan_slug(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let slug: String = path.into_inner();
    let kegiatan  = sqlx::query_as::<_, Kegiatan>(
        "SELECT id, kategori, nama_kegiatan, slug, photo, biaya, lokasi, tanggal, jam, batas_pendaftaran, map, link_pendaftaran, status FROM kegiatan WHERE slug = ?",
    )
    .bind(slug)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(kegiatan))
}

//pdp-all

#[derive(Serialize, FromRow, Debug)]
struct Pdp {
    id: i32,
    id_provinsi: i32,
    id_kabupaten: Option<i32>,
}

#[get("/api/pdp-all")]
pub async fn get_pdp(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let pdp: Vec<Pdp> = sqlx::query_as::<_, Pdp>("SELECT id, id_provinsi, id_kabupaten FROM pdp")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(pdp))
}

//pdp-provinsi

#[derive(Serialize, FromRow, Debug)]
struct PdpProvinsi {
    id: String,
    id_provinsi: i32,
    id_kabupaten: Option<i32>,
}
#[get("/api/pdp-provinsi")]
pub async fn get_pdp_provinsi(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let pdp_provinsi: Vec<PdpProvinsi> = sqlx::query_as::<_, PdpProvinsi>(
        "SELECT id, id_provinsi, id_kabupaten FROM pdp where id_kabupaten IS NULL",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(pdp_provinsi))
}

//pdp-kabupaten

#[derive(Serialize, FromRow, Debug)]
struct PdpKabupaten {
    id: String,
    id_provinsi: i32,
    id_kabupaten: Option<i32>,
}
#[get("/api/pdp-kabupaten")]
pub async fn get_pdp_kabupaten(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let pdp_kabupaten: Vec<PdpKabupaten> = sqlx::query_as::<_, PdpKabupaten>(
        "SELECT id, id_provinsi, id_kabupaten FROM pdp where id_kabupaten IS NOT NULL",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(pdp_kabupaten))
}

//kabupaten
#[derive(Serialize, FromRow, Debug)]
struct KabupatenPdp {
    id: i32,
    nama_kabupaten: String,
    id_provinsi: i32,
}
#[get("/api/kabupaten")]
pub async fn get_kabupaten(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let kabupaten: Vec<KabupatenPdp> =
        sqlx::query_as::<_, KabupatenPdp>("SELECT id, nama_kabupaten, id_provinsi FROM kabupaten")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(kabupaten))
}

//provinsi
#[derive(Serialize, FromRow, Debug)]
struct ProvinsiPdp {
    id: i32,
    nama_provinsi: String,
}
#[get("/api/provinsi")]
pub async fn get_provinsi(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let provinsi: Vec<ProvinsiPdp> =
        sqlx::query_as::<_, ProvinsiPdp>("SELECT id, nama_provinsi FROM provinsi")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(provinsi))
}

//profil
#[derive(Serialize, FromRow, Debug)]
struct Profil {
    id: i32,
    dasar_hukum: String,
    pengertian: String,
    peran: String,
    tupoksi: String,
    kepengurusan: String,
}
#[get("/api/profil")]
pub async fn get_profil(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let profil: Vec<Profil> = sqlx::query_as::<_, Profil>(
        "SELECT id, dasar_hukum, pengertian, peran, tupoksi, kepengurusan FROM profil_lembaga LIMIT 1",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(profil))
}

//pelaksana-pusat
#[derive(Serialize, FromRow, Debug)]
struct PelaksanaPusat {
    id: i32,
    id_pdp: Option<i32>,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: String,
}
#[get("/api/pelaksana-pusat")]
pub async fn get_pelaksana_pusat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let pelaksana_pusat: Vec<PelaksanaPusat> = sqlx::query_as::<_, PelaksanaPusat>(
        "SELECT id, id_pdp, nama_lengkap, photo, jabatan FROM pelaksana_pusat",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(pelaksana_pusat))
}

#[derive(Debug, Serialize, FromRow)]
struct ProvinsiByPelaksana {
    id_provinsi: Option<i32>,
    nama_provinsi: String,
}

//pelaksana-provinsi
#[get("/api/pelaksana-provinsi")]
pub async fn pelaksana_provinsi(pool: web::Data<MySqlPool>) -> impl Responder {
    let query = r#"
        SELECT
            pp.id_provinsi,
            p.nama_provinsi
        FROM
            provinsi p
        LEFT JOIN
            pelaksana_provinsi pp ON pp.id_provinsi = p.id
        GROUP BY
            pp.id_provinsi, p.nama_provinsi
        ORDER BY
            p.nama_provinsi ASC
    "#;

    match sqlx::query_as::<_, ProvinsiByPelaksana>(query)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(provinsi) => {
            #[derive(Serialize)]
            struct InertiaData {
                provinsi: Vec<ProvinsiByPelaksana>,
            }

            let data = InertiaData { provinsi };

            // Return the data as JSON
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("Database query error: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to fetch data: {}", e))
        }
    }
}

#[derive(Debug, Serialize, FromRow)]
struct PelaksanaProvinsiById {
    id: i32,
    id_pdp: Option<i32>,
    id_provinsi: Option<i32>,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_provinsi: String,
}

//pelaksana-provinsi-by-id
#[get("/api/pelaksana-provinsi/{id}")]
pub async fn get_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let pelaksana = sqlx::query_as::<_, PelaksanaProvinsiById>(
        r#"
        SELECT
            pp.id_provinsi,
            pp.id,
            pp.id_pdp,
            pp.nama_lengkap,
            pp.photo,
            pp.jabatan,
            p.nama_provinsi
        FROM
            provinsi p
        LEFT JOIN
            pelaksana_provinsi pp ON pp.id_provinsi = p.id
        WHERE
            pp.id_provinsi = ?
        GROUP BY
            pp.id_provinsi, p.nama_provinsi, pp.id, pp.nama_lengkap, pp.photo, pp.jabatan, pp.id_pdp
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref()) // gunakan fetch_one() kalau pasti 1 data
    .await
    .map_err(|e| {
        log::error!(
            "Gagal mengambil data pelaksana provinsi dengan id provinsi {}: {:?}",
            id,
            e
        );
        actix_web::error::ErrorNotFound("Pelaksana provinsi tidak ditemukan")
    })?;

    Ok(HttpResponse::Ok().json(pelaksana))
}

//pelaksana-kabupaten
#[get("/api/pelaksana-kabupaten/provinsi")]
pub async fn pelaksana_kabupaten_all_provinsi(pool: web::Data<MySqlPool>) -> impl Responder {
    let query = r#"
        SELECT
            pp.id_provinsi,
            p.nama_provinsi
        FROM
            provinsi p
        LEFT JOIN
            pelaksana_kabupaten pp ON pp.id_provinsi = p.id
        GROUP BY
            pp.id_provinsi, p.nama_provinsi
        ORDER BY
            p.nama_provinsi ASC
    "#;

    match sqlx::query_as::<_, ProvinsiByPelaksana>(query)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(provinsi) => {
            #[derive(Serialize)]
            struct InertiaData {
                provinsi: Vec<ProvinsiByPelaksana>,
            }

            let data = InertiaData { provinsi };

            // Return the data as JSON
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("Database query error: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to fetch data: {}", e))
        }
    }
}
// pelaksana-kabupaten.rs
#[derive(Debug, Serialize, sqlx::FromRow)]
struct KabupatenByPelaksana {
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
    nama_kabupaten: String,
}

#[get("/api/pelaksana-kabupaten/provinsi/{id}")]
pub async fn get_pelaksana_kabupaten_names(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let kabupaten = sqlx::query_as::<_, KabupatenByPelaksana>(
        r#"
        SELECT
            pp.id_provinsi,
            pp.id_kabupaten,
            k.nama_kabupaten
        FROM
            kabupaten k
        LEFT JOIN
            pelaksana_kabupaten pp ON pp.id_kabupaten = k.id
        WHERE
            k.id_provinsi = ?
        GROUP BY
            pp.id_provinsi, pp.id_kabupaten, k.nama_kabupaten
        ORDER BY
            k.nama_kabupaten ASC
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!(
            "Gagal mengambil data kabupaten dengan id provinsi {}: {:?}",
            id,
            e
        );
        actix_web::error::ErrorNotFound("Kabupaten tidak ditemukan")
    })?;

    Ok(HttpResponse::Ok().json(kabupaten))
}

#[derive(Debug, Serialize, FromRow)]
struct PelaksanaKabupatenById {
    id: i32,
    id_pdp: Option<i32>,
    id_kabupaten: Option<i32>,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_kabupaten: String,
}
//pelaksana-kabupaten-by-id
#[get("/api/pelaksana-kabupaten/{id}")]
pub async fn get_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let pelaksana = sqlx::query_as::<_, PelaksanaKabupatenById>(
        r#"
        SELECT
            pp.id_kabupaten,
            pp.id,
            pp.id_pdp,
            pp.nama_lengkap,
            pp.photo,
            pp.jabatan,
            p.nama_kabupaten
        FROM
            kabupaten p
        LEFT JOIN
            pelaksana_kabupaten pp ON pp.id_kabupaten = p.id
        WHERE
            pp.id_kabupaten = ?
        GROUP BY
            pp.id_kabupaten, p.nama_kabupaten, pp.id, pp.nama_lengkap, pp.photo, pp.jabatan, pp.id_pdp
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref()) // gunakan fetch_one() kalau pasti 1 data
    .await
    .map_err(|e| {
        log::error!(
            "Gagal mengambil data pelaksana kabupaten dengan id kabupaten {}: {:?}",
            id,
            e
        );
        actix_web::error::ErrorNotFound("Pelaksana kabupaten tidak ditemukan")
    })?;

    Ok(HttpResponse::Ok().json(pelaksana))
}
#[derive(Debug, Serialize, FromRow)]
struct Regulasi {
    id: i32,
    nama_regulasi: String,
    icon_regulasi: String,
    file_regulasi: String,
    created_at: DateTime<Utc>,
    created_by: i32,
    role: String,
}

#[get("/api/regulasi")]
pub async fn get_regulasi(
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<Pagination>,
) -> Result<impl Responder, Error> {
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = 8;
    let offset = (page - 1) * per_page;
    let keyword = pagination.q.unwrap_or_default();
    let keyword_like = format!("%{}%", keyword);

    let (regulasi, total): (Vec<Regulasi>, i64) = if keyword.is_empty() {
        // tanpa pencarian
        let regulasi = sqlx::query_as::<_, Regulasi>(
            "SELECT
                r.id,
                r.nama_regulasi,
                r.icon_regulasi,
                r.file_regulasi,
                r.created_at,
                r.created_by,
                u.role
             FROM regulasi r
             LEFT JOIN users u ON u.id = r.created_by
             ORDER BY r.id DESC
             LIMIT ? OFFSET ?",
        )
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM regulasi")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        (regulasi, total.0)
    } else {
        // dengan pencarian
        let regulasi = sqlx::query_as::<_, Regulasi>(
            "SELECT
                r.id,
                r.nama_regulasi,
                r.icon_regulasi,
                r.file_regulasi,
                r.created_at,
                r.created_by,
                u.role
             FROM regulasi r
             LEFT JOIN users u ON u.id = r.created_by
             WHERE r.nama_regulasi LIKE ? OR r.file_regulasi LIKE ?
             ORDER BY r.id DESC
             LIMIT ? OFFSET ?",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM regulasi r
             WHERE r.nama_regulasi LIKE ? OR r.file_regulasi LIKE ?",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        (regulasi, total.0)
    };

    let total_pages = if total == 0 {
        0
    } else {
        (total as f64 / per_page as f64).ceil() as u32
    };

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        per_page: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + per_page, total as u32);

    let response = PaginatedResponse {
        data: regulasi,
        current_page: page,
        total_pages,
        total_items: total,
        per_page,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

// controllers/regulasi_controller.rs
#[get("/api/regulasi/view/{filename}")]
pub async fn view_regulasi(path: web::Path<String>) -> Result<impl Responder, Error> {
    let filename = path.into_inner();

    // Security: Validasi filename
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(actix_web::error::ErrorBadRequest("Filename tidak valid"));
    }

    let file_path = format!("./uploads/assets/file/regulasi/{}", filename);

    // Cek apakah file exists
    if !Path::new(&file_path).exists() {
        return Err(actix_web::error::ErrorNotFound("File tidak ditemukan"));
    }

    // Tentukan content type untuk view (inline)
    let content_type = get_content_type(&filename);

    match NamedFile::open(&file_path) {
        Ok(file) => {
            let file = file
                .use_last_modified(true)
                .set_content_type(content_type.parse().unwrap())
                .set_content_disposition(actix_web::http::header::ContentDisposition {
                    disposition: actix_web::http::header::DispositionType::Inline,
                    parameters: vec![actix_web::http::header::DispositionParam::Filename(
                        filename.clone(),
                    )],
                });

            log::info!("File viewed inline: {}", filename);

            Ok(file)
        }
        Err(_) => Err(actix_web::error::ErrorInternalServerError(
            "Gagal membuka file",
        )),
    }
}

// Helper function untuk menentukan content type
fn get_content_type(filename: &str) -> &'static str {
    let ext = filename.split('.').last().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        _ => "application/octet-stream",
    }
}
// Kontak
#[derive(Deserialize)]
struct RecaptchaResponse {
    success: bool,
    #[serde(default)]
    score: f32,
}

#[post("/api/pesan")]
pub async fn post_pesan(
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut nama = String::new();
    let mut telepon = String::new();
    let mut email = String::new();
    let mut jenis_pesan = String::new();
    let mut pesan = String::new();
    let mut evidance_path = None::<String>;
    let mut recaptcha_token = String::new();

    // 1. Parsing Multipart
    while let Some(item) = payload.try_next().await? {
        let mut field = item;
        let name = field.name().unwrap_or("").to_string();

        if name == "evidance" {
            // Dapatkan content type
            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_default();

            // Validasi tipe file
            let allowed_types = vec![
                "image/jpeg",
                "image/jpg",
                "image/png",
                "image/gif",
                "image/webp",
                "application/pdf",
            ];

            if !allowed_types.contains(&content_type.as_str()) {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": "Tipe file tidak diizinkan. Hanya gambar (JPEG, PNG, GIF, WebP) dan PDF yang diperbolehkan"
                })));
            }

            // Tentukan ekstensi file berdasarkan content type
            let extension = match content_type.as_str() {
                "image/jpeg" | "image/jpg" => "jpg",
                "image/png" => "png",
                "image/gif" => "gif",
                "image/webp" => "webp",
                "application/pdf" => "pdf",
                _ => "bin", // fallback
            };

            // Pastikan direktori upload ada
            let upload_dir = std::path::Path::new("uploads/assets/bukti-pelaporan");
            if !upload_dir.exists() {
                std::fs::create_dir_all(upload_dir)?;
            }

            // Generate nama file yang unik
            let filename = format!(
                "uploads/assets/bukti-pelaporan/{}.{}",
                Uuid::new_v4(),
                extension
            );
            let filepath = std::path::Path::new(&filename);
            let mut f = std::fs::File::create(filepath)?;

            // Tulis file
            while let Some(chunk) = field.try_next().await? {
                f.write_all(&chunk)?;
            }

            evidance_path = Some(filename);
        } else {
            // Handle field teks biasa
            let mut value = Vec::new();
            while let Some(chunk) = field.try_next().await? {
                value.extend_from_slice(&chunk);
            }
            let value_str = String::from_utf8(value).unwrap_or_default();

            match name.as_str() {
                "nama" => nama = value_str,
                "telepon" => telepon = value_str,
                "email" => email = value_str,
                "jenis_pesan" => jenis_pesan = value_str,
                "pesan" => pesan = value_str,
                "recaptcha_token" => recaptcha_token = value_str,
                _ => {}
            }
        }
    }

    // 2. Validasi field wajib
    if nama.trim().is_empty()
        || telepon.trim().is_empty()
        || email.trim().is_empty()
        || pesan.trim().is_empty()
    {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Field wajib tidak boleh kosong"
        })));
    }

    // 3. Validasi jenis pesan untuk pelaporan
    if jenis_pesan == "Pelaporan" && evidance_path.is_none() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Jenis pesan Pelaporan wajib menyertakan bukti"
        })));
    }

    // 4. Verifikasi reCAPTCHA ke Google
    let secret_key = env::var("RECAPTCHA_SECRET_KEY").map_err(|_| {
        actix_web::error::ErrorInternalServerError("RECAPTCHA_SECRET_KEY tidak diatur")
    })?;

    if recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Token reCAPTCHA tidak ditemukan"
        })));
    }

    let client = reqwest::Client::new();
    let verify_url = "https://www.google.com/recaptcha/api/siteverify";
    let response = client
        .post(verify_url)
        .form(&[
            ("secret", secret_key.as_str()),
            ("response", recaptcha_token.as_str()),
        ])
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let text = response
        .text()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    println!("🔍 reCAPTCHA response: {}", text);

    let body: RecaptchaResponse =
        serde_json::from_str(&text).map_err(actix_web::error::ErrorInternalServerError)?;

    if !body.success || body.score < 0.5 {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "status": "error",
            "message": "Verifikasi reCAPTCHA gagal, kemungkinan bot."
        })));
    }

    // 5. Sanitasi Input
    nama = clean(&nama.trim());
    telepon = clean(&telepon.trim());
    email = clean(&email.trim());
    jenis_pesan = clean(&jenis_pesan.trim());
    pesan = clean(&pesan.trim());

    // 6. Validasi email
    if !email.contains('@') {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Format email tidak valid"
        })));
    }

    // 7. Simpan ke database
    let result = sqlx::query!(
        r#"
        INSERT INTO contacts (nama, telepon, email, jenis_pesan, pesan, evidance)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        nama,
        telepon,
        email,
        jenis_pesan,
        pesan,
        evidance_path
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Pesan berhasil dikirim dan diverifikasi reCAPTCHA"
        }))),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Gagal menyimpan pesan ke database"
            })))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PengumumanResponse {
    announce: FetchPengumuman,
}
#[get("/api/pengumuman")]
pub async fn get_pengumuman(pool: web::Data<MySqlPool>) -> Result<HttpResponse, Error> {
    let result = sqlx::query_as::<_, FetchPengumuman>(
        "SELECT
            id,
            image,
            link
        FROM pengumuman LIMIT 1",
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(announce) => Ok(HttpResponse::Ok().json(PengumumanResponse { announce })),
        Err(sqlx::Error::RowNotFound) => {
            log::warn!("Data pengumuman tidak ditemukan di database");
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "Tidak ada data pengumuman"
            })))
        }
        Err(e) => {
            log::error!("Gagal mengambil data pengumuman: {:?}", e);
            Err(actix_web::error::ErrorInternalServerError(
                "Gagal mengambil data pengumuman",
            ))
        }
    }
}
=======
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, Responder, get, post, web};
use ammonia::clean;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use futures_util::TryStreamExt as _;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, mysql::MySqlPool};
use std::{env, io::Write, path::Path};
use uuid::Uuid;

use crate::models::pengumuman::FetchPengumuman;

// setting, provinsi, kabupaten
#[derive(Serialize, FromRow, Debug)]
struct Setting {
    id: i32,
    nama: Option<String>,
    deskripsi: Option<String>,
    alamat: Option<String>,
    telepon: Option<String>,
    email: Option<String>,
}

#[derive(Serialize, FromRow, Debug)]
struct Provinsi {
    id: i32,
    nama_provinsi: String,
}

#[derive(Serialize, FromRow, Debug)]
struct Kabupaten {
    id: i32,
    nama_kabupaten: String,
    id_provinsi: i32,
}

#[derive(Serialize)]
struct DataSettingResponse {
    setting: Option<Setting>,
    provinsi: Vec<Provinsi>,
    kabupaten: Vec<Kabupaten>,
}

#[get("/api/data-setting")]
pub async fn get_data_setting(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // ambil setting pertama
    let setting: Option<Setting> = sqlx::query_as::<_, Setting>(
        "SELECT id, nama, deskripsi, alamat, telepon, email FROM settings LIMIT 1",
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    // ambil semua provinsi
    let provinsi: Vec<Provinsi> =
        sqlx::query_as::<_, Provinsi>("SELECT id, nama_provinsi FROM provinsi")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // ambil semua kabupaten
    let kabupaten: Vec<Kabupaten> =
        sqlx::query_as::<_, Kabupaten>("SELECT id, nama_kabupaten, id_provinsi FROM kabupaten")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(DataSettingResponse {
        setting,
        provinsi,
        kabupaten,
    }))
}

//gallery
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    page: Option<usize>,
    per_page: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub current_page: usize,
    pub per_page: usize,
    pub total: usize,
    pub last_page: usize,
    pub from: usize,
    pub to: usize,
}

#[derive(Serialize, FromRow, Debug)]
struct Gallery {
    id: i32,
    kegiatan: String,
    foto: String,
    keterangan: Option<String>,
    tanggal: NaiveDate,
}

#[get("/api/gallery")]
pub async fn get_gallery(
    pool: web::Data<MySqlPool>,
    query: web::Query<PaginationParams>,
) -> Result<impl Responder, Error> {
    // Default values
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(8);

    // Calculate offset
    let offset = (page - 1) * per_page;

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM galleries")
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let total = total as usize;

    // Calculate pagination metadata
    let last_page = (total as f64 / per_page as f64).ceil() as usize;
    let from = offset + 1;
    let to = std::cmp::min(offset + per_page, total);

    // Fetch paginated data
    let galleries: Vec<Gallery> = sqlx::query_as::<_, Gallery>(
        "SELECT id, kegiatan, foto, keterangan, tanggal FROM galleries ORDER BY tanggal DESC LIMIT ? OFFSET ?",
    )
    .bind(per_page as i32)
    .bind(offset as i32)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    // Create paginated response
    let response = PaginatedResponse {
        data: galleries,
        current_page: page,
        per_page,
        total,
        last_page,
        from,
        to,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/all-gallery")]
pub async fn get_all_gallery(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let galleries: Vec<Gallery> = sqlx::query_as::<_, Gallery>(
        "SELECT id, kegiatan, foto, keterangan, tanggal FROM galleries ORDER BY tanggal DESC LIMIT 8",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(galleries))
}

#[get("/api/gallery/{id}")]
pub async fn get_gallery_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();
    let galleries: Vec<Gallery> = sqlx::query_as::<_, Gallery>(
        "SELECT id, kegiatan, foto, keterangan, tanggal FROM galleries WHERE id=? ORDER BY tanggal DESC",
    )
     .bind(&id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(galleries))
}

//video
#[derive(Serialize, FromRow, Debug)]
struct Video {
    id: i32,
    file_video: String,
}
#[get("/api/video")]
pub async fn get_video(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let videos: Vec<Video> =
        sqlx::query_as::<_, Video>("SELECT id, file_video FROM videos ORDER BY id ASC LIMIT 1")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(videos))
}

//berita
#[derive(Serialize, FromRow, Debug)]
struct Post {
    id: i32,
    category_id: i32,
    title: String,
    slug: String,
    news_category: i32,
    tanggal: NaiveDate,
    view: i32,
    photo: String,
    caption: Option<String>,
    body: String,
    author: String,
    sumber: Option<String>,
    approval: i32,
    status: i32,
}
#[get("/api/berita")]
pub async fn get_berita(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let posts: Vec<Post> = sqlx::query_as::<_, Post>(
        "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status FROM posts WHERE status = 1 ORDER BY tanggal DESC LIMIT 3",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(posts))
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
    q: Option<String>,
}

#[get("/api/post")]

pub async fn get_all_berita(
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<Pagination>,
) -> Result<impl Responder, Error> {
    let page = pagination.page.unwrap_or(1).max(1); // Minimal page 1
    let limit = pagination.limit.unwrap_or(8).clamp(1, 50); // Batasi max 50 per page
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    let keyword_like = format!("%{}%", keyword);

    // Query dinamis dengan LIKE jika ada keyword
    let (posts, total): (Vec<Post>, i64) = if keyword.is_empty() {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE status = 1
             ORDER BY tanggal DESC
             LIMIT ? OFFSET ?",
        )
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE status = 1")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    } else {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE status = 1 AND (title LIKE ? OR body LIKE ?)
             ORDER BY tanggal DESC
             LIMIT ? OFFSET ?",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM posts WHERE status = 1 AND (title LIKE ? OR body LIKE ?)",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    };

    let total_pages = if total == 0 {
        0
    } else {
        (total as f64 / limit as f64).ceil() as u32
    };

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        per_page: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + limit, total as u32);

    let response = PaginatedResponse {
        data: posts,
        current_page: page,
        total_pages,
        total_items: total,
        per_page: limit,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Deserialize)]
pub struct Pagination2 {
    pub page: Option<u32>,
    pub per_page: Option<u32>, // Ganti dari 'limit' jadi 'per_page'
    pub q: Option<String>,
}

#[get("/api/post-random")]
pub async fn get_random_berita(
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<Pagination2>,
) -> Result<impl Responder, Error> {
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = pagination.per_page.unwrap_or(8).clamp(1, 50); // Batasi max 50 per page
    let offset = (page - 1) * per_page;
    let keyword = pagination.q.unwrap_or_default();

    let keyword_like = format!("%{}%", keyword);

    // Query dinamis dengan LIKE jika ada keyword
    let (posts, total): (Vec<Post>, i64) = if keyword.is_empty() {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE status = 1
           ORDER BY RAND()
             LIMIT ? OFFSET ?",
        )
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts WHERE status = 1")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    } else {
        let posts = sqlx::query_as::<_, Post>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE status = 1 AND (title LIKE ? OR body LIKE ?)
             ORDER BY RAND()
             LIMIT ? OFFSET ?",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM posts WHERE status = 1 AND (title LIKE ? OR body LIKE ?)",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    };

    let total_pages = if total == 0 {
        0
    } else {
        (total as f64 / per_page as f64).ceil() as u32
    };

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        per_page: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + per_page, total as u32);

    let response = PaginatedResponse {
        data: posts,
        current_page: page,
        total_pages,
        total_items: total,
        per_page,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/berita/{slug}")]
pub async fn get_berita_by_slug(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let slug: String = path.into_inner();

    // 1. Ambil data post terlebih dahulu (tanpa update view dulu)
    let post = sqlx::query_as::<_, Post>(
        "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
         FROM posts
         WHERE status = 1 AND slug = ?",
    )
    .bind(&slug)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let post = match post {
        Some(post) => post,
        None => return Err(actix_web::error::ErrorNotFound("Post tidak ditemukan")),
    };

    // 2. Update view count secara asynchronous (fire and forget)
    let pool_clone = pool.clone();
    let slug_clone = slug.clone();

    // Spawn task untuk update view tanpa blocking response
    actix_web::rt::spawn(async move {
        if let Err(e) =
            sqlx::query("UPDATE posts SET view = view + 1 WHERE status = 1 AND slug = ?")
                .bind(&slug_clone)
                .execute(pool_clone.get_ref())
                .await
        {
            log::error!("Gagal update view untuk post {}: {:?}", slug_clone, e);
        } else {
            log::info!("View increased for post: {}", slug_clone);
        }
    });

    Ok(HttpResponse::Ok().json(post))
}

//kegiatan

#[derive(Serialize, FromRow, Debug)]
struct Kegiatan {
    id: i32,
    kategori: String,
    nama_kegiatan: String,
    slug: String,
    photo: String,
    biaya: i32,
    lokasi: String,
    tanggal: NaiveDate,
    jam: Option<NaiveTime>,
    batas_pendaftaran: Option<NaiveDate>,
    map: Option<String>,
    link_pendaftaran: Option<String>,
    status: String,
}

#[get("/api/kegiatan")]
pub async fn get_kegiatan(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let kegiatan: Vec<Kegiatan> = sqlx::query_as::<_, Kegiatan>(
        "SELECT id, kategori, nama_kegiatan, slug, photo, biaya, lokasi, tanggal, jam, batas_pendaftaran, map, link_pendaftaran, status FROM kegiatan ORDER BY tanggal DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(kegiatan))
}

#[get("/api/kegiatan/{slug}")]
pub async fn get_kegiatan_slug(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let slug: String = path.into_inner();
    let kegiatan  = sqlx::query_as::<_, Kegiatan>(
        "SELECT id, kategori, nama_kegiatan, slug, photo, biaya, lokasi, tanggal, jam, batas_pendaftaran, map, link_pendaftaran, status FROM kegiatan WHERE slug = ?",
    )
    .bind(slug)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(kegiatan))
}

//pdp-all

#[derive(Serialize, FromRow, Debug)]
struct Pdp {
    id: i32,
    id_provinsi: i32,
    id_kabupaten: Option<i32>,
}

#[get("/api/pdp-all")]
pub async fn get_pdp(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let pdp: Vec<Pdp> = sqlx::query_as::<_, Pdp>("SELECT id, id_provinsi, id_kabupaten FROM pdp")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(pdp))
}

//pdp-provinsi

#[derive(Serialize, FromRow, Debug)]
struct PdpProvinsi {
    id: String,
    id_provinsi: i32,
    id_kabupaten: Option<i32>,
}
#[get("/api/pdp-provinsi")]
pub async fn get_pdp_provinsi(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let pdp_provinsi: Vec<PdpProvinsi> = sqlx::query_as::<_, PdpProvinsi>(
        "SELECT id, id_provinsi, id_kabupaten FROM pdp where id_kabupaten IS NULL",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(pdp_provinsi))
}

//pdp-kabupaten

#[derive(Serialize, FromRow, Debug)]
struct PdpKabupaten {
    id: String,
    id_provinsi: i32,
    id_kabupaten: Option<i32>,
}
#[get("/api/pdp-kabupaten")]
pub async fn get_pdp_kabupaten(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let pdp_kabupaten: Vec<PdpKabupaten> = sqlx::query_as::<_, PdpKabupaten>(
        "SELECT id, id_provinsi, id_kabupaten FROM pdp where id_kabupaten IS NOT NULL",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(pdp_kabupaten))
}

//kabupaten
#[derive(Serialize, FromRow, Debug)]
struct KabupatenPdp {
    id: i32,
    nama_kabupaten: String,
    id_provinsi: i32,
}
#[get("/api/kabupaten")]
pub async fn get_kabupaten(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let kabupaten: Vec<KabupatenPdp> =
        sqlx::query_as::<_, KabupatenPdp>("SELECT id, nama_kabupaten, id_provinsi FROM kabupaten")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(kabupaten))
}

//provinsi
#[derive(Serialize, FromRow, Debug)]
struct ProvinsiPdp {
    id: i32,
    nama_provinsi: String,
}
#[get("/api/provinsi")]
pub async fn get_provinsi(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let provinsi: Vec<ProvinsiPdp> =
        sqlx::query_as::<_, ProvinsiPdp>("SELECT id, nama_provinsi FROM provinsi")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(provinsi))
}

//profil
#[derive(Serialize, FromRow, Debug)]
struct Profil {
    id: i32,
    dasar_hukum: String,
    pengertian: String,
    peran: String,
    tupoksi: String,
    kepengurusan: String,
}
#[get("/api/profil")]
pub async fn get_profil(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let profil: Vec<Profil> = sqlx::query_as::<_, Profil>(
        "SELECT id, dasar_hukum, pengertian, peran, tupoksi, kepengurusan FROM profil_lembaga LIMIT 1",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(profil))
}

//pelaksana-pusat
#[derive(Serialize, FromRow, Debug)]
struct PelaksanaPusat {
    id: i32,
    id_pdp: Option<i32>,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: String,
}
#[get("/api/pelaksana-pusat")]
pub async fn get_pelaksana_pusat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let pelaksana_pusat: Vec<PelaksanaPusat> = sqlx::query_as::<_, PelaksanaPusat>(
        "SELECT id, id_pdp, nama_lengkap, photo, jabatan FROM pelaksana_pusat",
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(pelaksana_pusat))
}

#[derive(Debug, Serialize, FromRow)]
struct ProvinsiByPelaksana {
    id_provinsi: Option<i32>,
    nama_provinsi: String,
}

//pelaksana-provinsi
#[get("/api/pelaksana-provinsi")]
pub async fn pelaksana_provinsi(pool: web::Data<MySqlPool>) -> impl Responder {
    let query = r#"
        SELECT
            pp.id_provinsi,
            p.nama_provinsi
        FROM
            provinsi p
        LEFT JOIN
            pelaksana_provinsi pp ON pp.id_provinsi = p.id
        GROUP BY
            pp.id_provinsi, p.nama_provinsi
        ORDER BY
            p.nama_provinsi ASC
    "#;

    match sqlx::query_as::<_, ProvinsiByPelaksana>(query)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(provinsi) => {
            #[derive(Serialize)]
            struct InertiaData {
                provinsi: Vec<ProvinsiByPelaksana>,
            }

            let data = InertiaData { provinsi };

            // Return the data as JSON
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("Database query error: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to fetch data: {}", e))
        }
    }
}

#[derive(Debug, Serialize, FromRow)]
struct PelaksanaProvinsiById {
    id: i32,
    id_pdp: Option<i32>,
    id_provinsi: Option<i32>,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_provinsi: String,
}

//pelaksana-provinsi-by-id
#[get("/api/pelaksana-provinsi/{id}")]
pub async fn get_pelaksana_provinsi(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let pelaksana = sqlx::query_as::<_, PelaksanaProvinsiById>(
        r#"
        SELECT
            pp.id_provinsi,
            pp.id,
            pp.id_pdp,
            pp.nama_lengkap,
            pp.photo,
            pp.jabatan,
            p.nama_provinsi
        FROM
            provinsi p
        LEFT JOIN
            pelaksana_provinsi pp ON pp.id_provinsi = p.id
        WHERE
            pp.id_provinsi = ?
        GROUP BY
            pp.id_provinsi, p.nama_provinsi, pp.id, pp.nama_lengkap, pp.photo, pp.jabatan, pp.id_pdp
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref()) // gunakan fetch_one() kalau pasti 1 data
    .await
    .map_err(|e| {
        log::error!(
            "Gagal mengambil data pelaksana provinsi dengan id provinsi {}: {:?}",
            id,
            e
        );
        actix_web::error::ErrorNotFound("Pelaksana provinsi tidak ditemukan")
    })?;

    Ok(HttpResponse::Ok().json(pelaksana))
}

//pelaksana-kabupaten
#[get("/api/pelaksana-kabupaten/provinsi")]
pub async fn pelaksana_kabupaten_all_provinsi(pool: web::Data<MySqlPool>) -> impl Responder {
    let query = r#"
        SELECT
            pp.id_provinsi,
            p.nama_provinsi
        FROM
            provinsi p
        LEFT JOIN
            pelaksana_kabupaten pp ON pp.id_provinsi = p.id
        GROUP BY
            pp.id_provinsi, p.nama_provinsi
        ORDER BY
            p.nama_provinsi ASC
    "#;

    match sqlx::query_as::<_, ProvinsiByPelaksana>(query)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(provinsi) => {
            #[derive(Serialize)]
            struct InertiaData {
                provinsi: Vec<ProvinsiByPelaksana>,
            }

            let data = InertiaData { provinsi };

            // Return the data as JSON
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            eprintln!("Database query error: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to fetch data: {}", e))
        }
    }
}
// pelaksana-kabupaten.rs
#[derive(Debug, Serialize, sqlx::FromRow)]
struct KabupatenByPelaksana {
    id_provinsi: Option<i32>,
    id_kabupaten: Option<i32>,
    nama_kabupaten: String,
}

#[get("/api/pelaksana-kabupaten/provinsi/{id}")]
pub async fn get_pelaksana_kabupaten_names(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let kabupaten = sqlx::query_as::<_, KabupatenByPelaksana>(
        r#"
        SELECT
            pp.id_provinsi,
            pp.id_kabupaten,
            k.nama_kabupaten
        FROM
            kabupaten k
        LEFT JOIN
            pelaksana_kabupaten pp ON pp.id_kabupaten = k.id
        WHERE
            k.id_provinsi = ?
        GROUP BY
            pp.id_provinsi, pp.id_kabupaten, k.nama_kabupaten
        ORDER BY
            k.nama_kabupaten ASC
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!(
            "Gagal mengambil data kabupaten dengan id provinsi {}: {:?}",
            id,
            e
        );
        actix_web::error::ErrorNotFound("Kabupaten tidak ditemukan")
    })?;

    Ok(HttpResponse::Ok().json(kabupaten))
}

#[derive(Debug, Serialize, FromRow)]
struct PelaksanaKabupatenById {
    id: i32,
    id_pdp: Option<i32>,
    id_kabupaten: Option<i32>,
    nama_lengkap: String,
    photo: Option<String>,
    jabatan: Option<String>,
    nama_kabupaten: String,
}
//pelaksana-kabupaten-by-id
#[get("/api/pelaksana-kabupaten/{id}")]
pub async fn get_pelaksana_kabupaten(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let id: String = path.into_inner();

    let pelaksana = sqlx::query_as::<_, PelaksanaKabupatenById>(
        r#"
        SELECT
            pp.id_kabupaten,
            pp.id,
            pp.id_pdp,
            pp.nama_lengkap,
            pp.photo,
            pp.jabatan,
            p.nama_kabupaten
        FROM
            kabupaten p
        LEFT JOIN
            pelaksana_kabupaten pp ON pp.id_kabupaten = p.id
        WHERE
            pp.id_kabupaten = ?
        GROUP BY
            pp.id_kabupaten, p.nama_kabupaten, pp.id, pp.nama_lengkap, pp.photo, pp.jabatan, pp.id_pdp
        "#,
    )
    .bind(&id)
    .fetch_all(pool.get_ref()) // gunakan fetch_one() kalau pasti 1 data
    .await
    .map_err(|e| {
        log::error!(
            "Gagal mengambil data pelaksana kabupaten dengan id kabupaten {}: {:?}",
            id,
            e
        );
        actix_web::error::ErrorNotFound("Pelaksana kabupaten tidak ditemukan")
    })?;

    Ok(HttpResponse::Ok().json(pelaksana))
}
#[derive(Debug, Serialize, FromRow)]
struct Regulasi {
    id: i32,
    nama_regulasi: String,
    icon_regulasi: String,
    file_regulasi: String,
    created_at: DateTime<Utc>,
    created_by: i32,
    role: String,
}

#[get("/api/regulasi")]
pub async fn get_regulasi(
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<Pagination>,
) -> Result<impl Responder, Error> {
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = 8;
    let offset = (page - 1) * per_page;
    let keyword = pagination.q.unwrap_or_default();
    let keyword_like = format!("%{}%", keyword);

    let (regulasi, total): (Vec<Regulasi>, i64) = if keyword.is_empty() {
        // tanpa pencarian
        let regulasi = sqlx::query_as::<_, Regulasi>(
            "SELECT
                r.id,
                r.nama_regulasi,
                r.icon_regulasi,
                r.file_regulasi,
                r.created_at,
                r.created_by,
                u.role
             FROM regulasi r
             LEFT JOIN users u ON u.id = r.created_by
             ORDER BY r.id DESC
             LIMIT ? OFFSET ?",
        )
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM regulasi")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        (regulasi, total.0)
    } else {
        // dengan pencarian
        let regulasi = sqlx::query_as::<_, Regulasi>(
            "SELECT
                r.id,
                r.nama_regulasi,
                r.icon_regulasi,
                r.file_regulasi,
                r.created_at,
                r.created_by,
                u.role
             FROM regulasi r
             LEFT JOIN users u ON u.id = r.created_by
             WHERE r.nama_regulasi LIKE ? OR r.file_regulasi LIKE ?
             ORDER BY r.id DESC
             LIMIT ? OFFSET ?",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .bind(per_page as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM regulasi r
             WHERE r.nama_regulasi LIKE ? OR r.file_regulasi LIKE ?",
        )
        .bind(&keyword_like)
        .bind(&keyword_like)
        .fetch_one(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        (regulasi, total.0)
    };

    let total_pages = if total == 0 {
        0
    } else {
        (total as f64 / per_page as f64).ceil() as u32
    };

    #[derive(Serialize)]
    struct PaginatedResponse<T> {
        data: Vec<T>,
        current_page: u32,
        total_pages: u32,
        total_items: i64,
        per_page: u32,
        last_page: u32,
        from: u32,
        to: u32,
        query: String,
    }

    let from = if total == 0 { 0 } else { offset + 1 };
    let to = std::cmp::min(offset + per_page, total as u32);

    let response = PaginatedResponse {
        data: regulasi,
        current_page: page,
        total_pages,
        total_items: total,
        per_page,
        last_page: total_pages,
        from,
        to,
        query: keyword,
    };

    Ok(HttpResponse::Ok().json(response))
}

// controllers/regulasi_controller.rs
#[get("/api/regulasi/view/{filename}")]
pub async fn view_regulasi(path: web::Path<String>) -> Result<impl Responder, Error> {
    let filename = path.into_inner();

    // Security: Validasi filename
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(actix_web::error::ErrorBadRequest("Filename tidak valid"));
    }

    let file_path = format!("./uploads/assets/file/regulasi/{}", filename);

    // Cek apakah file exists
    if !Path::new(&file_path).exists() {
        return Err(actix_web::error::ErrorNotFound("File tidak ditemukan"));
    }

    // Tentukan content type untuk view (inline)
    let content_type = get_content_type(&filename);

    match NamedFile::open(&file_path) {
        Ok(file) => {
            let file = file
                .use_last_modified(true)
                .set_content_type(content_type.parse().unwrap())
                .set_content_disposition(actix_web::http::header::ContentDisposition {
                    disposition: actix_web::http::header::DispositionType::Inline,
                    parameters: vec![actix_web::http::header::DispositionParam::Filename(
                        filename.clone(),
                    )],
                });

            log::info!("File viewed inline: {}", filename);

            Ok(file)
        }
        Err(_) => Err(actix_web::error::ErrorInternalServerError(
            "Gagal membuka file",
        )),
    }
}

// Helper function untuk menentukan content type
fn get_content_type(filename: &str) -> &'static str {
    let ext = filename.split('.').last().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        _ => "application/octet-stream",
    }
}
// Kontak
#[derive(Deserialize)]
struct RecaptchaResponse {
    success: bool,
    #[serde(default)]
    score: f32,
}

#[post("/api/pesan")]
pub async fn post_pesan(
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut nama = String::new();
    let mut telepon = String::new();
    let mut email = String::new();
    let mut jenis_pesan = String::new();
    let mut pesan = String::new();
    let mut evidance_path = None::<String>;
    let mut recaptcha_token = String::new();

    // 1. Parsing Multipart
    while let Some(item) = payload.try_next().await? {
        let mut field = item;
        let name = field.name().unwrap_or("").to_string();

        if name == "evidance" {
            // Dapatkan content type
            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_default();

            // Validasi tipe file
            let allowed_types = vec![
                "image/jpeg",
                "image/jpg",
                "image/png",
                "image/gif",
                "image/webp",
                "application/pdf",
            ];

            if !allowed_types.contains(&content_type.as_str()) {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": "Tipe file tidak diizinkan. Hanya gambar (JPEG, PNG, GIF, WebP) dan PDF yang diperbolehkan"
                })));
            }

            // Tentukan ekstensi file berdasarkan content type
            let extension = match content_type.as_str() {
                "image/jpeg" | "image/jpg" => "jpg",
                "image/png" => "png",
                "image/gif" => "gif",
                "image/webp" => "webp",
                "application/pdf" => "pdf",
                _ => "bin", // fallback
            };

            // Pastikan direktori upload ada
            let upload_dir = std::path::Path::new("uploads/assets/bukti-pelaporan");
            if !upload_dir.exists() {
                std::fs::create_dir_all(upload_dir)?;
            }

            // Generate nama file yang unik
            let filename = format!(
                "uploads/assets/bukti-pelaporan/{}.{}",
                Uuid::new_v4(),
                extension
            );
            let filepath = std::path::Path::new(&filename);
            let mut f = std::fs::File::create(filepath)?;

            // Tulis file
            while let Some(chunk) = field.try_next().await? {
                f.write_all(&chunk)?;
            }

            evidance_path = Some(filename);
        } else {
            // Handle field teks biasa
            let mut value = Vec::new();
            while let Some(chunk) = field.try_next().await? {
                value.extend_from_slice(&chunk);
            }
            let value_str = String::from_utf8(value).unwrap_or_default();

            match name.as_str() {
                "nama" => nama = value_str,
                "telepon" => telepon = value_str,
                "email" => email = value_str,
                "jenis_pesan" => jenis_pesan = value_str,
                "pesan" => pesan = value_str,
                "recaptcha_token" => recaptcha_token = value_str,
                _ => {}
            }
        }
    }

    // 2. Validasi field wajib
    if nama.trim().is_empty()
        || telepon.trim().is_empty()
        || email.trim().is_empty()
        || pesan.trim().is_empty()
    {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Field wajib tidak boleh kosong"
        })));
    }

    // 3. Validasi jenis pesan untuk pelaporan
    if jenis_pesan == "Pelaporan" && evidance_path.is_none() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Jenis pesan Pelaporan wajib menyertakan bukti"
        })));
    }

    // 4. Verifikasi reCAPTCHA ke Google
    let secret_key = env::var("RECAPTCHA_SECRET_KEY").map_err(|_| {
        actix_web::error::ErrorInternalServerError("RECAPTCHA_SECRET_KEY tidak diatur")
    })?;

    if recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Token reCAPTCHA tidak ditemukan"
        })));
    }

    let client = reqwest::Client::new();
    let verify_url = "https://www.google.com/recaptcha/api/siteverify";
    let response = client
        .post(verify_url)
        .form(&[
            ("secret", secret_key.as_str()),
            ("response", recaptcha_token.as_str()),
        ])
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let text = response
        .text()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    println!("🔍 reCAPTCHA response: {}", text);

    let body: RecaptchaResponse =
        serde_json::from_str(&text).map_err(actix_web::error::ErrorInternalServerError)?;

    if !body.success || body.score < 0.5 {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "status": "error",
            "message": "Verifikasi reCAPTCHA gagal, kemungkinan bot."
        })));
    }

    // 5. Sanitasi Input
    nama = clean(&nama.trim());
    telepon = clean(&telepon.trim());
    email = clean(&email.trim());
    jenis_pesan = clean(&jenis_pesan.trim());
    pesan = clean(&pesan.trim());

    // 6. Validasi email
    if !email.contains('@') {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Format email tidak valid"
        })));
    }

    // 7. Simpan ke database
    let result = sqlx::query!(
        r#"
        INSERT INTO contacts (nama, telepon, email, jenis_pesan, pesan, evidance)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        nama,
        telepon,
        email,
        jenis_pesan,
        pesan,
        evidance_path
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Pesan berhasil dikirim dan diverifikasi reCAPTCHA"
        }))),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Gagal menyimpan pesan ke database"
            })))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PengumumanResponse {
    announce: FetchPengumuman,
}
#[get("/api/pengumuman")]
pub async fn get_pengumuman(pool: web::Data<MySqlPool>) -> Result<HttpResponse, Error> {
    let result = sqlx::query_as::<_, FetchPengumuman>(
        "SELECT
            id,
            image,
            link
        FROM pengumuman LIMIT 1",
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(announce) => Ok(HttpResponse::Ok().json(PengumumanResponse { announce })),
        Err(sqlx::Error::RowNotFound) => {
            log::warn!("Data pengumuman tidak ditemukan di database");
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "Tidak ada data pengumuman"
            })))
        }
        Err(e) => {
            log::error!("Gagal mengambil data pengumuman: {:?}", e);
            Err(actix_web::error::ErrorInternalServerError(
                "Gagal mengambil data pengumuman",
            ))
        }
    }
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
