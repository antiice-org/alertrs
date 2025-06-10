use crate::models::authentication::Authentication;
use crate::utils::time::{deserialize_offset_date_time, serialize_offset_date_time};
use crate::{find_one_resource_where_fields, models::authentication::AuthenticationError};
use rocket::{
    request::{FromRequest, Outcome},
    Request,
};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// Represents an authentication token with basic information
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Token {
    /// The ID of the user this token belongs to
    pub user_id: String,
    /// The actual token string
    pub token: String,
    /// When this token expires, stored as a string
    pub expires_at: String,
}

/// Represents a verified authentication token with additional metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct VerifiedToken {
    /// The original raw token string, if available
    pub raw_token: Option<String>,
    /// The user ID associated with this token (renamed to ssoToken in JSON)
    #[serde(rename = "ssoToken")]
    pub user_id: String,
    /// When this token expires, stored as an OffsetDateTime
    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub expires_at: Option<OffsetDateTime>,
}

impl VerifiedToken {
    /// Creates a new VerifiedToken instance
    pub fn new(raw_token: String, user_id: String, expires_at: Option<OffsetDateTime>) -> Self {
        Self {
            raw_token: Some(raw_token),
            user_id,
            expires_at,
        }
    }

    /// Attempts to create a VerifiedToken from a RawToken by validating it against the database
    /// Returns an error if the token is invalid or expired
    pub async fn from_raw(raw_token: RawToken) -> Result<Self, AuthenticationError> {
        let params = vec![("token", &raw_token.value)];
        let authentication = match find_one_resource_where_fields!(Authentication, params).await {
            Ok(authentication) => authentication,
            Err(_) => return Err(AuthenticationError::InvalidToken),
        };
        if authentication.expires_at.is_none()
            || authentication.expires_at.as_ref().unwrap().to_string()
                < OffsetDateTime::now_utc().format(&Rfc3339).unwrap()
        {
            return Err(AuthenticationError::TokenExpired);
        }
        Ok(Self::new(
            raw_token.value,
            authentication.user_id,
            Some(authentication.expires_at.unwrap().clone()),
        ))
    }
}

/// Represents an unverified raw token as received from the client
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct RawToken {
    /// The raw token string
    pub value: String,
}

/// Implements Rocket's FromRequest trait to extract the token from the Authorization header
#[rocket::async_trait]
impl<'r> FromRequest<'r> for RawToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let token = request
            .headers()
            .get_one("Authorization")
            .map(|header| header.split(" ").nth(1).unwrap_or(""));
        Outcome::Success(
            request
                .local_cache(|| RawToken {
                    value: token.unwrap_or("").to_string(),
                })
                .clone(),
        )
    }
}

/// Validates a RawToken and converts it into a VerifiedToken
/// Returns an error if the token is empty, invalid, or expired
pub async fn validate_token(token: RawToken) -> Result<VerifiedToken, AuthenticationError> {
    if token.value.is_empty() {
        println!("Token is empty");
        return Err(AuthenticationError::SessionNotFound);
    }

    match VerifiedToken::from_raw(token).await {
        Ok(token) => Ok(token),
        Err(err) => {
            println!("Error verifying token: {:?}", err);
            return Err(AuthenticationError::InvalidToken);
        }
    }
}
