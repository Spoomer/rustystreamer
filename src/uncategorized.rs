use std::collections::HashSet;
use actix_web::web;
use crate::config;
use crate::db_connection::Pool;
use crate::queries::get_all_videos;
use crate::util::MultiThreadableError;

pub(crate) async fn get_uncategorized_videos(
    db_connection: web::Data<Pool>,
    data: web::Data<config::Config>,
) -> Result<Vec<String>, Box<MultiThreadableError>> {
    let all_videos: Vec<String> =
        get_all_videos_in_video_path(data)?;

    let categorized_videos: HashSet<String> = get_all_videos(db_connection)
        .await?
        .into_iter()
        .map(|v| String::from(v.get_file_name()))
        .collect();
    let mut uncategorized_videos: Vec<String> = Vec::new();
    for video in all_videos.into_iter() {
        if !categorized_videos.contains(&video) {
            uncategorized_videos.push(video);
        }
    }
    Ok(uncategorized_videos)
}

fn get_all_videos_in_video_path(
    data: web::Data<config::Config>,
) -> Result<Vec<String>, Box<MultiThreadableError>> {
    let read_dir = std::fs::read_dir(&data.video_path)?;
    let videos = read_dir
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(String::from))
            })
        })
        .collect();
    Ok(videos)
}
