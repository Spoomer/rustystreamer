use std::{collections::HashMap, sync::Mutex};

use crate::consts;
pub struct VideoIndex {
    video_index: Mutex<HashMap<String, VideoIndexEntry>>,
}

impl VideoIndex {
    pub fn new() -> Self {
        VideoIndex {
            video_index: Mutex::new(VideoIndexEntry::load_entries_hashmap()),
        }
    }
    pub fn get_index<'a>(&'a self) -> &'a Mutex<HashMap<String, VideoIndexEntry>> {
        &self.video_index
    }
    pub fn add_to_index(self,key : String, entry: VideoIndexEntry) {
        let mut index = self.video_index.lock().unwrap();
        index.insert(key,entry);
        let file_content = serde_json::to_string_pretty(&index.clone()).unwrap();
        std::fs::write(consts::VIDEO_INDEX_PATH, file_content).unwrap();
    }
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct VideoIndexEntry {
    pub filename: String,
    pub title: String,
}

impl VideoIndexEntry {
    fn load_entries_hashmap() -> HashMap<String, VideoIndexEntry> {
        let index_file = std::fs::read_to_string(consts::VIDEO_INDEX_PATH).unwrap();
        serde_json::from_str(&index_file).unwrap()
    }
}
