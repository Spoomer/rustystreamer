use actix_web::web;

use crate::{
    collection_id::CollectionId,
    consts,
    db_connection::{execute_get_vec, Pool},
    queries::{get_root_collections, get_video_if_single_in_collection},
    util::MultiThreadableError,
    video_collection::VideoCollection,
    video_entry::VideoEntry,
};

pub(crate) async fn get_video_html_list(
    collection_id: CollectionId,
    db_connection: &web::Data<Pool>,
) -> Result<Vec<String>, Box<MultiThreadableError>> {
    let mut video_list: Vec<String> = Vec::new();
    let videos: Vec<VideoEntry> = execute_get_vec(db_connection, move |conn| {
        let mut statement =
            conn.prepare("SELECT * FROM Videos WHERE collection_id = ?1 ORDER BY title;")?;
        let result: Result<Vec<VideoEntry>, rusqlite::Error> = statement
            .query_map([collection_id], VideoEntry::from_rusqlite_row)?
            .collect();
        Ok(result?)
    })
    .await?;
    for entry in videos {
        let string_id = entry.get_id().0.to_string();
        video_list.push(
            consts::VIDEO_LIST_HTML
                .replace("{itemLink}", &format!("/video/{}", &string_id))
                .replace("{title}", entry.get_title())
                .replace(
                    "{thumbnailLink}",
                    &format!("/thumbnail/video/{}", &string_id),
                ),
        )
    }
    Ok(video_list)
}
pub(crate) async fn get_root_collection_html_list(
    db_connection: web::Data<Pool>,
) -> Result<Vec<String>, Box<MultiThreadableError>> {
    let collections = get_root_collections(&db_connection).await?;
    get_collection_html_list(&db_connection, &collections).await
}

pub(crate) async fn get_collection_html_list(
    db_connection: &web::Data<Pool>,
    collections: &Vec<VideoCollection>,
) -> Result<Vec<String>, Box<MultiThreadableError>> {
    let mut video_list: Vec<String> = Vec::new();
    for entry in collections {
        let string_id = entry.get_id().0.to_string();
        if let Some(video) =
            get_video_if_single_in_collection(db_connection, entry.get_id()).await?
        {
            video_list.push(
                consts::VIDEO_LIST_HTML
                    .replace("{itemLink}", &format!("/video/{}", &video.get_id().0))
                    .replace("{title}", entry.get_title())
                    .replace(
                        "{thumbnailLink}",
                        &format!("/thumbnail/video/{}", &video.get_id().0),
                    ),
            );
        } else {
            video_list.push(
                consts::VIDEO_LIST_HTML
                    .replace("{itemLink}", &format!("/collection/{}", &string_id))
                    .replace("{title}", entry.get_title())
                    .replace(
                        "{thumbnailLink}",
                        &format!("/thumbnail/collection/{}", &string_id),
                    ),
            );
        }
    }
    Ok(video_list)
}
