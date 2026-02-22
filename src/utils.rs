<<<<<<< HEAD
//utils.rs
use actix_multipart::Multipart;
use actix_web::{Error, error::ErrorBadRequest};
use blake3::{self, Hasher};
use chrono::Datelike;
use deunicode::deunicode_with_tofu;
use futures::TryStreamExt;
use mail_send::{Credentials, SmtpClientBuilder, mail_builder::MessageBuilder};
use rand::Rng;
use sanitize_filename::sanitize;
use sodiumoxide::crypto::secretbox;
use std::{env, fmt};
use std::{fs, io::Write, path::Path};
use uuid::Uuid;
// Enkripsi
pub fn encrypt_data(data: &[u8], key: &secretbox::Key) -> (secretbox::Nonce, Vec<u8>) {
    let nonce = secretbox::gen_nonce();
    let ciphertext = secretbox::seal(data, &nonce, key);
    (nonce, ciphertext)
}

// Dekripsi
#[derive(Debug)]
pub enum DecryptionError {
    InvalidCiphertext,
    Utf8Error(std::string::FromUtf8Error),
    InvalidNonce,
}

impl fmt::Display for DecryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecryptionError::InvalidCiphertext => write!(f, "Ciphertext tidak valid atau rusak"),
            DecryptionError::Utf8Error(e) => write!(f, "Kesalahan konversi UTF-8: {}", e),
            DecryptionError::InvalidNonce => write!(f, "Nonce tidak valid atau rusak"),
        }
    }
}

impl std::error::Error for DecryptionError {}

// Fungsi untuk mendekripsi data
pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8],
    key: &secretbox::Key,
) -> Result<String, DecryptionError> {
    // Log untuk debugging
    log::debug!(
        "Decrypt: ciphertext_len={}, nonce_len={}",
        ciphertext.len(),
        nonce.len()
    );

    // Convert slice to Nonce
    let nonce = secretbox::Nonce::from_slice(nonce).ok_or_else(|| {
        log::error!("Invalid nonce length: {} bytes, expected 24", nonce.len());
        DecryptionError::InvalidNonce
    })?;

    // Decrypt the ciphertext
    let plaintext_bytes = secretbox::open(ciphertext, &nonce, key).map_err(|e| {
        log::error!("Decryption failed: {:?}", e);
        DecryptionError::InvalidCiphertext
    })?;

    // Convert plaintext bytes to String
    let plaintext_string = String::from_utf8(plaintext_bytes).map_err(|e| {
        log::error!("UTF-8 conversion failed: {:?}", e);
        DecryptionError::Utf8Error(e)
    })?;

    log::debug!(
        "Decrypt successful: plaintext_len={}",
        plaintext_string.len()
    );
    Ok(plaintext_string)
}

// Fungsi untuk membuat blind index
pub fn generate_blind_index(data: &[u8], key: &[u8]) -> Vec<u8> {
    let key_array: &[u8; 32] = key.try_into().expect("Blind index key must be 32 bytes");
    let mut hasher = Hasher::new_keyed(key_array);
    hasher.update(data);
    hasher.finalize().as_bytes().to_vec()
}

pub fn get_encryption_key() -> Result<secretbox::Key, Box<dyn std::error::Error>> {
    let encryption_key_hex =
        env::var("ENCRYPTION_KEY").map_err(|e| format!("ENCRYPTION_KEY not set: {}", e))?;

    let encryption_key_bytes = hex::decode(&encryption_key_hex)
        .map_err(|e| format!("Invalid hex encryption key: {}", e))?;

    let key = secretbox::Key::from_slice(&encryption_key_bytes)
        .ok_or("Invalid encryption key size, expected 32 bytes")?;

    log::debug!("Encryption key loaded successfully");
    Ok(key)
}
// Fungsi normalize_phone yang lebih robust
pub fn normalize_phone(phone: &str) -> String {
    // Menghapus semua karakter selain angka
    let normalized_phone: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    // Normalisasi nomor telepon
    if normalized_phone.starts_with("08") {
        format!("62{}", &normalized_phone[1..])
    } else if normalized_phone.starts_with("8") {
        format!("62{}", normalized_phone)
    } else if !normalized_phone.starts_with("62") {
        format!("62{}", normalized_phone)
    } else {
        normalized_phone
    }
}

pub async fn send_verified_email(to: &str, name: &str, password: &str) -> Result<(), String> {
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

    let subject = format!("[{}] Akun Anda Telah Diverifikasi", app_name);
    let text = format!(
        "Halo, {name}\n\nPendaftaran Anda telah Diverifikasi.\n\
         Silakan login menggunakan email ini dengan password: {password}\n\
         Untuk keamanan, disarankan untuk mengganti password setelah login pertama.\n\n— Tim {app_name}"
    );
    let html = format!(
        r#"<div style="font-family:Arial,sans-serif">
            <h1>SALAM PANCASILA</h1>
            <h2>Halo, {name}</h2>
            <p>Pendaftaran Anda telah <b>Diverifikasi</b>.</p>
            <p><strong>Password login Anda:</strong> {password}</p>
            <p>Silakan login menggunakan email ini dengan password di atas.
               Untuk keamanan, disarankan untuk mengganti password setelah login pertama.</p>
            <p>Terima kasih.<br>— Tim {app_name}</p>
          </div>"#
    );

    // ......
    println!("SMTP Configuration:");
    println!("  Host: {}", host);
    println!("  Port: {}", port);
    println!("  Encryption: {}", enc);
    println!("  User: {}", user);

    // Bangun message
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
        _ => client_builder.implicit_tls(false), // STARTTLS
    };

    // Kirim email
    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .send(message)
        .await
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}

pub fn generate_random_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

    let mut rng = rand::rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    password
}

pub const GALLERY_DIR: &str = "uploads/assets/images/gallery";

/// Pastikan folder tersedia
pub fn ensure_gallery_dir() -> std::io::Result<()> {
    fs::create_dir_all(GALLERY_DIR)
}

