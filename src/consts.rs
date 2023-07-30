pub const CHUNK_SIZE: usize = usize::pow(2, 20); // 1MB;
pub const VIEW_PATH: &str = "views";
pub const VIDEO_LIST_HTML: &str = r#"<a href="/video/{videoId}" class="video-list-item"><div class="max-w-sm rounded overflow-hidden"><img src="{thumbnailLink}" class="w-full" alt="cover-picture"><h5 class="font-medium text-xl mb-2">{title}</h5></div></a>"#;
pub const VIDEO_INDEX_PATH: &str = "videoIndex.json";
pub const VIDEO_TIMESTAMPS_PATH: &str = "videoTimeStamps.json";
