use actix_web::{web, App, HttpServer};
use rustystreamer::db_connection::open_connection;
use std::fs;

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
