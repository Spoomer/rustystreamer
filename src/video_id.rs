use rusqlite::{types::FromSql, ToSql};
use serde::{Deserialize, Serialize};

/// Wrapper for u32 as VideoId
#[derive(Serialize, Deserialize, Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct VideoId(pub u32);

impl From<u32> for VideoId {
    fn from(value: u32) -> Self {
        VideoId(value)
    }
}
impl FromSql for VideoId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let int = value.as_i64()?;
        Ok(VideoId(int as u32))
    }
}
impl ToSql for VideoId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        u32::to_sql(&self.0)
    }
}
