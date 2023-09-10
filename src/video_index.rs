use std::{
    collections::HashMap,
    fs,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::video_id::VideoId;
use crate::{collection_id::CollectionId, consts};

pub struct VideoIndex {
    video_index: Mutex<HashMap<VideoId, VideoIndexEntry>>,
    timestamps: Mutex<HashMap<VideoId, u32>>,
    last_timestamp_save: Mutex<u64>,
    last_video_index_hash: Mutex<blake3::Hash>,
}

impl VideoIndex {
    pub fn new() -> Result<Self, std::io::Error> {
        let now: u64;
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(seconds) => now = seconds.as_secs(),
            Err(err) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    err.to_string(),
                ))
            }
        }
        let hash = Self::get_video_index_file_hash()?;
        Self::set_video_db_hash_file(hash)?;
        Ok(VideoIndex {
            video_index: Mutex::new(VideoIndexEntry::load_entries_hashmap()?),
            timestamps: Mutex::new(Self::load_timestamps()?),
            last_timestamp_save: Mutex::new(now),
            last_video_index_hash: Mutex::new(hash),
        })
    }
    pub fn get_index<'a>(&'a self) -> &'a Mutex<HashMap<VideoId, VideoIndexEntry>> {
        &self.video_index
    }
    pub fn set_index(&self, hash: blake3::Hash) -> Result<(), std::io::Error> {
        *self.last_video_index_hash.lock().unwrap() = hash;
        Self::set_video_db_hash_file(hash)?;
        Ok(())
    }
    pub fn reload_index(&self) -> Result<bool, std::io::Error> {
        let hash = Self::get_video_index_file_hash()?;
        if !self.last_video_index_hash.lock().unwrap().eq(&hash) {
            self.set_index(hash)?;
            *self.get_index().lock().unwrap() = VideoIndexEntry::load_entries_hashmap()?;
            return Ok(true);
        }
        Ok(false)
    }

    fn get_video_index_file_hash() -> Result<blake3::Hash, std::io::Error> {
        let source_file = fs::read(consts::VIDEO_INDEX_PATH)?;
        let index_hash = blake3::hash(&source_file);
        Ok(index_hash)
    }
    fn set_video_db_hash_file(hash: blake3::Hash) -> Result<(), std::io::Error> {
        fs::write(consts::VIDEO_DB_HASH_FILE, hash.as_bytes())?;
        Ok(())
    }
    pub fn add_to_index(
        self,
        key: VideoId,
        entry: VideoIndexEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut index = self.video_index.lock().unwrap();
        index.insert(key, entry);
        let file_content = serde_json::to_string_pretty(&index.clone())?;
        std::fs::write(consts::VIDEO_INDEX_PATH, file_content)?;
        Ok(())
    }
    pub fn get_timestamps<'a>(&'a self) -> &'a Mutex<HashMap<VideoId, u32>> {
        &self.timestamps
    }
    fn load_timestamps() -> Result<HashMap<VideoId, u32>, std::io::Error> {
        let path = std::path::Path::new(consts::VIDEO_TIMESTAMPS_PATH);
        if !path.exists() {
            std::fs::write(path, "{}")?;
            return Ok(HashMap::new());
        }
        let timestamp_files = std::fs::read_to_string(consts::VIDEO_TIMESTAMPS_PATH)?;
        Ok(serde_json::from_str(&timestamp_files)?)
    }
    pub fn update_timestamp(
        &self,
        key: VideoId,
        value: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.timestamps
            .lock()
            .unwrap()
            .entry(key)
            .and_modify(|timestamp| *timestamp = value)
            .or_insert(value);
        if SystemTime::now().duration_since(UNIX_EPOCH)?
            - Duration::from_secs(*self.last_timestamp_save.lock().unwrap())
            > Duration::from_secs(15)
        {
            self.save_timestamps()?;
            *self.last_timestamp_save.lock().unwrap() =
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        }
        Ok(())
    }
    pub fn save_timestamps(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_content = serde_json::to_string_pretty(&self.timestamps)?;
        std::fs::write(consts::VIDEO_TIMESTAMPS_PATH, file_content)?;
        Ok(())
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct VideoIndexEntry {
    video_id: VideoId,
    file_name: String,
    title: String,
    file_type: String,
    collection_id: CollectionId,
}

impl VideoIndexEntry {
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
    fn load_entries_hashmap() -> Result<HashMap<VideoId, VideoIndexEntry>, std::io::Error> {
        let index_file = std::fs::read_to_string(consts::VIDEO_INDEX_PATH)?;
        let entries: Vec<VideoIndexEntry> = serde_json::from_str(&index_file)?;
        let mut map: HashMap<VideoId, VideoIndexEntry> = HashMap::new();
        for entry in entries {
            map.entry(entry.video_id).or_insert(entry);
        }
        Ok(map)
    }
    pub fn from_rusqlite_row(row: &rusqlite::Row) -> Result<VideoIndexEntry, rusqlite::Error> {
        Ok(VideoIndexEntry {
            video_id: row.get(0)?,
            file_name: row.get(1)?,
            title: row.get(2)?,
            collection_id: row.get(3)?,
            file_type: row.get(4)?,
        })
    }
}
