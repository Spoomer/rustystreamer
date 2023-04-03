use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct VideoTimeStamp{
    pub id : String,
    pub timestamp : u32
}