use diesel::backend::Backend;
use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    serialize::{self, ToSql},
    sql_types::Integer,
    sqlite::Sqlite,
};
use serde::{Deserialize, Serialize};

/// Wrapper for u32 as VideoId
#[derive(FromSqlRow, Serialize, Deserialize, Clone, Copy, Eq, Hash, PartialEq, Debug)]
#[diesel(sql_type = Integer)]
pub struct VideoId(pub u32);

impl From<u32> for VideoId {
    fn from(value: u32) -> Self {
        VideoId(value)
    }
}

impl FromSql<diesel::sql_types::Integer, Sqlite> for VideoId {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let id = i32::from_sql(bytes)? as u32;
        Ok(VideoId(id))
    }
}
impl ToSql<diesel::sql_types::Integer, Sqlite> for VideoId {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
        let id = self.0 as i32;
        <i32 as serialize::ToSql<Integer, Sqlite>>::to_sql(&id, out)
    }
}
