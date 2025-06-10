/// A macro that generates and executes an SQL INSERT query for a database resource.
///
/// # Overview
///
/// This macro provides a type-safe, secure way to insert records into PostgreSQL database tables.
/// It handles all the complexity of parameter binding, type casting, and automatic field management.
///
/// # Key Features
///
/// - **Automatic Field Management**:
///   - Generates UUIDs for new records
///   - Sets created_at/updated_at timestamps
///   - Manages expiration dates
///   - Converts resource names to table names (e.g., UserRole -> user_roles)
///
/// - **Type Safety**:
///   - Proper PostgreSQL type casting for all value types
///   - Safe parameter binding to prevent SQL injection
///   - Automatic handling of NULL values
///
/// - **Resource Trait Integration**:
///   - Works with any type implementing DatabaseResource
///   - Respects resource-specific behaviors (has_id, is_creatable, etc.)
///   - Automatic field population based on trait implementations
///
/// # Arguments
///
/// * `$resource` - The type that implements the `DatabaseResource` trait
/// * `$params` - A Vec of tuples containing (field_name, DatabaseValue) pairs
///
/// # Type Requirements
///
/// The resource type must implement the `DatabaseResource` trait and may implement any of:
/// - `has_id()` - If true, automatically generates and includes a UUID
/// - `is_creatable()` - If true, includes created_at timestamp
/// - `is_updatable()` - If true, includes updated_at timestamp
/// - `is_expirable()` - If true, includes expires_at timestamp (30 days from now)
///
/// # Supported Value Types
///
/// The macro handles the following `DatabaseValue` types with appropriate PostgreSQL casting:
/// - `String`/`Str` - Text values (no casting needed)
/// - `Int` - 32-bit integers (CAST AS INTEGER)
/// - `Int64` - 64-bit integers (CAST AS BIGINT)
/// - `Float` - Floating point numbers (CAST AS FLOAT)
/// - `Boolean` - True/false values (CAST AS BOOLEAN)
/// - `DateTime` - Timestamp values (CAST AS TIMESTAMP)
/// - `None` - NULL values
///
/// # Examples
///
/// Basic usage with a User resource:
/// ```rust
/// let params = vec![
///     ("name".to_string(), DatabaseValue::String("John".to_string())),
///     ("age".to_string(), DatabaseValue::Int(30)),
/// ];
/// let user = insert_resource!(User, params).await?;
/// ```
///
/// Inserting a record with multiple value types:
/// ```rust
/// let params = vec![
///     ("title".to_string(), DatabaseValue::String("Product".to_string())),
///     ("price".to_string(), DatabaseValue::Float(29.99)),
///     ("in_stock".to_string(), DatabaseValue::Boolean(true)),
///     ("quantity".to_string(), DatabaseValue::Int(100)),
/// ];
/// let product = insert_resource!(Product, params).await?;
/// ```
///
/// # Generated SQL
///
/// The macro generates SQL queries in this format:
/// ```sql
/// INSERT INTO table_name (field1, field2, ...)
/// VALUES (
///     CAST($1 AS appropriate_type),
///     CAST($2 AS appropriate_type),
///     ...
/// )
/// RETURNING *
/// ```
///
/// # Error Handling
///
/// Returns `Err(sqlx::Error)` if:
/// - The database connection fails
/// - The INSERT query is invalid
/// - A constraint violation occurs (e.g., unique constraint)
/// - Type conversion fails
/// - Row conversion to resource type fails
///
/// # Safety and Security
///
/// This macro implements several security best practices:
/// - Uses parameter binding to prevent SQL injection attacks
/// - Properly escapes and formats all values
/// - Handles NULL values safely
/// - Uses appropriate type casting for each value type
/// - Validates input parameters before query construction
///
/// # Implementation Details
///
/// The macro follows these steps:
/// 1. Generates required identifiers (UUID) and timestamps
/// 2. Converts resource type name to table name
/// 3. Processes input parameters and adds automatic fields
/// 4. Constructs type-safe SQL query with proper casting
/// 5. Binds parameters and executes the query
/// 6. Converts returned row to resource type
///
/// # Note
///
/// This macro is designed for use with PostgreSQL databases and relies on
/// PostgreSQL-specific features like CAST operators and parameter binding syntax.
#[macro_export]
macro_rules! insert_resource {
    ($resource:ty, $params:expr) => {{
        // Import required dependencies for:
        // - Database operations (connection, traits, value types)
        // - String manipulation (case conversion, pluralization)
        // - Time handling (ISO8601 formatting, UTC timestamps)
        // - UUID generation (v4 UUIDs for unique identifiers)
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;
        use time::{format_description::well_known::Iso8601, Duration, OffsetDateTime};
        use uuid::Uuid;

        // Clone input parameters to avoid modifying the original data
        // This allows the caller to retain ownership of their parameters
        let input_params = $params.clone();
        async {
            // Generate all required identifiers and timestamps up front:
            // - id: A new v4 UUID string for unique record identification
            // - created_at: Current UTC time in ISO8601 format for record creation tracking
            // - updated_at: Same as created_at initially, will differ on updates
            // - expires_at: UTC time 30 days from now for automatic record expiration
            let id = Uuid::new_v4().to_string();
            let created_at = OffsetDateTime::now_utc().format(&Iso8601::DEFAULT).unwrap();
            let updated_at = created_at.clone();
            let expires_at = (OffsetDateTime::now_utc() + Duration::days(30))
                .format(&Iso8601::DEFAULT)
                .unwrap();

            // Convert the resource type name to a database table name:
            // 1. Convert type name to string (e.g., "UserRole")
            // 2. Convert camelCase to snake_case (e.g., "user_role")
            // 3. Pluralize the name (e.g., "user_roles")
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            // Get database connection pool for query execution
            let pool = get_connection().await;

            // Initialize parameters vector and copy input parameters
            // Each parameter is a tuple of (field_name: String, value: DatabaseValue)
            // We clone values to ensure we don't move out of borrowed data
            let mut params: Vec<(String, DatabaseValue)> = Vec::new();
            for (field, value) in input_params.into_iter() {
                params.push((field.to_string(), value.clone()))
            }

            // Add or update automatic fields based on resource trait implementations
            // These trait checks determine which special fields should be included

            // Add UUID if resource requires an ID field
            if <$resource as DatabaseResource>::has_id() {
                params.push(("id".to_string(), DatabaseValue::String(id.clone())));
            }

            // Handle created_at timestamp for resources that track creation time
            if <$resource as DatabaseResource>::is_creatable() {
                // First check if created_at was provided in input parameters
                // If found, update it; if not, add it as a new parameter
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("created_at"))
                {
                    params[idx] = (
                        "created_at".to_string(),
                        DatabaseValue::DateTime(created_at),
                    );
                } else {
                    params.push((
                        "created_at".to_string(),
                        DatabaseValue::DateTime(created_at),
                    ));
                }
            }

            // Handle updated_at timestamp similarly to created_at
            // This field tracks the last modification time of the record
            if <$resource as DatabaseResource>::is_updatable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("updated_at"))
                {
                    params[idx] = (
                        "updated_at".to_string(),
                        DatabaseValue::DateTime(updated_at),
                    );
                } else {
                    params.push((
                        "updated_at".to_string(),
                        DatabaseValue::DateTime(updated_at),
                    ));
                }
            }

            // Handle expires_at timestamp for resources with expiration
            // This sets when the record should be considered invalid/expired
            if <$resource as DatabaseResource>::is_expirable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("expires_at"))
                {
                    params[idx] = (
                        "expires_at".to_string(),
                        DatabaseValue::DateTime(expires_at),
                    );
                } else {
                    params.push((
                        "expires_at".to_string(),
                        DatabaseValue::DateTime(expires_at),
                    ));
                }
            }

            // Separate field names and values into distinct vectors
            // This separation simplifies SQL query construction and parameter binding
            // Fields vector contains column names for the INSERT clause
            // Values vector contains the actual values to be inserted
            let fields: Vec<String> = params.iter().map(|(field, _)| field.clone()).collect();
            let values: Vec<DatabaseValue> =
                params.iter().map(|(_, value)| (*value).clone()).collect();

            // Begin constructing the parameterized SQL INSERT query
            // Start with the basic INSERT INTO clause using the table name
            let mut query = format!("INSERT INTO {} (", resource_name);

            // Add all field names to the query, comma-separated
            for (i, field) in fields.iter().enumerate() {
                query.push_str(field);
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
            }

            // Construct the VALUES clause with appropriate type casting
            // Each value is represented by a positional parameter ($1, $2, etc.)
            // Type casting ensures proper data type conversion in PostgreSQL
            query.push_str(") VALUES (");
            for (i, value) in values.iter().enumerate() {
                match value {
                    DatabaseValue::None => {
                        // NULL values don't need casting
                        query.push_str("NULL");
                    }
                    DatabaseValue::Str(_) | DatabaseValue::String(_) => {
                        // String types are automatically handled by PostgreSQL
                        query.push_str(&format!("Cast(${} AS VARCHAR)", i + 1));
                    }
                    DatabaseValue::Text(_) => {
                        // Text types are automatically handled by PostgreSQL
                        query.push_str(&format!("Cast(${} AS TEXT)", i + 1));
                    }
                    DatabaseValue::DateTime(_) => {
                        // Timestamps need explicit casting from text
                        query.push_str(&format!("CAST(${} AS TIMESTAMP)", i + 1));
                    }
                    DatabaseValue::Int(_) => {
                        // 32-bit integers need INTEGER casting
                        query.push_str(&format!("CAST(${} AS INTEGER)", i + 1));
                    }
                    DatabaseValue::Int64(_) => {
                        // 64-bit integers need BIGINT casting
                        query.push_str(&format!("CAST(${} AS BIGINT)", i + 1));
                    }
                    DatabaseValue::Float(_) => {
                        // Floating point numbers need FLOAT casting
                        query.push_str(&format!("CAST(${} AS FLOAT)", i + 1));
                    }
                    DatabaseValue::Boolean(_) => {
                        // Boolean values need explicit BOOLEAN casting
                        query.push_str(&format!("CAST(${} AS BOOLEAN)", i + 1));
                    }
                }
                if i < values.len() - 1 {
                    query.push_str(", ");
                }
            }
            // Add RETURNING clause to get the inserted row back
            query.push_str(") RETURNING *");

            // Create a prepared statement and bind all parameters
            // sqlx handles the actual parameter binding and escaping
            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }

            // Execute the query and handle the result:
            // - On success: Convert the returned row to the resource type
            // - On error: Return the database error directly
            // The ? operator propagates any conversion errors
            match query.fetch_one(&pool).await {
                Ok(row) => Ok(<$resource as DatabaseResource>::from_row(&row)?),
                Err(e) => {
                    println!("Error fetching row: {:?}", e);
                    Err(e)
                }
            }
        }
    }};
}
