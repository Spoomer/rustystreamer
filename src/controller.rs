use actix_web::{
    get,
    http::header::{self, ContentType},
    web, HttpRequest, HttpResponse, Responder,
};
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

use super::{consts, config};
use super::range_header::RangeHeader;
#[get("/")]
async fn index() -> impl Responder {
    let path: PathBuf = [consts::VIEW_PATH, "index.html"].iter().collect();
    let mut file = std::fs::read_to_string(path).unwrap();
    file = file
        .replace("{title}", "Testvideo")
        .replace("{videoId}", "Testvideo");
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file)
}

#[get("/video/{name}")]
async fn video_page(name: web::Path<String>) -> impl Responder {
    if name.to_string() == "Testvideo" {
        let path: PathBuf = [consts::VIEW_PATH, "video.html"].iter().collect();
        let mut file = std::fs::read_to_string(path).unwrap();
        file = file
            .replace("{title}", "Testvideo")
            .replace("{videoId}", "test.mp4");
        return HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(file);
    }
    HttpResponse::NotFound().finish()
}

#[get("/video-resource/{name}")]
async fn load_video(name: web::Path<String>, request: HttpRequest, data: web::Data<config::Config>) -> impl Responder {
    if name.to_string() == "test.mp4" {
        let header_map = request.headers();
        if !header_map.contains_key(header::RANGE) {
            return HttpResponse::BadRequest().finish();
        }
        let path: PathBuf = [&(data.videopath), "test.mp4"].iter().collect();
        let size = std::fs::metadata(&path).unwrap().len();
        let range = header_map.get(header::RANGE).unwrap();
        let range_header = RangeHeader::parse(range, size).unwrap();
        if range_header.unit != "bytes" {
            return HttpResponse::BadRequest().finish();
        }
        let content_length = range_header.end - range_header.start + 1;
        let file = std::fs::File::open(path).unwrap();
        let mut buf_reader = BufReader::new(file);
        let _position = buf_reader
            .seek(SeekFrom::Start(range_header.start))
            .unwrap();
        let mut content = Vec::<u8>::new();
        let mut buffer = [0; consts::CHUNK_SIZE];
        let size_read = buf_reader.read(&mut buffer[..]).unwrap();
        if size_read > consts::CHUNK_SIZE {
            return HttpResponse::InternalServerError().finish();
        }
        content.extend_from_slice(&buffer[..size_read]);

        let response = HttpResponse::PartialContent()
            .content_type(ContentType("video/mp4".parse::<mime::Mime>().unwrap()))
            .append_header(("Content-Length", content_length))
            .append_header((
                "Content-Range",
                format!("bytes {}-{}/{}", range_header.start, range_header.end, size),
            ))
            .append_header(("Accept-Range", "bytes"))
            .body(content);
        return response;
    }
    HttpResponse::NotFound().finish()
}