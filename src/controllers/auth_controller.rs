<<<<<<< HEAD
//auth_controller.rs
use crate::models::user::{User, UserProfile};
use crate::{auth, utils};
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder,
    cookie::{Cookie, SameSite, time::Duration},
    get, post, web,
};
use ammonia::clean;
use bcrypt::{DEFAULT_COST, verify};
use chrono::Utc;
use futures::TryStreamExt;
use mail_send::mail_builder::MessageBuilder;
use mail_send::{Credentials, SmtpClientBuilder};
use mime::{APPLICATION_PDF, IMAGE_JPEG, IMAGE_PNG};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sodiumoxide::crypto::secretbox;
use sqlx::{MySqlPool, Row, prelude::FromRow};
use std::io::Write;
use std::{env, fs};
use uuid::Uuid;
use validator::Validate;
//register
#[derive(Deserialize, Validate, Debug)]
pub struct RegisterData {
    #[validate(length(equal = 16, message = "NIK harus 16 digit"))]
    pub nik: String,

    #[validate(length(min = 3, message = "Nama minimal 3 karakter"))]
    pub nama_lengkap: String,
    pub photo: Option<String>,

    #[validate(length(min = 1))]
    pub tempat_lahir: String,

    #[validate(length(min = 1))]
    pub tgl_lahir: String,

    pub jk: String,

    #[validate(email(message = "Email tidak valid"))]
    pub email: String,
    pub telepon: String,
    pub alamat: String,
    pub pendidikan_terakhir: String,
    pub jurusan: String,
    pub nama_instansi_pendidikan: String,
    pub id_hobi: Option<String>,
    pub posisi: String,
    pub tingkat_kepengurusan: String,
    pub jabatan: String,
    pub tingkat_penugasan: String,
    pub id_provinsi_domisili: Option<i32>,
    pub id_kabupaten_domisili: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub thn_tugas: Option<String>,
    pub id_minat: Option<String>,
    pub detail_minat: Option<String>,
    pub id_minat_2: Option<String>,
    pub detail_minat_2: Option<String>,
    pub id_bakat: Option<String>,
    pub detail_bakat: Option<String>,
    pub keterangan: Option<String>,
    pub agreement: Option<String>,
    pub status: Option<String>,
    pub no_piagam: Option<String>,
}
// Fungsi helper untuk parse ke i32 dengan debugging
fn parse_int(value: &str) -> Option<i32> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        println!("🔄 parse_int: empty string -> None");
        None
    } else {
        match trimmed.parse() {
            Ok(num) => {
                println!("🔄 parse_int: '{}' -> {}", trimmed, num);
                Some(num)
            }
            Err(e) => {
                println!("❌ parse_int: Gagal parse '{}': {}", trimmed, e);
                None
            }
        }
    }
}

