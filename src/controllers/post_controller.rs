<<<<<<< HEAD
// src/controllers/post_controller.rs
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, Result, delete, error::ErrorInternalServerError,
    get, post, put, web,
};
use chrono::NaiveDate;
use futures_util::TryStreamExt as _;
use rand::Rng;
use rand_distr::Alphanumeric;
use serde::{Deserialize, Serialize};
use slug::slugify;
use sqlx::{MySqlPool, prelude::FromRow, query, query_as};
use std::path::{Path, PathBuf};

use tokio::{fs, io::AsyncWriteExt};

use crate::auth;

//berita
#[derive(Serialize, FromRow, Debug)]
struct Category {
    id: i32,
    category_name: String,
}

#[get("/api/adminpanel/kategori-berita")]
pub async fn get_category(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    let kategori: Vec<Category> =
        sqlx::query_as::<_, Category>("SELECT id, category_name FROM categories")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(kategori))
}

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
    user_id: i32,
    author: String,
    sumber: Option<String>,
    approval: i32,
    status: i32,
}

#[get("/api/adminpanel/post/{id}")]
pub async fn get_post_by_id(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    let id: String = path.into_inner();

    let post = sqlx::query_as::<_, Post>(
        "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, user_id, status
         FROM posts
         WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let post = match post {
        Some(post) => post,
        None => return Err(actix_web::error::ErrorNotFound("Post tidak ditemukan")),
    };

    Ok(HttpResponse::Ok().json(post))
}

#[derive(serde::Serialize)]
struct ApiResponse<T: serde::Serialize> {
    message: String,
    data: Option<T>,
}

#[derive(serde::Serialize)]
struct PostIdResp {
    id: i64,
}

// ----------- Util -----------
fn random_string(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}

fn ensure_jpeg_ext(name_hint: Option<&str>) -> &'static str {
    // sebagian besar kiriman dari canvas.toBlob('image/jpeg') → .jpg
    // kalau mau fleksibel: deteksi mime. Di sini dipaksa jpg.
    let _ = name_hint;
    "jpg"
}

fn to_i32(s: &str) -> Option<i32> {
    s.trim().parse::<i32>().ok()
}

fn to_naivedate(s: &str) -> Option<NaiveDate> {
    // input "YYYY-MM-DD"
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

async fn ensure_dir(dir: &str) -> std::io::Result<()> {
    if !Path::new(dir).exists() {
        fs::create_dir_all(dir).await?;
    }
    Ok(())
}

// kumpulkan field teks dari multipart
#[derive(Default, Debug)]
struct PostFields {
    title: Option<String>,
    news_category: Option<String>,
    tanggal: Option<String>,
    caption: Option<String>,
    author: Option<String>,
    sumber: Option<String>,
    status: Option<String>,
    body: Option<String>,
    user_id: Option<i32>,
    created_by: Option<String>,
}

// parse multipart → fields + optional saved_path (photo baru)
async fn parse_post_multipart(
    mut payload: Multipart,
) -> Result<(PostFields, Option<String>), Error> {
    let mut fields = PostFields::default();
    let mut saved_file_path: Option<String> = None;

    // pastikan folder ada
    let upload_dir = "uploads/assets/posts";
    ensure_dir(upload_dir)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().cloned();
        let name = cd.as_ref().and_then(|c| c.get_name()).unwrap_or("");

        if name == "photo" {
            // simpan file
            // nama final: random + .jpg (default)
            let ext = ensure_jpeg_ext(cd.as_ref().and_then(|c| c.get_filename()));
            let filename = format!(
                "{}_{}.{}",
                chrono::Utc::now().timestamp_millis(),
                random_string(12),
                ext
            );
            let relative_path = format!("{}/{}", upload_dir, filename);

            let mut f = fs::File::create(&relative_path)
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                f.write_all(&chunk)
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
            }
            saved_file_path = Some(format!("{}", relative_path)); // simpan sebagai path relatif dari root server
        } else {
            // field teks
            let mut bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                bytes.extend_from_slice(&chunk);
            }
            let val = String::from_utf8(bytes).unwrap_or_default();

            match name {
                "title" => fields.title = Some(val),
                "news_category" => fields.news_category = Some(val),
                "tanggal" => fields.tanggal = Some(val),
                "caption" => fields.caption = Some(val),
                "author" => fields.author = Some(val),
                "sumber" => fields.sumber = Some(val),
                "status" => fields.status = Some(val),
                "body" => fields.body = Some(val),
                "created_by" => fields.created_by = Some(val),
                "user_id" => fields.user_id = Some(0),
                _ => {}
            }
        }
    }

    Ok((fields, saved_file_path))
}

