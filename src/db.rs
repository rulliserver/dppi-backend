use sqlx::{ Pool, MySql };
use std::env;
use dotenv::dotenv;

pub async fn establish_connection() -> Result<Pool<MySql>, sqlx::Error> {
    dotenv().ok();

    let database_url = env
        ::var("DATABASE_URL")
        .map_err(|_| sqlx::Error::Configuration("DATABASE_URL tidak ditemukan di .env".into()))?;

    let pool = sqlx::mysql::MySqlPoolOptions
        ::new()
        .max_connections(5)
        .connect(&database_url).await
        .map_err(|e| {
            log::error!("Gagal membuat pool database: {:?}", e);
            e
        })?;

    Ok(pool)
}
    