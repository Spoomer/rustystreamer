use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::consts;
pub struct VideoIndex {
    video_index: Mutex<HashMap<String, VideoIndexEntry>>,
    timestamps: Mutex<HashMap<String, u32>>,
    last_timestamp_save: Mutex<u64>,
}

impl VideoIndex {
    pub fn new() -> Self {
        VideoIndex {
            video_index: Mutex::new(VideoIndexEntry::load_entries_hashmap()),
            timestamps: Mutex::new(Self::load_timestamps()),
            last_timestamp_save: Mutex::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        }
    }
    pub fn get_index<'a>(&'a self) -> &'a Mutex<HashMap<String, VideoIndexEntry>> {
        &self.video_index
    }
    pub fn add_to_index(self, key: String, entry: VideoIndexEntry) {
        let mut index = self.video_index.lock().unwrap();
        index.insert(key, entry);
        let file_content = serde_json::to_string_pretty(&index.clone()).unwrap();
        std::fs::write(consts::VIDEO_INDEX_PATH, file_content).unwrap();
    }
    pub fn get_timestamps<'a>(&'a self) -> &'a Mutex<HashMap<String, u32>> {
        &self.timestamps
    }
    fn load_timestamps() -> HashMap<String, u32> {
        let timestamp_files = std::fs::read_to_string(consts::VIDEO_TIMESTAMPS_PATH).unwrap();
        serde_json::from_str(&timestamp_files).unwrap()
    }
    pub fn update_timestamp(&self, key: String, value: u32) {
        self.timestamps
            .lock()
            .unwrap()
            .entry(key)
            .and_modify(|timestamp| *timestamp = value)
            .or_insert(value);
        if SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
            - Duration::from_secs(*self.last_timestamp_save.lock().unwrap())
            > Duration::from_secs(15)
        {
            self.save_timestamps();
            *self.last_timestamp_save.lock().unwrap() = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
        }
    }
    pub fn save_timestamps(&self) {
        let file_content = serde_json::to_string_pretty(&self.timestamps).unwrap();
        std::fs::write(consts::VIDEO_TIMESTAMPS_PATH, file_content).unwrap();
    }
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct VideoIndexEntry {
    pub filename: String,
    pub title: String,
    pub filetype: String,
}

impl VideoIndexEntry {
    fn load_entries_hashmap() -> HashMap<String, VideoIndexEntry> {
        let index_file = std::fs::read_to_string(consts::VIDEO_INDEX_PATH).unwrap();
        serde_json::from_str(&index_file).unwrap()
    }
}
