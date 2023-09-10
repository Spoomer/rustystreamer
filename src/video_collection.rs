use crate::{collection_id::CollectionId, consts};
use std::{collections::HashMap, fs, sync::Mutex};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VideoCollection {
    collection_id: CollectionId,
    title: String,
    parent_id: Option<CollectionId>,
}
impl VideoCollection {
    pub fn from_rusqlite_row(row: &rusqlite::Row) -> Result<VideoCollection, rusqlite::Error> {
        Ok(VideoCollection {
            collection_id: row.get(0)?,
            title: row.get(1)?,
            parent_id: row.get(2)?,
        })
    }
    pub fn get_id(&self) -> CollectionId {
        self.collection_id
    }
    pub fn get_title(&self) -> &str {
        &self.title
    }
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
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
    pub fn get_collections(&self) -> &Mutex<HashMap<CollectionId, VideoCollection>> {
        &self.video_collection
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
        fs::write(consts::COLLECTION_DB_HASH_FILE, hash.as_bytes())?;
        Ok(())
    }
    fn read_from_db() -> Result<HashMap<CollectionId, VideoCollection>, std::io::Error> {
        let index_file = std::fs::read_to_string(consts::VIDEO_COLLECTION_INDEX_PATH)?;
        let entries: Vec<VideoCollection> = serde_json::from_str(&index_file)?;
        let mut map: HashMap<CollectionId, VideoCollection> = HashMap::new();
        for entry in entries {
            map.entry(entry.collection_id).or_insert(entry);
        }
        Ok(map)
    }
}
