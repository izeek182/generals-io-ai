#![allow(clippy::needless_range_loop)]

use ai::Ai;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use game_state::{GameState, BOARD_SIZE};
use model::Space;
use std::process;
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    time::sleep,
};
use tower_http::services::ServeDir;

mod ai;
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
            .is_err()
        {
            println!("Unable to send ws message, closing socket");
            return;
        }
    }

    panic!("Shouldn't ever reach the end of a websocket connection");
}

#[tokio::main]
async fn main() {
    let players: Vec<Ai> = std::env::args()
        .skip(1)
        .map(|arg| Ai::from_arg(&arg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|err| {
            eprintln!("Error: {}", err);
            process::exit(1);
        });

    let mut game_state = GameState::new(players.len());

    let (game_state_sender, _) = broadcast::channel::<GameState>(16);

    let websocket_sender = game_state_sender.clone();

    tokio::spawn(async move {
        let port: u16 = std::env::var("FORCE_PORT")
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(0); // Default to port 0 if FORCE_PORT is not set or invalid

        let host = std::env::var("HOST_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
        let ip: IpAddr = host.parse().expect("Invalid IP address");

        let addr = SocketAddr::from((ip, port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        println!(
            "Starting server at http://{}",
            listener.local_addr().unwrap()
        );
        axum::serve(
            listener,
            Router::new()
                .route("/spectate", get(ws_handler))
                .fallback_service(ServeDir::new("game/data"))
                .with_state(websocket_sender),
        )
        .await
        .unwrap();
    });

    loop {
        let mut moves = vec![];

        for (i, ai) in players.iter().enumerate() {
            if let Some(m) = ai.make_move(game_state.turn, &game_state.spaces, i).await {
                moves.push((i, m));
            }
        }

        let moves = moves
            .into_iter()
            .filter(|(player, m)| {
                if [m.to.x, m.to.y, m.from.x, m.from.y]
                    .iter()
                    .any(|coord| *coord > BOARD_SIZE)
                {
                    println!("Player {player} tried to make a move that was out of bounds. {m:?}");
                    false
                } else if game_state.spaces[m.from.x][m.from.y].owner() != Some(*player) {
                    println!(
                        "Player {player} tried to make a move from a space they didn't own. {m:?}"
                    );
                    false
                } else if game_state.spaces[m.from.x][m.from.y] == Space::Mountain {
                    println!("Player {player} tried to make a move onto a mountain. {m:?}");
                    false
                } else {
                    true
                }
            })
            .map(|(_, m)| m)
            .collect();

        game_state.handle_moves(moves);

        game_state.populate_spaces();

        // TODO: Handle player elimination, game over

        game_state.turn += 1;

        // Ignore errors because there might be no subcribers
        let _ = game_state_sender.send(game_state.clone());

        sleep(Duration::from_millis(50)).await;
    }
}