#[post("/api/register")]
pub async fn register_user(
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let upload_dir = "uploads/assets/file/piagam";
    if let Err(e) = fs::create_dir_all(upload_dir) {
        eprintln!("Gagal buat folder upload: {:?}", e);
    }
    let mut recaptcha_token = String::new();
    let mut form_data = RegisterData {
        nik: "".into(),
        nama_lengkap: "".into(),
        photo: None,
        tempat_lahir: "".into(),
        tgl_lahir: "".into(),
        jk: "".into(),
        email: "".into(),
        telepon: "".into(),
        alamat: "".into(),
        pendidikan_terakhir: "".into(),
        jurusan: "".into(),
        nama_instansi_pendidikan: "".into(),
        id_hobi: None,
        posisi: "".into(),
        tingkat_kepengurusan: "".into(),
        jabatan: "".into(),
        tingkat_penugasan: "".into(),
        id_provinsi_domisili: None,
        id_kabupaten_domisili: None,
        id_provinsi: None,
        id_kabupaten: None,
        thn_tugas: None,
        id_minat: None,
        detail_minat: None,
        id_minat_2: None,
        detail_minat_2: None,
        id_bakat: None,
        detail_bakat: None,
        keterangan: None,
        agreement: None,
        status: None,
        no_piagam: None,
    };

    let mut file_piagam_path: Option<String> = None;

    // === Baca seluruh field dari multipart form ===
    while let Ok(Some(mut field)) = payload.try_next().await {
        let name = field.name().unwrap_or("").to_string();

        // === File Piagam ===
        if name == "file_piagam" {
            // jika user tidak mengupload file sama sekali, skip
            if let Some(cd) = field.content_disposition() {
                if cd.get_filename().is_none() {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(content_type) = field.content_type() {
                if content_type == &IMAGE_JPEG
                    || content_type == &IMAGE_PNG
                    || content_type == &APPLICATION_PDF
                {
                    let ext = if content_type == &APPLICATION_PDF {
                        "pdf"
                    } else {
                        "jpg"
                    };
                    let filename = format!("{}_{}.{}", Utc::now().timestamp(), Uuid::new_v4(), ext);
                    let filepath = format!("{}/{}", upload_dir, filename);

                    match fs::File::create(&filepath) {
                        Ok(mut f) => {
                            let mut file_size: u64 = 0;
                            while let Ok(Some(chunk)) = field.try_next().await {
                                file_size += chunk.len() as u64;
                                if file_size > 10 * 1024 * 1024 {
                                    return Ok(HttpResponse::BadRequest()
                                        .json("Ukuran file piagam maksimal 10MB"));
                                }
                                if let Err(_) = f.write_all(&chunk) {
                                    return Ok(HttpResponse::InternalServerError()
                                        .json("Gagal menulis file piagam"));
                                }
                            }
                            file_piagam_path = Some(filepath);
                        }
                        Err(_) => {
                            return Ok(HttpResponse::InternalServerError()
                                .json("Gagal menulis file piagam"));
                        }
                    }
                }
            }

            continue;
        }

        if name == "avatar" {
            eprintln!("📸 menerima field avatar...");

            let ct = field.content_type();
            eprintln!("📸 content_type = {:?}", ct);

            if let Some(content_type) = ct {
                if content_type == &IMAGE_JPEG || content_type == &IMAGE_PNG {
                    eprintln!("📸 tipe valid {:?}", content_type);
                    let ext = if content_type == &IMAGE_PNG {
                        "png"
                    } else {
                        "jpg"
                    };
                    let filename = format!(
                        "avatar_{}_{}.{}",
                        Utc::now().timestamp(),
                        Uuid::new_v4(),
                        ext
                    );

                    let upload_avatar_dir = "uploads/assets/images/avatars";
                    if let Err(e) = fs::create_dir_all(upload_avatar_dir) {
                        eprintln!("Gagal membuat direktori avatar: {:?}", e);
                        return Ok(HttpResponse::InternalServerError()
                            .json("Gagal membuat direktori avatar"));
                    }
                    fs::create_dir_all(upload_avatar_dir).ok();
                    let filepath = format!("{}/{}", upload_avatar_dir, filename);

                    match fs::File::create(&filepath) {
                        Ok(mut f) => {
                            let mut file_size: u64 = 0;
                            while let Ok(Some(chunk)) = field.try_next().await {
                                file_size += chunk.len() as u64;
                                if file_size > 10 * 1024 * 1024 {
                                    return Ok(HttpResponse::BadRequest()
                                        .json("Ukuran foto maksimal 10MB"));
                                }

                                // ✅ ganti map_err() dengan if let Err
                                if let Err(_) = f.write_all(&chunk) {
                                    return Ok(HttpResponse::InternalServerError()
                                        .json("Gagal menulis foto"));
                                }
                            }
                            eprintln!("📸 tersimpan di {:?}", filepath);
                            form_data.photo = Some(filepath);
                        }
                        Err(e) => {
                            eprintln!("❌ Gagal membuat file: {:?}", e);
                            return Ok(
                                HttpResponse::InternalServerError().json("Gagal menulis foto")
                            );
                        }
                    }
                } else {
                    eprintln!("⚠️ Tipe foto tidak valid: {:?}", content_type);
                }
            } else {
                eprintln!("⚠️ Tidak ada content-type di multipart field photo");
            }
            continue;
        }

        // === Field teks ===
        let mut value = Vec::new();
        while let Ok(Some(chunk)) = field.try_next().await {
            value.extend_from_slice(&chunk);
        }
        let text = String::from_utf8_lossy(&value).trim().to_string();
        let value_str = String::from_utf8(value).unwrap_or_default();

        // DEBUG: Tampilkan nilai yang diterima
        println!("📥 Field '{}' = '{}'", name, text);

        match name.as_str() {
            "nik" => form_data.nik = clean(&text),
            "nama_lengkap" => form_data.nama_lengkap = clean(&text),
            "tempat_lahir" => form_data.tempat_lahir = clean(&text),
            "tgl_lahir" => form_data.tgl_lahir = clean(&text),
            "jk" => form_data.jk = clean(&text),
            "email" => form_data.email = clean(&text),
            "telepon" => form_data.telepon = clean(&text),
            "alamat" => form_data.alamat = clean(&text),
            "pendidikan_terakhir" => form_data.pendidikan_terakhir = clean(&text),
            "jurusan" => form_data.jurusan = clean(&text),
            "nama_instansi_pendidikan" => form_data.nama_instansi_pendidikan = clean(&text),
            "id_hobi" => form_data.id_hobi = Some(clean(&text)),
            "posisi" => form_data.posisi = clean(&text),
            "tingkat_kepengurusan" => form_data.tingkat_kepengurusan = clean(&text),
            "jabatan" => form_data.jabatan = clean(&text),
            "tingkat_penugasan" => form_data.tingkat_penugasan = clean(&text),
            "id_provinsi_domisili" => {
                form_data.id_provinsi_domisili = parse_int(&text);
                println!(
                    "🔍 id_provinsi_domisili setelah parse: {:?}",
                    form_data.id_provinsi_domisili
                );
            }
            "id_kabupaten_domisili" => {
                form_data.id_kabupaten_domisili = parse_int(&text);
                println!(
                    "🔍 id_kabupaten_domisili setelah parse: {:?}",
                    form_data.id_kabupaten_domisili
                );
            }
            "id_provinsi" => {
                form_data.id_provinsi = parse_int(&text);
                println!("🔍 id_provinsi setelah parse: {:?}", form_data.id_provinsi);
            }
            "id_kabupaten" => {
                form_data.id_kabupaten = parse_int(&text);
                println!(
                    "🔍 id_kabupaten setelah parse: {:?}",
                    form_data.id_kabupaten
                );
            }
            "thn_tugas" => form_data.thn_tugas = Some(clean(&text)),
            "id_minat" => form_data.id_minat = Some(clean(&text)),
            "detail_minat" => form_data.detail_minat = Some(clean(&text)),
            "id_minat_2" => form_data.id_minat_2 = Some(clean(&text)),
            "detail_minat_2" => form_data.detail_minat_2 = Some(clean(&text)),
            "id_bakat" => form_data.id_bakat = Some(clean(&text)),
            "detail_bakat" => form_data.detail_bakat = Some(clean(&text)),
            "keterangan" => form_data.keterangan = Some(clean(&text)),
            "agreement" => form_data.agreement = Some(clean(&text)),
            "status" => form_data.status = Some(clean(&text)),
            "no_piagam" => form_data.no_piagam = Some(clean(&text)),
            "recaptchaToken" => recaptcha_token = value_str,
            _ => {}
        }
    }
    // Debug sebelum eksekusi query
    println!("🔍 DEBUG sebelum query:");
    println!("  id_provinsi: {:?}", form_data.id_provinsi);
    println!("  id_kabupaten: {:?}", form_data.id_kabupaten);
    println!(
        "  id_provinsi_domisili: {:?}",
        form_data.id_provinsi_domisili
    );
    println!(
        "  id_kabupaten_domisili: {:?}",
        form_data.id_kabupaten_domisili
    );
    // Verifikasi reCAPTCHA ke Google
    let secret_key =
        env::var("RECAPTCHA_SECRET_KEY").expect("RECAPTCHA_SECRET_KEY harus diatur di .env");

    if recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Token reCAPTCHA tidak ditemukan"
        })));
    }

    let client = Client::new();
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

    // === Validasi field ===
    if let Err(e) = form_data.validate() {
        return Ok(HttpResponse::BadRequest().json(e));
    }
    let phone = Some(utils::normalize_phone(&form_data.telepon));

    //key enkripsi

    let encryption_key_hex = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");
    let blind_index_key_hex = env::var("BLIND_INDEX_KEY").expect("BLIND_INDEX_KEY must be set");

    let encryption_key_bytes = hex::decode(&encryption_key_hex)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid hex encryption key"))?;
    let blind_index_key_bytes = hex::decode(&blind_index_key_hex)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid hex blind index key"))?;

    let key = secretbox::Key::from_slice(&encryption_key_bytes)
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Invalid encryption key size"))?;

    // Hasilkan blind index
    let nik_blind_index =
        utils::generate_blind_index(form_data.nik.as_bytes(), &blind_index_key_bytes);

    let email_norm = form_data.email.trim().to_ascii_lowercase();

    // Blind index
    let email_blind_index =
        utils::generate_blind_index(email_norm.as_bytes(), &blind_index_key_bytes);
    let telepon_blind_index = phone
        .as_ref() // Option<&String>
        .map(|p| utils::generate_blind_index(p.as_bytes(), &blind_index_key_bytes));

    // Enkripsi data pribadi
    let (nik_nonce, nik_ciphertext) = utils::encrypt_data(&form_data.nik.as_bytes(), &key);
    let (nama_nonce, nama_ciphertext) =
        utils::encrypt_data(&form_data.nama_lengkap.as_bytes(), &key);
    let (email_nonce, email_ciphertext) = utils::encrypt_data(email_norm.as_bytes(), &key);
    let (telepon_nonce, telepon_ciphertext) = utils::encrypt_data(&phone.unwrap().as_bytes(), &key);

    // === Cek apakah no_piagam sudah ada ===
    if let Some(ref no_piagam) = form_data.no_piagam {
        let existing_record = sqlx::query("SELECT id FROM pdp WHERE no_piagam = ?")
            .bind(no_piagam)
            .fetch_optional(pool.get_ref())
            .await;

        match existing_record {
            Ok(Some(row)) => {
                let existing_id: i64 = row.get("id");
                println!("Record ditemukan dengan ID: {}", existing_id);
                // === UPDATE data yang sudah ada ===
                let result = sqlx::query!(
                    r#"
                    UPDATE pdp SET
                    nik = ?, nama_lengkap = ?, tempat_lahir = ?, tgl_lahir = ?, jk = ?, email = ?, telepon = ?, alamat = ?,
                    pendidikan_terakhir = ?, jurusan = ?, nama_instansi_pendidikan = ?, id_hobi = ?,
                    posisi = ?, tingkat_kepengurusan = ?, jabatan = ?, tingkat_penugasan = ?,
                    id_provinsi_domisili = ?, id_kabupaten_domisili = ?, id_provinsi = ?, id_kabupaten = ?,
                    thn_tugas = ?, id_minat = ?, detail_minat = ?, id_minat_2 = ?, detail_minat_2 = ?,
                    id_bakat = ?, detail_bakat = ?, keterangan = ?, agreement = ?, status = ?,
                    file_piagam = ?, photo = ?,
                    nik_blind_index = ?, email_blind_index = ?, telepon_blind_index = ?,
                    nik_nonce = ?, nama_nonce = ?, email_nonce = ?, telepon_nonce = ?,
                    updated_at = CURRENT_TIMESTAMP
                    WHERE no_piagam = ?
                    "#,
                    nik_ciphertext,
                    nama_ciphertext,
                    form_data.tempat_lahir,
                    form_data.tgl_lahir,
                    form_data.jk,
                    email_ciphertext,
                    telepon_ciphertext,
                    form_data.alamat,
                    form_data.pendidikan_terakhir,
                    form_data.jurusan,
                    form_data.nama_instansi_pendidikan,
                    form_data.id_hobi,
                    form_data.posisi,
                    form_data.tingkat_kepengurusan,
                    form_data.jabatan,
                    form_data.tingkat_penugasan,
                    form_data.id_provinsi_domisili,
                    form_data.id_kabupaten_domisili,
                    form_data.id_provinsi,
                    form_data.id_kabupaten,
                    form_data.thn_tugas,
                    form_data.id_minat,
                    form_data.detail_minat,
                    form_data.id_minat_2,
                    form_data.detail_minat_2,
                    form_data.id_bakat,
                    form_data.detail_bakat,
                    form_data.keterangan,
                    form_data.agreement,
                    form_data.status,
                    file_piagam_path,
                    form_data.photo,
                    nik_blind_index,
                    email_blind_index,
                    telepon_blind_index,
                    nik_nonce.as_ref(),
                    nama_nonce.as_ref(),
                    email_nonce.as_ref(),
                    telepon_nonce.as_ref(),
                    no_piagam
                )
                .execute(pool.get_ref())
                .await;

                match result {
                    Ok(_) => return Ok(HttpResponse::Ok().json("Data berhasil diupdate")),
                    Err(e) => {
                        eprintln!("DB Update Error: {:?}", e);
                        return Ok(HttpResponse::InternalServerError()
                            .json("Terjadi kesalahan saat mengupdate data"));
                    }
                }
            }
            Ok(None) => {
                // Lanjut dengan INSERT baru
            }
            Err(e) => {
                eprintln!("DB Check Error: {:?}", e);
                return Ok(HttpResponse::InternalServerError()
                    .json("Terjadi kesalahan saat mengecek data"));
            }
        }
    }

    let id = generate_short_uuid(); // Generate UUID 10 karakter

    let result = sqlx::query!(
        r#"
        INSERT INTO pdp
        (id, nik, nama_lengkap, tempat_lahir, tgl_lahir, jk, email, telepon, alamat,
         pendidikan_terakhir, jurusan, nama_instansi_pendidikan, id_hobi,
         posisi, tingkat_kepengurusan, jabatan, tingkat_penugasan,
         id_provinsi_domisili, id_kabupaten_domisili, id_provinsi, id_kabupaten,
         thn_tugas, id_minat, detail_minat, id_minat_2, detail_minat_2,
         id_bakat, detail_bakat, keterangan, agreement, status, no_piagam, file_piagam, photo,
         nik_blind_index, email_blind_index, telepon_blind_index, nik_nonce, nama_nonce, email_nonce, telepon_nonce)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
            id,
            nik_ciphertext,
            nama_ciphertext,
            form_data.tempat_lahir,
            form_data.tgl_lahir,
            form_data.jk,
            email_ciphertext,
            telepon_ciphertext,
            form_data.alamat,
            form_data.pendidikan_terakhir,
            form_data.jurusan,
            form_data.nama_instansi_pendidikan,
            form_data.id_hobi,
            form_data.posisi,
            form_data.tingkat_kepengurusan,
            form_data.jabatan,
            form_data.tingkat_penugasan,
            form_data.id_provinsi_domisili,
            form_data.id_kabupaten_domisili,
            form_data.id_provinsi,
            form_data.id_kabupaten,
            form_data.thn_tugas,
            form_data.id_minat,
            form_data.detail_minat,
            form_data.id_minat_2,
            form_data.detail_minat_2,
            form_data.id_bakat,
            form_data.detail_bakat,
            form_data.keterangan,
            form_data.agreement,
            form_data.status,
            form_data.no_piagam,
            file_piagam_path,
            form_data.photo,
            nik_blind_index,
            email_blind_index,
            telepon_blind_index,
            nik_nonce.as_ref(),
            nama_nonce.as_ref(),
            email_nonce.as_ref(),
            telepon_nonce.as_ref(),
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Pendaftaran berhasil disimpan",
            "id": id // Kembalikan ID ke frontend jika diperlukan
        }))),
        Err(e) => {
            eprintln!("DB Error: {:?}", e);
            if let sqlx::Error::Database(db_err) = &e {
                if let Some(code) = db_err.code() {
                    if code == "23000" && db_err.message().contains("Duplicate entry") {
                        eprintln!("❌ Duplikat NIK terdeteksi");
                        return Ok(HttpResponse::Conflict().json("NIK sudah terdaftar"));
                    }
                }
            }

            Ok(HttpResponse::InternalServerError()
                .json("Terjadi kesalahan saat menyimpan ke database"))
        }
    }
}

