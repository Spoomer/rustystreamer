use std::{collections::HashMap, fs, sync::Mutex};

use crate::{collection_id::CollectionId, consts};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VideoCollection {
    id: CollectionId,
    child_collection: Option<Box<VideoCollection>>,
    name: String,
    videos: Vec<u32>,
}
pub struct VideoCollectionIndex {
    hash: Mutex<blake3::Hash>,
    video_collection: Mutex<HashMap<CollectionId, VideoCollection>>,
}
impl VideoCollectionIndex {
    pub fn new() -> Result<Self, std::io::Error> {
        let map = Self::read_from_db()?;
        let hash = Self::get_video_collection_file_hash()?;
        Self::set_video_collection_db_hash_file(hash)?;
        Ok(VideoCollectionIndex {
            hash: Mutex::new(hash),
            video_collection: Mutex::new(map),
        })
    }
    pub fn reload_collections(&self) -> Result<bool, std::io::Error> {
        let hash = Self::get_video_collection_file_hash()?;
        if !self.hash.lock().unwrap().eq(&hash) {
            *self.hash.lock().unwrap() = hash;
            *self.video_collection.lock().unwrap() = Self::read_from_db()?;
            return Ok(true);
        }
        Ok(false)
    }

    fn get_video_collection_file_hash() -> Result<blake3::Hash, std::io::Error> {
        let source_file = fs::read(consts::VIDEO_COLLECTION_INDEX_PATH)?;
        let index_hash = blake3::hash(&source_file);
        Ok(index_hash)
    }
    fn set_video_collection_db_hash_file(hash: blake3::Hash) -> Result<(), std::io::Error> {
        fs::write(consts::VIDEO_COLLECTION_INDEX_PATH, hash.as_bytes())?;
        Ok(())
    }
    fn read_from_db() -> Result<HashMap<CollectionId, VideoCollection>, std::io::Error> {
        let index_file = std::fs::read_to_string(consts::VIDEO_COLLECTION_INDEX_PATH)?;
        let entries: Vec<VideoCollection> = serde_json::from_str(&index_file)?;
        let mut map: HashMap<CollectionId, VideoCollection> = HashMap::new();
        for entry in entries {
            map.entry(entry.id).or_insert(entry);
        }
        Ok(map)
    }
}
