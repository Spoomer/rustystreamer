use super::range_header::RangeHeader;
use super::{config, consts};
use crate::db_connection::Pool;
use crate::queries::{get_timestamp_store_by_id, get_video_entry_by_id};
use crate::util::MultiThreadableError;
use crate::video_id::VideoId;
use crate::video_time_stamps::VideoTimeStamp;
use actix_web::error::ErrorInternalServerError;
use actix_web::{
    get,
    http::header::{self, ContentType},
    post, web, HttpRequest, HttpResponse, Responder,
};
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

#[get("/video/{id}")]
async fn video_page(
    id: web::Path<u32>,
    db_connection: web::Data<Pool>,
) -> Result<impl Responder, actix_web::Error> {
    match get_video_entry_by_id(db_connection, VideoId(*id)).await {
        Ok(video_details) => {
            let path: PathBuf = [consts::VIEW_PATH, "video.html"].iter().collect();
            let mut file = std::fs::read_to_string(path)?;
            file = file
                .replace("{title}", video_details.get_title())
                .replace("{videoId}", &id.to_string());
            return Ok(HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(file));
        }
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/video-timestamp/{id}")]
async fn timestamp(
    id: web::Path<u32>,
    db_connection: web::Data<Pool>,
) -> Result<impl Responder, actix_web::Error> {
    match get_timestamp_store_by_id(&db_connection, VideoId(*id))
        .await
        .map_err(ErrorInternalServerError)?
    {
        Some(timestamp) => Ok(HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(serde_json::to_string(&timestamp)?)),
        None => Ok(HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(format!("{{\"timestamp\":0, \"id\":{}}}", id))),
    }
}

#[post("/update-video-timestamp")]
async fn update_timestamp(
    video_time_stamp: web::Json<VideoTimeStamp>,
    db_connection: web::Data<Pool>,
) -> Result<impl Responder, actix_web::Error> {
    let vts = video_time_stamp.0;
    crate::queries::update_timestamp(&db_connection, vts)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok())
}

#[get("/video-resource/{id}")]
async fn load_video(
    id: web::Path<u32>,
    request: HttpRequest,
    data: web::Data<config::Config>,
    db_connection: web::Data<Pool>,
) -> Result<impl Responder, actix_web::Error> {
    let video = get_video_entry_by_id(db_connection, VideoId(*id))
        .await
        .map_err(ErrorInternalServerError)?;
    let response = get_video_resource(request, data, video.get_file_name(), video.get_file_type())
        .map_err(ErrorInternalServerError)?;

    Ok(response)
}

#[get("/video-resource/uncategorized/{file_name}")]
async fn load_video_by_file_name(
    file_name: web::Path<String>,
    request: HttpRequest,
    data: web::Data<config::Config>,
) -> Result<impl Responder, actix_web::Error> {
    let file_name = file_name.replace("%2E", ".");
    let decoded_string = html_escape::decode_html_entities(&file_name);
    let response = get_video_resource(
        request,
        data,
        &decoded_string,
        decoded_string.split('.').nth(1).unwrap(),
    )
    .map_err(ErrorInternalServerError)?;
    Ok(response)
}

fn get_video_resource(
    request: HttpRequest,
    data: web::Data<config::Config>,
    file_name: &str,
    file_type: &str,
) -> Result<HttpResponse, Box<MultiThreadableError>> {
    let path: PathBuf = [&(data.video_path), file_name].iter().collect();

    let header_map = request.headers();
    if !header_map.contains_key(header::RANGE) {
        return Ok(HttpResponse::BadRequest().finish());
    }
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
            format!("video/{}", file_type)
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
    Ok(response)
}
