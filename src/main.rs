#[macro_use]
extern crate rocket;

// External dependencies
use rocket_cors::CorsOptions;
use sqlx::postgres::PgPoolOptions;
use std::env;

// Internal modules
mod api; // API routes and handlers
mod database; // Database connection and operations
mod models; // Data models and structures
mod utils; // Utility functions and helpers

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL");
    let pool = PgPoolOptions::new()
        .connect(&*database_url.unwrap())
        .await?;
    let cors = CorsOptions::default().to_cors().unwrap();

    rocket::build()
        .mount(
            "/api/auth",
            routes![
                api::authentications::login,
                api::authentications::logout,
                api::authentications::register,
                api::authentications::reset_password,
                api::authentications::check_username,
            ],
        )
        .manage(pool)
        .attach(cors)
        .launch()
        .await?;
    Ok(())
}
