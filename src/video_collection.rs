use crate::collection_id::CollectionId;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VideoCollection {
    collection_id: CollectionId,
    title: String,
    parent_id: Option<CollectionId>,
}
impl VideoCollection {
    pub fn from_rusqlite_row(row: &rusqlite::Row) -> Result<VideoCollection, rusqlite::Error> {
        Ok(VideoCollection {
            collection_id: row.get("collection_id")?,
            title: row.get("title")?,
            parent_id: row.get("parent_id")?,
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

    pub(crate) fn get_parent_id(&self) -> Option<CollectionId> {
        self.parent_id
    }
    pub(crate) fn new(title: String, parent_id: Option<CollectionId>) -> Self {
        VideoCollection {
            collection_id: CollectionId(0),
            title,
            parent_id,
        }
    }
}
