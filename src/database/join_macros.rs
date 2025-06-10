/// Performs a SQL JOIN query between two database tables and filters results based on provided parameters.
///
/// # Macro Arguments
/// * `$resource` - The primary resource type that implements DatabaseResource trait
/// * `$join_resource` - The resource type to join with
/// * `$params` - A vector of tuples containing (field_name, value) pairs for WHERE clause filtering
///
/// # Returns
/// * `Result<Vec<$resource>, sqlx::Error>` - Returns a vector of primary resource instances or an error
///
/// # Examples
/// ```rust
/// // Join Users with Stores and filter by store_id and active status
/// let params = vec![("store_id", "123"), ("active", "true")];
/// let users = join_all_resources_where_fields_on!(User, Store, params).await?;
///
/// // Join UserRoles with Roles and filter by role_name
/// let params = vec![("role_name", "admin")];
/// let user_roles = join_all_resources_where_fields_on!(UserRole, Role, params).await?;
/// ```
///
/// # Details
/// This macro generates a SQL query that:
/// 1. Converts resource names from CamelCase to snake_case (e.g., UserRole -> user_role)
/// 2. Pluralizes table names (e.g., user_role -> user_roles)
/// 3. Creates JOIN conditions using `{resource}_id` format (e.g., user_role_id)
/// 4. Adds WHERE clause conditions based on provided parameters
/// 5. Maps the results to the primary resource type using the DatabaseResource trait
///
/// # Generated SQL Example
/// For `join_all_resources_where_fields_on!(User, Store, vec![("active", "true")])`:
/// ```sql
/// SELECT * FROM users
/// JOIN stores ON store_id = user_id
/// WHERE active = $1
/// ```
///
/// # Notes
/// - The primary resource type must implement the DatabaseResource trait
/// - Table names are automatically pluralized and converted to snake_case
/// - Join conditions assume conventional ID naming (`{resource}_id`)
/// - WHERE clause parameters are automatically parameterized to prevent SQL injection
/// - The macro is asynchronous and must be awaited
///
/// # Panics
/// - Will panic if the DatabaseResource::from_row conversion fails
/// - May panic if the provided field names don't exist in the database
#[macro_export]
macro_rules! join_all_resources_where_fields_on {
    ($resource:ty, $join_resource:ty, $params:expr) => {{
        use crate::database::{connection::get_connection, traits::DatabaseResource};
        use crate::utils::strings::camel_to_snake_case;
        use pluralizer::pluralize;

        async {
            // Step 1: Process the primary resource name
            // Convert CamelCase type name (e.g., UserRole) to snake_case (user_role)
            let resource_name = camel_to_snake_case(stringify!($resource).to_string());
            // Convert singular to plural for table name (e.g., user_role -> user_roles)
            let resource_table_name = pluralize(&resource_name, 2, false);
            // Create the foreign key column name (e.g., user_role -> user_role_id)
            let resource_join_name = format!("{}_id", resource_name);

            // Step 2: Process the joined resource name using the same pattern
            let join_resource_name = camel_to_snake_case(stringify!($join_resource).to_string());
            let join_resource_table_name = pluralize(&join_resource_name, 2, false);
            let join_resource_join_name = format!("{}_id", join_resource_name);

            // Step 3: Get database connection from the connection pool
            let pool = get_connection().await;

            // Step 4: Process the WHERE clause parameters
            // Split the input params tuple vec into separate field names and values
            // Example: vec![("store_id", "123"), ("active", "true")]
            // Becomes: fields=["store_id", "active"], values=["123", "true"]
            let fields = $params
                .iter()
                .map(|field| field.0.to_string())
                .collect::<Vec<String>>();
            let values = $params
                .iter()
                .map(|field| field.1.to_string())
                .collect::<Vec<String>>();

            // Step 5: Construct the base JOIN query
            // Creates: "SELECT * FROM {table1} JOIN {table2} ON {fk} = {pk}"
            let mut query = format!(
                "SELECT * FROM {} JOIN {} ON {} = {}",
                resource_table_name,      // First table (e.g., user_roles)
                join_resource_table_name, // Second table (e.g., roles)
                join_resource_join_name,  // Foreign key (e.g., role_id)
                resource_join_name        // Primary key (e.g., user_role_id)
            );

            // Step 6: Add WHERE clause conditions
            // Adds parameterized conditions: "WHERE field1 = $1 AND field2 = $2"
            query.push_str(" WHERE ");
            for (i, field) in fields.iter().enumerate() {
                // Add each condition with a numbered parameter placeholder
                query.push_str(&format!("{} = ${}", field, i + 1));
                // Add AND between conditions, but not after the last one
                if i < fields.len() - 1 {
                    query.push_str(" AND ");
                }
            }

            // Step 7: Create and prepare the SQL query
            let mut query = sqlx::query(&query);
            // Bind all parameter values in order
            for (_, value) in values.iter().enumerate() {
                query = query.bind(value);
            }

            // Step 8: Execute query and map results
            match query.fetch_all(&pool).await {
                Ok(rows) => {
                    // Convert each database row into the requested resource type
                    // using the DatabaseResource trait implementation
                    Ok(rows
                        .iter()
                        .map(|row| <$resource as DatabaseResource>::from_row(row).unwrap())
                        .collect::<Vec<$resource>>())
                }
                Err(e) => Err(e),
            }
        }
    }};
}
