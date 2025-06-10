#[macro_use]
extern crate rocket;

use rocket_cors::CorsOptions;

mod websocket;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let cors = CorsOptions::default().to_cors().unwrap();

    rocket::build()
        .mount("/ws", routes![websocket::ws_handler,])
        .attach(cors)
        .launch()
        .await?;
    Ok(())
}
