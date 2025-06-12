use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use warp::ws::{Message, WebSocket};

pub async fn handle_connection(ws: WebSocket, tx: Arc<Mutex<broadcast::Sender<String>>>) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let mut rx = tx.lock().unwrap().subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    println!("Received message: {}", text);
                    tx.lock()
                        .unwrap()
                        .send(text.to_string())
                        .expect("Failed to broadcast message");
                }
            }
            Err(_) => break,
        }
    }
}