// ----------- CREATE POST -----------
#[post("/api/adminpanel/berita")]
pub async fn create_post(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: Multipart,
) -> Result<impl Responder, Error> {
    // auth
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let (fields, saved_photo) = parse_post_multipart(payload).await?;
    let title = fields
        .title
        .ok_or_else(|| actix_web::error::ErrorBadRequest("title required"))?;
    let news_category = fields.news_category.unwrap_or_default();
    let caption = fields.caption.unwrap_or_default();
    let author = fields.author.unwrap_or_default();
    let sumber = fields.sumber.unwrap_or_default();
    let user_id = fields.user_id.unwrap_or_default();
    let created_by = fields.created_by.unwrap_or_default();
    let body = fields.body.unwrap_or_default();

    // tanggal
    let tanggal_str = fields
        .tanggal
        .ok_or_else(|| actix_web::error::ErrorBadRequest("tanggal required (YYYY-MM-DD)"))?;
    let tanggal = to_naivedate(&tanggal_str)
        .ok_or_else(|| actix_web::error::ErrorBadRequest("invalid tanggal format"))?;

    // status: hanya admin/superadmin boleh set; lainnya = 0
    let status_val = match claims.role.as_str() {
        "Superadmin" | "Administrator" | "Jurnalis" => {
            fields.status.and_then(|s| to_i32(&s)).unwrap_or(0)
        }
        _ => 0,
    };

    let slug = slugify(&title);
    let view: i64 = 0;
    let photo = saved_photo.unwrap_or_else(|| "".into());

    let id = sqlx::query(
        r#"
        INSERT INTO posts (title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, user_id, created_by, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&title)
    .bind(&slug)
    .bind(&news_category)
    .bind(tanggal) // NaiveDate akan dipetakan ke DATE
    .bind(view)
    .bind(&photo)
    .bind(&caption)
    .bind(&body)
    .bind(&author)
    .bind(&sumber)
    .bind(&user_id)
    .bind(&created_by)
    .bind(status_val)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    .last_insert_id();

    Ok(HttpResponse::Ok().json(ApiResponse {
        message: "Post created".into(),
        data: Some(PostIdResp { id: id as i64 }),
    }))
}

// ----------- UPDATE POST -----------
#[put("/api/adminpanel/berita/update/{id}")]
pub async fn update_post(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
    payload: Multipart,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();

    // auth
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // ambil data lama (untuk photo lama)
    let old_row = sqlx::query_as::<_, (Option<String>,)>("SELECT photo FROM posts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if old_row.is_none() {
        return Err(actix_web::error::ErrorNotFound("Post not found"));
    }
    let old_photo = old_row.unwrap().0;

    let (fields, saved_photo) = parse_post_multipart(payload).await?;

    // build field untuk update
    let title = fields.title;
    let mut slug: Option<String> = None;
    if let Some(ref t) = title {
        slug = Some(slugify(t));
    }

    let news_category = fields.news_category;
    let caption = fields.caption;
    let author = fields.author;
    let sumber = fields.sumber;
    let body = fields.body;

    // tanggal opsional
    let tanggal_opt = fields.tanggal.and_then(|s| to_naivedate(&s));

    // status: hanya admin/superadmin yang boleh mengubah
    let status_opt = match claims.role.as_str() {
        "Superadmin" | "Administrator" | "Jurnalis" => fields.status.and_then(|s| to_i32(&s)),
        _ => None, // selain itu tidak boleh mengubah status
    };

    // jika ada photo baru → pakai itu, dan hapus lama (jika ada)
    let final_photo = if let Some(newphoto) = saved_photo {
        // hapus foto lama di disk
        if let Some(old) = old_photo.as_deref() {
            if old.trim().len() > 0 {
                // old adalah path relatif yang kita simpan, ex: /uploads/assets/posts/xxx.jpg
                let disk_path = old.trim_start_matches('/');
                if Path::new(disk_path).exists() {
                    // ignore error delete
                    let _ = fs::remove_file(disk_path).await;
                }
            }
        }
        Some(newphoto)
    } else {
        None
    };

    let mut tx = pool
        .begin()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if let Some(ref t) = title {
        sqlx::query("UPDATE posts SET title = ? WHERE id = ?")
            .bind(t)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(s) = slug {
        sqlx::query("UPDATE posts SET slug = ? WHERE id = ?")
            .bind(s)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(nc) = news_category {
        sqlx::query("UPDATE posts SET news_category = ? WHERE id = ?")
            .bind(nc)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(c) = caption {
        sqlx::query("UPDATE posts SET caption = ? WHERE id = ?")
            .bind(c)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(a) = author {
        sqlx::query("UPDATE posts SET author = ? WHERE id = ?")
            .bind(a)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(s) = sumber {
        sqlx::query("UPDATE posts SET sumber = ? WHERE id = ?")
            .bind(s)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(b) = body {
        sqlx::query("UPDATE posts SET body = ? WHERE id = ?")
            .bind(b)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(d) = tanggal_opt {
        sqlx::query("UPDATE posts SET tanggal = ? WHERE id = ?")
            .bind(d)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(st) = status_opt {
        sqlx::query("UPDATE posts SET status = ? WHERE id = ?")
            .bind(st)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(p) = final_photo {
        sqlx::query("UPDATE posts SET photo = ? WHERE id = ?")
            .bind(p)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    tx.commit()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(ApiResponse::<PostIdResp> {
        message: "Post updated".into(),
        data: Some(PostIdResp { id }),
    }))
}

// ================== Delete Post ==================
#[derive(Serialize, FromRow, Debug)]
struct PostDelete {
    id: i32,
    photo: Option<String>,
}

#[delete("/api/adminpanel/post/{id}")]
pub async fn delete_post(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let id_to_delete = path.into_inner();
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    // Langkah 1: Ambil path file dari database
    let post_to_delete: Option<PostDelete> = query_as!(
        PostDelete,
        "SELECT id, photo FROM posts WHERE id = ?",
        id_to_delete
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    if post_to_delete.is_none() {
        return Ok(
            HttpResponse::NotFound().body(format!("Post with id {} not found", id_to_delete))
        );
    }

    let photo_path: Option<PathBuf> = post_to_delete.and_then(|c| c.photo).map(|filename| {
        // Gabungkan nama file dengan direktori upload
        Path::new("./").join(filename)
    });

    // Langkah 2: Hapus entri dari database
    let result = query!("DELETE FROM posts WHERE id = ?", id_to_delete)
        .execute(pool.get_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(
            HttpResponse::NotFound().body(format!("Post with id {} not found", id_to_delete))
        );
    }

    // Langkah 3: Hapus file dari sistem file (jika ada)
    if let Some(path_to_delete) = photo_path {
        if path_to_delete.exists() {
            if let Err(e) = fs::remove_file(&path_to_delete).await {
                eprintln!("Failed to delete file {}: {}", path_to_delete.display(), e);
            }
        }
    }

    Ok(HttpResponse::Ok().body(format!(
        "Post with id {} and its evidence deleted successfully",
        id_to_delete
    )))
}


#[derive(Serialize, FromRow, Debug)]
struct AllPostResponse {
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
#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
    q: Option<String>,
}

#[get("/api/adminpanel/post")]

pub async fn admin_get_all_berita(
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<Pagination>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    let page = pagination.page.unwrap_or(1).max(1); // Minimal page 1
    let limit = pagination.limit.unwrap_or(8).clamp(1, 50); // Batasi max 50 per page
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    let keyword_like = format!("%{}%", keyword);

    // Query dinamis dengan LIKE jika ada keyword
    let (posts, total): (Vec<AllPostResponse>, i64) = if keyword.is_empty() {
        let posts = sqlx::query_as::<_, AllPostResponse>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             ORDER BY tanggal DESC
             LIMIT ? OFFSET ?",
        )
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    } else {
        let posts = sqlx::query_as::<_, AllPostResponse>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE title LIKE ? OR body LIKE ?
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

        let total: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM posts WHERE title LIKE ? OR body LIKE ?")
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
=======
// src/controllers/post_controller.rs
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, Result, delete, error::ErrorInternalServerError,
    get, post, put, web,
};
use chrono::NaiveDate;
use futures_util::TryStreamExt as _;
use rand::Rng;
use rand_distr::Alphanumeric;
use serde::{Deserialize, Serialize};
use slug::slugify;
use sqlx::{MySqlPool, prelude::FromRow, query, query_as};
use std::path::{Path, PathBuf};

use tokio::{fs, io::AsyncWriteExt};

use crate::auth;

//berita
#[derive(Serialize, FromRow, Debug)]
struct Category {
    id: i32,
    category_name: String,
}

#[get("/api/adminpanel/kategori-berita")]
pub async fn get_category(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    let kategori: Vec<Category> =
        sqlx::query_as::<_, Category>("SELECT id, category_name FROM categories")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(kategori))
}

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
    user_id: i32,
    author: String,
    sumber: Option<String>,
    approval: i32,
    status: i32,
}

#[get("/api/adminpanel/post/{id}")]
pub async fn get_post_by_id(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    let id: String = path.into_inner();

    let post = sqlx::query_as::<_, Post>(
        "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, user_id, status
         FROM posts
         WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let post = match post {
        Some(post) => post,
        None => return Err(actix_web::error::ErrorNotFound("Post tidak ditemukan")),
    };

    Ok(HttpResponse::Ok().json(post))
}

#[derive(serde::Serialize)]
struct ApiResponse<T: serde::Serialize> {
    message: String,
    data: Option<T>,
}

#[derive(serde::Serialize)]
struct PostIdResp {
    id: i64,
}

// ----------- Util -----------
fn random_string(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
}

fn ensure_jpeg_ext(name_hint: Option<&str>) -> &'static str {
    // sebagian besar kiriman dari canvas.toBlob('image/jpeg') → .jpg
    // kalau mau fleksibel: deteksi mime. Di sini dipaksa jpg.
    let _ = name_hint;
    "jpg"
}

fn to_i32(s: &str) -> Option<i32> {
    s.trim().parse::<i32>().ok()
}

fn to_naivedate(s: &str) -> Option<NaiveDate> {
    // input "YYYY-MM-DD"
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

async fn ensure_dir(dir: &str) -> std::io::Result<()> {
    if !Path::new(dir).exists() {
        fs::create_dir_all(dir).await?;
    }
    Ok(())
}

// kumpulkan field teks dari multipart
#[derive(Default, Debug)]
struct PostFields {
    title: Option<String>,
    news_category: Option<String>,
    tanggal: Option<String>,
    caption: Option<String>,
    author: Option<String>,
    sumber: Option<String>,
    status: Option<String>,
    body: Option<String>,
    user_id: Option<i32>,
    created_by: Option<String>,
}

// parse multipart → fields + optional saved_path (photo baru)
async fn parse_post_multipart(
    mut payload: Multipart,
) -> Result<(PostFields, Option<String>), Error> {
    let mut fields = PostFields::default();
    let mut saved_file_path: Option<String> = None;

    // pastikan folder ada
    let upload_dir = "uploads/assets/posts";
    ensure_dir(upload_dir)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?
    {
        let cd = field.content_disposition().cloned();
        let name = cd.as_ref().and_then(|c| c.get_name()).unwrap_or("");

        if name == "photo" {
            // simpan file
            // nama final: random + .jpg (default)
            let ext = ensure_jpeg_ext(cd.as_ref().and_then(|c| c.get_filename()));
            let filename = format!(
                "{}_{}.{}",
                chrono::Utc::now().timestamp_millis(),
                random_string(12),
                ext
            );
            let relative_path = format!("{}/{}", upload_dir, filename);

            let mut f = fs::File::create(&relative_path)
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                f.write_all(&chunk)
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;
            }
            saved_file_path = Some(format!("{}", relative_path)); // simpan sebagai path relatif dari root server
        } else {
            // field teks
            let mut bytes = Vec::new();
            while let Some(chunk) = field
                .try_next()
                .await
                .map_err(actix_web::error::ErrorBadRequest)?
            {
                bytes.extend_from_slice(&chunk);
            }
            let val = String::from_utf8(bytes).unwrap_or_default();

            match name {
                "title" => fields.title = Some(val),
                "news_category" => fields.news_category = Some(val),
                "tanggal" => fields.tanggal = Some(val),
                "caption" => fields.caption = Some(val),
                "author" => fields.author = Some(val),
                "sumber" => fields.sumber = Some(val),
                "status" => fields.status = Some(val),
                "body" => fields.body = Some(val),
                "created_by" => fields.created_by = Some(val),
                "user_id" => fields.user_id = Some(0),
                _ => {}
            }
        }
    }

    Ok((fields, saved_file_path))
}

// ----------- CREATE POST -----------
#[post("/api/adminpanel/berita")]
pub async fn create_post(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    payload: Multipart,
) -> Result<impl Responder, Error> {
    // auth
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    let (fields, saved_photo) = parse_post_multipart(payload).await?;
    let title = fields
        .title
        .ok_or_else(|| actix_web::error::ErrorBadRequest("title required"))?;
    let news_category = fields.news_category.unwrap_or_default();
    let caption = fields.caption.unwrap_or_default();
    let author = fields.author.unwrap_or_default();
    let sumber = fields.sumber.unwrap_or_default();
    let user_id = fields.user_id.unwrap_or_default();
    let created_by = fields.created_by.unwrap_or_default();
    let body = fields.body.unwrap_or_default();

    // tanggal
    let tanggal_str = fields
        .tanggal
        .ok_or_else(|| actix_web::error::ErrorBadRequest("tanggal required (YYYY-MM-DD)"))?;
    let tanggal = to_naivedate(&tanggal_str)
        .ok_or_else(|| actix_web::error::ErrorBadRequest("invalid tanggal format"))?;

    // status: hanya admin/superadmin boleh set; lainnya = 0
    let status_val = match claims.role.as_str() {
        "Superadmin" | "Administrator" | "Jurnalis" => {
            fields.status.and_then(|s| to_i32(&s)).unwrap_or(0)
        }
        _ => 0,
    };

    let slug = slugify(&title);
    let view: i64 = 0;
    let photo = saved_photo.unwrap_or_else(|| "".into());

    let id = sqlx::query(
        r#"
        INSERT INTO posts (title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, user_id, created_by, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&title)
    .bind(&slug)
    .bind(&news_category)
    .bind(tanggal) // NaiveDate akan dipetakan ke DATE
    .bind(view)
    .bind(&photo)
    .bind(&caption)
    .bind(&body)
    .bind(&author)
    .bind(&sumber)
    .bind(&user_id)
    .bind(&created_by)
    .bind(status_val)
    .execute(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    .last_insert_id();

    Ok(HttpResponse::Ok().json(ApiResponse {
        message: "Post created".into(),
        data: Some(PostIdResp { id: id as i64 }),
    }))
}

// ----------- UPDATE POST -----------
#[put("/api/adminpanel/berita/update/{id}")]
pub async fn update_post(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    path: web::Path<i64>,
    payload: Multipart,
) -> Result<impl Responder, Error> {
    let id = path.into_inner();

    // auth
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    // ambil data lama (untuk photo lama)
    let old_row = sqlx::query_as::<_, (Option<String>,)>("SELECT photo FROM posts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if old_row.is_none() {
        return Err(actix_web::error::ErrorNotFound("Post not found"));
    }
    let old_photo = old_row.unwrap().0;

    let (fields, saved_photo) = parse_post_multipart(payload).await?;

    // build field untuk update
    let title = fields.title;
    let mut slug: Option<String> = None;
    if let Some(ref t) = title {
        slug = Some(slugify(t));
    }

    let news_category = fields.news_category;
    let caption = fields.caption;
    let author = fields.author;
    let sumber = fields.sumber;
    let body = fields.body;

    // tanggal opsional
    let tanggal_opt = fields.tanggal.and_then(|s| to_naivedate(&s));

    // status: hanya admin/superadmin yang boleh mengubah
    let status_opt = match claims.role.as_str() {
        "Superadmin" | "Administrator" | "Jurnalis" => fields.status.and_then(|s| to_i32(&s)),
        _ => None, // selain itu tidak boleh mengubah status
    };

    // jika ada photo baru → pakai itu, dan hapus lama (jika ada)
    let final_photo = if let Some(newphoto) = saved_photo {
        // hapus foto lama di disk
        if let Some(old) = old_photo.as_deref() {
            if old.trim().len() > 0 {
                // old adalah path relatif yang kita simpan, ex: /uploads/assets/posts/xxx.jpg
                let disk_path = old.trim_start_matches('/');
                if Path::new(disk_path).exists() {
                    // ignore error delete
                    let _ = fs::remove_file(disk_path).await;
                }
            }
        }
        Some(newphoto)
    } else {
        None
    };

    let mut tx = pool
        .begin()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if let Some(ref t) = title {
        sqlx::query("UPDATE posts SET title = ? WHERE id = ?")
            .bind(t)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(s) = slug {
        sqlx::query("UPDATE posts SET slug = ? WHERE id = ?")
            .bind(s)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(nc) = news_category {
        sqlx::query("UPDATE posts SET news_category = ? WHERE id = ?")
            .bind(nc)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(c) = caption {
        sqlx::query("UPDATE posts SET caption = ? WHERE id = ?")
            .bind(c)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(a) = author {
        sqlx::query("UPDATE posts SET author = ? WHERE id = ?")
            .bind(a)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(s) = sumber {
        sqlx::query("UPDATE posts SET sumber = ? WHERE id = ?")
            .bind(s)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(b) = body {
        sqlx::query("UPDATE posts SET body = ? WHERE id = ?")
            .bind(b)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(d) = tanggal_opt {
        sqlx::query("UPDATE posts SET tanggal = ? WHERE id = ?")
            .bind(d)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(st) = status_opt {
        sqlx::query("UPDATE posts SET status = ? WHERE id = ?")
            .bind(st)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }
    if let Some(p) = final_photo {
        sqlx::query("UPDATE posts SET photo = ? WHERE id = ?")
            .bind(p)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    tx.commit()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(ApiResponse::<PostIdResp> {
        message: "Post updated".into(),
        data: Some(PostIdResp { id }),
    }))
}

// ================== Delete Post ==================
#[derive(Serialize, FromRow, Debug)]
struct PostDelete {
    id: i32,
    photo: Option<String>,
}

#[delete("/api/adminpanel/post/{id}")]
pub async fn delete_post(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let id_to_delete = path.into_inner();
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    // Langkah 1: Ambil path file dari database
    let post_to_delete: Option<PostDelete> = query_as!(
        PostDelete,
        "SELECT id, photo FROM posts WHERE id = ?",
        id_to_delete
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(ErrorInternalServerError)?;

    if post_to_delete.is_none() {
        return Ok(
            HttpResponse::NotFound().body(format!("Post with id {} not found", id_to_delete))
        );
    }

    let photo_path: Option<PathBuf> = post_to_delete.and_then(|c| c.photo).map(|filename| {
        // Gabungkan nama file dengan direktori upload
        Path::new("./").join(filename)
    });

    // Langkah 2: Hapus entri dari database
    let result = query!("DELETE FROM posts WHERE id = ?", id_to_delete)
        .execute(pool.get_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    if result.rows_affected() == 0 {
        return Ok(
            HttpResponse::NotFound().body(format!("Post with id {} not found", id_to_delete))
        );
    }

    // Langkah 3: Hapus file dari sistem file (jika ada)
    if let Some(path_to_delete) = photo_path {
        if path_to_delete.exists() {
            if let Err(e) = fs::remove_file(&path_to_delete).await {
                eprintln!("Failed to delete file {}: {}", path_to_delete.display(), e);
            }
        }
    }

    Ok(HttpResponse::Ok().body(format!(
        "Post with id {} and its evidence deleted successfully",
        id_to_delete
    )))
}


#[derive(Serialize, FromRow, Debug)]
struct AllPostResponse {
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
#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
    q: Option<String>,
}

#[get("/api/adminpanel/post")]

pub async fn admin_get_all_berita(
    pool: web::Data<MySqlPool>,
    web::Query(pagination): web::Query<Pagination>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // autentikasi
    let claims =
        auth::verify_jwt(&req).map_err(|e| actix_web::error::ErrorUnauthorized(e.to_string()))?;

    if !["Superadmin", "Administrator", "Jurnalis"].contains(&claims.role.as_str()) {
        return Err(actix_web::error::ErrorForbidden(
            "Hanya Superadmin atau Administrator yang dapat mengakses",
        ));
    }
    let page = pagination.page.unwrap_or(1).max(1); // Minimal page 1
    let limit = pagination.limit.unwrap_or(8).clamp(1, 50); // Batasi max 50 per page
    let offset = (page - 1) * limit;
    let keyword = pagination.q.unwrap_or_default();

    let keyword_like = format!("%{}%", keyword);

    // Query dinamis dengan LIKE jika ada keyword
    let (posts, total): (Vec<AllPostResponse>, i64) = if keyword.is_empty() {
        let posts = sqlx::query_as::<_, AllPostResponse>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             ORDER BY tanggal DESC
             LIMIT ? OFFSET ?",
        )
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
            .fetch_one(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        (posts, total.0)
    } else {
        let posts = sqlx::query_as::<_, AllPostResponse>(
            "SELECT id, category_id, title, slug, news_category, tanggal, view, photo, caption, body, author, sumber, approval, status
             FROM posts
             WHERE title LIKE ? OR body LIKE ?
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

        let total: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM posts WHERE title LIKE ? OR body LIKE ?")
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
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