#[get("/api/user")]
pub async fn get_user(req: HttpRequest, pool: web::Data<MySqlPool>) -> Result<HttpResponse, Error> {
    let claims = auth::verify_jwt(&req).map_err(|e| {
        log::error!("Verify JWT failed: {}", e);
        actix_web::error::ErrorUnauthorized(e.to_string())
    })?;

    let user = sqlx
        ::query_as::<_, UserProfile>(
            "SELECT id, name, email, role, address, avatar, phone, id_pdp, id_provinsi, id_kabupaten, created_at FROM users WHERE email = ?"
        )
        .bind(&claims.sub)
        .fetch_one(pool.get_ref()).await
        .map_err(|e| {
            log::error!("Gagal mengambil data pengguna: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengambil data pengguna")
        })?;

    let user_response = json!({
        "id": user.id,
        "name": user.name,
        "email": user.email,
        "role": user.role,
        "address": user.address.unwrap_or_default(),
        "avatar": user.avatar.unwrap_or_default(),
        "phone": user.phone.unwrap_or_default(),
        "id_pdp": user.id_pdp,
        "id_provinsi": user.id_provinsi.unwrap_or_default(),
        "id_kabupaten": user.id_kabupaten.unwrap_or_default(),
        "created_at": user.created_at,
    });

    Ok(HttpResponse::Ok().json(user_response))
}

//=======login====================================================================

// ---- Payload dari Frontend (JSON) ----
#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
    #[serde(rename = "recaptchaToken")]
    pub recaptcha_token: String,
}

#[derive(Deserialize)]
struct RecaptchaResponse {
    success: bool,
    #[serde(default)]
    score: f32,
}

#[post("/api/login")]
pub async fn login(
    pool: web::Data<MySqlPool>,
    payload: web::Json<LoginPayload>,
) -> Result<impl Responder, Error> {
    let email = payload.email.trim();
    let password = payload.password.trim();

    if email.is_empty() || password.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status":"error","message":"Email atau password kosong"
        })));
    }

    // ---- reCAPTCHA ----
    let secret_key = env::var("RECAPTCHA_SECRET_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("RECAPTCHA_SECRET_KEY not set"))?;
    if payload.recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status":"error","message":"Token reCAPTCHA tidak ditemukan"
        })));
    }
    let resp_text = Client::new()
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&[
            ("secret", secret_key.as_str()),
            ("response", payload.recaptcha_token.as_str()),
        ])
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .text()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let rec: RecaptchaResponse =
        serde_json::from_str(&resp_text).map_err(actix_web::error::ErrorInternalServerError)?;
    if !rec.success || rec.score < 0.5 {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "status":"error","message":"Verifikasi reCAPTCHA gagal/bot"
        })));
    }

    // ---- Ambil user ----
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, avatar, name, password, email, role, address, phone, id_pdp, id_provinsi, id_kabupaten, email_verified_at, remember_token, created_at FROM users WHERE email = ? LIMIT 1"#,
    )
    .bind(email)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("DB error get user: {:?}", e);
        actix_web::error::ErrorInternalServerError("DB error")
    })?
    .ok_or_else(|| actix_web::error::ErrorUnauthorized("Email tidak terdaftar"))?;

    // ---- Verifikasi password ----
    let ok = verify(password, &user.password).map_err(|e| {
        log::error!("bcrypt verify: {:?}", e);
        actix_web::error::ErrorInternalServerError("Verify error")
    })?;

    if !ok {
        return Err(actix_web::error::ErrorUnauthorized(
            "Kredensial tidak valid",
        ));
    }
    let row = sqlx::query_as::<_, (String, String, String, Option<String>)>(
        "SELECT id, password, role, id_pdp FROM users WHERE email=? LIMIT 1",
    )
    .bind(&payload.email)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorUnauthorized)?;

    if !bcrypt::verify(&payload.password, &row.1).map_err(actix_web::error::ErrorUnauthorized)? {
        return Err(actix_web::error::ErrorUnauthorized("Password salah"));
    }

    let token = auth::generate_jwt(&user).map_err(|e| {
        log::error!("Gagal menghasilkan JWT: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menghasilkan token")
    })?;

    let access_cookie = Cookie::build("access_token", token.clone())
        .path("/")
        .http_only(true)
        .secure(false) // false untuk development (HTTP)
        .same_site(SameSite::Lax) // Lax untuk cross-origin
        .max_age(Duration::days(4))
        .finish();

    Ok(HttpResponse::Ok().cookie(access_cookie).json(json!({
        "message": "Berhasil login",
        "role": user.role,
    })))
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
    #[serde(rename = "recaptchaToken")]
    pub recaptcha_token: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
    pub confirm_password: String,
    #[serde(rename = "recaptchaToken")]
    pub recaptcha_token: String,
}

#[post("/api/forgot-password")]
pub async fn forgot_password(
    pool: web::Data<MySqlPool>,
    payload: web::Json<ForgotPasswordRequest>,
) -> Result<impl Responder, Error> {
    let email = payload.email.trim().to_lowercase();

    if email.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Email harus diisi"
        })));
    }

    // Verifikasi reCAPTCHA
    let secret_key = env::var("RECAPTCHA_SECRET_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("RECAPTCHA_SECRET_KEY not set"))?;

    if payload.recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Token reCAPTCHA tidak ditemukan"
        })));
    }

    let client = Client::new();
    let verify_url = "https://www.google.com/recaptcha/api/siteverify";

    let response = client
        .post(verify_url)
        .form(&[
            ("secret", secret_key.as_str()),
            ("response", payload.recaptcha_token.as_str()),
        ])
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let recaptcha_response: RecaptchaResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if !recaptcha_response.success || recaptcha_response.score < 0.5 {
        return Ok(HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Verifikasi reCAPTCHA gagal"
        })));
    }

    // Cek apakah email terdaftar
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, password, address, avatar, phone, email_verified_at, remember_token, id_pdp, id_provinsi, id_kabupaten, created_at FROM users WHERE email = ?"
    )
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error checking email: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Untuk keamanan, selalu return success meskipun email tidak ditemukan
    if user.is_none() {
        log::info!("Forgot password request for non-existent email: {}", email);
        return Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Jika email terdaftar, tautan reset password akan dikirim"
        })));
    }

    let user = user.unwrap();

    // Generate reset token
    let reset_token = Uuid::new_v4().to_string();
    let token_expires = chrono::Utc::now() + chrono::Duration::hours(1); // Token berlaku 1 jam

    // Simpan token ke database
    sqlx::query("INSERT INTO password_reset_tokens (email, token, expires_at) VALUES (?, ?, ?)")
        .bind(&email)
        .bind(&reset_token)
        .bind(token_expires)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to save reset token: {}", e);
            actix_web::error::ErrorInternalServerError("Gagal menyimpan token reset")
        })?;

    // Kirim email reset password
    let reset_url = format!(
        "{}/reset-password/{}",
        env::var("FRONTEND_URL").unwrap_or_else(|_| "https://dppi.bpip.go.id".into()),
        reset_token
    );

    let email_result = send_reset_password_email(&user.email, &user.name, &reset_url).await;

    if let Err(e) = email_result {
        log::error!("Failed to send reset password email: {}", e);
        // Tetap return success untuk keamanan
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Jika email terdaftar, tautan reset password akan dikirim"
    })))
}

