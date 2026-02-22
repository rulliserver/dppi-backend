// src/models/majelis_pertimbangan.rs
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MajelisPertimbangan {
    pub id: i64,
    pub id_pdp: Option<String>,
    pub nama_lengkap: String,
    pub photo: Option<String>,
    pub jabatan: String,
}
