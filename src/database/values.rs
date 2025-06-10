use sqlx::postgres::PgArgumentBuffer;
use sqlx::{encode::IsNull, error::BoxDynError, Encode, Postgres, Type};
use std::fmt::{self, Display};
use std::iter::FromIterator;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

/// Represents a value that can be stored in and retrieved from the database.
/// This enum provides type-safe handling of different data types commonly used
/// in database operations.
///
/// Each variant stores its value as a String to provide a uniform interface
/// for database operations while maintaining type information through the variant.
#[derive(Debug, Clone)]
pub enum DatabaseValue {
    /// Represents a NULL value in the database
    #[allow(dead_code)]
    None,
    /// Represents a static string value
    #[allow(dead_code)]
    Str(&'static str),
    /// Represents an owned String value
    #[allow(dead_code)]
    String(String),
    /// Represents an owned String value as a text type
    #[allow(dead_code)]
    Text(String),
    /// Represents an integer value stored as a String
    #[allow(dead_code)]
    Int(String),
    /// Represents a 64-bit integer value stored as a String
    #[allow(dead_code)]
    Int64(String),
    /// Represents a floating-point value stored as a String
    #[allow(dead_code)]
    Float(String),
    /// Represents a boolean value stored as a String
    #[allow(dead_code)]
    Boolean(String),
    /// Represents a datetime value stored as an ISO 8601 formatted String
    #[allow(dead_code)]
    DateTime(String),
}

/// Implements string representation for DatabaseValue for debugging and logging purposes
impl Display for DatabaseValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Implements encoding for PostgreSQL database operations.
/// This allows DatabaseValue to be used directly in SQL queries with sqlx.
impl<'q> Encode<'q, Postgres> for DatabaseValue {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        match self {
            DatabaseValue::None => Ok(IsNull::Yes),
            DatabaseValue::Str(s) => Encode::<Postgres>::encode_by_ref(s, buf),
            DatabaseValue::String(s) => Encode::<Postgres>::encode_by_ref(s, buf),
            DatabaseValue::Text(s) => Encode::<Postgres>::encode_by_ref(s, buf),
            DatabaseValue::Int(i) => Encode::<Postgres>::encode_by_ref(i, buf),
            DatabaseValue::Int64(i) => Encode::<Postgres>::encode_by_ref(i, buf),
            DatabaseValue::Float(f) => Encode::<Postgres>::encode_by_ref(f, buf),
            DatabaseValue::Boolean(b) => Encode::<Postgres>::encode_by_ref(b, buf),
            DatabaseValue::DateTime(dt) => Encode::<Postgres>::encode_by_ref(dt, buf),
        }
    }
}

/// Implements type information for PostgreSQL.
/// All variants are encoded as text type for maximum flexibility.
impl Type<Postgres> for DatabaseValue {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        // Most general type that can handle all our variants
        sqlx::postgres::PgTypeInfo::with_name("text")
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        // OIDs for text-based types in PostgreSQL
        let text_oids = [25, 1043, 1042, 19, 1042]; // text, varchar, char, name, bpchar
        ty.oid()
            .map(|oid| text_oids.contains(&oid.0))
            .unwrap_or(false)
    }
}

/// Collection of FromIterator implementations to allow convenient conversion
/// from iterators of various types into DatabaseValue.
/// These implementations enable collecting iterators directly into DatabaseValue.

impl<'a> FromIterator<&'a str> for DatabaseValue {
    /// Collects an iterator of string slices into a DatabaseValue::String
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        DatabaseValue::String(iter.into_iter().collect::<String>())
    }
}

impl FromIterator<String> for DatabaseValue {
    /// Collects an iterator of Strings into a DatabaseValue::String
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        DatabaseValue::String(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a String> for DatabaseValue {
    /// Collects an iterator of String references into a DatabaseValue::String
    fn from_iter<I: IntoIterator<Item = &'a String>>(iter: I) -> Self {
        DatabaseValue::String(iter.into_iter().cloned().collect())
    }
}

impl FromIterator<bool> for DatabaseValue {
    /// Collects an iterator of booleans into a DatabaseValue::Boolean
    /// Each boolean is converted to its string representation
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
        DatabaseValue::Boolean(iter.into_iter().map(|b| b.to_string()).collect())
    }
}

impl FromIterator<OffsetDateTime> for DatabaseValue {
    /// Collects an iterator of OffsetDateTime into a DatabaseValue::DateTime
    /// Each datetime is formatted according to ISO 8601 standard
    fn from_iter<I: IntoIterator<Item = OffsetDateTime>>(iter: I) -> Self {
        DatabaseValue::DateTime(
            iter.into_iter()
                .map(|dt| dt.format(&Iso8601::DEFAULT).unwrap())
                .collect(),
        )
    }
}

impl FromIterator<i64> for DatabaseValue {
    /// Collects an iterator of 64-bit integers into a DatabaseValue::Int64
    /// Each integer is converted to its string representation
    fn from_iter<I: IntoIterator<Item = i64>>(iter: I) -> Self {
        DatabaseValue::Int64(iter.into_iter().map(|i| i.to_string()).collect())
    }
}

impl FromIterator<f64> for DatabaseValue {
    /// Collects an iterator of floating-point numbers into a DatabaseValue::Float
    /// Each number is converted to its string representation
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        DatabaseValue::Float(iter.into_iter().map(|f| f.to_string()).collect())
    }
}