#[post("/api/reset-password")]
pub async fn reset_password(
    pool: web::Data<MySqlPool>,
    payload: web::Json<ResetPasswordRequest>,
) -> Result<impl Responder, Error> {
    let data = payload.into_inner();

    // Validasi input
    if data.new_password != data.confirm_password {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Password baru dan konfirmasi password tidak sama"
        })));
    }

    if data.new_password.len() < 6 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Password minimal 6 karakter"
        })));
    }

    // Verifikasi reCAPTCHA
    let secret_key = env::var("RECAPTCHA_SECRET_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("RECAPTCHA_SECRET_KEY not set"))?;

    if data.recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Token reCAPTCHA tidak ditemukan"
        })));
    }

    let client = Client::new();
    let verify_url = "https://www.google.com/recaptcha/api/siteverify";

    let response = client
        .post(verify_url)
        .form(&[
            ("secret", secret_key.as_str()),
            ("response", data.recaptcha_token.as_str()),
        ])
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let recaptcha_response: RecaptchaResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if !recaptcha_response.success || recaptcha_response.score < 0.5 {
        return Ok(HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Verifikasi reCAPTCHA gagal"
        })));
    }

    // Validasi token
    let token_record: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT email, expires_at FROM password_reset_tokens WHERE token = ? AND used = false",
    )
    .bind(&data.token)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error checking token: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let (email, expires_at) = match token_record {
        Some(record) => record,
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Token reset password tidak valid atau sudah digunakan"
            })));
        }
    };

    // Cek expiry
    if chrono::Utc::now() > expires_at {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Token reset password sudah kadaluarsa"
        })));
    }

    // Hash password baru
    let hashed_password = bcrypt::hash(&data.new_password, DEFAULT_COST).map_err(|e| {
        log::error!("Failed to hash password: {}", e);
        actix_web::error::ErrorInternalServerError("Gagal mengenkripsi password")
    })?;

    // Update password user
    let result = sqlx::query(
        "UPDATE users SET password = ?, updated_at = CURRENT_TIMESTAMP WHERE email = ?",
    )
    .bind(&hashed_password)
    .bind(&email)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Failed to update password: {}", e);
        actix_web::error::ErrorInternalServerError("Gagal mengupdate password")
    })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "User tidak ditemukan"
        })));
    }

    // PERBAIKAN: Tandai token sebagai sudah digunakan (tanpa operator ?)
    let _ = sqlx::query("UPDATE password_reset_tokens SET used = true WHERE token = ?")
        .bind(&data.token)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to mark token as used: {}", e);
        });

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Password berhasil direset"
    })))
}
// Fungsi helper untuk mengirim email reset password
async fn send_reset_password_email(to: &str, name: &str, reset_url: &str) -> Result<(), String> {
    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "DPPI".into());
    let from_name =
        env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Sekretariat DPPI BPIP RI".into());
    let from_addr =
        env::var("SMTP_FROM_ADDRESS").map_err(|_| "SMTP_FROM_ADDRESS missing".to_string())?;

    let host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST missing".to_string())?;
    let port: u16 = env::var("SMTP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(587);
    let user = env::var("SMTP_USER").map_err(|_| "SMTP_USER missing".to_string())?;
    let pass = env::var("SMTP_PASS").map_err(|_| "SMTP_PASS missing".to_string())?;
    let enc = env::var("SMTP_ENCRYPTION")
        .unwrap_or_else(|_| "STARTTLS".into())
        .to_uppercase();

    let subject = format!("[{}] Reset Password", app_name);

    let text = format!(
        "Halo, {name}\n\n\
        Anda menerima email ini karena meminta reset password untuk akun Anda.\n\n\
        Silakan klik tautan berikut untuk reset password (berlaku 1 jam):\n\
        {reset_url}\n\n\
        Jika Anda tidak meminta reset password, abaikan email ini.\n\n\
        — Tim {app_name}"
    );

    let html = format!(
        r#"<div style="font-family:Arial,sans-serif; max-width:600px; margin:0 auto;">
            <h1 style="color:#2c5aa0; text-align:center;">SALAM PANCASILA</h1>
            <h2 style="color:#333;">Halo, {name}</h2>

            <p>Anda menerima email ini karena meminta reset password untuk akun Anda.</p>

            <div style="text-align:center; margin:30px 0;">
                <a href="{reset_url}" style="background-color:#2c5aa0; color:white; padding:12px 24px; text-decoration:none; border-radius:4px; display:inline-block;">
                    Reset Password
                </a>
            </div>

            <p style="color:#666; font-size:14px;">
                Tautan ini berlaku selama 1 jam.<br>
                Jika Anda tidak meminta reset password, abaikan email ini.
            </p>

            <p style="margin-top:30px;">Terima kasih.<br>— Tim {app_name}</p>

            <div style="margin-top:30px; padding:15px; background:#f8f9fa; border:1px solid #e9ecef; border-radius:5px; text-align:center; font-size:12px; color:#6c757d;">
                <p style="margin:0;">
                    <strong>📧 EMAIL NO-REPLY</strong><br>
                    Email ini dikirim secara otomatis oleh sistem. <br>
                    <strong>Mohon untuk tidak membalas email ini.</strong><br>
                    Jika Anda membutuhkan bantuan, silakan hubungi administrator.
                </p>
            </div>
        </div>"#,
        name = name,
        reset_url = reset_url,
        app_name = app_name
    );

    let message = MessageBuilder::new()
        .from((from_name.as_str(), from_addr.as_str()))
        .to((name, to))
        .subject(subject)
        .text_body(text)
        .html_body(html);

    let mut client_builder = SmtpClientBuilder::new(host.as_str(), port)
        .credentials(Credentials::new(user.as_str(), pass.as_str()));

    client_builder = match enc.as_str() {
        "SSL" | "SMTPS" => client_builder.implicit_tls(true),
        "PLAIN" | "NONE" => client_builder.implicit_tls(false),
        _ => client_builder.implicit_tls(false),
    };

    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .send(message)
        .await
        .map_err(|e| format!("Failed to send reset password email: {}", e))?;

    Ok(())
}

#[post("/api/logout")]
pub async fn logout() -> Result<impl Responder, Error> {
    // ✅ FIXED LOGOUT COOKIE (harus sama persis dengan login)
    let access_cookie = Cookie::build("access_token", "")
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax)
        // .domain("127.0.0.1")
        .max_age(Duration::seconds(0))
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .json(json!({ "message": "Berhasil logout" })))
}
//jabatan
// struktur Jabatan
#[derive(Serialize, FromRow, Debug)]
struct Jabatan {
    id: i32,
    nama_jabatan: String,
}

#[derive(Serialize, Debug)]
struct JabatanResponse {
    jabatan_pusat: Vec<Jabatan>,
    jabatan_provinsi: Vec<Jabatan>,
    jabatan_kabupaten: Vec<Jabatan>,
}

#[get("/api/jabatan")]
pub async fn get_jabatan(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // A. Ambil data Jabatan Pusat
    let jabatan: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // B. Ambil data Jabatan Provinsi
    let jabatan_provinsi: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan_provinsi")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // C. Ambil data Jabatan Kabupaten
    let jabatan_kabupaten: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan_kabupaten")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    let response_data = JabatanResponse {
        jabatan_pusat: jabatan,
        jabatan_provinsi: jabatan_provinsi,
        jabatan_kabupaten: jabatan_kabupaten,
    };

    Ok(HttpResponse::Ok().json(response_data))
}

// minat
// struktur minat
#[derive(Serialize, FromRow, Debug)]
struct Minat {
    id: i32,
    kategori_minat: String,
}

#[derive(Serialize, FromRow, Debug)]
struct DetailMinat {
    id: i32,
    id_minat: i32,
    detail_minat: String,
}

#[get("/api/minat")]
pub async fn get_minat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Ambil data minat
    let minat: Vec<Minat> = sqlx::query_as::<_, Minat>("SELECT id, kategori_minat FROM minat")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(minat))
}

#[get("/api/detail-minat")]
pub async fn get_detail_minat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Ambil data minat
    let detail_minat: Vec<DetailMinat> =
        sqlx::query_as::<_, DetailMinat>("SELECT id, id_minat, detail_minat FROM detail_minat")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(detail_minat))
}

//bakat
// struktur bakat
#[derive(Serialize, FromRow, Debug)]
struct Bakat {
    id: i32,
    kategori_bakat: String,
}

#[derive(Serialize, FromRow, Debug)]
struct DetailBakat {
    id: i32,
    id_bakat: i32,
    detail_bakat: String,
}
#[get("/api/bakat")]
pub async fn get_bakat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Ambil data bakat
    let bakat: Vec<Bakat> = sqlx::query_as::<_, Bakat>("SELECT id, kategori_bakat FROM bakat")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(bakat))
}

#[get("/api/detail-bakat")]
pub async fn get_detail_bakat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Ambil data bakat
    let detail_bakat: Vec<DetailBakat> =
        sqlx::query_as::<_, DetailBakat>("SELECT id, id_bakat, detail_bakat FROM detail_bakat")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(detail_bakat))
}

//hobi
#[derive(Serialize, FromRow, Debug)]
struct Hobi {
    id: i32,
    kategori_hobi: String,
}

#[get("/api/hobi")]
pub async fn get_hobi(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let hobi: Vec<Hobi> = sqlx::query_as::<_, Hobi>("SELECT id, kategori_hobi FROM hobi")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(hobi))
}

fn generate_short_uuid() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    raw[..10].to_uppercase()
}
=======
//auth_controller.rs
use crate::models::user::{User, UserProfile};
use crate::{auth, utils};
use actix_multipart::Multipart;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder,
    cookie::{Cookie, SameSite, time::Duration},
    get, post, web,
};
use ammonia::clean;
use bcrypt::{DEFAULT_COST, verify};
use chrono::Utc;
use futures::TryStreamExt;
use mail_send::mail_builder::MessageBuilder;
use mail_send::{Credentials, SmtpClientBuilder};
use mime::{APPLICATION_PDF, IMAGE_JPEG, IMAGE_PNG};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sodiumoxide::crypto::secretbox;
use sqlx::{MySqlPool, Row, prelude::FromRow};
use std::io::Write;
use std::{env, fs};
use uuid::Uuid;
use validator::Validate;
//register
#[derive(Deserialize, Validate, Debug)]
pub struct RegisterData {
    #[validate(length(equal = 16, message = "NIK harus 16 digit"))]
    pub nik: String,

