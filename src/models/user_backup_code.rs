use crate::{
    database::traits::DatabaseResource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Error, Row};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub enum UserBackupCodeError {
    CodeAlreadyUsed,
    CodeNotFound,
    CodeExpired,
    CodeNotValid,
    CodeCreationFailed,
    CodeVerificationFailed,
    CodeDeletionFailed,
    CodeUpdateFailed,
}

impl UserBackupCodeError {
    pub fn to_string(&self) -> String {
        match self {
            UserBackupCodeError::CodeAlreadyUsed => "Code already used".to_string(),
            UserBackupCodeError::CodeNotFound => "Code not found".to_string(),
            UserBackupCodeError::CodeExpired => "Code expired".to_string(),
            UserBackupCodeError::CodeNotValid => "Code not valid".to_string(),
            UserBackupCodeError::CodeCreationFailed => "Code creation failed".to_string(),
            UserBackupCodeError::CodeVerificationFailed => "Code verification failed".to_string(),
            UserBackupCodeError::CodeDeletionFailed => "Code deletion failed".to_string(),
            UserBackupCodeError::CodeUpdateFailed => "Code update failed".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBackupCode {
    pub id: Option<String>,
    pub code: Option<String>,
    pub user_id: Option<String>,
    pub used: Option<bool>,

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

impl DatabaseResource for UserBackupCode {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(Self {
            id: row.get("id"),
            code: row.get("code"),
            user_id: row.get("user_id"),
            used: row.get("used"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            archived_at: row.get("archived_at"),
        })
    }

    fn has_id() -> bool {
        true
    }

    fn is_archivable() -> bool {
        true
    }

    fn is_updatable() -> bool {
        true
    }

    fn is_creatable() -> bool {
        true
    }

    fn is_expirable() -> bool {
        false
    }

    fn is_verifiable() -> bool {
        false
    }
}