/// Simpan semua file image dari multipart field `foto[]`
/// Return: Vec<filename_only>
pub async fn save_gallery_images(mut payload: Multipart) -> Result<Vec<String>, Error> {
    ensure_gallery_dir().map_err(|e| ErrorBadRequest(format!("MkDir: {e}")))?;
    let mut saved = Vec::<String>::new();

    while let Some(field) = payload.try_next().await.map_err(ErrorBadRequest)? {
        let name = field.name().unwrap_or_default().to_string();

        // hanya proses field yang namanya `foto` atau `foto[]`
        if name != "foto" && name != "foto[]" {
            // skip field lain (mis. kegiatan/tanggal bila ikut multipart)
            continue;
        }

        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_default();
        if !content_type.starts_with("image/") {
            return Err(ErrorBadRequest("File bukan image"));
        }

        let cd = field.content_disposition().cloned();
        let orig = cd
            .and_then(|d| d.get_filename().map(|s| s.to_string()))
            .unwrap_or_else(|| "image".to_string());

        let ext = Path::new(&orig)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg");

        let filename = format!("{}.{ext}", Uuid::new_v4().simple());
        let safe = sanitize(&filename);
        let filepath = Path::new(GALLERY_DIR).join(&safe);

        let mut f = std::fs::File::create(&filepath)
            .map_err(|e| ErrorBadRequest(format!("Create file: {e}")))?;

        // tulis chunk
        let mut field_stream = field;
        while let Some(chunk) = field_stream.try_next().await.map_err(ErrorBadRequest)? {
            f.write_all(&chunk)
                .map_err(|e| ErrorBadRequest(format!("Write file: {e}")))?;
        }

        saved.push(safe);
    }

    Ok(saved)
}

/// Hapus 1 file fisik (diabaikan kalau tidak ada)
pub fn delete_gallery_image(filename: &str) -> std::io::Result<()> {
    let p = Path::new(GALLERY_DIR).join(filename);
    if p.exists() {
        std::fs::remove_file(p)?;
    }
    Ok(())
}

/// Hapus semua file dalam array
pub fn delete_gallery_images_all(files: &[String]) {
    for f in files {
        let _ = delete_gallery_image(f);
    }
}

// src/utils.rs
pub fn generate_slug(text: &str) -> String {
    let slug = deunicode_with_tofu(text, "-")
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();

    slug.trim_matches('-')
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// attachment: Some(&(filename, bytes, mime)), contoh: ("file.pdf", vec![..], "application/pdf")
pub async fn send_generic_email_mail_send(
    to: &str,
    cc: Option<&str>,  // koma-separeted
    bcc: Option<&str>, // koma-separeted
    subject: &str,
    text: &str,
    html: Option<&str>,
    attachment: Option<&(String, Vec<u8>, String)>,
) -> Result<(), String> {
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

    // Bangun message (To bisa banyak, tapi di sini satu sesuai UI)
    let mut msg = MessageBuilder::new()
        .from((from_name.as_str(), from_addr.as_str()))
        .subject(subject)
        .text_body(text);

    if let Some(html_body) = html {
        msg = msg.html_body(html_body);
    }

    // To
    msg = msg.to(("", to));

    // CC (koma dipisah)
    if let Some(cc_s) = cc {
        for addr in cc_s.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            msg = msg.cc(("", addr));
        }
    }
    // BCC
    if let Some(bcc_s) = bcc {
        for addr in bcc_s.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            msg = msg.bcc(("", addr));
        }
    }

    // Attachment
    if let Some((filename, bytes, mime)) = attachment {
        // Format: .attachment(mime_type, filename, data)
        msg = msg.attachment(mime.as_str(), filename.as_str(), bytes.as_slice());
    }

    // SMTP client
    let mut client_builder = SmtpClientBuilder::new(host.as_str(), port)
        .credentials(Credentials::new(user.as_str(), pass.as_str()));

    client_builder = match enc.as_str() {
        "SSL" | "SMTPS" => client_builder.implicit_tls(true),
        "PLAIN" | "NONE" => client_builder.implicit_tls(false),
        _ => client_builder.implicit_tls(false), // STARTTLS default
    };

    // Opsional: logging ringan biar sama kayak verified_email mu
    println!(
        "[{}] SMTP → host:{} port:{} enc:{} user:{}",
        app_name, host, port, enc, user
    );

    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect SMTP: {e}"))?
        .send(msg)
        .await
        .map_err(|e| format!("Failed to send email: {e}"))?;

    Ok(())
}

