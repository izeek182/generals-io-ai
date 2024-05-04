use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use game_state::GameState;
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    time::sleep,
};
use tower_http::services::ServeDir;

mod game_state;

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(web_socket_sender): State<Sender<GameState>>,
) -> impl IntoResponse {
    println!("New user connected.");

    let reciever = web_socket_sender.subscribe();

    ws.on_upgrade(move |socket| handle_socket(socket, reciever))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, mut reciever: Receiver<GameState>) {
    while let Ok(message) = reciever.recv().await {
        if socket
            .send(Message::Text(serde_json::to_string(&message).unwrap()))
            .await
            .is_ok()
        {
            println!("Sent message to websocket");
        } else {
            println!("Unable to send ws message, closing socket");
            return;
        }
    }

    panic!("Shouldn't ever reach the end of a websocket connection");
}

#[tokio::main]
async fn main() {
    let mut game_state = GameState::new(2);

    let (game_state_sender, _) = broadcast::channel::<GameState>(16);

    let websocket_sender = game_state_sender.clone();

    tokio::spawn(async move {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8082));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        axum::serve(
            listener,
            Router::new()
                .route("/spectate", get(ws_handler))
                .fallback_service(ServeDir::new("data"))
                .with_state(websocket_sender),
        )
        .await
        .unwrap();
    });

    loop {
        // TODO: Get moves here
        // game_state.handle_moves()
        game_state.populate_spaces();

        // Ignore errors because there might be no subcribers
        let _ = game_state_sender.send(game_state.clone());

        sleep(Duration::from_secs(1)).await;
    }
}
