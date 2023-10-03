use actix_web::web;

use crate::queries::get_video_entry_by_collection_id;
use crate::{
    collection_id::CollectionId,
    consts,
    db_connection::Pool,
    queries::{get_root_collections, get_video_if_single_in_collection},
    util::MultiThreadableError,
    video_collection::VideoCollection,
};

pub(crate) async fn get_video_html_list(
    collection_id: CollectionId,
    db_connection: &web::Data<Pool>,
) -> Result<Vec<String>, Box<MultiThreadableError>> {
    let video_entries = get_video_entry_by_collection_id(db_connection, collection_id).await?;
    let videos: Vec<CreateVideoHtmlListParameter> = video_entries
        .iter()
        .map(|entry| {
            let string_id = entry.get_id().0.to_string();
            CreateVideoHtmlListParameter {
                item_link: format!("/video/{}", string_id),
                title: entry.get_title().to_string(),
                thumbnail_id: string_id,
            }
        })
        .collect();
    let video_list = create_video_html_list(videos);
    Ok(video_list)
}

pub(crate) struct CreateVideoHtmlListParameter {
    pub item_link: String,
    pub title: String,
    pub thumbnail_id: String,
}

pub(crate) fn create_video_html_list(
    create_video_html_list_param: Vec<CreateVideoHtmlListParameter>,
) -> Vec<String> {
    let mut video_list = Vec::<String>::new();
    for param in create_video_html_list_param {
        video_list.push(
            consts::VIDEO_LIST_HTML
                .replace("{itemLink}", &param.item_link)
                .replace("{editLink}", &param.item_link)
                .replace("{title}", &param.title)
                .replace(
                    "{thumbnailLink}",
                    &format!("/thumbnail/video/{}", &param.thumbnail_id),
                ),
        )
    }
    video_list
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
                    )
                    .replace(
                        "{editLink}",
                        &format!("/video-entry/edit/{}", &video.get_id().0),
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
                    )
                    .replace(
                        "{editLink}",
                        &format!("/collection-entry/edit/{}", &string_id),
                    ),
            );
        }
    }
    Ok(video_list)
}
