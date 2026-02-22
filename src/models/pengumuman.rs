<<<<<<< HEAD
// src/models/pengumuman.rs
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Pengumuman {
    pub id: i32,
    pub image: String,
    pub link: Option<String>,
    pub created_by: i32,
    pub created_at: DateTime<chrono::Local>,
    pub updated_at: DateTime<chrono::Local>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FetchPengumuman {
    pub id: i32,
    pub image: String,
    pub link: Option<String>,
}
=======
// src/models/pengumuman.rs
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Pengumuman {
    pub id: i32,
    pub image: String,
    pub link: Option<String>,
    pub created_by: i32,
    pub created_at: DateTime<chrono::Local>,
    pub updated_at: DateTime<chrono::Local>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FetchPengumuman {
    pub id: i32,
    pub image: String,
    pub link: Option<String>,
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
