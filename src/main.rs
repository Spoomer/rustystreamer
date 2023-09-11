use actix_web::{web, App, HttpServer};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rustystreamer::consts::DROP_DB;
use rustystreamer::consts::INIT_DB;
use rustystreamer::db_connection::execute_single;
use rustystreamer::db_connection::open_connection;
use std::fs;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;

use rustystreamer::config;

use rustystreamer::controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!(
        "Starting in working directory: {}",
        std::env::current_dir()?.to_string_lossy()
    );
    let ip = "0.0.0.0";
    let pool = open_connection().unwrap();
    if std::env::args().any(|x| x == "--init") {
        println!("Initialize DB!");
        init(&pool).await.unwrap();
    }

    let config = get_config()?;
    let port = config.port;
    println!("Hosting at http://{}:{}", ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .service(controller::index_page)
            .service(controller::video_page)
            .service(controller::collection_page)
            .service(controller::load_video)
            .service(actix_files::Files::new("/assets", "./assets"))
            .service(controller::update_timestamp)
            .service(controller::timestamp)
            .service(controller::get_thumbnail)
    })
    .bind((ip, port))?
    .run()
    .await
}

fn get_config() -> std::io::Result<config::Config> {
    let config_json = fs::read_to_string("./config.json")?;
    Ok(serde_json::from_str(&config_json)?)
}

async fn init(
    pool: &Pool<SqliteConnectionManager>,
) -> Result<(), Box<rustystreamer::util::MultiThreadableError>> {
    println!("The existing database will be erased. Do want to continue? y/n");
    let mut input = String::new();
    let _ = stdout().flush();
    stdin().read_line(&mut input)?;
    if !input.chars().next().is_some_and(|c| c == 'y' || c == 'Y') {
        println!("Abort!");
        return Ok(());
    }
    println!("Dropping DB!");
    execute_single(pool, |conn| {
        let result = conn.execute_batch(DROP_DB);
        Ok(result?)
    })
    .await?;
    println!("Creating DB!");
    execute_single(pool, |conn| {
        let result = conn.execute_batch(INIT_DB);
        Ok(result?)
    })
    .await
}
