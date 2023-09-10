use rusqlite::Row;
use serde::{Deserialize, Serialize};

use crate::video_id::VideoId;

#[derive(Serialize, Deserialize)]
pub(crate) struct VideoTimeStamp {
    video_id: VideoId,
    timestamp: u64,
}
impl VideoTimeStamp {
    pub fn get_video_id(&self) -> VideoId {
        self.video_id
    }
    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
    pub fn from_rusqlite_row(row: &Row) -> Result<VideoTimeStamp, rusqlite::Error> {
        Ok(VideoTimeStamp {
            video_id: row.get(0)?,
            timestamp: row.get(1)?,
        })
    }
}
