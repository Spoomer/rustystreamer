use serde::{Deserialize, Serialize};

/// Wrapper for u32 as VideoId
#[derive(Serialize, Deserialize, Clone, Copy, Eq, Hash, PartialEq)]
pub struct VideoId(pub u32);
