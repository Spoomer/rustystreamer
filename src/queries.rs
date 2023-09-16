use actix_web::web;

use crate::{
    collection_id::CollectionId,
    db_connection::{execute_get_vec, execute_single, get_connection, Pool},
    util::MultiThreadableError,
    video_collection::VideoCollection,
    video_entry::VideoEntry,
    video_id::VideoId,
    video_time_stamps::VideoTimeStamp,
};
use rusqlite::OptionalExtension;

pub(crate) async fn get_video_entry_by_id(
    db_connection: web::Data<Pool>,
    id: VideoId,
) -> Result<VideoEntry, Box<MultiThreadableError>> {
    execute_single(&db_connection, move |conn| {
        let mut stmt = conn.prepare("SELECT * FROM Videos WHERE video_id = ?1;")?;
        let result = stmt.query_row([id], VideoEntry::from_rusqlite_row)?;
        Ok(result)
    })
    .await
}

pub(crate) async fn get_video_entry_by_collection_id(
    db_connection: &web::Data<Pool>,
    collection_id: CollectionId,
) -> Result<Vec<VideoEntry>, Box<MultiThreadableError>> {
    let videos: Vec<VideoEntry> = execute_get_vec(&db_connection, move |conn| {
        let mut statement =
            conn.prepare("SELECT * FROM Videos WHERE collection_id = ?1 ORDER BY title;")?;
        let result: Result<Vec<VideoEntry>, rusqlite::Error> = statement
            .query_map([collection_id], VideoEntry::from_rusqlite_row)?
            .collect();
        Ok(result?)
    })
    .await?;
    Ok(videos)
}

pub(crate) async fn get_all_videos(
    db_connection: web::Data<Pool>,
) -> Result<Vec<VideoEntry>, Box<MultiThreadableError>> {
    execute_get_vec(&db_connection, move |conn| {
        let mut stmt = conn.prepare("SELECT * FROM Videos;")?;
        let result: Result<Vec<VideoEntry>, rusqlite::Error> =
            stmt.query_map([], VideoEntry::from_rusqlite_row)?.collect();
        Ok(result?)
    })
    .await
}

/// Gets a video entry, if there is only one video in the collection
pub(crate) async fn get_video_if_single_in_collection(
    db_connection: &web::Data<Pool>,
    collection_id: CollectionId,
) -> Result<Option<VideoEntry>, Box<MultiThreadableError>> {
    let conn = get_connection(db_connection).await?;
    let mut stmt = conn.prepare("Select * FROM Videos WHERE collection_id = ?1;")?;
    let mut query = stmt.query([collection_id.0])?;
    let mut single: Option<VideoEntry> = None;
    loop {
        if let Some(row) = query.next()? {
            //found 2. row - not single
            if single.is_some() {
                return Ok(None);
            }
            single = Some(VideoEntry::from_rusqlite_row(row)?);
            continue;
        }
        break;
    }
    Ok(single)
}

pub(crate) async fn get_all_collections(
    db_connection: web::Data<Pool>,
) -> Result<Vec<VideoCollection>, Box<MultiThreadableError>> {
    execute_get_vec(&db_connection, move |conn| {
        let mut stmt = conn.prepare("SELECT * FROM Collections;")?;
        let result: Result<Vec<VideoCollection>, rusqlite::Error> = stmt
            .query_map([], VideoCollection::from_rusqlite_row)?
            .collect();
        Ok(result?)
    })
    .await
}

pub(crate) async fn get_root_collections(
    db_connection: &web::Data<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>>,
) -> Result<Vec<VideoCollection>, Box<MultiThreadableError>> {
    let collections: Vec<VideoCollection> = execute_get_vec(db_connection, |conn| {
        let mut stmt =
            conn.prepare("SELECT * FROM Collections WHERE parent_id IS NULL ORDER BY title;")?;
        let result: Result<Vec<VideoCollection>, rusqlite::Error> = stmt
            .query_map([], VideoCollection::from_rusqlite_row)?
            .collect();
        Ok(result?)
    })
    .await?;
    Ok(collections)
}