pub async fn send_rejection_email(
    to: &str,
    name: &str,
    alasan_penolakan: &str,
) -> Result<(), String> {
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

    let subject = format!("[{}] Pemberitahuan Penolakan Registrasi PDP", app_name);

    let text = format!(
        "Halo, {name}\n\n\
         Dengan hormat kami sampaikan bahwa registrasi Anda sebagai PDP (Purnapaskibraka Duta Pancasila) \
         TIDAK DAPAT DISETUJUI.\n\n\
         Alasan Penolakan:\n\
         {alasan_penolakan}\n\n\
         Anda dapat memperbaiki data dan mengajukan kembali registrasi melalui sistem.\n\
         Jika Anda memiliki pertanyaan lebih lanjut, silakan hubungi administrator.\n\n\
         — Tim {app_name} \n\n
         EMAIL INI DIKIRIM SECARA OTOMATIS, MOHON UNTUK TIDAK MEMBALAS EMAIL INI."
    );

    let html = format!(
        r#"<div style="font-family:Arial,sans-serif; max-width:600px; margin:0 auto;">
            <h1 style="color:#dc3545; text-align:center;">SALAM PANCASILA</h1>
            <h2 style="color:#333;">Halo, {name}</h2>

            <p>Dengan hormat kami sampaikan bahwa registrasi Anda sebagai <strong>PDP (Purnapaskibraka Duta Pancasila)</strong>
            <strong style="color:#dc3545;">TIDAK DAPAT DISETUJUI</strong>.</p>

            <div style="background:#fff3cd; border:1px solid #ffeaa7; padding:15px; margin:15px 0; border-radius:5px;">
                <h4 style="color:#856404; margin-top:0;">Alasan Penolakan:</h4>
                <p style="color:#333; margin-bottom:0;">{alasan_penolakan}</p>
            </div>

            <p>Anda dapat memperbaiki data dan mengajukan kembali registrasi melalui sistem.</p>
            <p style="margin-top:30px;">Terima kasih.<br>— Tim {app_name}</p>

        </div>
        <div style="margin-top:30px; padding:15px; background:#f8f9fa; border:1px solid #e9ecef; border-radius:5px; text-align:center; font-size:12px; color:#6c757d;">
                <p style="margin:0;">
                    <strong>📧 EMAIL NO-REPLY</strong><br>
                    Email ini dikirim secara otomatis oleh sistem. <br>
                    <strong>Mohon untuk tidak membalas email ini.</strong><br>
                    Jika Anda membutuhkan bantuan, silakan hubungi administrator melalui <a href="https://dppi.bpip.go.id/kontak" target="_blank" rel="noopener">https://dppi.bpip.go.id/kontak</a>.
                </p>
            </div>
        "#,
        name = name,
        alasan_penolakan = alasan_penolakan,
        app_name = app_name
    );

    // Debug info (opsional)
    println!("SMTP Configuration for Rejection Email:");
    println!("  Host: {}", host);
    println!("  Port: {}", port);
    println!("  Encryption: {}", enc);
    println!("  User: {}", user);
    println!("  To: {}", to);

    // Bangun message
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
        _ => client_builder.implicit_tls(false), // STARTTLS
    };

    // Kirim email
    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .send(message)
        .await
        .map_err(|e| format!("Failed to send rejection email: {}", e))?;

    println!("Rejection email successfully sent to {}", to);
    Ok(())
}

/// Fungsi untuk mengirim email bukti submit dokumen pengangkatan DPPI {daerah}
/// - to: email penerima
/// - data: struct berisi data submit dokumen
pub async fn send_submit_confirmation_email(
    to: &str,
    data: &SubmitConfirmationData,
) -> Result<(), String> {
    let from_name: String = "Portal Informasi DPPI".into();
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

    // Subject email
    let subject = format!("Bukti Submit Dokumen Pengangkatan DPPI {}", data.daerah);

    // HTML body sesuai dengan gambar
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Bukti Submit Dokumen Pengangkatan DPPI</title>
  <style>
            body {{
                font-family: Arial, sans-serif;
                line-height: 1.6;
                color: #333;
                max-width: 800px;
                margin: 0 auto;
                padding: 20px;
            }}
            .header {{
                text-align: center;
                margin-bottom: 30px;
                padding-bottom: 20px;
                border-bottom: 2px solid #1a365d;
            }}

            .logo {{
                font-size: 48px;
                font-weight: bold;
                margin: 20px 0;
                display: inline-flex;
                align-items: center;
                justify-content: center;
                gap: 15px;
                color: #b80000;
            }}
            .logo-text {{
                margin-top: 20px;
                margin-left: 20px;
                font-size: 40px;
                font-weight: bold;
            }}
            .logo-bpip {{
                color: #b80000;
            }}
            .logo-dppi {{
                color: #000000;
            }}
            .logo-images {{
                display: flex;
                gap: 10px;
                align-items: center;
            }}
            .logo-image {{
                width: 80px;
                height: auto;
            }}
            .section {{
                margin-bottom: 25px;
                padding: 20px;
                border: 1px solid #e2e8f0;
                border-radius: 8px;
                background-color: #f8fafc;
            }}
            .section-title {{
                color: #2d3748;
                margin-top: 0;
                font-size: 18px;
                font-weight: bold;
            }}
            .info-item {{
                margin-bottom: 10px;
                display: flex;
            }}
            .info-label {{
                font-weight: bold;
                min-width: 180px;
                color: #4a5568;
            }}
            .info-value {{
                color: #2d3748;
            }}
            .participant-table {{
                width: 100%;
                border-collapse: collapse;
                margin-top: 10px;
            }}
            .participant-table th {{
                background-color: #edf2f7;
                padding: 12px;
                text-align: left;
                border: 1px solid #cbd5e0;
                font-weight: bold;
                color: #2d3748;
            }}
            .participant-table td {{
                padding: 12px;
                border: 1px solid #cbd5e0;
            }}
            .participant-table tr:nth-child(even) {{
                background-color: #f7fafc;
            }}
            .footer {{
                margin-top: 30px;
                padding-top: 20px;
                border-top: 1px solid #e2e8f0;
                font-size: 14px;
                color: #718096;
                text-align: center;
            }}
            .note-box {{
                background-color: #fff3cd;
                border: 1px solid #ffeaa7;
                border-radius: 5px;
                padding: 15px;
                margin-top: 20px;
                color: #856404;
            }}
            @media (max-width: 600px) {{
                .logo {{
                    flex-direction: column;
                    gap: 10px;
                }}
                .logo-images {{
                    flex-direction: row;
                    justify-content: center;
                }}
                .logo-image {{
                    width: 60px;
                }}
                .logo-text {{
                    font-size: 24px;
                }}
            }}
        </style>
        </head>
        <body>
            <div class="header">
                <div class="logo">
                    <div class="logo-images">
                        <img src="https://dppi.bpip.go.id/assets/images/logo-bpip.png"
                             alt="Logo BPIP"
                             class="logo-image">
                        <img src="https://dppi.bpip.go.id/assets/images/logo-dppi.png"
                             alt="Logo DPPI"
                             class="logo-image">
                    </div>
                    <div class="logo-text">
                        <span class="logo-dppi">DPPI</span>
                        <span class="logo-bpip">BPIP</span>
                    </div>
                </div>
                <h1 style="color: #1a365d;">Bukti Submit Dokumen Pengangkatan DPPI {daerah}</h1>
                <hr style="border: none; border-top: 2px dashed #cbd5e0; margin: 20px 0;">
            </div>
            <div class="section">
                <h2 class="section-title">DPPI Pusat</h2>
                <p>Berikut ini adalah Bukti Submit <strong>Dokumen Pengangkatan DPPI {daerah}</strong></p>

                <div class="info-item">
                    <span class="info-label">Nama PIC:</span>
                    <span class="info-value">{nama_pic}</span>
                </div>
                <div class="info-item">
                    <span class="info-label">ID Registrasi:</span>
                    <span class="info-value">{id_registrasi}</span>
                </div>
                <div class="info-item">
                    <span class="info-label">Tanggal Submit:</span>
                    <span class="info-value">{tanggal_submit}</span>
                </div>
            </div>

            <div class="section">
                <h2 class="section-title">Nama Calon Peserta</h2>
                <table class="participant-table">
                    <thead>
                        <tr>
                            <th>Jabatan</th>
                            <th>Nama</th>
                        </tr>
                    </thead>
                    <tbody>
                        {participant_rows}
                    </tbody>
                </table>
            </div>

            <div class="note-box">
                <strong>✅ Mohon untuk simpan/screenshot/foto/print halaman ini</strong><br>
                Bukti submit juga telah dikirimkan ke email Anda.
            </div>

            <div class="footer">
                <p>
                    <strong>EMAIL INI DIKIRIM SECARA OTOMATIS</strong><br>
                    Mohon untuk tidak membalas email ini.<br>
                    Jika ada pertanyaan, silakan hubungi administrator melalui portal https://dppi.bpip.go.id/kontak
                </p>
                <p style="margin-top: 15px; font-size: 12px;">
                    © {year} DPPI BPIP - Semua Hak Dilindungi Undang-Undang
                </p>
            </div>
        </body>
        </html>
        "#,
        daerah = data.daerah,
        nama_pic = data.nama_pic,
        id_registrasi = data.id_registrasi,
        tanggal_submit = data.tanggal_submit,
        participant_rows = generate_participant_rows(&data.participants),
        year = chrono::Local::now().year()
    );

    // Plain text version (fallback)
    let text = format!(
        "BUKTI SUBMIT DOKUMEN PENGANGKATAN DPPI {daerah}

DPPI Pusat

Berikut ini adalah Bukti Submit Dokumen Pengangkatan DPPI {daerah}
Daerah: {daerah}
Nama PIC: {nama_pic}
ID Registrasi: {id_registrasi}
Tanggal Submit: {tanggal_submit}

Nama Calon Peserta:
{participants_text}

Mohon untuk simpan/screenshot/foto/print halaman ini, bukti submit juga telah dikirimkan ke email Anda.

---
EMAIL INI DIKIRIM SECARA OTOMATIS
Mohon untuk tidak membalas email ini.
© {year} DPPI BPLP",
        daerah = data.daerah,
        nama_pic = data.nama_pic,
        id_registrasi = data.id_registrasi,
        tanggal_submit = data.tanggal_submit,
        participants_text = generate_participants_text(&data.participants),
        year = chrono::Local::now().year()
    );

    // Bangun message
    let message = MessageBuilder::new()
        .from((from_name.as_str(), from_addr.as_str()))
        .to(("", to))
        .subject(subject)
        .text_body(text)
        .html_body(html);

    // Konfigurasi SMTP client
    let mut client_builder = SmtpClientBuilder::new(host.as_str(), port)
        .credentials(Credentials::new(user.as_str(), pass.as_str()));

    client_builder = match enc.as_str() {
        "SSL" | "SMTPS" => client_builder.implicit_tls(true),
        "PLAIN" | "NONE" => client_builder.implicit_tls(false),
        _ => client_builder.implicit_tls(false), // STARTTLS default
    };

    // Kirim email
    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .send(message)
        .await
        .map_err(|e| format!("Failed to send submit confirmation email: {}", e))?;

    log::info!("Submit confirmation email sent to {}", to);
    Ok(())
}

