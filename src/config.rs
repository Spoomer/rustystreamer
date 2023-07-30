#[derive(serde::Deserialize, Clone)]
pub struct Config {
    pub video_path: String,
    pub thumbnail_path: String,
    pub port: u16,
}
