use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use warp::Filter;

mod websockets;

#[tokio::main]
async fn main() {
    let tx = Arc::new(Mutex::new(broadcast::channel(100).0));
    let tx_ws = tx.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            println!("WebSocket connection established");
            let tx = tx_ws.clone();
            ws.on_upgrade(move |websocket| websockets::handle_connection(websocket, tx))
        });
    warp::serve(ws_route).run(([0, 0, 0, 0], 8000)).await;
}
