//! Backup code generation and management utilities.
//!
//! This module provides functionality for generating secure backup codes that can be
//! used as a fallback authentication method. The codes are generated using a
//! combination of random numbers and timestamps to ensure uniqueness, and are then
//! hashed for security.
//!
//! The generated codes are guaranteed to be unique within the database, with automatic
//! regeneration if a collision occurs.

use rand::{rng, Rng};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::find_all_resources_where_fields;
use crate::models::user_backup_code::UserBackupCode;

/// Generates a single backup code using a cryptographically secure process.
///
/// The code generation process:
/// 1. Generates a random 6-digit number
/// 2. Combines it with the current timestamp
/// 3. Creates a SHA-256 hash of the combination
/// 4. Takes the first 7 bytes of the hash and converts them to hexadecimal
///
/// # Returns
/// A String containing the generated backup code in hexadecimal format
fn generate_code() -> String {
    let mut rng = rng();
    let timestamp = OffsetDateTime::now_utc().unix_timestamp();
    let code = format!("{:06}", rng.random_range(0..1000000));
    let hash = Sha256::digest(format!("{:?}{}", timestamp, code).as_bytes()).to_vec();
    hash[..7]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}

/// Generates a unique backup code and ensures it doesn't exist in the database.
///
/// This function will recursively generate new codes until it finds one that
/// doesn't already exist in the database.
///
/// # Returns
/// A String containing a unique backup code
///
/// # Note
/// If a database error occurs, the function will recursively try again
pub async fn generate_backup_code() -> String {
    let backup_code = generate_code();
    match find_all_resources_where_fields!(
        UserBackupCode,
        vec![("code", DatabaseValue::String(backup_code.clone()))]
    )
    .await
    {
        Ok(backup_codes) => {
            if backup_codes.is_empty() {
                backup_code
            } else {
                Box::pin(generate_backup_code()).await
            }
        }
        Err(err) => {
            println!("Error generating backup code: {:?}", err);
            Box::pin(generate_backup_code()).await
        }
    }
}

/// Generates a set of 10 unique backup codes.
///
/// This function calls `generate_backup_code()` 10 times to create a set of
/// unique backup codes that can be provided to a user.
///
/// # Returns
/// A Vec<String> containing 10 unique backup codes
///
/// # Example
/// ```no_run
/// use crate::utils::backup_codes::generate_backup_codes;
///
/// async fn example() {
///     let codes = generate_backup_codes().await;
///     assert_eq!(codes.len(), 10);
/// }
/// ```
pub async fn generate_backup_codes() -> Vec<String> {
    let mut codes = Vec::new();
    for _ in 0..10 {
        let code = generate_backup_code().await;
        codes.push(code);
    }
    codes
}
