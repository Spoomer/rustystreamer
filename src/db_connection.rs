use diesel::prelude::*;
use diesel::SqliteConnection;
use dotenvy::dotenv;
use std::env;
pub fn open_connection() -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let connection = SqliteConnection::establish(&database_url)?;
    Ok(connection)
}
