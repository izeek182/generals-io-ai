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
    if let Some((from, to)) = my_spaces
        .iter()
        .flat_map(|from| {
            from.surrounding()
                .into_iter()
                .filter(|to| {
                    body.spaces[to.x][to.y] != Space::Mountain
                        && body.spaces[to.x][to.y].owner() != Some(body.player)
                        && body.spaces[to.x][to.y].get_units() + 2
                            < body.spaces[from.x][from.y].get_units()
                })
                .map(|to| (*from, to))
        })
        .choose(&mut thread_rng())
    {
        Json(Some(TurnResponse { from, to }))
    } else if let Ok((from, to)) = my_spaces
        .iter()
        .flat_map(|from| {
            let mut possible_tos = from
                .surrounding()
                .into_iter()
                .filter(|to| body.spaces[to.x][to.y].owner() == Some(body.player))
                .map(|to| (*from, to))
                .collect::<Vec<_>>();

            // Shuffle to prevent moving in a loop
            possible_tos.shuffle(&mut thread_rng());

            possible_tos.into_iter()
        })
        .collect::<Vec<_>>()
        .choose_weighted(&mut thread_rng(), |(from, _)| {
            body.spaces[from.x][from.y].get_units()
        })
    {
        Json(Some(TurnResponse {
            from: *from,
            to: *to,
        }))
    } else {
        Json(None)
    }
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