/// Helper function untuk generate participant rows dalam format HTML table
fn generate_participant_rows(participants: &[Participant]) -> String {
    participants
        .iter()
        .map(|p| {
            format!(
                "<tr><td>{}</td><td>{}</td></tr>",
                p.jabatan,
                p.nama
                    .iter()
                    .map(|n| n.as_str())
                    .collect::<Vec<&str>>()
                    .join(" ; ")
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Helper function untuk generate participant text dalam format plain text
fn generate_participants_text(participants: &[Participant]) -> String {
    participants
        .iter()
        .map(|p| {
            format!(
                "{}: {}",
                p.jabatan,
                p.nama
                    .iter()
                    .map(|n| n.as_str())
                    .collect::<Vec<&str>>()
                    .join(" ; ")
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[derive(Debug, Clone)]
pub struct SubmitConfirmationData {
    pub daerah: String,
    pub nama_pic: String,
    pub id_registrasi: String,
    pub tanggal_submit: String,
    pub participants: Vec<Participant>,
}

/// Data structure untuk peserta
#[derive(Debug, Clone)]
pub struct Participant {
    pub jabatan: String,
    pub nama: Vec<String>,
}

/// Contoh implementasi builder untuk SubmitConfirmationData
impl SubmitConfirmationData {
    pub fn new(
        daerah: impl Into<String>,
        nama_pic: impl Into<String>,
        id_registrasi: impl Into<String>,
        tanggal_submit: impl Into<String>,
    ) -> Self {
        Self {
            daerah: daerah.into(),
            nama_pic: nama_pic.into(),
            id_registrasi: id_registrasi.into(),
            tanggal_submit: tanggal_submit.into(),
            participants: Vec::new(),
        }
    }

    pub fn add_participant(mut self, jabatan: impl Into<String>, nama: Vec<String>) -> Self {
        self.participants.push(Participant {
            jabatan: jabatan.into(),
            nama,
        });
        self
    }
}
=======
//utils.rs
use actix_multipart::Multipart;
use actix_web::{Error, error::ErrorBadRequest};
use blake3::{self, Hasher};
use chrono::Datelike;
use deunicode::deunicode_with_tofu;
use futures::TryStreamExt;
use mail_send::{Credentials, SmtpClientBuilder, mail_builder::MessageBuilder};
use rand::Rng;
use sanitize_filename::sanitize;
use sodiumoxide::crypto::secretbox;
use std::{env, fmt};
use std::{fs, io::Write, path::Path};
use uuid::Uuid;
// Enkripsi
pub fn encrypt_data(data: &[u8], key: &secretbox::Key) -> (secretbox::Nonce, Vec<u8>) {
    let nonce = secretbox::gen_nonce();
    let ciphertext = secretbox::seal(data, &nonce, key);
    (nonce, ciphertext)
}

// Dekripsi
#[derive(Debug)]
pub enum DecryptionError {
    InvalidCiphertext,
    Utf8Error(std::string::FromUtf8Error),
    InvalidNonce,
}

impl fmt::Display for DecryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecryptionError::InvalidCiphertext => write!(f, "Ciphertext tidak valid atau rusak"),
            DecryptionError::Utf8Error(e) => write!(f, "Kesalahan konversi UTF-8: {}", e),
            DecryptionError::InvalidNonce => write!(f, "Nonce tidak valid atau rusak"),
        }
    }
}

impl std::error::Error for DecryptionError {}

// Fungsi untuk mendekripsi data
pub fn decrypt_data(
    ciphertext: &[u8],
    nonce: &[u8],
    key: &secretbox::Key,
) -> Result<String, DecryptionError> {
    // Log untuk debugging
    log::debug!(
        "Decrypt: ciphertext_len={}, nonce_len={}",
        ciphertext.len(),
        nonce.len()
    );

    // Convert slice to Nonce
    let nonce = secretbox::Nonce::from_slice(nonce).ok_or_else(|| {
        log::error!("Invalid nonce length: {} bytes, expected 24", nonce.len());
        DecryptionError::InvalidNonce
    })?;

    // Decrypt the ciphertext
    let plaintext_bytes = secretbox::open(ciphertext, &nonce, key).map_err(|e| {
        log::error!("Decryption failed: {:?}", e);
        DecryptionError::InvalidCiphertext
    })?;

    // Convert plaintext bytes to String
    let plaintext_string = String::from_utf8(plaintext_bytes).map_err(|e| {
        log::error!("UTF-8 conversion failed: {:?}", e);
        DecryptionError::Utf8Error(e)
    })?;

    log::debug!(
        "Decrypt successful: plaintext_len={}",
        plaintext_string.len()
    );
    Ok(plaintext_string)
}

// Fungsi untuk membuat blind index
pub fn generate_blind_index(data: &[u8], key: &[u8]) -> Vec<u8> {
    let key_array: &[u8; 32] = key.try_into().expect("Blind index key must be 32 bytes");
    let mut hasher = Hasher::new_keyed(key_array);
    hasher.update(data);
    hasher.finalize().as_bytes().to_vec()
}

pub fn get_encryption_key() -> Result<secretbox::Key, Box<dyn std::error::Error>> {
    let encryption_key_hex =
        env::var("ENCRYPTION_KEY").map_err(|e| format!("ENCRYPTION_KEY not set: {}", e))?;

    let encryption_key_bytes = hex::decode(&encryption_key_hex)
        .map_err(|e| format!("Invalid hex encryption key: {}", e))?;

    let key = secretbox::Key::from_slice(&encryption_key_bytes)
        .ok_or("Invalid encryption key size, expected 32 bytes")?;

    log::debug!("Encryption key loaded successfully");
    Ok(key)
}
// Fungsi normalize_phone yang lebih robust
pub fn normalize_phone(phone: &str) -> String {
    // Menghapus semua karakter selain angka
    let normalized_phone: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    // Normalisasi nomor telepon
    if normalized_phone.starts_with("08") {
        format!("62{}", &normalized_phone[1..])
    } else if normalized_phone.starts_with("8") {
        format!("62{}", normalized_phone)
    } else if !normalized_phone.starts_with("62") {
        format!("62{}", normalized_phone)
    } else {
        normalized_phone
    }
}

pub async fn send_verified_email(to: &str, name: &str, password: &str) -> Result<(), String> {
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

    let subject = format!("[{}] Akun Anda Telah Diverifikasi", app_name);
    let text = format!(
        "Halo, {name}\n\nPendaftaran Anda telah Diverifikasi.\n\
         Silakan login menggunakan email ini dengan password: {password}\n\
         Untuk keamanan, disarankan untuk mengganti password setelah login pertama.\n\n— Tim {app_name}"
    );
    let html = format!(
        r#"<div style="font-family:Arial,sans-serif">
            <h1>SALAM PANCASILA</h1>
            <h2>Halo, {name}</h2>
            <p>Pendaftaran Anda telah <b>Diverifikasi</b>.</p>
            <p><strong>Password login Anda:</strong> {password}</p>
            <p>Silakan login menggunakan email ini dengan password di atas.
               Untuk keamanan, disarankan untuk mengganti password setelah login pertama.</p>
            <p>Terima kasih.<br>— Tim {app_name}</p>
          </div>"#
    );

    // ......
    println!("SMTP Configuration:");
    println!("  Host: {}", host);
    println!("  Port: {}", port);
    println!("  Encryption: {}", enc);
    println!("  User: {}", user);

    // Bangun message
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
        _ => client_builder.implicit_tls(false), // STARTTLS
    };

    // Kirim email
    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .send(message)
        .await
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}

pub fn generate_random_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";

    let mut rng = rand::rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    password
}

pub const GALLERY_DIR: &str = "uploads/assets/images/gallery";

/// Pastikan folder tersedia
pub fn ensure_gallery_dir() -> std::io::Result<()> {
    fs::create_dir_all(GALLERY_DIR)
}

/// Simpan semua file image dari multipart field `foto[]`
/// Return: Vec<filename_only>
pub async fn save_gallery_images(mut payload: Multipart) -> Result<Vec<String>, Error> {
    ensure_gallery_dir().map_err(|e| ErrorBadRequest(format!("MkDir: {e}")))?;
    let mut saved = Vec::<String>::new();

    while let Some(field) = payload.try_next().await.map_err(ErrorBadRequest)? {
        let name = field.name().unwrap_or_default().to_string();

        // hanya proses field yang namanya `foto` atau `foto[]`
        if name != "foto" && name != "foto[]" {
            // skip field lain (mis. kegiatan/tanggal bila ikut multipart)
            continue;
        }

        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_default();
        if !content_type.starts_with("image/") {
            return Err(ErrorBadRequest("File bukan image"));
        }

        let cd = field.content_disposition().cloned();
        let orig = cd
            .and_then(|d| d.get_filename().map(|s| s.to_string()))
            .unwrap_or_else(|| "image".to_string());

        let ext = Path::new(&orig)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg");

        let filename = format!("{}.{ext}", Uuid::new_v4().simple());
        let safe = sanitize(&filename);
        let filepath = Path::new(GALLERY_DIR).join(&safe);

        let mut f = std::fs::File::create(&filepath)
            .map_err(|e| ErrorBadRequest(format!("Create file: {e}")))?;

        // tulis chunk
        let mut field_stream = field;
        while let Some(chunk) = field_stream.try_next().await.map_err(ErrorBadRequest)? {
            f.write_all(&chunk)
                .map_err(|e| ErrorBadRequest(format!("Write file: {e}")))?;
        }

        saved.push(safe);
    }

    Ok(saved)
}

/// Hapus 1 file fisik (diabaikan kalau tidak ada)
pub fn delete_gallery_image(filename: &str) -> std::io::Result<()> {
    let p = Path::new(GALLERY_DIR).join(filename);
    if p.exists() {
        std::fs::remove_file(p)?;
    }
    Ok(())
}

/// Hapus semua file dalam array
pub fn delete_gallery_images_all(files: &[String]) {
    for f in files {
        let _ = delete_gallery_image(f);
    }
}

// src/utils.rs
pub fn generate_slug(text: &str) -> String {
    let slug = deunicode_with_tofu(text, "-")
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();

    slug.trim_matches('-')
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// attachment: Some(&(filename, bytes, mime)), contoh: ("file.pdf", vec![..], "application/pdf")
pub async fn send_generic_email_mail_send(
    to: &str,
    cc: Option<&str>,  // koma-separeted
    bcc: Option<&str>, // koma-separeted
    subject: &str,
    text: &str,
    html: Option<&str>,
    attachment: Option<&(String, Vec<u8>, String)>,
) -> Result<(), String> {
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

    // Bangun message (To bisa banyak, tapi di sini satu sesuai UI)
    let mut msg = MessageBuilder::new()
        .from((from_name.as_str(), from_addr.as_str()))
        .subject(subject)
        .text_body(text);

    if let Some(html_body) = html {
        msg = msg.html_body(html_body);
    }

    // To
    msg = msg.to(("", to));

    // CC (koma dipisah)
    if let Some(cc_s) = cc {
        for addr in cc_s.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            msg = msg.cc(("", addr));
        }
    }
    // BCC
    if let Some(bcc_s) = bcc {
        for addr in bcc_s.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            msg = msg.bcc(("", addr));
        }
    }

    // Attachment
    if let Some((filename, bytes, mime)) = attachment {
        // Format: .attachment(mime_type, filename, data)
        msg = msg.attachment(mime.as_str(), filename.as_str(), bytes.as_slice());
    }

    // SMTP client
    let mut client_builder = SmtpClientBuilder::new(host.as_str(), port)
        .credentials(Credentials::new(user.as_str(), pass.as_str()));

    client_builder = match enc.as_str() {
        "SSL" | "SMTPS" => client_builder.implicit_tls(true),
        "PLAIN" | "NONE" => client_builder.implicit_tls(false),
        _ => client_builder.implicit_tls(false), // STARTTLS default
    };

    // Opsional: logging ringan biar sama kayak verified_email mu
    println!(
        "[{}] SMTP → host:{} port:{} enc:{} user:{}",
        app_name, host, port, enc, user
    );

    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect SMTP: {e}"))?
        .send(msg)
        .await
        .map_err(|e| format!("Failed to send email: {e}"))?;

    Ok(())
}

pub async fn send_rejection_email(
    to: &str,
    name: &str,
    alasan_penolakan: &str,
) -> Result<(), String> {
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

    let subject = format!("[{}] Pemberitahuan Penolakan Registrasi PDP", app_name);

    let text = format!(
        "Halo, {name}\n\n\
         Dengan hormat kami sampaikan bahwa registrasi Anda sebagai PDP (Purnapaskibraka Duta Pancasila) \
         TIDAK DAPAT DISETUJUI.\n\n\
         Alasan Penolakan:\n\
         {alasan_penolakan}\n\n\
         Anda dapat memperbaiki data dan mengajukan kembali registrasi melalui sistem.\n\
         Jika Anda memiliki pertanyaan lebih lanjut, silakan hubungi administrator.\n\n\
         — Tim {app_name} \n\n
         EMAIL INI DIKIRIM SECARA OTOMATIS, MOHON UNTUK TIDAK MEMBALAS EMAIL INI."
    );

    let html = format!(
        r#"<div style="font-family:Arial,sans-serif; max-width:600px; margin:0 auto;">
            <h1 style="color:#dc3545; text-align:center;">SALAM PANCASILA</h1>
            <h2 style="color:#333;">Halo, {name}</h2>

            <p>Dengan hormat kami sampaikan bahwa registrasi Anda sebagai <strong>PDP (Purnapaskibraka Duta Pancasila)</strong>
            <strong style="color:#dc3545;">TIDAK DAPAT DISETUJUI</strong>.</p>

            <div style="background:#fff3cd; border:1px solid #ffeaa7; padding:15px; margin:15px 0; border-radius:5px;">
                <h4 style="color:#856404; margin-top:0;">Alasan Penolakan:</h4>
                <p style="color:#333; margin-bottom:0;">{alasan_penolakan}</p>
            </div>

            <p>Anda dapat memperbaiki data dan mengajukan kembali registrasi melalui sistem.</p>
            <p style="margin-top:30px;">Terima kasih.<br>— Tim {app_name}</p>

        </div>
        <div style="margin-top:30px; padding:15px; background:#f8f9fa; border:1px solid #e9ecef; border-radius:5px; text-align:center; font-size:12px; color:#6c757d;">
                <p style="margin:0;">
                    <strong>📧 EMAIL NO-REPLY</strong><br>
                    Email ini dikirim secara otomatis oleh sistem. <br>
                    <strong>Mohon untuk tidak membalas email ini.</strong><br>
                    Jika Anda membutuhkan bantuan, silakan hubungi administrator melalui <a href="https://dppi.bpip.go.id/kontak" target="_blank" rel="noopener">https://dppi.bpip.go.id/kontak</a>.
                </p>
            </div>
        "#,
        name = name,
        alasan_penolakan = alasan_penolakan,
        app_name = app_name
    );

    // Debug info (opsional)
    println!("SMTP Configuration for Rejection Email:");
    println!("  Host: {}", host);
    println!("  Port: {}", port);
    println!("  Encryption: {}", enc);
    println!("  User: {}", user);
    println!("  To: {}", to);

    // Bangun message
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
        _ => client_builder.implicit_tls(false), // STARTTLS
    };

    // Kirim email
    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .send(message)
        .await
        .map_err(|e| format!("Failed to send rejection email: {}", e))?;

    println!("Rejection email successfully sent to {}", to);
    Ok(())
}

/// Fungsi untuk mengirim email bukti submit dokumen pengangkatan DPPI {daerah}
/// - to: email penerima
/// - data: struct berisi data submit dokumen
pub async fn send_submit_confirmation_email(
    to: &str,
    data: &SubmitConfirmationData,
) -> Result<(), String> {
    let from_name: String = "Portal Informasi DPPI".into();
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

    // Subject email
    let subject = format!("Bukti Submit Dokumen Pengangkatan DPPI {}", data.daerah);

    // HTML body sesuai dengan gambar
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Bukti Submit Dokumen Pengangkatan DPPI</title>
  <style>
            body {{
                font-family: Arial, sans-serif;
                line-height: 1.6;
                color: #333;
                max-width: 800px;
                margin: 0 auto;
                padding: 20px;
            }}
            .header {{
                text-align: center;
                margin-bottom: 30px;
                padding-bottom: 20px;
                border-bottom: 2px solid #1a365d;
            }}

            .logo {{
                font-size: 48px;
                font-weight: bold;
                margin: 20px 0;
                display: inline-flex;
                align-items: center;
                justify-content: center;
                gap: 15px;
                color: #b80000;
            }}
            .logo-text {{
                margin-top: 20px;
                margin-left: 20px;
                font-size: 40px;
                font-weight: bold;
            }}
            .logo-bpip {{
                color: #b80000;
            }}
            .logo-dppi {{
                color: #000000;
            }}
            .logo-images {{
                display: flex;
                gap: 10px;
                align-items: center;
            }}
            .logo-image {{
                width: 80px;
                height: auto;
            }}
            .section {{
                margin-bottom: 25px;
                padding: 20px;
                border: 1px solid #e2e8f0;
                border-radius: 8px;
                background-color: #f8fafc;
            }}
            .section-title {{
                color: #2d3748;
                margin-top: 0;
                font-size: 18px;
                font-weight: bold;
            }}
            .info-item {{
                margin-bottom: 10px;
                display: flex;
            }}
            .info-label {{
                font-weight: bold;
                min-width: 180px;
                color: #4a5568;
            }}
            .info-value {{
                color: #2d3748;
            }}
            .participant-table {{
                width: 100%;
                border-collapse: collapse;
                margin-top: 10px;
            }}
            .participant-table th {{
                background-color: #edf2f7;
                padding: 12px;
                text-align: left;
                border: 1px solid #cbd5e0;
                font-weight: bold;
                color: #2d3748;
            }}
            .participant-table td {{
                padding: 12px;
                border: 1px solid #cbd5e0;
            }}
            .participant-table tr:nth-child(even) {{
                background-color: #f7fafc;
            }}
            .footer {{
                margin-top: 30px;
                padding-top: 20px;
                border-top: 1px solid #e2e8f0;
                font-size: 14px;
                color: #718096;
                text-align: center;
            }}
            .note-box {{
                background-color: #fff3cd;
                border: 1px solid #ffeaa7;
                border-radius: 5px;
                padding: 15px;
                margin-top: 20px;
                color: #856404;
            }}
            @media (max-width: 600px) {{
                .logo {{
                    flex-direction: column;
                    gap: 10px;
                }}
                .logo-images {{
                    flex-direction: row;
                    justify-content: center;
                }}
                .logo-image {{
                    width: 60px;
                }}
                .logo-text {{
                    font-size: 24px;
                }}
            }}
        </style>
        </head>
        <body>
            <div class="header">
                <div class="logo">
                    <div class="logo-images">
                        <img src="https://dppi.bpip.go.id/assets/images/logo-bpip.png"
                             alt="Logo BPIP"
                             class="logo-image">
                        <img src="https://dppi.bpip.go.id/assets/images/logo-dppi.png"
                             alt="Logo DPPI"
                             class="logo-image">
                    </div>
                    <div class="logo-text">
                        <span class="logo-dppi">DPPI</span>
                        <span class="logo-bpip">BPIP</span>
                    </div>
                </div>
                <h1 style="color: #1a365d;">Bukti Submit Dokumen Pengangkatan DPPI {daerah}</h1>
                <hr style="border: none; border-top: 2px dashed #cbd5e0; margin: 20px 0;">
            </div>
            <div class="section">
                <h2 class="section-title">DPPI Pusat</h2>
                <p>Berikut ini adalah Bukti Submit <strong>Dokumen Pengangkatan DPPI {daerah}</strong></p>

                <div class="info-item">
                    <span class="info-label">Nama PIC:</span>
                    <span class="info-value">{nama_pic}</span>
                </div>
                <div class="info-item">
                    <span class="info-label">ID Registrasi:</span>
                    <span class="info-value">{id_registrasi}</span>
                </div>
                <div class="info-item">
                    <span class="info-label">Tanggal Submit:</span>
                    <span class="info-value">{tanggal_submit}</span>
                </div>
            </div>

            <div class="section">
                <h2 class="section-title">Nama Calon Peserta</h2>
                <table class="participant-table">
                    <thead>
                        <tr>
                            <th>Jabatan</th>
                            <th>Nama</th>
                        </tr>
                    </thead>
                    <tbody>
                        {participant_rows}
                    </tbody>
                </table>
            </div>

            <div class="note-box">
                <strong>✅ Mohon untuk simpan/screenshot/foto/print halaman ini</strong><br>
                Bukti submit juga telah dikirimkan ke email Anda.
            </div>

            <div class="footer">
                <p>
                    <strong>EMAIL INI DIKIRIM SECARA OTOMATIS</strong><br>
                    Mohon untuk tidak membalas email ini.<br>
                    Jika ada pertanyaan, silakan hubungi administrator melalui portal https://dppi.bpip.go.id/kontak
                </p>
                <p style="margin-top: 15px; font-size: 12px;">
                    © {year} DPPI BPIP - Semua Hak Dilindungi Undang-Undang
                </p>
            </div>
        </body>
        </html>
        "#,
        daerah = data.daerah,
        nama_pic = data.nama_pic,
        id_registrasi = data.id_registrasi,
        tanggal_submit = data.tanggal_submit,
        participant_rows = generate_participant_rows(&data.participants),
        year = chrono::Local::now().year()
    );

    // Plain text version (fallback)
    let text = format!(
        "BUKTI SUBMIT DOKUMEN PENGANGKATAN DPPI {daerah}

DPPI Pusat

Berikut ini adalah Bukti Submit Dokumen Pengangkatan DPPI {daerah}
Daerah: {daerah}
Nama PIC: {nama_pic}
ID Registrasi: {id_registrasi}
Tanggal Submit: {tanggal_submit}

Nama Calon Peserta:
{participants_text}

Mohon untuk simpan/screenshot/foto/print halaman ini, bukti submit juga telah dikirimkan ke email Anda.

---
EMAIL INI DIKIRIM SECARA OTOMATIS
Mohon untuk tidak membalas email ini.
© {year} DPPI BPLP",
        daerah = data.daerah,
        nama_pic = data.nama_pic,
        id_registrasi = data.id_registrasi,
        tanggal_submit = data.tanggal_submit,
        participants_text = generate_participants_text(&data.participants),
        year = chrono::Local::now().year()
    );

    // Bangun message
    let message = MessageBuilder::new()
        .from((from_name.as_str(), from_addr.as_str()))
        .to(("", to))
        .subject(subject)
        .text_body(text)
        .html_body(html);

    // Konfigurasi SMTP client
    let mut client_builder = SmtpClientBuilder::new(host.as_str(), port)
        .credentials(Credentials::new(user.as_str(), pass.as_str()));

    client_builder = match enc.as_str() {
        "SSL" | "SMTPS" => client_builder.implicit_tls(true),
        "PLAIN" | "NONE" => client_builder.implicit_tls(false),
        _ => client_builder.implicit_tls(false), // STARTTLS default
    };

    // Kirim email
    client_builder
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to SMTP server: {}", e))?
        .send(message)
        .await
        .map_err(|e| format!("Failed to send submit confirmation email: {}", e))?;

    log::info!("Submit confirmation email sent to {}", to);
    Ok(())
}

