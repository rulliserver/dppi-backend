<<<<<<< HEAD
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
=======
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
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
