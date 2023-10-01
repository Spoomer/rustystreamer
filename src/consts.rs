pub const CHUNK_SIZE: usize = usize::pow(2, 20); // 1MB
pub const VIEW_PATH: &str = "views";
pub const VIDEO_LIST_HTML: &str = r#"
<div class="video-list-item">
    <div class="max-w-sm rounded overflow-hidden focus:outline focus:outline-2 focus:outline-orange-400">
        <a href="{itemLink}"><img src="{thumbnailLink}" class="w-full" alt="cover-picture"></a>
        <div class="flex justify-between"><a href="{itemLink}" class="font-medium text-xl mb-2 grow">{title}</a><a href="{editLink}" class="edit-link">:</a></div>
    </div>
</div>"#;
pub const VIDEO_INDEX_PATH: &str = "videoIndex.json";
pub const VIDEO_TIMESTAMPS_PATH: &str = "videoTimeStamps.json";
pub const VIDEO_DB_HASH_FILE: &str = "video_db.hash";
pub const COLLECTION_DB_HASH_FILE: &str = "collection_db.hash";
pub const EMPTY_U8_ARRAY: [u8; 0] = [];
pub const VIDEO_COLLECTION_INDEX_PATH: &str = "videoCollectionIndex.json";
pub const DEFAULT_THUMBNAIL_PATH: &str = "assets/images/default_thumbnail.png";
pub const DB_URL: &str = "DATABASE_URL";
pub const INIT_DB: &str = "
CREATE TABLE Videos (
    video_id INTEGER PRIMARY KEY ASC NOT NULL,
    title NVARCHAR(100) NOT NULL,
    file_name NVARCHAR(100) NOT NULL,
    file_type NVARCHAR(10) NOT NULL,
    collection_id INTEGER NOT NULL
);

CREATE INDEX VideosCollectionId ON Videos(collection_id);

CREATE TABLE Collections (
    collection_id INTEGER PRIMARY KEY ASC NOT NULL,
    title NVARCHAR(100) NOT NULL,
    parent_id INTEGER NULL   
);

CREATE INDEX CollectionsParentId ON Collections(parent_id);

CREATE TABLE VideoTimeStamps (
    video_id INTEGER PRIMARY KEY ASC NOT NULL,
    timestamp INTEGER NOT NULL
);  
";
pub const DROP_DB: &str = "
DROP INDEX IF EXISTS VideosCollectionId;
DROP TABLE IF EXISTS Videos;
DROP INDEX IF EXISTS CollectionsParentId;
DROP TABLE IF EXISTS Collections;
DROP TABLE IF EXISTS VideoTimeStamps;
";