    #[validate(length(min = 3, message = "Nama minimal 3 karakter"))]
    pub nama_lengkap: String,
    pub photo: Option<String>,

    #[validate(length(min = 1))]
    pub tempat_lahir: String,

    #[validate(length(min = 1))]
    pub tgl_lahir: String,

    pub jk: String,

    #[validate(email(message = "Email tidak valid"))]
    pub email: String,
    pub telepon: String,
    pub alamat: String,
    pub pendidikan_terakhir: String,
    pub jurusan: String,
    pub nama_instansi_pendidikan: String,
    pub id_hobi: Option<String>,
    pub posisi: String,
    pub tingkat_kepengurusan: String,
    pub jabatan: String,
    pub tingkat_penugasan: String,
    pub id_provinsi_domisili: Option<i32>,
    pub id_kabupaten_domisili: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub thn_tugas: Option<String>,
    pub id_minat: Option<String>,
    pub detail_minat: Option<String>,
    pub id_minat_2: Option<String>,
    pub detail_minat_2: Option<String>,
    pub id_bakat: Option<String>,
    pub detail_bakat: Option<String>,
    pub keterangan: Option<String>,
    pub agreement: Option<String>,
    pub status: Option<String>,
    pub no_piagam: Option<String>,
}
// Fungsi helper untuk parse ke i32 dengan debugging
fn parse_int(value: &str) -> Option<i32> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        println!("🔄 parse_int: empty string -> None");
        None
    } else {
        match trimmed.parse() {
            Ok(num) => {
                println!("🔄 parse_int: '{}' -> {}", trimmed, num);
                Some(num)
            }
            Err(e) => {
                println!("❌ parse_int: Gagal parse '{}': {}", trimmed, e);
                None
            }
        }
    }
}

