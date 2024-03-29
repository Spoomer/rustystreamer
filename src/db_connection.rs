use actix_web::web;
use dotenvy::dotenv;
use r2d2::PooledConnection;
use r2d2_sqlite::{self, SqliteConnectionManager};
use std::env;

use crate::consts::DB_URL;
pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;
pub fn open_connection() -> Result<Pool, Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var(DB_URL)?;
    let manager = SqliteConnectionManager::file(database_url);
    let pool = Pool::new(manager)?;
    Ok(pool)
}

pub async fn get_connection(
    pool: &Pool,
) -> Result<PooledConnection<SqliteConnectionManager>, Box<dyn std::error::Error + Send + Sync>> {
    let pool = pool.clone();

    let result = web::block(move || pool.get()).await??;
    Ok(result)
}

pub async fn execute_get_vec<T: Send + Sized + 'static>(
    pool: &Pool,
    func: impl FnOnce(Connection) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + 'static,
) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get()).await??;

    let result = web::block(move || func(conn)).await??;
    Ok(result)
}

pub async fn execute_single<T: Send + Sized + 'static>(
    pool: &Pool,
    func: impl FnOnce(Connection) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + 'static,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let pool = pool.clone();

    let conn = web::block(move || pool.get()).await??;

    let result = web::block(move || func(conn)).await??;
    Ok(result)
}
