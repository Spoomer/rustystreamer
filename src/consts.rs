pub const CHUNK_SIZE: usize = usize::pow(2, 20); // 1MB;
pub const VIEW_PATH: &str = "views";
pub const VIDEO_LIST_HTML: &str =
    r#"<li><a href="/video/{videoId}" class="video-list-item">{title}</a></li>"#;
pub const VIDEO_INDEX_PATH: &str = "videoIndex.json";
pub const VIDEO_TIMESTAMPS_PATH: &str = "videoTimeStamps.json";
