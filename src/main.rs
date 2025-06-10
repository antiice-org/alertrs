#[macro_use]
extern crate rocket;

// External dependencies
use rocket_cors::CorsOptions;
use sqlx::postgres::PgPoolOptions;
use std::env;

// Internal modules
mod websocket;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL");
    let pool = PgPoolOptions::new()
        .connect(&*database_url.unwrap())
        .await?;
    let cors = CorsOptions::default().to_cors().unwrap();

    rocket::build()
        .mount("/ws", routes![websocket::ws_handler,])
        .manage(pool)
        .attach(cors)
        .launch()
        .await?;
    Ok(())
}
