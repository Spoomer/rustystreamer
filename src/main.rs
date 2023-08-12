use actix_web::{web, App, HttpServer};
use rustystreamer::video_collection::VideoCollectionIndex;
use rustystreamer::video_index::VideoIndex;
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
    let video_index = web::Data::new(VideoIndex::new()?);
    let collection_index = web::Data::new(VideoCollectionIndex::new()?);
    let config = get_config()?;
    let port = config.port;
    println!("Hosting at http://{}:{}", ip, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(video_index.clone())
            .app_data(collection_index.clone())
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
