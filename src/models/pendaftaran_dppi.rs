use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PendaftaranDppiWithKabupaten {
    pub id: i32,
    pub id_kabupaten: i32,
    pub nama_kabupaten: String,

    // Data PIC
    pub nama_pic: String,
    pub jabatan_pic: String,
    pub nip_pic: String,
    pub no_telp_pic: String,
    pub email_pic: String,

    // Struktur Organisasi
    pub ketua_1: String,
    pub ketua_2: String,
    pub wakil_ketua_1: String,
    pub wakil_ketua_2: String,
    pub sekretaris_1: String,
    pub sekretaris_2: String,
    pub kepala_bidang_dukungan_1: String,
    pub kepala_bidang_dukungan_2: String,
    pub kepala_bidang_kompetensi_1: String,
    pub kepala_bidang_kompetensi_2: String,
    pub kepala_bidang_aktualisasi_1: String,
    pub kepala_bidang_aktualisasi_2: String,
    pub kepala_bidang_kominfo_1: String,
    pub kepala_bidang_kominfo_2: String,

    // Path dokumen
    pub path_surat_sekda: Option<String>,
    pub path_daftar_riwayat_hidup: Option<String>,
    pub path_portofolio: Option<String>,
    pub path_kartu_keluarga: Option<String>,
    pub path_sertifikat_pdp: Option<String>,
    pub path_sertifikat_diktat_pip: Option<String>,
    pub rekomendasi: Option<String>,

    // Status
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,

    // Tambahan dari join
    pub nama_provinsi: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPendaftaranDppi {
    pub id_kabupaten: i32,
    pub nama_kabupaten: String,

    // Data PIC
    pub nama_pic: String,
    pub jabatan_pic: String,
    pub nip_pic: String,
    pub no_telp_pic: String,
    pub email_pic: String,

    // Struktur Organisasi
    pub ketua_1: String,
    pub ketua_2: Option<String>,
    pub wakil_ketua_1: String,
    pub wakil_ketua_2: Option<String>,
    pub sekretaris_1: String,
    pub sekretaris_2: Option<String>,
    pub kepala_bidang_dukungan_1: String,
    pub kepala_bidang_dukungan_2: Option<String>,
    pub kepala_bidang_kompetensi_1: String,
    pub kepala_bidang_kompetensi_2: Option<String>,
    pub kepala_bidang_aktualisasi_1: String,
    pub kepala_bidang_aktualisasi_2: Option<String>,
    pub kepala_bidang_kominfo_1: String,
    pub kepala_bidang_kominfo_2: Option<String>,
    pub rekomendasi: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadDocumentRequest {
    pub field_name: String,
    pub file_name: String,
    pub base64_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
    pub rekomendasi: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterParams {
    pub status: Option<String>,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PendaftaranDppiToEmailKabupaten {
    pub id: i32,
    pub id_kabupaten: i32,
    pub nama_kabupaten: String,

    // Data PIC
    pub nama_pic: String,
    pub jabatan_pic: String,
    pub nip_pic: String,
    pub no_telp_pic: String,
    pub email_pic: String,

    // Struktur Organisasi
    pub ketua_1: String,
    pub ketua_2: String,
    pub wakil_ketua_1: String,
    pub wakil_ketua_2: String,
    pub sekretaris_1: String,
    pub sekretaris_2: String,
    pub kepala_bidang_dukungan_1: String,
    pub kepala_bidang_dukungan_2: String,
    pub kepala_bidang_kompetensi_1: String,
    pub kepala_bidang_kompetensi_2: String,
    pub kepala_bidang_aktualisasi_1: String,
    pub kepala_bidang_aktualisasi_2: String,
    pub kepala_bidang_kominfo_1: String,
    pub kepala_bidang_kominfo_2: String,

    // Path dokumen
    pub path_surat_sekda: Option<String>,
    pub path_daftar_riwayat_hidup: Option<String>,
    pub path_portofolio: Option<String>,
    pub path_kartu_keluarga: Option<String>,
    pub path_sertifikat_pdp: Option<String>,
    pub path_sertifikat_diktat_pip: Option<String>,
    pub rekomendasi: Option<String>,

    // Status
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
}