#[post("/api/register")]
pub async fn register_user(
    pool: web::Data<MySqlPool>,
    mut payload: Multipart,
) -> Result<impl Responder, Error> {
    let upload_dir = "uploads/assets/file/piagam";
    if let Err(e) = fs::create_dir_all(upload_dir) {
        eprintln!("Gagal buat folder upload: {:?}", e);
    }
    let mut recaptcha_token = String::new();
    let mut form_data = RegisterData {
        nik: "".into(),
        nama_lengkap: "".into(),
        photo: None,
        tempat_lahir: "".into(),
        tgl_lahir: "".into(),
        jk: "".into(),
        email: "".into(),
        telepon: "".into(),
        alamat: "".into(),
        pendidikan_terakhir: "".into(),
        jurusan: "".into(),
        nama_instansi_pendidikan: "".into(),
        id_hobi: None,
        posisi: "".into(),
        tingkat_kepengurusan: "".into(),
        jabatan: "".into(),
        tingkat_penugasan: "".into(),
        id_provinsi_domisili: None,
        id_kabupaten_domisili: None,
        id_provinsi: None,
        id_kabupaten: None,
        thn_tugas: None,
        id_minat: None,
        detail_minat: None,
        id_minat_2: None,
        detail_minat_2: None,
        id_bakat: None,
        detail_bakat: None,
        keterangan: None,
        agreement: None,
        status: None,
        no_piagam: None,
    };

    let mut file_piagam_path: Option<String> = None;

    // === Baca seluruh field dari multipart form ===
    while let Ok(Some(mut field)) = payload.try_next().await {
        let name = field.name().unwrap_or("").to_string();

        // === File Piagam ===
        if name == "file_piagam" {
            // jika user tidak mengupload file sama sekali, skip
            if let Some(cd) = field.content_disposition() {
                if cd.get_filename().is_none() {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(content_type) = field.content_type() {
                if content_type == &IMAGE_JPEG
                    || content_type == &IMAGE_PNG
                    || content_type == &APPLICATION_PDF
                {
                    let ext = if content_type == &APPLICATION_PDF {
                        "pdf"
                    } else {
                        "jpg"
                    };
                    let filename = format!("{}_{}.{}", Utc::now().timestamp(), Uuid::new_v4(), ext);
                    let filepath = format!("{}/{}", upload_dir, filename);

                    match fs::File::create(&filepath) {
                        Ok(mut f) => {
                            let mut file_size: u64 = 0;
                            while let Ok(Some(chunk)) = field.try_next().await {
                                file_size += chunk.len() as u64;
                                if file_size > 10 * 1024 * 1024 {
                                    return Ok(HttpResponse::BadRequest()
                                        .json("Ukuran file piagam maksimal 10MB"));
                                }
                                if let Err(_) = f.write_all(&chunk) {
                                    return Ok(HttpResponse::InternalServerError()
                                        .json("Gagal menulis file piagam"));
                                }
                            }
                            file_piagam_path = Some(filepath);
                        }
                        Err(_) => {
                            return Ok(HttpResponse::InternalServerError()
                                .json("Gagal menulis file piagam"));
                        }
                    }
                }
            }

            continue;
        }

        if name == "avatar" {
            eprintln!("📸 menerima field avatar...");

            let ct = field.content_type();
            eprintln!("📸 content_type = {:?}", ct);

            if let Some(content_type) = ct {
                if content_type == &IMAGE_JPEG || content_type == &IMAGE_PNG {
                    eprintln!("📸 tipe valid {:?}", content_type);
                    let ext = if content_type == &IMAGE_PNG {
                        "png"
                    } else {
                        "jpg"
                    };
                    let filename = format!(
                        "avatar_{}_{}.{}",
                        Utc::now().timestamp(),
                        Uuid::new_v4(),
                        ext
                    );

                    let upload_avatar_dir = "uploads/assets/images/avatars";
                    if let Err(e) = fs::create_dir_all(upload_avatar_dir) {
                        eprintln!("Gagal membuat direktori avatar: {:?}", e);
                        return Ok(HttpResponse::InternalServerError()
                            .json("Gagal membuat direktori avatar"));
                    }
                    fs::create_dir_all(upload_avatar_dir).ok();
                    let filepath = format!("{}/{}", upload_avatar_dir, filename);

                    match fs::File::create(&filepath) {
                        Ok(mut f) => {
                            let mut file_size: u64 = 0;
                            while let Ok(Some(chunk)) = field.try_next().await {
                                file_size += chunk.len() as u64;
                                if file_size > 10 * 1024 * 1024 {
                                    return Ok(HttpResponse::BadRequest()
                                        .json("Ukuran foto maksimal 10MB"));
                                }

                                // ✅ ganti map_err() dengan if let Err
                                if let Err(_) = f.write_all(&chunk) {
                                    return Ok(HttpResponse::InternalServerError()
                                        .json("Gagal menulis foto"));
                                }
                            }
                            eprintln!("📸 tersimpan di {:?}", filepath);
                            form_data.photo = Some(filepath);
                        }
                        Err(e) => {
                            eprintln!("❌ Gagal membuat file: {:?}", e);
                            return Ok(
                                HttpResponse::InternalServerError().json("Gagal menulis foto")
                            );
                        }
                    }
                } else {
                    eprintln!("⚠️ Tipe foto tidak valid: {:?}", content_type);
                }
            } else {
                eprintln!("⚠️ Tidak ada content-type di multipart field photo");
            }
            continue;
        }

        // === Field teks ===
        let mut value = Vec::new();
        while let Ok(Some(chunk)) = field.try_next().await {
            value.extend_from_slice(&chunk);
        }
        let text = String::from_utf8_lossy(&value).trim().to_string();
        let value_str = String::from_utf8(value).unwrap_or_default();

        // DEBUG: Tampilkan nilai yang diterima
        println!("📥 Field '{}' = '{}'", name, text);

        match name.as_str() {
            "nik" => form_data.nik = clean(&text),
            "nama_lengkap" => form_data.nama_lengkap = clean(&text),
            "tempat_lahir" => form_data.tempat_lahir = clean(&text),
            "tgl_lahir" => form_data.tgl_lahir = clean(&text),
            "jk" => form_data.jk = clean(&text),
            "email" => form_data.email = clean(&text),
            "telepon" => form_data.telepon = clean(&text),
            "alamat" => form_data.alamat = clean(&text),
            "pendidikan_terakhir" => form_data.pendidikan_terakhir = clean(&text),
            "jurusan" => form_data.jurusan = clean(&text),
            "nama_instansi_pendidikan" => form_data.nama_instansi_pendidikan = clean(&text),
            "id_hobi" => form_data.id_hobi = Some(clean(&text)),
            "posisi" => form_data.posisi = clean(&text),
            "tingkat_kepengurusan" => form_data.tingkat_kepengurusan = clean(&text),
            "jabatan" => form_data.jabatan = clean(&text),
            "tingkat_penugasan" => form_data.tingkat_penugasan = clean(&text),
            "id_provinsi_domisili" => {
                form_data.id_provinsi_domisili = parse_int(&text);
                println!(
                    "🔍 id_provinsi_domisili setelah parse: {:?}",
                    form_data.id_provinsi_domisili
                );
            }
            "id_kabupaten_domisili" => {
                form_data.id_kabupaten_domisili = parse_int(&text);
                println!(
                    "🔍 id_kabupaten_domisili setelah parse: {:?}",
                    form_data.id_kabupaten_domisili
                );
            }
            "id_provinsi" => {
                form_data.id_provinsi = parse_int(&text);
                println!("🔍 id_provinsi setelah parse: {:?}", form_data.id_provinsi);
            }
            "id_kabupaten" => {
                form_data.id_kabupaten = parse_int(&text);
                println!(
                    "🔍 id_kabupaten setelah parse: {:?}",
                    form_data.id_kabupaten
                );
            }
            "thn_tugas" => form_data.thn_tugas = Some(clean(&text)),
            "id_minat" => form_data.id_minat = Some(clean(&text)),
            "detail_minat" => form_data.detail_minat = Some(clean(&text)),
            "id_minat_2" => form_data.id_minat_2 = Some(clean(&text)),
            "detail_minat_2" => form_data.detail_minat_2 = Some(clean(&text)),
            "id_bakat" => form_data.id_bakat = Some(clean(&text)),
            "detail_bakat" => form_data.detail_bakat = Some(clean(&text)),
            "keterangan" => form_data.keterangan = Some(clean(&text)),
            "agreement" => form_data.agreement = Some(clean(&text)),
            "status" => form_data.status = Some(clean(&text)),
            "no_piagam" => form_data.no_piagam = Some(clean(&text)),
            "recaptchaToken" => recaptcha_token = value_str,
            _ => {}
        }
    }
    // Debug sebelum eksekusi query
    println!("🔍 DEBUG sebelum query:");
    println!("  id_provinsi: {:?}", form_data.id_provinsi);
    println!("  id_kabupaten: {:?}", form_data.id_kabupaten);
    println!(
        "  id_provinsi_domisili: {:?}",
        form_data.id_provinsi_domisili
    );
    println!(
        "  id_kabupaten_domisili: {:?}",
        form_data.id_kabupaten_domisili
    );
    // Verifikasi reCAPTCHA ke Google
    let secret_key =
        env::var("RECAPTCHA_SECRET_KEY").expect("RECAPTCHA_SECRET_KEY harus diatur di .env");

    if recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Token reCAPTCHA tidak ditemukan"
        })));
    }

    let client = Client::new();
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

    // === Validasi field ===
    if let Err(e) = form_data.validate() {
        return Ok(HttpResponse::BadRequest().json(e));
    }
    let phone = Some(utils::normalize_phone(&form_data.telepon));

    //key enkripsi

    let encryption_key_hex = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set");
    let blind_index_key_hex = env::var("BLIND_INDEX_KEY").expect("BLIND_INDEX_KEY must be set");

    let encryption_key_bytes = hex::decode(&encryption_key_hex)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid hex encryption key"))?;
    let blind_index_key_bytes = hex::decode(&blind_index_key_hex)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid hex blind index key"))?;

    let key = secretbox::Key::from_slice(&encryption_key_bytes)
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Invalid encryption key size"))?;

    // Hasilkan blind index
    let nik_blind_index =
        utils::generate_blind_index(form_data.nik.as_bytes(), &blind_index_key_bytes);

    let email_norm = form_data.email.trim().to_ascii_lowercase();

    // Blind index
    let email_blind_index =
        utils::generate_blind_index(email_norm.as_bytes(), &blind_index_key_bytes);
    let telepon_blind_index = phone
        .as_ref() // Option<&String>
        .map(|p| utils::generate_blind_index(p.as_bytes(), &blind_index_key_bytes));

    // Enkripsi data pribadi
    let (nik_nonce, nik_ciphertext) = utils::encrypt_data(&form_data.nik.as_bytes(), &key);
    let (nama_nonce, nama_ciphertext) =
        utils::encrypt_data(&form_data.nama_lengkap.as_bytes(), &key);
    let (email_nonce, email_ciphertext) = utils::encrypt_data(email_norm.as_bytes(), &key);
    let (telepon_nonce, telepon_ciphertext) = utils::encrypt_data(&phone.unwrap().as_bytes(), &key);

    // === Cek apakah no_piagam sudah ada ===
    if let Some(ref no_piagam) = form_data.no_piagam {
        let existing_record = sqlx::query("SELECT id FROM pdp WHERE no_piagam = ?")
            .bind(no_piagam)
            .fetch_optional(pool.get_ref())
            .await;

        match existing_record {
            Ok(Some(row)) => {
                let existing_id: i64 = row.get("id");
                println!("Record ditemukan dengan ID: {}", existing_id);
                // === UPDATE data yang sudah ada ===
                let result = sqlx::query!(
                    r#"
                    UPDATE pdp SET
                    nik = ?, nama_lengkap = ?, tempat_lahir = ?, tgl_lahir = ?, jk = ?, email = ?, telepon = ?, alamat = ?,
                    pendidikan_terakhir = ?, jurusan = ?, nama_instansi_pendidikan = ?, id_hobi = ?,
                    posisi = ?, tingkat_kepengurusan = ?, jabatan = ?, tingkat_penugasan = ?,
                    id_provinsi_domisili = ?, id_kabupaten_domisili = ?, id_provinsi = ?, id_kabupaten = ?,
                    thn_tugas = ?, id_minat = ?, detail_minat = ?, id_minat_2 = ?, detail_minat_2 = ?,
                    id_bakat = ?, detail_bakat = ?, keterangan = ?, agreement = ?, status = ?,
                    file_piagam = ?, photo = ?,
                    nik_blind_index = ?, email_blind_index = ?, telepon_blind_index = ?,
                    nik_nonce = ?, nama_nonce = ?, email_nonce = ?, telepon_nonce = ?,
                    updated_at = CURRENT_TIMESTAMP
                    WHERE no_piagam = ?
                    "#,
                    nik_ciphertext,
                    nama_ciphertext,
                    form_data.tempat_lahir,
                    form_data.tgl_lahir,
                    form_data.jk,
                    email_ciphertext,
                    telepon_ciphertext,
                    form_data.alamat,
                    form_data.pendidikan_terakhir,
                    form_data.jurusan,
                    form_data.nama_instansi_pendidikan,
                    form_data.id_hobi,
                    form_data.posisi,
                    form_data.tingkat_kepengurusan,
                    form_data.jabatan,
                    form_data.tingkat_penugasan,
                    form_data.id_provinsi_domisili,
                    form_data.id_kabupaten_domisili,
                    form_data.id_provinsi,
                    form_data.id_kabupaten,
                    form_data.thn_tugas,
                    form_data.id_minat,
                    form_data.detail_minat,
                    form_data.id_minat_2,
                    form_data.detail_minat_2,
                    form_data.id_bakat,
                    form_data.detail_bakat,
                    form_data.keterangan,
                    form_data.agreement,
                    form_data.status,
                    file_piagam_path,
                    form_data.photo,
                    nik_blind_index,
                    email_blind_index,
                    telepon_blind_index,
                    nik_nonce.as_ref(),
                    nama_nonce.as_ref(),
                    email_nonce.as_ref(),
                    telepon_nonce.as_ref(),
                    no_piagam
                )
                .execute(pool.get_ref())
                .await;

                match result {
                    Ok(_) => return Ok(HttpResponse::Ok().json("Data berhasil diupdate")),
                    Err(e) => {
                        eprintln!("DB Update Error: {:?}", e);
                        return Ok(HttpResponse::InternalServerError()
                            .json("Terjadi kesalahan saat mengupdate data"));
                    }
                }
            }
            Ok(None) => {
                // Lanjut dengan INSERT baru
            }
            Err(e) => {
                eprintln!("DB Check Error: {:?}", e);
                return Ok(HttpResponse::InternalServerError()
                    .json("Terjadi kesalahan saat mengecek data"));
            }
        }
    }

    let id = generate_short_uuid(); // Generate UUID 10 karakter

    let result = sqlx::query!(
        r#"
        INSERT INTO pdp
        (id, nik, nama_lengkap, tempat_lahir, tgl_lahir, jk, email, telepon, alamat,
         pendidikan_terakhir, jurusan, nama_instansi_pendidikan, id_hobi,
         posisi, tingkat_kepengurusan, jabatan, tingkat_penugasan,
         id_provinsi_domisili, id_kabupaten_domisili, id_provinsi, id_kabupaten,
         thn_tugas, id_minat, detail_minat, id_minat_2, detail_minat_2,
         id_bakat, detail_bakat, keterangan, agreement, status, no_piagam, file_piagam, photo,
         nik_blind_index, email_blind_index, telepon_blind_index, nik_nonce, nama_nonce, email_nonce, telepon_nonce)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
            id,
            nik_ciphertext,
            nama_ciphertext,
            form_data.tempat_lahir,
            form_data.tgl_lahir,
            form_data.jk,
            email_ciphertext,
            telepon_ciphertext,
            form_data.alamat,
            form_data.pendidikan_terakhir,
            form_data.jurusan,
            form_data.nama_instansi_pendidikan,
            form_data.id_hobi,
            form_data.posisi,
            form_data.tingkat_kepengurusan,
            form_data.jabatan,
            form_data.tingkat_penugasan,
            form_data.id_provinsi_domisili,
            form_data.id_kabupaten_domisili,
            form_data.id_provinsi,
            form_data.id_kabupaten,
            form_data.thn_tugas,
            form_data.id_minat,
            form_data.detail_minat,
            form_data.id_minat_2,
            form_data.detail_minat_2,
            form_data.id_bakat,
            form_data.detail_bakat,
            form_data.keterangan,
            form_data.agreement,
            form_data.status,
            form_data.no_piagam,
            file_piagam_path,
            form_data.photo,
            nik_blind_index,
            email_blind_index,
            telepon_blind_index,
            nik_nonce.as_ref(),
            nama_nonce.as_ref(),
            email_nonce.as_ref(),
            telepon_nonce.as_ref(),
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Pendaftaran berhasil disimpan",
            "id": id // Kembalikan ID ke frontend jika diperlukan
        }))),
        Err(e) => {
            eprintln!("DB Error: {:?}", e);
            if let sqlx::Error::Database(db_err) = &e {
                if let Some(code) = db_err.code() {
                    if code == "23000" && db_err.message().contains("Duplicate entry") {
                        eprintln!("❌ Duplikat NIK terdeteksi");
                        return Ok(HttpResponse::Conflict().json("NIK sudah terdaftar"));
                    }
                }
            }

            Ok(HttpResponse::InternalServerError()
                .json("Terjadi kesalahan saat menyimpan ke database"))
        }
    }
}

