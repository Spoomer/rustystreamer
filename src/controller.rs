use crate::video_id::VideoId;
use crate::video_index::VideoIndexEntry;
use crate::webmodels::VideoTimeStamp;

use super::range_header::RangeHeader;
use super::video_index::VideoIndex;
use super::{config, consts};
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::{
    get,
    http::header::{self, ContentType},
    post, web, HttpRequest, HttpResponse, Responder,
};
use rand::Rng;
use std::fs::{self};
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

#[get("/")]
async fn index(video_index: web::Data<VideoIndex>) -> Result<impl Responder, actix_web::Error> {
    let path: PathBuf = [consts::VIEW_PATH, "index.html"].iter().collect();
    let mut file = std::fs::read_to_string(path)?;
    let mut video_list: Vec<String> = Vec::new();
    let index_map = video_index.get_index().lock().unwrap();
    let mut index: Vec<&VideoIndexEntry> = index_map.values().collect();
    index.sort_by_key(|e| &e.title);
    for entry in index {
        let string_id = entry.id.to_string();
        video_list.push(
            consts::VIDEO_LIST_HTML
                .replace("{videoId}", &string_id)
                .replace("{title}", &entry.title)
                .replace("{thumbnailLink}", &format!("thumbnail/{}", &string_id)),
        )
    }
    file = file.replace("{videoListEntries}", &video_list.concat());
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file))
}

#[get("/video/{id}")]
async fn video_page(
    id: web::Path<u32>,
    video_index: web::Data<VideoIndex>,
) -> Result<impl Responder, actix_web::Error> {
    match video_index.get_index().lock().unwrap().get(&VideoId(*id)) {
        Some(video_details) => {
            let path: PathBuf = [consts::VIEW_PATH, "video.html"].iter().collect();
            let mut file = std::fs::read_to_string(path)?;
            file = file
                .replace("{title}", &video_details.title)
                .replace("{videoId}", &id.to_string());
            return Ok(HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(file));
        }
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/video-timestamp/{id}")]
async fn timestamp(
    id: web::Path<u32>,
    video_index: web::Data<VideoIndex>,
) -> Result<impl Responder, actix_web::Error> {
    let video_id = VideoId(*id);
    match video_index.get_timestamps().lock().unwrap().get(&video_id) {
        Some(seconds) => Ok(HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(serde_json::to_string(&VideoTimeStamp {
                id: video_id,
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
    video_index.update_timestamp(video_time_stamp.id, video_time_stamp.timestamp)?;
    Ok(HttpResponse::Ok())
}

#[get("/video-resource/{id}")]
async fn load_video(
    id: web::Path<u32>,
    request: HttpRequest,
    data: web::Data<config::Config>,
    video_index: web::Data<VideoIndex>,
) -> Result<impl Responder, actix_web::Error> {
    match video_index.get_index().lock().unwrap().get(&VideoId(*id)) {
        Some(video) => {
            let header_map = request.headers();
            if !header_map.contains_key(header::RANGE) {
                return Ok(HttpResponse::BadRequest().finish());
            }
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
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/thumbnail/{id}")]
async fn get_thumbnail(id: web::Path<String>) -> Result<impl Responder, actix_web::Error> {
    let path = get_thumbnail_path(id)?;
    let file = actix_files::NamedFile::open(path)?;
    Ok(file
        .use_etag(true)
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}

fn get_thumbnail_path<'a>(id: web::Path<String>) -> Result<PathBuf, actix_web::Error> {
    let files: Vec<_> = fs::read_dir(format!("./thumbnails/{}/", id))?.collect();

    let mut rng = rand::thread_rng();
    let random = rng.gen_range(0..files.len() - 1);
    match &files[random] {
        Ok(dir) => Ok(dir.path()),
        Err(err) => {
            Err(err).map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))
        }
    }
}