/// Helper function untuk generate participant rows dalam format HTML table
fn generate_participant_rows(participants: &[Participant]) -> String {
    participants
        .iter()
        .map(|p| {
            format!(
                "<tr><td>{}</td><td>{}</td></tr>",
                p.jabatan,
                p.nama
                    .iter()
                    .map(|n| n.as_str())
                    .collect::<Vec<&str>>()
                    .join(" ; ")
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Helper function untuk generate participant text dalam format plain text
fn generate_participants_text(participants: &[Participant]) -> String {
    participants
        .iter()
        .map(|p| {
            format!(
                "{}: {}",
                p.jabatan,
                p.nama
                    .iter()
                    .map(|n| n.as_str())
                    .collect::<Vec<&str>>()
                    .join(" ; ")
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[derive(Debug, Clone)]
pub struct SubmitConfirmationData {
    pub daerah: String,
    pub nama_pic: String,
    pub id_registrasi: String,
    pub tanggal_submit: String,
    pub participants: Vec<Participant>,
}

/// Data structure untuk peserta
#[derive(Debug, Clone)]
pub struct Participant {
    pub jabatan: String,
    pub nama: Vec<String>,
}

/// Contoh implementasi builder untuk SubmitConfirmationData
impl SubmitConfirmationData {
    pub fn new(
        daerah: impl Into<String>,
        nama_pic: impl Into<String>,
        id_registrasi: impl Into<String>,
        tanggal_submit: impl Into<String>,
    ) -> Self {
        Self {
            daerah: daerah.into(),
            nama_pic: nama_pic.into(),
            id_registrasi: id_registrasi.into(),
            tanggal_submit: tanggal_submit.into(),
            participants: Vec::new(),
        }
    }

    pub fn add_participant(mut self, jabatan: impl Into<String>, nama: Vec<String>) -> Self {
        self.participants.push(Participant {
            jabatan: jabatan.into(),
            nama,
        });
        self
    }
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
