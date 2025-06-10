//! Database Query Macros
//!
//! This module provides a collection of macros for performing common database query operations.
//! These macros generate and execute SQL queries dynamically based on resource types and parameters.
//! They handle both archived and non-archived resources, supporting soft-delete functionality.
//!
//! # Common Parameters
//! - `$resource:ty`: The type of the resource (e.g., User, Store)
//! - `$params:expr`: Vector of tuples containing field names and values: `vec![("field", value)]`
//!
//! # Resource Requirements
//! Resources must implement the `DatabaseResource` trait which provides `from_row` functionality
//! to convert database rows into the appropriate type.
//!
//! # Examples
//!
//! ```rust
//! // Find all active users with a specific role
//! let admins = find_all_unarchived_resources_where_fields!(
//!     User,
//!     vec![("role", "admin")]
//! ).await?;
//!
//! // Find a single user by email
//! let user = find_one_resource_where_fields!(
//!     User,
//!     vec![("email", "user@example.com")]
//! ).await?;
//! ```

#[macro_export]
macro_rules! find_all_resources_where_fields {
    ($resource:ty, $params:expr) => {{
        // Import required traits and types for database operations
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            // Convert resource type name to table name (e.g., "User" -> "users")
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,     // Plural form
                false, // Don't include count
            );
            // Get database connection pool
            let pool = get_connection().await;

            // Extract field names and values from parameters
            // Example: vec![("email", value)] -> ["email"]
            let fields = $params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            // Example: vec![("email", value)] -> [value]
            let values = $params
                .iter()
                .map(|field| field.1.clone())
                .collect::<Vec<DatabaseValue>>();

            // Build the SQL query string with parameterized values
            // Example: "SELECT * FROM users WHERE email = $1 AND role = $2"
            let mut query = format!("SELECT * FROM {} WHERE ", resource_name);
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            // Create the SQL query and bind parameter values
            let mut query = sqlx::query(&query);
            for value in values.iter() {
                query = query.bind(value);
            }

            // Execute query and convert results to resource type
            match query.fetch_all(&pool).await {
                Ok(rows) => rows
                    .into_iter()
                    .map(|row| <$resource as DatabaseResource>::from_row(&row))
                    .collect::<Result<Vec<$resource>, _>>(),
                Err(e) => Err(e),
            }
        }
    }};
}

/// Finds all non-archived resources matching the specified conditions.
/// Only returns resources where archived_at IS NULL.
///
/// # Arguments
/// * `$resource:ty` - The type of resource to query
/// * `$params:expr` - Vector of (field_name, value) tuples for WHERE conditions
///
/// # Returns
/// * `Result<Vec<Resource>, Error>` - Collection of matching non-archived resources
///
/// # Example
/// ```rust
/// let active_stores = find_all_unarchived_resources_where_fields!(
///     Store,
///     vec![("owner_id", user_id)]
/// ).await?;
/// ```
#[macro_export]
macro_rules! find_all_unarchived_resources_where_fields {
    ($resource:ty, $params:expr) => {{
        // Import required traits and types
        use crate::database::{connection::get_connection, traits::DatabaseResource};
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            // Convert type name to plural table name
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            // Extract query parameters
            let fields = $params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            // Note: Using references to values here instead of cloning
            let values = $params.iter().map(|field| &field.1).collect::<Vec<_>>();

            // Build query with archived_at IS NULL condition
            let mut query = format!(
                "SELECT * FROM {} WHERE archived_at IS NULL AND ",
                resource_name
            );
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            // Create and execute parameterized query
            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }
            match query.fetch_all(&pool).await {
                Ok(rows) => rows
                    .into_iter()
                    .map(|row| <$resource as DatabaseResource>::from_row(&row))
                    .collect::<Result<Vec<$resource>, _>>(),
                Err(e) => Err(e),
            }
        }
    }};
}

/// Finds all archived resources matching the specified conditions.
/// Only returns resources where archived_at IS NOT NULL.
///
/// # Arguments
/// * `$resource:ty` - The type of resource to query
/// * `$params:expr` - Vector of (field_name, value) tuples for WHERE conditions
///
/// # Returns
/// * `Result<Vec<Resource>, Error>` - Collection of matching archived resources
///
/// # Example
/// ```rust
/// let deleted_users = find_all_archived_resources_where_fields!(
///     User,
///     vec![("department", "sales")]
/// ).await?;
/// ```
#[macro_export]
macro_rules! find_all_archived_resources_where_fields {
    ($resource:ty, $params:expr) => {{
        use crate::database::{connection::get_connection, traits::DatabaseResource};
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let fields = $params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = $params.iter().map(|field| &field.1).collect::<Vec<_>>();
            let mut query = format!(
                "SELECT * FROM {} WHERE archived_at IS NOT NULL AND ",
                resource_name
            );
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }
            match query.fetch_all(&pool).await {
                Ok(rows) => rows
                    .into_iter()
                    .map(|row| <$resource as DatabaseResource>::from_row(&row))
                    .collect::<Result<Vec<$resource>, _>>(),
                Err(e) => Err(e),
            }
        }
    }};
}

