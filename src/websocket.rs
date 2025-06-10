use ws::{Stream, WebSocket};

/// WebSocket handler for real-time communication
///
/// This endpoint establishes a WebSocket connection and streams messages between the client and server.
/// It uses Rocket's streaming capabilities to handle WebSocket communication efficiently.
///
/// # Route
/// `GET /ws`
///
/// # Returns
/// A stream of WebSocket messages that can be:
/// - Text messages
/// - Binary messages
/// - Ping/Pong frames
/// - Close frames
///
/// # Example Client Usage
/// ```javascript
/// const ws = new WebSocket('ws://localhost:8000/ws');
///
/// ws.onmessage = (event) => {
///     console.log('Received:', event.data);
/// };
///
/// ws.onopen = () => {
///     ws.send('Hello Server!');
/// };
/// ```
///
/// # Error Handling
/// - Connection errors are propagated through the stream
/// - Invalid messages are handled gracefully
/// - Client disconnections are handled automatically
#[get("/")]
pub fn ws_handler(ws: WebSocket) -> Stream!['static] {
    println!("WebSocket connection established");
    Stream! { ws =>
        for await message in ws {
            println!("Received message: {:?}", message);
            yield message?;
        }
    }
}
