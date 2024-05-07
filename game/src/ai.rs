use crate::game_state::Move;

use model::{Spaces, TurnRequest, TurnResponse};

pub struct Ai {
    port: u16,
}
impl Ai {
    pub fn from_arg(arg: &str) -> Self {
        Self {
            port: arg.parse().unwrap(),
        }
    }

    pub async fn make_move(&self, turn: usize, spaces: &Spaces, player: usize) -> Option<Move> {
        let request_body = TurnRequest {
            turn,
            player,
            spaces: *spaces,
        };
        let response: Option<TurnResponse> = reqwest::Client::new()
            .post(format!("http://localhost:{}", self.port))
            .json(&request_body)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        response.map(|r| Move {
            owner: player,
            units: spaces[r.from.x][r.from.y].get_units(),
            from: r.from,
            to: r.to,
        })
    }
}