/// Finds a single resource matching the specified conditions.
/// Returns the first match if multiple records exist.
///
/// # Arguments
/// * `$resource:ty` - The type of resource to query
/// * `$params:expr` - Vector of (field_name, value) tuples for WHERE conditions
///
/// # Returns
/// * `Result<Resource, Error>` - The matching resource or error if not found
///
/// # Example
/// ```rust
/// let user = find_one_resource_where_fields!(
///     User,
///     vec![("id", user_id)]
/// ).await?;
/// ```
#[macro_export]
macro_rules! find_one_resource_where_fields {
    ($resource:ty, $params:expr) => {{
        use crate::database::{connection::get_connection, traits::DatabaseResource};
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let fields = $params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = $params.iter().map(|field| &field.1).collect::<Vec<_>>();
            let mut query = format!("SELECT * FROM {} WHERE ", resource_name);
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }
            query.push_str(" LIMIT 1");

            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }
            match query.fetch_one(&pool).await {
                Ok(row) => <$resource as DatabaseResource>::from_row(&row),
                Err(e) => Err(e),
            }
        }
    }};
}

/// Finds a single non-archived resource matching the specified conditions.
/// Only searches resources where archived_at IS NULL.
///
/// # Arguments
/// * `$resource:ty` - The type of resource to query
/// * `$params:expr` - Vector of (field_name, value) tuples for WHERE conditions
///
/// # Returns
/// * `Result<Resource, Error>` - The matching non-archived resource or error if not found
///
/// # Example
/// ```rust
/// let active_user = find_one_unarchived_resource_where_fields!(
///     User,
///     vec![("email", email)]
/// ).await?;
/// ```
#[macro_export]
macro_rules! find_one_unarchived_resource_where_fields {
    ($resource:ty, $params:expr) => {{
        use crate::database::{connection::get_connection, traits::DatabaseResource};
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let fields = $params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = $params.iter().map(|field| &field.1).collect::<Vec<_>>();
            let mut query = format!(
                "SELECT * FROM {} WHERE archived_at IS NULL AND ",
                resource_name
            );
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }
            query.push_str(" LIMIT 1");

            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }
            match query.fetch_one(&pool).await {
                Ok(row) => <$resource as DatabaseResource>::from_row(&row),
                Err(e) => Err(e),
            }
        }
    }};
}

/// Finds a single archived resource matching the specified conditions.
/// Only searches resources where archived_at IS NOT NULL.
///
/// # Arguments
/// * `$resource:ty` - The type of resource to query
/// * `$params:expr` - Vector of (field_name, value) tuples for WHERE conditions
///
/// # Returns
/// * `Result<Resource, Error>` - The matching archived resource or error if not found
///
/// # Example
/// ```rust
/// let deleted_store = find_one_archived_resource_where_fields!(
///     Store,
///     vec![("id", store_id)]
/// ).await?;
/// ```
#[macro_export]
macro_rules! find_one_archived_resource_where_fields {
    ($resource:ty, $params:expr) => {{
        use crate::database::{connection::get_connection, traits::DatabaseResource};
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            // Generate table name from resource type
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            // Build query for archived records (archived_at IS NOT NULL)
            let mut query = format!(
                "SELECT * FROM {} WHERE archived_at IS NOT NULL AND ",
                resource_name
            );

            // Extract field names for WHERE clause
            let fields = $params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();

            // Build WHERE conditions with parameter placeholders
            for (i, field) in fields.iter().enumerate() {
                query.push_str(&format!("{} = ${}", field, i + 1));
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }
            // Limit to single result
            query.push_str(" LIMIT 1");

            // Create parameterized query and bind values
            let mut query = sqlx::query(&query);
            for (_, value) in $params.iter().enumerate() {
                query = query.bind(value.1);
            }

            // Execute query and convert result to resource type
            match query.fetch_one(&pool).await {
                Ok(row) => Ok(<$resource as DatabaseResource>::from_row(&row)?),
                Err(e) => Err(e),
            }
        }
    }};
}
