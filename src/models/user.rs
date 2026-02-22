use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub address: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub id_pdp: Option<String>,
    pub id_provinsi: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub created_at: DateTime<chrono::Local>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub password: String,
    pub address: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub remember_token: Option<String>,
    pub id_pdp: Option<String>,
    pub id_provinsi: Option<i32>,
    pub id_kabupaten: Option<i32>,
    pub created_at: DateTime<chrono::Local>,
}

#[derive(Deserialize)]
pub struct UserForm {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub role: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub id_pdp: Option<String>,
    pub id_provinsi: Option<i32>,
    pub id_kabupaten: Option<i32>,
}
