use crate::collection_id::CollectionId;
use crate::thumbnails::get_thumbnail_path;
use crate::video_id::VideoId;
use super::range_header::RangeHeader;
use super::{config, consts};
use crate::db_connection::Pool;
use crate::html_helper::{
    get_collection_html_list, get_root_collection_html_list, get_video_html_list,
};
use crate::queries::{get_child_collections, get_timestamp_store_by_id, get_video_entry_by_id};
use crate::video_time_stamps::VideoTimeStamp;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::{ContentDisposition, DispositionType};
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
async fn index_page(db_connection: web::Data<Pool>) -> Result<impl Responder, actix_web::Error> {
    let path: PathBuf = [consts::VIEW_PATH, "index.html"].iter().collect();
    let mut file = std::fs::read_to_string(path)?;

    let video_list = get_root_collection_html_list(db_connection)
        .await
        .map_err(ErrorInternalServerError)?;

    file = file.replace("{videoListEntries}", &video_list.concat());
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file))
}

#[get("/collection/{id}")]
async fn collection_page(
    id: web::Path<u32>,
    db_connection: web::Data<Pool>,
) -> Result<impl Responder, actix_web::Error> {
    let path: PathBuf = [consts::VIEW_PATH, "index.html"].iter().collect();
    let mut file = std::fs::read_to_string(path)?;
    let mut video_list: Vec<String> = Vec::new();
    let child_collections = get_child_collections(&db_connection, CollectionId(*id))
        .await
        .map_err(ErrorInternalServerError)?;
    let mut collection_html_list = get_collection_html_list(&db_connection, &child_collections)
        .await
        .map_err(ErrorInternalServerError)?;
    video_list.append(&mut collection_html_list);
    let collection_id = CollectionId(*id);
    let mut video_hmtl_list = get_video_html_list(collection_id, &db_connection)
        .await
        .map_err(ErrorInternalServerError)?;
    video_list.append(&mut video_hmtl_list);
    file = file.replace("{videoListEntries}", &video_list.concat());
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file))
}

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

    let header_map = request.headers();
    if !header_map.contains_key(header::RANGE) {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let path: PathBuf = [&(data.video_path), video.get_file_name()].iter().collect();
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
            format!("video/{}", video.get_file_type())
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

#[get("/thumbnail/{category}/{id}")]
async fn get_thumbnail(
    params: web::Path<(String, String)>,
    data: web::Data<config::Config>,
) -> Result<impl Responder, actix_web::Error> {
    let id = &params.1;
    let category = &params.0;
    let path = get_thumbnail_path(id, category, &data.thumbnail_path)?;
    let file = actix_files::NamedFile::open(path)?;
    Ok(file
        .use_etag(true)
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}
