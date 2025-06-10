use sqlx::postgres::PgPool;

pub async fn get_connection() -> PgPool {
    let mut database_url = std::env::var("DATABASE_URL").unwrap();
    if !database_url.contains("sslmode=") {
        database_url.push_str("?sslmode=prefer");
    }
    PgPool::connect(&database_url).await.unwrap()
}
