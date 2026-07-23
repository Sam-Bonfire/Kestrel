use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::database::{HasArguments, HasValueRef};
use sqlx::decode::Decode;
use sqlx::encode::{Encode, IsNull};
use sqlx::error::BoxDynError;
use sqlx::sqlite::Sqlite;
use uuid::Uuid;

/// A UUID wrapper that correctly handles SQLite TEXT encoding/decoding.
/// SQLite stores UUIDs as TEXT (36-char strings), but sqlx's default `Uuid`
/// decode expects 16-byte BLOB. This wrapper decodes from TEXT for SQLite
/// and delegates to the native UUID decode for Postgres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DbUuid(pub Uuid);

impl DbUuid {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn parse_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Deref for DbUuid {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Uuid> for DbUuid {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<DbUuid> for Uuid {
    fn from(db: DbUuid) -> Self {
        db.0
    }
}

impl From<DbUuid> for String {
    fn from(db: DbUuid) -> Self {
        db.0.to_string()
    }
}

// --- SQLite: decode from TEXT ---

impl<'r> Decode<'r, Sqlite> for DbUuid {
    fn decode(value: <Sqlite as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, Sqlite>>::decode(value)?;
        let uuid = Uuid::parse_str(&s)?;
        Ok(DbUuid(uuid))
    }
}

impl Encode<'_, Sqlite> for DbUuid {
    fn encode_by_ref(&self, args: &mut <Sqlite as HasArguments<'_>>::ArgumentBuffer) -> IsNull {
        <String as Encode<'_, Sqlite>>::encode(self.0.to_string(), args)
    }
}

// --- Postgres: decode from native UUID ---

impl<'r> Decode<'r, sqlx::Postgres> for DbUuid {
    fn decode(
        value: <sqlx::Postgres as HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, BoxDynError> {
        let uuid = <Uuid as Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(DbUuid(uuid))
    }
}

impl Encode<'_, sqlx::Postgres> for DbUuid {
    fn encode_by_ref(
        &self,
        args: &mut <sqlx::Postgres as HasArguments<'_>>::ArgumentBuffer,
    ) -> IsNull {
        <Uuid as Encode<'_, sqlx::Postgres>>::encode_by_ref(&self.0, args)
    }
}

// --- Serde: transparent serialization ---

impl Serialize for DbUuid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for DbUuid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let uuid = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
        Ok(DbUuid(uuid))
    }
}

impl sqlx::Type<Sqlite> for DbUuid {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl sqlx::Type<sqlx::Postgres> for DbUuid {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}
