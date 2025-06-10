/// Macro for deleting or archiving database resources based on specified field conditions
///
/// # Description
/// This macro generates an async function that either deletes or "soft deletes" (archives)
/// database resources based on the provided field conditions. For archivable resources,
/// it sets the `archived_at` timestamp instead of performing a hard delete.
///
/// The macro automatically:
/// - Converts resource names from CamelCase to snake_case
/// - Pluralizes table names
/// - Handles parameter binding safely to prevent SQL injection
/// - Supports both hard deletes and soft deletes (archiving)
///
/// # Parameters
/// * `$resource` - The type implementing `DatabaseResource` trait
/// * `$params` - A vector of tuples `Vec<(String, DatabaseValue)>` where:
///   - First element is the field name
///   - Second element is the value to match against
///
/// # Returns
/// * `Result<(), anyhow::Error>` - Ok(()) on success, Error on failure
///
/// # Implementation Details
/// For archivable resources (where `is_archivable()` returns true):
/// ```sql
/// UPDATE table_name SET archived_at = CURRENT_TIMESTAMP WHERE field1 = $1 AND field2 = $2
/// ```
///
/// For non-archivable resources:
/// ```sql
/// DELETE FROM table_name WHERE field1 = $1 AND field2 = $2
/// ```
///
/// # Example
/// ```rust
/// use crate::database::values::DatabaseValue;
///
/// // Delete/archive user roles matching specific conditions
/// let conditions = vec![
///     ("user_id".to_string(), DatabaseValue::Uuid(user_id)),
///     ("store_id".to_string(), DatabaseValue::Uuid(store_id))
/// ];
/// delete_resource_where_fields!(UserRole, conditions).await?;
/// ```
///
/// # Note
/// The macro requires the following traits and types to be in scope:
/// - `DatabaseResource` trait implementation for the resource type
/// - `DatabaseValue` enum for type-safe value handling
/// - Database connection pool access via `get_connection()`
#[macro_export]
macro_rules! delete_resource_where_fields {
    ($resource:ty, $params:expr) => {{
        use crate::database::connection::get_connection;
        use crate::database::traits::DatabaseResource;
        use crate::database::values::DatabaseValue;
        use crate::utils::strings::camel_to_snake_case;
        use anyhow::anyhow;
        use pluralizer::pluralize;
        use time::OffsetDateTime;

        async {
            let archived_at = OffsetDateTime::now_utc();

            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            let params = $params.clone();

            let fields: Vec<String> = params.iter().map(|field| field.0.to_string()).collect();
            let values: Vec<DatabaseValue> = params.iter().map(|field| field.1.clone()).collect();

            let mut query: String;
            if <$resource as DatabaseResource>::is_archivable() {
                query = format!(
                    "UPDATE {} SET archived_at = CAST(${} AS TIMESTAMP) WHERE ",
                    resource_name,
                    fields.len() + 1
                );
            } else {
                query = format!("DELETE FROM {} WHERE ", resource_name);
            }

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
            if <$resource as DatabaseResource>::is_archivable() {
                query = query.bind(archived_at);
            }

            match query.execute(&pool).await {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!(e)),
            }
        }
    }};
}