#[get("/api/user")]
pub async fn get_user(req: HttpRequest, pool: web::Data<MySqlPool>) -> Result<HttpResponse, Error> {
    let claims = auth::verify_jwt(&req).map_err(|e| {
        log::error!("Verify JWT failed: {}", e);
        actix_web::error::ErrorUnauthorized(e.to_string())
    })?;

    let user = sqlx
        ::query_as::<_, UserProfile>(
            "SELECT id, name, email, role, address, avatar, phone, id_pdp, id_provinsi, id_kabupaten, created_at FROM users WHERE email = ?"
        )
        .bind(&claims.sub)
        .fetch_one(pool.get_ref()).await
        .map_err(|e| {
            log::error!("Gagal mengambil data pengguna: {:?}", e);
            actix_web::error::ErrorInternalServerError("Gagal mengambil data pengguna")
        })?;

    let user_response = json!({
        "id": user.id,
        "name": user.name,
        "email": user.email,
        "role": user.role,
        "address": user.address.unwrap_or_default(),
        "avatar": user.avatar.unwrap_or_default(),
        "phone": user.phone.unwrap_or_default(),
        "id_pdp": user.id_pdp,
        "id_provinsi": user.id_provinsi.unwrap_or_default(),
        "id_kabupaten": user.id_kabupaten.unwrap_or_default(),
        "created_at": user.created_at,
    });

    Ok(HttpResponse::Ok().json(user_response))
}

//=======login====================================================================

// ---- Payload dari Frontend (JSON) ----
#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
    #[serde(rename = "recaptchaToken")]
    pub recaptcha_token: String,
}

#[derive(Deserialize)]
struct RecaptchaResponse {
    success: bool,
    #[serde(default)]
    score: f32,
}

#[post("/api/login")]
pub async fn login(
    pool: web::Data<MySqlPool>,
    payload: web::Json<LoginPayload>,
) -> Result<impl Responder, Error> {
    let email = payload.email.trim();
    let password = payload.password.trim();

    if email.is_empty() || password.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status":"error","message":"Email atau password kosong"
        })));
    }

    // ---- reCAPTCHA ----
    let secret_key = env::var("RECAPTCHA_SECRET_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("RECAPTCHA_SECRET_KEY not set"))?;
    if payload.recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "status":"error","message":"Token reCAPTCHA tidak ditemukan"
        })));
    }
    let resp_text = Client::new()
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&[
            ("secret", secret_key.as_str()),
            ("response", payload.recaptcha_token.as_str()),
        ])
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .text()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    let rec: RecaptchaResponse =
        serde_json::from_str(&resp_text).map_err(actix_web::error::ErrorInternalServerError)?;
    if !rec.success || rec.score < 0.5 {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "status":"error","message":"Verifikasi reCAPTCHA gagal/bot"
        })));
    }

    // ---- Ambil user ----
    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, avatar, name, password, email, role, address, phone, id_pdp, id_provinsi, id_kabupaten, email_verified_at, remember_token, created_at FROM users WHERE email = ? LIMIT 1"#,
    )
    .bind(email)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("DB error get user: {:?}", e);
        actix_web::error::ErrorInternalServerError("DB error")
    })?
    .ok_or_else(|| actix_web::error::ErrorUnauthorized("Email tidak terdaftar"))?;

    // ---- Verifikasi password ----
    let ok = verify(password, &user.password).map_err(|e| {
        log::error!("bcrypt verify: {:?}", e);
        actix_web::error::ErrorInternalServerError("Verify error")
    })?;

    if !ok {
        return Err(actix_web::error::ErrorUnauthorized(
            "Kredensial tidak valid",
        ));
    }
    let row = sqlx::query_as::<_, (String, String, String, Option<String>)>(
        "SELECT id, password, role, id_pdp FROM users WHERE email=? LIMIT 1",
    )
    .bind(&payload.email)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorUnauthorized)?;

    if !bcrypt::verify(&payload.password, &row.1).map_err(actix_web::error::ErrorUnauthorized)? {
        return Err(actix_web::error::ErrorUnauthorized("Password salah"));
    }

    let token = auth::generate_jwt(&user).map_err(|e| {
        log::error!("Gagal menghasilkan JWT: {:?}", e);
        actix_web::error::ErrorInternalServerError("Gagal menghasilkan token")
    })?;

    let access_cookie = Cookie::build("access_token", token.clone())
        .path("/")
        .http_only(true)
        .secure(false) // false untuk development (HTTP)
        .same_site(SameSite::Lax) // Lax untuk cross-origin
        .max_age(Duration::days(4))
        .finish();

    Ok(HttpResponse::Ok().cookie(access_cookie).json(json!({
        "message": "Berhasil login",
        "role": user.role,
    })))
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
    #[serde(rename = "recaptchaToken")]
    pub recaptcha_token: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
    pub confirm_password: String,
    #[serde(rename = "recaptchaToken")]
    pub recaptcha_token: String,
}

#[post("/api/forgot-password")]
pub async fn forgot_password(
    pool: web::Data<MySqlPool>,
    payload: web::Json<ForgotPasswordRequest>,
) -> Result<impl Responder, Error> {
    let email = payload.email.trim().to_lowercase();

    if email.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Email harus diisi"
        })));
    }

    // Verifikasi reCAPTCHA
    let secret_key = env::var("RECAPTCHA_SECRET_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("RECAPTCHA_SECRET_KEY not set"))?;

    if payload.recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Token reCAPTCHA tidak ditemukan"
        })));
    }

    let client = Client::new();
    let verify_url = "https://www.google.com/recaptcha/api/siteverify";

    let response = client
        .post(verify_url)
        .form(&[
            ("secret", secret_key.as_str()),
            ("response", payload.recaptcha_token.as_str()),
        ])
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let recaptcha_response: RecaptchaResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if !recaptcha_response.success || recaptcha_response.score < 0.5 {
        return Ok(HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Verifikasi reCAPTCHA gagal"
        })));
    }

    // Cek apakah email terdaftar
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, name, email, role, password, address, avatar, phone, email_verified_at, remember_token, id_pdp, id_provinsi, id_kabupaten, created_at FROM users WHERE email = ?"
    )
    .bind(&email)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error checking email: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Untuk keamanan, selalu return success meskipun email tidak ditemukan
    if user.is_none() {
        log::info!("Forgot password request for non-existent email: {}", email);
        return Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Jika email terdaftar, tautan reset password akan dikirim"
        })));
    }

    let user = user.unwrap();

    // Generate reset token
    let reset_token = Uuid::new_v4().to_string();
    let token_expires = chrono::Utc::now() + chrono::Duration::hours(1); // Token berlaku 1 jam

    // Simpan token ke database
    sqlx::query("INSERT INTO password_reset_tokens (email, token, expires_at) VALUES (?, ?, ?)")
        .bind(&email)
        .bind(&reset_token)
        .bind(token_expires)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to save reset token: {}", e);
            actix_web::error::ErrorInternalServerError("Gagal menyimpan token reset")
        })?;

    // Kirim email reset password
    let reset_url = format!(
        "{}/reset-password/{}",
        env::var("FRONTEND_URL").unwrap_or_else(|_| "https://dppi.bpip.go.id".into()),
        reset_token
    );

    let email_result = send_reset_password_email(&user.email, &user.name, &reset_url).await;

    if let Err(e) = email_result {
        log::error!("Failed to send reset password email: {}", e);
        // Tetap return success untuk keamanan
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Jika email terdaftar, tautan reset password akan dikirim"
    })))
}

