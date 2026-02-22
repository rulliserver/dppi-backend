use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PendaftaranDppiWithProvinsi {
    pub id: i32,
    pub id_provinsi: i32,
    pub nama_provinsi: String,

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
    pub kepala_divisi_dukungan_1: String,
    pub kepala_divisi_dukungan_2: String,
    pub kepala_divisi_kompetensi_1: String,
    pub kepala_divisi_kompetensi_2: String,
    pub kepala_divisi_aktualisasi_1: String,
    pub kepala_divisi_aktualisasi_2: String,
    pub kepala_divisi_kominfo_1: String,
    pub kepala_divisi_kominfo_2: String,

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
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPendaftaranDppiProvinsi {
    pub id_provinsi: i32,
    pub nama_provinsi: String,

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
    pub kepala_divisi_dukungan_1: String,
    pub kepala_divisi_dukungan_2: String,
    pub kepala_divisi_kompetensi_1: String,
    pub kepala_divisi_kompetensi_2: String,
    pub kepala_divisi_aktualisasi_1: String,
    pub kepala_divisi_aktualisasi_2: String,
    pub kepala_divisi_kominfo_1: String,
    pub kepala_divisi_kominfo_2: String,
    pub rekomendasi: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadDocumentRequestProvinsi {
    pub field_name: String,
    pub file_name: String,
    pub base64_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStatusRequestProvinsi {
    pub status: String,
    pub rekomendasi: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterParamsProvinsi {
    pub status: Option<String>,
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
