use crate::database::traits::DatabaseResource;
use crate::utils::time::{deserialize_offset_date_time, serialize_offset_date_time};
use rocket::serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Error, Row};
use std::fmt;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub enum UserTokenError {
    UserTokenCreationFailed,
    UserTokenUpdateFailed,
    UserTokenDeletionFailed,
    UserTokenNotFound,
}

impl fmt::Display for UserTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserTokenError::UserTokenCreationFailed => write!(f, "User token creation failed"),
            UserTokenError::UserTokenUpdateFailed => write!(f, "User token update failed"),
            UserTokenError::UserTokenDeletionFailed => write!(f, "User token deletion failed"),
            UserTokenError::UserTokenNotFound => write!(f, "User token not found"),
        }
    }
}

impl std::error::Error for UserTokenError {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserToken {
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub token_value: Option<String>,
    pub token_type: Option<String>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub created_at: Option<OffsetDateTime>, 

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub verified_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub archived_at: Option<OffsetDateTime>,
}

impl DatabaseResource for UserToken {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(UserToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token_value: row.get("token_value"),
            token_type: row.get("token_type"),
            created_at: row.get("created_at"),
            verified_at: row.get("verified_at"),
            archived_at: row.get("archived_at"),
        })
    }

    fn has_id() -> bool {
        true
    }

    fn is_archivable() -> bool {
        false
    }

    fn is_updatable() -> bool {
        false
    }

    fn is_creatable() -> bool {
        true
    }

    fn is_expirable() -> bool {
        false
    }

    fn is_verifiable() -> bool {
        true
    }
}