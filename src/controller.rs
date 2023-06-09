use crate::webmodels::VideoTimeStamp;

use super::range_header::RangeHeader;
use super::video_index::VideoIndex;
use super::{config, consts};
use actix_web::{
    get,
    http::header::{self, ContentType},
    post, web, HttpRequest, HttpResponse, Responder,
};
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

#[get("/")]
async fn index(video_index: web::Data<VideoIndex>) -> Result<impl Responder, actix_web::Error> {
    let path: PathBuf = [consts::VIEW_PATH, "index.html"].iter().collect();
    let mut file = std::fs::read_to_string(path)?;
    let mut video_list: Vec<String> = Vec::new();
    for entry in video_index.get_index().lock().unwrap().iter() {
        video_list.push(
            consts::VIDEO_LIST_HTML
                .replace("{videoId}", &entry.0.to_string())
                .replace("{title}", &entry.1.title),
        )
    }
    file = file.replace("{videoListEntries}", &video_list.concat());
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file))
}

#[get("/video/{name}")]
async fn video_page(
    name: web::Path<String>,
    video_index: web::Data<VideoIndex>,
) -> Result<impl Responder, actix_web::Error> {
    if video_index
        .get_index()
        .lock()
        .unwrap()
        .contains_key(name.as_str())
    {
        let video_details = &video_index.get_index().lock().unwrap()[name.as_str()];
        let path: PathBuf = [consts::VIEW_PATH, "video.html"].iter().collect();
        let mut file = std::fs::read_to_string(path)?;
        file = file
            .replace("{title}", &video_details.title)
            .replace("{videoId}", &name);
        return Ok(HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(file));
    }
    Ok(HttpResponse::NotFound().finish())
}

#[get("/video-timestamp/{id}")]
async fn timestamp(
    id: web::Path<String>,
    video_index: web::Data<VideoIndex>,
) -> Result<impl Responder, actix_web::Error> {
    match video_index
        .get_timestamps()
        .lock()
        .unwrap()
        .get(&id.to_string())
    {
        Some(seconds) => Ok(HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(serde_json::to_string(&VideoTimeStamp {
                id: id.to_string(),
                timestamp: *seconds,
            })?)),
        None => Ok(HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(format!("{{\"timestamp\":0, \"id\":{}}}", id))),
    }
}

#[post("/update-video-timestamp")]
async fn update_timestamp(
    video_time_stamp: web::Json<VideoTimeStamp>,
    video_index: web::Data<VideoIndex>,
) -> Result<impl Responder, actix_web::Error> {
    video_index.update_timestamp(video_time_stamp.id.to_string(), video_time_stamp.timestamp)?;
    Ok(HttpResponse::Ok())
}

#[get("/video-resource/{name}")]
async fn load_video(
    name: web::Path<String>,
    request: HttpRequest,
    data: web::Data<config::Config>,
    video_index: web::Data<VideoIndex>,
) -> Result<impl Responder, actix_web::Error> {
    if video_index
        .get_index()
        .lock()
        .unwrap()
        .contains_key(name.as_str())
    {
        let header_map = request.headers();
        if !header_map.contains_key(header::RANGE) {
            return Ok(HttpResponse::BadRequest().finish());
        }
        let video = &video_index.get_index().lock().unwrap()[name.as_str()];
        let path: PathBuf = [&(data.videopath), &video.filename].iter().collect();
        let size = std::fs::metadata(&path)?.len();
        let range = header_map.get(header::RANGE).unwrap();
        let range_header = RangeHeader::parse(range, size)?;
        if range_header.unit != "bytes" {
            return Ok(HttpResponse::BadRequest().finish());
        }
        let content_length = range_header.end - range_header.start + 1;
        let file = std::fs::File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        let _position = buf_reader.seek(SeekFrom::Start(range_header.start))?;
        let mut content = Vec::<u8>::new();
        let mut buffer = [0; consts::CHUNK_SIZE];
        let size_read = buf_reader.read(&mut buffer[..])?;
        if size_read > consts::CHUNK_SIZE {
            return Ok(HttpResponse::InternalServerError().finish());
        }
        content.extend_from_slice(&buffer[..size_read]);

        let response = HttpResponse::PartialContent()
            .content_type(ContentType(
                format!("video/{}", &video.filetype)
                    .parse::<mime::Mime>()
                    .unwrap(),
            ))
            .append_header(("Content-Length", content_length))
            .append_header((
                "Content-Range",
                format!("bytes {}-{}/{}", range_header.start, range_header.end, size),
            ))
            .append_header(("Accept-Range", "bytes"))
            .body(content);
        return Ok(response);
    }
    Ok(HttpResponse::NotFound().finish())
}
