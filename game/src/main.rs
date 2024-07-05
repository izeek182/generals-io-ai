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
use futures::future::join_all;
use game_state::{DeltafiedGameState, GameState, BOARD_SIZE};
use model::Space;
use std::collections::BTreeMap;
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::{
    sync::watch::{self, Receiver, Sender},
    time::sleep,
};
use tower_http::services::ServeDir;
use uuid::Uuid;

mod ai;
mod game_state;

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(web_socket_sender): State<Sender<DeltafiedGameState>>,
) -> impl IntoResponse {
    println!("New user connected.");

    let receiver = web_socket_sender.subscribe();

    ws.on_upgrade(move |socket| handle_socket(socket, receiver))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, mut receiver: Receiver<DeltafiedGameState>) {
    let deltafied_state = receiver.borrow_and_update().clone();
    if socket
        .send(Message::Text(
            serde_json::to_string(&deltafied_state).unwrap(),
        ))
        .await
        .is_err()
    {
        println!("Unable to send initial state, closing socket");
        return;
    }

    let mut deltas_sent = deltafied_state.deltas.len();

    while receiver.changed().await.is_ok() {
        let full_deltas = receiver.borrow_and_update().deltas.clone();

        let deltas = full_deltas[deltas_sent..].to_vec();
        let json_deltas = format!("{{\"deltas\":{}}}", serde_json::to_string(&deltas).unwrap());
        if socket.send(Message::Text(json_deltas)).await.is_err() {
            println!("Unable to send ws message, closing socket");
            return;
        }
        deltas_sent = full_deltas.len();
    }

    panic!("Shouldn't ever reach the end of a websocket connection");
}

#[tokio::main]
async fn main() {
    let game_id = Uuid::new_v4().to_string();
    let players: BTreeMap<String, Ai> = std::env::args()
        .skip(1)
        .map(|arg| (Uuid::new_v4().to_string(), Ai::from_arg(&arg).unwrap()))
        .collect();

    let mut game_state = GameState::new(game_id, players.keys().cloned().collect());
    let mut deltafied_game_state = DeltafiedGameState::new(&game_state);

    let (game_state_sender, _) = watch::channel::<DeltafiedGameState>(deltafied_game_state.clone());

    let websocket_sender = game_state_sender.clone();

    let spectate_listener = tokio::spawn(async move {
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

    let reqwest_client = reqwest::Client::new();

    loop {
        let remaining_players_at_start = game_state.remaining_players();

        let moves = join_all(
            players
                .iter()
                .filter(|(player_id, _)| remaining_players_at_start.contains(*player_id))
                .map(|(player_id, ai)| {
                    ai.make_move(
                        &reqwest_client,
                        game_state.turn,
                        &game_state.spaces,
                        player_id.to_string(),
                        game_state.game_id.clone(),
                    )
                }),
        )
        .await
        .into_iter()
        .flatten()
        .filter(|m| {
            if [m.to.x, m.to.y, m.from.x, m.from.y]
                .iter()
                .any(|coord| *coord > BOARD_SIZE)
            {
                println!("Player tried to make a move that was out of bounds. {m:?}");
                false
            } else if game_state.spaces[m.from.x][m.from.y].owner() != Some(&m.owner) {
                println!("Player tried to make a move from a space they didn't own. {m:?}");
                false
            } else if game_state.spaces[m.to.x][m.to.y] == Space::Mountain {
                println!("Player tried to make a move onto a mountain. {m:?}");
                false
            } else {
                true
            }
        })
        .collect();

        let prev_game_state = game_state.clone();

        game_state.handle_moves(moves);

        game_state.populate_spaces();

        game_state.turn += 1;

        deltafied_game_state.add_delta(&prev_game_state, &game_state);

        game_state_sender.send_replace(deltafied_game_state.clone());

        if game_state.remaining_players().len() <= 1 {
            println!("Game over");
            break;
        }

        sleep(Duration::from_millis(50)).await;
    }

    spectate_listener.await.unwrap();
}