#[post("/api/reset-password")]
pub async fn reset_password(
    pool: web::Data<MySqlPool>,
    payload: web::Json<ResetPasswordRequest>,
) -> Result<impl Responder, Error> {
    let data = payload.into_inner();

    // Validasi input
    if data.new_password != data.confirm_password {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Password baru dan konfirmasi password tidak sama"
        })));
    }

    if data.new_password.len() < 6 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Password minimal 6 karakter"
        })));
    }

    // Verifikasi reCAPTCHA
    let secret_key = env::var("RECAPTCHA_SECRET_KEY")
        .map_err(|_| actix_web::error::ErrorInternalServerError("RECAPTCHA_SECRET_KEY not set"))?;

    if data.recaptcha_token.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Token reCAPTCHA tidak ditemukan"
        })));
    }

    let client = Client::new();
    let verify_url = "https://www.google.com/recaptcha/api/siteverify";

    let response = client
        .post(verify_url)
        .form(&[
            ("secret", secret_key.as_str()),
            ("response", data.recaptcha_token.as_str()),
        ])
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let recaptcha_response: RecaptchaResponse = response
        .json()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if !recaptcha_response.success || recaptcha_response.score < 0.5 {
        return Ok(HttpResponse::Forbidden().json(json!({
            "status": "error",
            "message": "Verifikasi reCAPTCHA gagal"
        })));
    }

    // Validasi token
    let token_record: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT email, expires_at FROM password_reset_tokens WHERE token = ? AND used = false",
    )
    .bind(&data.token)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error checking token: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let (email, expires_at) = match token_record {
        Some(record) => record,
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Token reset password tidak valid atau sudah digunakan"
            })));
        }
    };

    // Cek expiry
    if chrono::Utc::now() > expires_at {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Token reset password sudah kadaluarsa"
        })));
    }

    // Hash password baru
    let hashed_password = bcrypt::hash(&data.new_password, DEFAULT_COST).map_err(|e| {
        log::error!("Failed to hash password: {}", e);
        actix_web::error::ErrorInternalServerError("Gagal mengenkripsi password")
    })?;

    // Update password user
    let result = sqlx::query(
        "UPDATE users SET password = ?, updated_at = CURRENT_TIMESTAMP WHERE email = ?",
    )
    .bind(&hashed_password)
    .bind(&email)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Failed to update password: {}", e);
        actix_web::error::ErrorInternalServerError("Gagal mengupdate password")
    })?;

    if result.rows_affected() == 0 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "User tidak ditemukan"
        })));
    }

    // PERBAIKAN: Tandai token sebagai sudah digunakan (tanpa operator ?)
    let _ = sqlx::query("UPDATE password_reset_tokens SET used = true WHERE token = ?")
        .bind(&data.token)
        .execute(pool.get_ref())
        .await
        .map_err(|e| {
            log::error!("Failed to mark token as used: {}", e);
        });

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Password berhasil direset"
    })))
}
// Fungsi helper untuk mengirim email reset password
async fn send_reset_password_email(to: &str, name: &str, reset_url: &str) -> Result<(), String> {
    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "DPPI".into());
    let from_name =
        env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Sekretariat DPPI BPIP RI".into());
    let from_addr =
        env::var("SMTP_FROM_ADDRESS").map_err(|_| "SMTP_FROM_ADDRESS missing".to_string())?;

    let host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST missing".to_string())?;
    let port: u16 = env::var("SMTP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(587);
    let user = env::var("SMTP_USER").map_err(|_| "SMTP_USER missing".to_string())?;
    let pass = env::var("SMTP_PASS").map_err(|_| "SMTP_PASS missing".to_string())?;
    let enc = env::var("SMTP_ENCRYPTION")
        .unwrap_or_else(|_| "STARTTLS".into())
        .to_uppercase();

    let subject = format!("[{}] Reset Password", app_name);

    let text = format!(
        "Halo, {name}\n\n\
        Anda menerima email ini karena meminta reset password untuk akun Anda.\n\n\
        Silakan klik tautan berikut untuk reset password (berlaku 1 jam):\n\
        {reset_url}\n\n\
        Jika Anda tidak meminta reset password, abaikan email ini.\n\n\
        — Tim {app_name}"
    );

    let html = format!(
        r#"<div style="font-family:Arial,sans-serif; max-width:600px; margin:0 auto;">
            <h1 style="color:#2c5aa0; text-align:center;">SALAM PANCASILA</h1>
            <h2 style="color:#333;">Halo, {name}</h2>

            <p>Anda menerima email ini karena meminta reset password untuk akun Anda.</p>

            <div style="text-align:center; margin:30px 0;">
                <a href="{reset_url}" style="background-color:#2c5aa0; color:white; padding:12px 24px; text-decoration:none; border-radius:4px; display:inline-block;">
                    Reset Password
                </a>
            </div>

            <p style="color:#666; font-size:14px;">
                Tautan ini berlaku selama 1 jam.<br>
                Jika Anda tidak meminta reset password, abaikan email ini.
            </p>

            <p style="margin-top:30px;">Terima kasih.<br>— Tim {app_name}</p>

            <div style="margin-top:30px; padding:15px; background:#f8f9fa; border:1px solid #e9ecef; border-radius:5px; text-align:center; font-size:12px; color:#6c757d;">
                <p style="margin:0;">
                    <strong>📧 EMAIL NO-REPLY</strong><br>
                    Email ini dikirim secara otomatis oleh sistem. <br>
                    <strong>Mohon untuk tidak membalas email ini.</strong><br>
                    Jika Anda membutuhkan bantuan, silakan hubungi administrator.
                </p>
            </div>
        </div>"#,
        name = name,
        reset_url = reset_url,
        app_name = app_name
    );

    let message = MessageBuilder::new()
        .from((from_name.as_str(), from_addr.as_str()))
        .to((name, to))
        .subject(subject)
        .text_body(text)
        .html_body(html);

    let mut client_builder = SmtpClientBuilder::new(host.as_str(), port)
        .credentials(Credentials::new(user.as_str(), pass.as_str()));

    client_builder = match enc.as_str() {
        "SSL" | "SMTPS" => client_builder.implicit_tls(true),
        "PLAIN" | "NONE" => client_builder.implicit_tls(false),
        _ => client_builder.implicit_tls(false),
    };

    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .send(message)
        .await
        .map_err(|e| format!("Failed to send reset password email: {}", e))?;

    Ok(())
}

#[post("/api/logout")]
pub async fn logout() -> Result<impl Responder, Error> {
    // ✅ FIXED LOGOUT COOKIE (harus sama persis dengan login)
    let access_cookie = Cookie::build("access_token", "")
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax)
        // .domain("127.0.0.1")
        .max_age(Duration::seconds(0))
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .json(json!({ "message": "Berhasil logout" })))
}
//jabatan
// struktur Jabatan
#[derive(Serialize, FromRow, Debug)]
struct Jabatan {
    id: i32,
    nama_jabatan: String,
}

#[derive(Serialize, Debug)]
struct JabatanResponse {
    jabatan_pusat: Vec<Jabatan>,
    jabatan_provinsi: Vec<Jabatan>,
    jabatan_kabupaten: Vec<Jabatan>,
}

#[get("/api/jabatan")]
pub async fn get_jabatan(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // A. Ambil data Jabatan Pusat
    let jabatan: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // B. Ambil data Jabatan Provinsi
    let jabatan_provinsi: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan_provinsi")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    // C. Ambil data Jabatan Kabupaten
    let jabatan_kabupaten: Vec<Jabatan> =
        sqlx::query_as::<_, Jabatan>("SELECT id, nama_jabatan FROM jabatan_kabupaten")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    let response_data = JabatanResponse {
        jabatan_pusat: jabatan,
        jabatan_provinsi: jabatan_provinsi,
        jabatan_kabupaten: jabatan_kabupaten,
    };

    Ok(HttpResponse::Ok().json(response_data))
}

// minat
// struktur minat
#[derive(Serialize, FromRow, Debug)]
struct Minat {
    id: i32,
    kategori_minat: String,
}

#[derive(Serialize, FromRow, Debug)]
struct DetailMinat {
    id: i32,
    id_minat: i32,
    detail_minat: String,
}

#[get("/api/minat")]
pub async fn get_minat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Ambil data minat
    let minat: Vec<Minat> = sqlx::query_as::<_, Minat>("SELECT id, kategori_minat FROM minat")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(minat))
}

#[get("/api/detail-minat")]
pub async fn get_detail_minat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Ambil data minat
    let detail_minat: Vec<DetailMinat> =
        sqlx::query_as::<_, DetailMinat>("SELECT id, id_minat, detail_minat FROM detail_minat")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(detail_minat))
}

//bakat
// struktur bakat
#[derive(Serialize, FromRow, Debug)]
struct Bakat {
    id: i32,
    kategori_bakat: String,
}

#[derive(Serialize, FromRow, Debug)]
struct DetailBakat {
    id: i32,
    id_bakat: i32,
    detail_bakat: String,
}
#[get("/api/bakat")]
pub async fn get_bakat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Ambil data bakat
    let bakat: Vec<Bakat> = sqlx::query_as::<_, Bakat>("SELECT id, kategori_bakat FROM bakat")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(bakat))
}

#[get("/api/detail-bakat")]
pub async fn get_detail_bakat(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    // Ambil data bakat
    let detail_bakat: Vec<DetailBakat> =
        sqlx::query_as::<_, DetailBakat>("SELECT id, id_bakat, detail_bakat FROM detail_bakat")
            .fetch_all(pool.get_ref())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(detail_bakat))
}

//hobi
#[derive(Serialize, FromRow, Debug)]
struct Hobi {
    id: i32,
    kategori_hobi: String,
}

#[get("/api/hobi")]
pub async fn get_hobi(pool: web::Data<MySqlPool>) -> Result<impl Responder, Error> {
    let hobi: Vec<Hobi> = sqlx::query_as::<_, Hobi>("SELECT id, kategori_hobi FROM hobi")
        .fetch_all(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(hobi))
}

fn generate_short_uuid() -> String {
    let raw = Uuid::new_v4().simple().to_string();
    raw[..10].to_uppercase()
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
