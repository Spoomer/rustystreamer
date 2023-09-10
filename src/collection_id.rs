use rusqlite::{types::FromSql, ToSql};
use serde::{Deserialize, Serialize};

/// Wrapper for u32 as VideoId
#[derive(Serialize, Deserialize, Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct CollectionId(pub u32);

impl FromSql for CollectionId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let int = value.as_i64()?;
        Ok(CollectionId(int as u32))
    }
}
impl ToSql for CollectionId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        u32::to_sql(&self.0)
    }
}
impl From<u32> for CollectionId {
    fn from(value: u32) -> Self {
        CollectionId(value)
    }
}
