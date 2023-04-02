use actix_web::{web, App, HttpServer};
use rustystreamer::video_index::VideoIndex;
use std::fs;

use rustystreamer::config;

use rustystreamer::controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let video_index = web::Data::new(VideoIndex::new());
    let config = get_config();
    let port = config.port;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(video_index.clone())
            .service(controller::index)
            .service(controller::video_page)
            .service(controller::load_video)
            .service(actix_files::Files::new("/assets", "./assets"))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

fn get_config() -> config::Config {
    let config_json = fs::read_to_string("./config.json").unwrap();
    serde_json::from_str(&config_json).unwrap()
}
