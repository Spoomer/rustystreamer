use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VideoTimeStamp {
    pub id: String,
    pub timestamp: u32,
}
