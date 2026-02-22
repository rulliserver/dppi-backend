<<<<<<< HEAD
use actix_web::HttpRequest;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub id_pdp: Option<String>,
    pub user_id: String,
    pub nama_user: String,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
}

pub fn generate_jwt(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let now = Utc::now();
    let claims = Claims {
        sub: user.email.clone(),
        role: user.role.clone(),
        id_pdp: user.id_pdp.clone(),
        nama_user: user.name.clone(),
        user_id: user.id.clone(),
        id_kabupaten: user.id_kabupaten.clone(),
        id_provinsi: user.id_provinsi.clone(),
        exp: (now + chrono::Duration::days(2)).timestamp() as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_jwt(req: &HttpRequest) -> Result<Claims, actix_web::Error> {
    let token = req
        .cookie("access_token")
        .ok_or_else(|| {
            log::error!("No access_token cookie found in request to {}", req.path());
            actix_web::error::ErrorUnauthorized("Token tidak ditemukan")
        })?
        .value()
        .to_string();

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| {
        log::error!("JWT verification failed for token {}: {:?}", token, e);
        actix_web::error::ErrorUnauthorized(format!("Invalid or expired token: {}", e))
    })?;

    Ok(token_data.claims)
}
=======
use actix_web::HttpRequest;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub id_pdp: Option<String>,
    pub user_id: String,
    pub nama_user: String,
    pub id_kabupaten: Option<i32>,
    pub id_provinsi: Option<i32>,
}

pub fn generate_jwt(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let now = Utc::now();
    let claims = Claims {
        sub: user.email.clone(),
        role: user.role.clone(),
        id_pdp: user.id_pdp.clone(),
        nama_user: user.name.clone(),
        user_id: user.id.clone(),
        id_kabupaten: user.id_kabupaten.clone(),
        id_provinsi: user.id_provinsi.clone(),
        exp: (now + chrono::Duration::days(2)).timestamp() as usize,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_jwt(req: &HttpRequest) -> Result<Claims, actix_web::Error> {
    let token = req
        .cookie("access_token")
        .ok_or_else(|| {
            log::error!("No access_token cookie found in request to {}", req.path());
            actix_web::error::ErrorUnauthorized("Token tidak ditemukan")
        })?
        .value()
        .to_string();

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| {
        log::error!("JWT verification failed for token {}: {:?}", token, e);
        actix_web::error::ErrorUnauthorized(format!("Invalid or expired token: {}", e))
    })?;

    Ok(token_data.claims)
}
>>>>>>> 84a9b1b1877d3e277f4e3c4af63ae68d6cdc7179
