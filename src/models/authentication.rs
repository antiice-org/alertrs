use crate::database::traits::DatabaseResource;
use crate::utils::time::{deserialize_offset_date_time, serialize_offset_date_time};
use rocket::serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Error, Row};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthenticationError {
    UserNotFound,
    InvalidCredentials,
    SessionCreationFailed,
    SessionDeletionFailed,
    SessionUpdateFailed,
    SessionNotFound,
    InvalidToken,
    TokenExpired,
}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthenticationError::UserNotFound => write!(f, "User not found"),
            AuthenticationError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthenticationError::SessionCreationFailed => write!(f, "Failed to create session"),
            AuthenticationError::SessionDeletionFailed => write!(f, "Failed to delete session"),
            AuthenticationError::SessionUpdateFailed => write!(f, "Failed to update session"),
            AuthenticationError::SessionNotFound => write!(f, "Session not found"),
            AuthenticationError::InvalidToken => write!(f, "Invalid token"),
            AuthenticationError::TokenExpired => write!(f, "Token expired"),
        }
    }
}

impl std::error::Error for AuthenticationError {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Authentication {
    pub id: String,
    pub user_id: String,
    pub token: String,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub expires_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub created_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub updated_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub archived_at: Option<OffsetDateTime>,
}

impl DatabaseResource for Authentication {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(Authentication {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token: row.get("token"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
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
        true
    }

    fn is_creatable() -> bool {
        true
    }

    fn is_expirable() -> bool {
        true
    }

    fn is_verifiable() -> bool {
        false
    }
}
