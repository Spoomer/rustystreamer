#[derive(serde::Deserialize, Clone)]
pub struct Config{
    pub videopath : String,
    pub port: u16
}