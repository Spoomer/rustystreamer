use actix_web::{error, http::Error, web};
use dotenvy::dotenv;
use r2d2_sqlite::{self, SqliteConnectionManager};
use rusqlite::Connection;
use std::env;
pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

pub fn open_connection() -> Result<Pool, Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let manager = SqliteConnectionManager::file("weather.db");
    let pool = Pool::new(manager).unwrap();
    Ok(pool)
}
pub async fn execute<T>(
    pool: &Pool,
    f: fn(Connection) -> Result<Vec<T>, Error>,
) -> Result<Vec<T>, Error> {
    let pool = pool.clone();

    let conn: Connection = web::block(move || pool.get())
        .await?
        .map_err(error::ErrorInternalServerError)?;

    web::block(f(conn))
        .await?
        .map_err(error::ErrorInternalServerError)
}
const GET_ALL_QUERY: &str = "SELECT * FROM ";
