use crate::collection_id::CollectionId;
use crate::video_id::VideoId;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct VideoEntry {
    video_id: VideoId,
    file_name: String,
    title: String,
    file_type: String,
    collection_id: CollectionId,
}

impl VideoEntry {
    pub fn get_id(&self) -> VideoId {
        self.video_id
    }
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn get_file_name(&self) -> &str {
        &self.file_name
    }
    pub fn get_file_type(&self) -> &str {
        &self.file_type
    }

    pub fn from_rusqlite_row(row: &rusqlite::Row) -> Result<VideoEntry, rusqlite::Error> {
        Ok(VideoEntry {
            video_id: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            collection_id: row.get(3)?,
            file_type: row.get(4)?,
        })
    }
}