pub(crate) async fn get_child_collections(
    db_connection: &web::Data<Pool>,
    collection_id: CollectionId,
) -> Result<Vec<VideoCollection>, Box<MultiThreadableError>> {
    execute_get_vec(db_connection, move |conn| {
        let mut stmt =
            conn.prepare("SELECT * FROM Collections WHERE parent_id = ?1 ORDER BY title;")?;
        let result: Result<Vec<VideoCollection>, rusqlite::Error> = stmt
            .query_map([collection_id], VideoCollection::from_rusqlite_row)?
            .collect();
        Ok(result?)
    })
    .await
}

pub(crate) async fn _get_collection_by_id(
    db_connection: &web::Data<Pool>,
    collection_id: CollectionId,
) -> Result<VideoCollection, Box<MultiThreadableError>> {
    execute_single(db_connection, move |conn| {
        let mut stmt = conn.prepare("SELECT * FROM Collections WHERE collection_id = ?1;")?;
        let result = stmt.query_row([collection_id], VideoCollection::from_rusqlite_row)?;
        Ok(result)
    })
    .await
}

pub(crate) async fn get_collection_by_title(
    db_connection: &web::Data<Pool>,
    title: String,
) -> Result<VideoCollection, Box<MultiThreadableError>> {
    execute_single(db_connection, move |conn| {
        let mut stmt = conn.prepare("SELECT * FROM Collections WHERE title= ?1;")?;
        let result = stmt.query_row([title], VideoCollection::from_rusqlite_row)?;
        Ok(result)
    })
    .await
}

pub(crate) async fn get_timestamp_store_by_id(
    db_connection: &web::Data<Pool>,
    video_id: VideoId,
) -> Result<Option<VideoTimeStamp>, Box<MultiThreadableError>> {
    execute_single(db_connection, move |conn| {
        let mut stmnt = conn.prepare("SELECT * FROM VideoTimeStamps WHERE video_id = ?1;")?;
        let result = stmnt
            .query_row([video_id.0], VideoTimeStamp::from_rusqlite_row)
            .optional()?;
        Ok(result)
    })
    .await
}

pub(crate) async fn update_timestamp(
    db_connection: &web::Data<Pool>,
    video_time_stamp: VideoTimeStamp,
) -> Result<(), Box<MultiThreadableError>> {
    let connection = get_connection(db_connection).await?;
    let vts = video_time_stamp;
    let changed_rows = web::block(move || {
        connection.execute(
            "UPDATE VideoTimeStamps SET timestamp = ?1 WHERE video_id = ?2",
            [vts.get_timestamp(), vts.get_video_id().0 as u64],
        )
    })
    .await??;
    let connection = get_connection(db_connection).await?;
    if changed_rows == 0 {
        let _ = web::block(move || {
            connection.execute(
                "INSERT INTO VideoTimeStamps(video_id, timestamp) VALUES(?1, ?2);",
                [
                    video_time_stamp.get_video_id().0 as u64,
                    video_time_stamp.get_timestamp(),
                ],
            )
        })
        .await??;
    }
    Ok(())
}
pub(crate) async fn insert_collection(
    db_connection: &web::Data<Pool>,
    collection: VideoCollection,
) -> Result<(), Box<MultiThreadableError>> {
    let connection = get_connection(db_connection).await?;
    let _ = web::block(move || {
        match collection.get_parent_id() {
            Some(id) => {
                connection.execute(
                    "INSERT INTO Collections(title, parent_id) VALUES(?1, ?2);",
                    [collection.get_title(), &id.0.to_string()],
                )},
            None => {
                connection.execute(
                    "INSERT INTO Collections(title) VALUES(?1);",
                    [collection.get_title()],
                )},
        }
    })
    .await??;
    Ok(())
}
pub(crate) async fn insert_video_entry(
    db_connection: &web::Data<Pool>,
    video_entry: VideoEntry,
) -> Result<(), Box<MultiThreadableError>> {
    let connection = get_connection(db_connection).await?;
    let _ = web::block(move || {
        connection.execute(
            "INSERT INTO Videos(title, file_name, file_type, collection_id) VALUES(?1, ?2, ?3, ?4);",
            [video_entry.get_title(), video_entry.get_file_name(),video_entry.get_file_type(), &video_entry.get_collection_id().0.to_string()],
        )
    })
    .await??;
    Ok(())
}
