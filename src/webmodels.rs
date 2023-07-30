use serde::{Deserialize, Serialize};

use crate::video_id::VideoId;

#[derive(Serialize, Deserialize)]
pub struct VideoTimeStamp {
    pub id: VideoId,
    pub timestamp: u32,
}
