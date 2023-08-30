use diesel::backend::Backend;
use diesel::AsExpression;
use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    serialize::{self, ToSql},
    sql_types::Integer,
    sqlite::Sqlite,
};
use serde::{Deserialize, Serialize};

/// Wrapper for u32 as VideoId
#[derive(
    FromSqlRow, AsExpression, Serialize, Deserialize, Clone, Copy, Eq, Hash, PartialEq, Debug,
)]
#[diesel(sql_type = Integer)]
pub struct CollectionId(pub u32);

impl From<u32> for CollectionId {
    fn from(value: u32) -> Self {
        CollectionId(value)
    }
}

impl FromSql<diesel::sql_types::Integer, Sqlite> for CollectionId {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let id = i32::from_sql(bytes)? as u32;
        Ok(CollectionId(id))
    }
}
impl ToSql<diesel::sql_types::Integer, Sqlite> for CollectionId {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
        let id = self.0 as i32;
        <i32 as serialize::ToSql<Integer, Sqlite>>::to_sql(&id, out)
    }
}
