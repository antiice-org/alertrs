/// Updates a database resource with the given parameters and returns the updated resource.
///
/// # Macro Arguments
///
/// * `$resource:ty` - The type of the resource to update (must implement `DatabaseResource` trait)
/// * `$id:expr` - The ID of the resource to update
/// * `$params:expr` - A vector of tuples containing field names and their new values
///
/// # Returns
///
/// Returns a `Result` containing either:
/// * `Ok(T)` - The updated resource of type T
/// * `Err(sqlx::Error)` - A database error if the update fails
///
/// # Features
///
/// * Automatically handles `updated_at` timestamp for resources implementing `is_updatable()`
/// * Automatically handles `expires_at` field for resources implementing `is_expirable()`
/// * Supports various database value types including:
///   - Strings
///   - Integers (32-bit and 64-bit)
///   - Floats
///   - Booleans
///   - DateTime values
///   - NULL values
///
/// # Example
///
/// ```rust
/// let params = vec![
///     ("name", DatabaseValue::String("New Name".to_string())),
///     ("active", DatabaseValue::Boolean(true))
/// ];
/// let updated_user = update_resource!(User, user_id, params).await?;
/// ```
///
/// # Implementation Details
///
/// 1. Constructs an UPDATE query for the specified resource table
/// 2. Handles automatic timestamp updates for updatable/expirable resources
/// 3. Properly formats and casts different value types in the SQL query
/// 4. Executes the update query with proper parameter binding
/// 5. Returns the updated resource by performing a follow-up select query
#[macro_export]
macro_rules! update_resource {
    ($resource:ty, $id:expr, $params:expr) => {{
        use crate::database::{
            connection::get_connection, traits::DatabaseResource, values::DatabaseValue,
        };
        use crate::find_one_resource_where_fields;
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;
        use time::{format_description::well_known::Iso8601, Duration, OffsetDateTime};

        async {
            // Generate current timestamp for updated_at field
            let updated_at = OffsetDateTime::now_utc().format(&Iso8601::DEFAULT).unwrap();
            // Calculate expiration date (30 days from now) for expires_at field
            let expires_at = (OffsetDateTime::now_utc() + Duration::days(30))
                .format(&Iso8601::DEFAULT)
                .unwrap();

            // Convert resource type name to plural snake_case for table name
            // e.g., "UserProfile" becomes "user_profiles"
            let resource_name = pluralize(
                camel_to_snake_case(stringify!($resource).to_string()).as_str(),
                2,
                false,
            );
            let pool = get_connection().await;

            // Initialize parameters vector for SQL query
            let mut params: Vec<(&str, DatabaseValue)> = Vec::new();

            // Copy input parameters to our working vector
            let input_params: Vec<(&str, DatabaseValue)> = $params;
            if !input_params.is_empty() {
                for (field, value) in input_params {
                    params.push((field, value.clone()));
                }
            }

            // Add or update the updated_at timestamp if resource is updatable
            if <$resource as DatabaseResource>::is_updatable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("updated_at"))
                {
                    params[idx] = ("updated_at", DatabaseValue::DateTime(updated_at));
                } else {
                    params.push(("updated_at", DatabaseValue::DateTime(updated_at)));
                }
            }

            // Add or update the expires_at timestamp if resource is expirable
            if <$resource as DatabaseResource>::is_expirable() {
                if let Some(idx) = params
                    .iter()
                    .position(|(field, _)| field.contains("expires_at"))
                {
                    params[idx] = ("expires_at", DatabaseValue::DateTime(expires_at));
                } else {
                    params.push(("expires_at", DatabaseValue::DateTime(expires_at)));
                }
            }

            // Separate field names and values for query construction
            let fields = params
                .iter()
                .map(|(field, _)| field.to_string())
                .collect::<Vec<String>>();
            let values: Vec<&DatabaseValue> = params.iter().map(|(_, value)| value).collect();

            // Begin constructing the UPDATE query
            let mut query = format!("UPDATE {} SET ", resource_name);

            // Build the SET clause with proper type casting for each field
            for (i, field) in fields.iter().enumerate() {
                let value = values[i];
                match value {
                    // Handle NULL values
                    DatabaseValue::None => {
                        query.push_str(&format!("{} = NULL", field));
                    }
                    // Handle string types (no casting needed)
                    DatabaseValue::Str(_) | DatabaseValue::String(_) | DatabaseValue::Text(_) => {
                        query.push_str(&format!("{} = ${}", field, i + 1));
                    }
                    // Cast timestamp strings to TIMESTAMP type
                    DatabaseValue::DateTime(_) => {
                        query.push_str(&format!("{} = CAST(${} AS TIMESTAMP)", field, i + 1));
                    }
                    // Cast integers to appropriate size
                    DatabaseValue::Int(_) => {
                        query.push_str(&format!("{} = CAST(${} AS INTEGER)", field, i + 1));
                    }
                    DatabaseValue::Int64(_) => {
                        query.push_str(&format!("{} = CAST(${} AS BIGINT)", field, i + 1));
                    }
                    // Cast floating point numbers
                    DatabaseValue::Float(_) => {
                        query.push_str(&format!("{} = CAST(${} AS FLOAT)", field, i + 1));
                    }
                    // Cast boolean values
                    DatabaseValue::Boolean(_) => {
                        query.push_str(&format!("{} = CAST(${} AS BOOLEAN)", field, i + 1));
                    }
                }
                // Add comma separator between fields
                if i < fields.len() - 1 {
                    query.push_str(", ");
                }
            }

            // Add WHERE clause and RETURNING clause
            query.push_str(&format!(" WHERE id = ${}", fields.len() + 1));
            query.push_str(&format!(" RETURNING *"));

            // Prepare the query with parameter bindings
            let mut query = sqlx::query(&query);
            for (_, value) in values.iter().enumerate() {
                match value {
                    DatabaseValue::None => query = query.bind(Option::<String>::None),
                    _ => query = query.bind(value),
                }
            }
            // Bind the ID parameter
            query = query.bind(&$id);

            // Execute the UPDATE query
            match query.execute(&pool).await {
                Ok(_) => (),
                Err(e) => return Err(e),
            };

            // Fetch and return the updated resource
            let params = vec![("id", &$id)];
            match find_one_resource_where_fields!($resource, params).await {
                Ok(resource) => Ok(resource),
                Err(e) => Err(e),
            }
        }
    }};
}
