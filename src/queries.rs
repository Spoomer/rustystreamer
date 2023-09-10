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
                return Ok(Option::None);
            }
            single = Some(VideoEntry::from_rusqlite_row(row)?);
            continue;
        }
        break;
    }
    Ok(single)
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
    let connection = get_connection(&db_connection).await?;
    let _ = web::block(move || {
        connection.execute(
            "UPDATE VideoTimeStamps SET timestamp = ?1 WHERE video_id = ?2",
            [
                video_time_stamp.get_timestamp(),
                video_time_stamp.get_video_id().0 as u64,
            ],
        )
    })
    .await??;
    Ok(())
}