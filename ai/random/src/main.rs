use std::net::SocketAddr;

use axum::{routing::post, Json, Router};
use model::{Coordinate, Space, TurnRequest, TurnResponse, BOARD_SIZE};
use rand::prelude::*;

async fn turn_handler(Json(body): Json<TurnRequest>) -> Json<Option<TurnResponse>> {
    let mut my_spaces = vec![];
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            if body.spaces[x][y].owner() == Some(body.player) && body.spaces[x][y].get_units() > 0 {
                my_spaces.push(Coordinate { x, y });
            }
        }
    }
    Json(my_spaces.into_iter().choose(&mut thread_rng()).map(|from| {
        let to = from
            .surrounding()
            .into_iter()
            .filter(|to| body.spaces[to.x][to.y] != Space::Mountain)
            .choose(&mut thread_rng())
            .expect("Should always be a path out of a space");

        TurnResponse { from, to }
    }))
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .expect("Should pass one argument, the port to run on")
        .parse()
        .expect("First argument should be a valid port");

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, Router::new().route("/", post(turn_handler)))
        .await
        .unwrap();
}
