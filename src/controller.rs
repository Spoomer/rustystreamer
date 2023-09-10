use super::range_header::RangeHeader;
use super::video_index::VideoIndex;
use super::{config, consts};
use crate::collection_id::CollectionId;
use crate::db_connection::{execute_get_vec, get_connection, Pool};
use crate::video_collection::VideoCollection;
use crate::video_id::VideoId;
use crate::video_index::VideoIndexEntry;
use crate::webmodels::VideoTimeStamp;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::{
    get,
    http::header::{self, ContentType},
    post, web, HttpRequest, HttpResponse, Responder,
};
use rand::Rng;
use std::error::Error;
use std::fs;
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
    // let id1 = id.clone();
    // let collection = execute_single(&db_connection, move |conn| {
    //     let mut stmt = conn.prepare("SELECT * FROM Collections WHERE collection_id = ?1;")?;
    //     let result = stmt.query_row([id1], VideoCollection::from_rusqlite_row)?;
    //     Ok(result)
    // })
    // .await
    // .map_err(ErrorInternalServerError)?;
    let collection_id_cloned_2 = id.clone();
    let child_collection = execute_get_vec(&db_connection, move |conn| {
        let mut stmt =
            conn.prepare("SELECT * FROM Collections WHERE parent_id = ?1 ORDER BY title;")?;
        let result: Result<Vec<VideoCollection>, rusqlite::Error> = stmt
            .query_map([collection_id_cloned_2], VideoCollection::from_rusqlite_row)?
            .map(|r| r)
            .collect();
        Ok(result?)
    })
    .await
    .map_err(ErrorInternalServerError)?;
    let mut collection_html_list = get_collection_html_list(&db_connection, &child_collection)
        .await
        .map_err(ErrorInternalServerError)?;
    video_list.append(&mut collection_html_list);
    let collection_id = CollectionId(id.clone());
    let mut video_hmtl_list = get_video_html_list(collection_id, &db_connection)
        .await
        .map_err(ErrorInternalServerError)?;
    video_list.append(&mut video_hmtl_list);
    file = file.replace("{videoListEntries}", &video_list.concat());
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file))
}

async fn get_video_html_list(
    collection_id: CollectionId,
    db_connection: &web::Data<Pool>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut video_list: Vec<String> = Vec::new();
    let videos: Vec<VideoIndexEntry> = execute_get_vec(&db_connection, move |conn| {
        let mut statement =
            conn.prepare("SELECT * FROM Videos WHERE collection_id = ?1 ORDER BY title;")?;
        let result: Result<Vec<VideoIndexEntry>, rusqlite::Error> = statement
            .query_map([collection_id], VideoIndexEntry::from_rusqlite_row)?
            .map(|r| r)
            .collect();
        Ok(result?)
    })
    .await?;
    for entry in videos {
        let string_id = entry.get_id().0.to_string();
        video_list.push(
            consts::VIDEO_LIST_HTML
                .replace("{itemLink}", &format!("/video/{}", &string_id))
                .replace("{title}", &entry.get_title())
                .replace(
                    "{thumbnailLink}",
                    &format!("/thumbnail/video/{}", &string_id),
                ),
        )
    }
    Ok(video_list)
}
async fn get_root_collection_html_list(
    db_connection: web::Data<Pool>,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let collections: Vec<VideoCollection> = execute_get_vec(&db_connection, |conn| {
        let mut stmt =
            conn.prepare("SELECT * FROM Collections WHERE parent_id IS NULL ORDER BY title;")?;
        let result: Result<Vec<VideoCollection>, rusqlite::Error> = stmt
            .query_map([], VideoCollection::from_rusqlite_row)?
            .map(|r| r)
            .collect();
        Ok(result?)
    })
    .await?;
    Ok(get_collection_html_list(&db_connection, &collections).await?)
}
async fn get_collection_html_list(
    db_connection: &web::Data<Pool>,
    collections: &Vec<VideoCollection>,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let mut video_list: Vec<String> = Vec::new();
    for entry in collections {
        let string_id = entry.get_id().0.to_string();
        if let Some(video) =
            get_video_if_single_in_collection(db_connection, entry.get_id()).await?
        {
            video_list.push(
                consts::VIDEO_LIST_HTML
                    .replace("{itemLink}", &format!("/video/{}", &video.get_id().0))
                    .replace("{title}", &entry.get_title())
                    .replace(
                        "{thumbnailLink}",
                        &format!("/thumbnail/video/{}", &video.get_id().0),
                    ),
            );
        } else {
            video_list.push(
                consts::VIDEO_LIST_HTML
                    .replace("{itemLink}", &format!("/collection/{}", &string_id))
                    .replace("{title}", &entry.get_title())
                    .replace(
                        "{thumbnailLink}",
                        &format!("/thumbnail/collection/{}", &string_id),
                    ),
            );
        }
    }
    Ok(video_list)
}
/// Gets a video entry, if there is only one video in the collection
async fn get_video_if_single_in_collection(
    db_connection: &web::Data<Pool>,
    collection_id: CollectionId,
) -> Result<Option<VideoIndexEntry>, Box<dyn Error + Send + Sync>> {
    let conn = get_connection(&db_connection).await?;
    let mut stmt = conn.prepare("Select * FROM Videos WHERE collection_id = ?1;")?;
    let mut query = stmt.query([collection_id.0])?;
    let mut single: Option<VideoIndexEntry> = None;
    loop {
        if let Some(row) = query.next()? {
            //found 2. row - not single
            if single.is_some() {
                return Ok(Option::None);
            }
            single = Some(VideoIndexEntry::from_rusqlite_row(row)?);
            continue;
        }
        break;
    }
    Ok(single)
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
                .replace("{title}", &video_details.get_title())
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
            return Ok(response);
        }
        None => Ok(HttpResponse::NotFound().finish()),
    }
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

fn get_thumbnail_path<'a>(
    id: &String,
    category: &String,
    thumbnail_root_path: &str,
) -> Result<PathBuf, actix_web::Error> {
    let striped_option = thumbnail_root_path.strip_suffix('/');
    let striped_thumbnail_root_path: &str;
    match striped_option {
        Some(striped) => striped_thumbnail_root_path = striped,
        None => striped_thumbnail_root_path = &thumbnail_root_path,
    }
    let files: Vec<_> = fs::read_dir(format!(
        "{}/{}/{}",
        striped_thumbnail_root_path, category, id
    ))?
    .collect();
    if files.len() == 0 {
        return Err(actix_web::error::ErrorNotFound("no thumbnail available"));
    }
    let mut rng = rand::thread_rng();
    let random = rng.gen_range(0..(files.len() - 1));
    match &files[random] {
        Ok(dir) => Ok(dir.path()),
        Err(err) => {
            Err(err).map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))
        }
    }
}
