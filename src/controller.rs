use super::{config, consts};
use crate::collection_id::CollectionId;
use crate::db_connection::Pool;
use crate::html_helper::{
    create_video_html_list, get_collection_html_list, get_root_collection_html_list,
    get_video_html_list, CreateVideoHtmlListParameter,
};
use crate::queries::{
    get_all_collections, get_child_collections, get_collection_by_title, insert_collection,
    insert_video_entry,
};
use crate::thumbnails::get_thumbnail_path;
use crate::uncategorized::get_uncategorized_videos;
use crate::video_collection::VideoCollection;
use crate::video_entry::VideoEntry;
use crate::video_id::VideoId;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::post;
use actix_web::{get, http::header::ContentType, web, HttpResponse, Responder};
use html_escape;
use std::collections::HashMap;
use std::path::PathBuf;

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

#[get("collections")]
async fn get_collections(
    db_connection: web::Data<Pool>,
) -> Result<impl Responder, actix_web::Error> {
    let collections = get_all_collections(db_connection)
        .await
        .map_err(ErrorInternalServerError)?;
    let map = collections
        .iter()
        .map(|c| (c.get_id(), c.get_title()))
        .collect::<HashMap<_, _>>();
    let json = serde_json::to_string(&map)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(json))
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
    let mut video_html_list = get_video_html_list(collection_id, &db_connection)
        .await
        .map_err(ErrorInternalServerError)?;
    video_list.append(&mut video_html_list);
    file = file.replace("{videoListEntries}", &video_list.concat());
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file))
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

#[get("/uncategorized")]
async fn get_uncategorized_videos_page(
    db_connection: web::Data<Pool>,
    data: web::Data<config::Config>,
) -> Result<impl Responder, actix_web::Error> {
    let uncategorized_videos = get_uncategorized_videos(db_connection, data)
        .await
        .map_err(ErrorInternalServerError)?;
    let path: PathBuf = [consts::VIEW_PATH, "uncategorized-videos.html"]
        .iter()
        .collect();
    let mut file = std::fs::read_to_string(path)?;
    let mut params = Vec::<CreateVideoHtmlListParameter>::new();
    for video in uncategorized_videos {
        let video = video.replace('.', "%2E");
        params.push(CreateVideoHtmlListParameter {
            item_link: format!("uncategorized/{}", html_escape::encode_text(&video)),
            title: video,
            thumbnail_id: String::from("-1"),
        })
    }
    let video_list = create_video_html_list(params);

    file = file.replace("{videoListEntries}", &video_list.concat());
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file))
}

#[get("/uncategorized/{file_name}")]
async fn get_uncategorized_video_page(
    file_name: web::Path<String>,
) -> Result<impl Responder, actix_web::Error> {
    let path: PathBuf = [consts::VIEW_PATH, "uncategorized-video.html"]
        .iter()
        .collect();
    let mut file = std::fs::read_to_string(path)?;
    file = file.replace("{title}", &file_name).replace("{file_name}", &file_name);
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(file))
}
#[derive(serde::Deserialize)]
struct PostVideoEntry {
    video_id: VideoId,
    title: String,
    file_name: String,
    collection_id: String,
}

#[post("/uncategorized")]
async fn post_uncategorized_video(
    categorized_video: web::Form<PostVideoEntry>,
    db_connection: web::Data<Pool>,
) -> Result<impl Responder, actix_web::Error> {
    let new_entry: VideoEntry;
    if categorized_video.collection_id.is_empty() {
        insert_collection(
            &db_connection,
            VideoCollection::new(categorized_video.title.clone(), None),
        )
        .await
        .map_err(ErrorInternalServerError)?;

        let collection = get_collection_by_title(&db_connection, categorized_video.title.clone())
            .await
            .map_err(ErrorInternalServerError)?;

        new_entry = VideoEntry::new(
            categorized_video.video_id,
            categorized_video.title.clone(),
            categorized_video.file_name.clone(),
            String::from(categorized_video.file_name.split('.').nth(1).unwrap()),
            collection.get_id(),
        );
    } else {
        new_entry = VideoEntry::new(
            categorized_video.video_id,
            categorized_video.title.clone(),
            categorized_video.file_name.clone(),
            String::from(categorized_video.file_name.split('.').nth(1).unwrap()),
            CollectionId(categorized_video.collection_id.parse().map_err(ErrorInternalServerError)?),
        );
    }
    insert_video_entry(&db_connection, new_entry)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok())
}

#[post("/collection")]
async fn post_collection(
    collection: web::Json<VideoCollection>,
    db_connection: web::Data<Pool>,
) -> Result<impl Responder, actix_web::Error> {
    insert_collection(&db_connection, collection.0)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok())
}
