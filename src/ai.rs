use std::{net::Ipv4Addr, str::FromStr};

use crate::game_state::{Coordinate, Move, Space, Spaces, BOARD_SIZE};

use rand::prelude::*;

pub enum Ai {
    RandomMove,
    JroylanceCustom,
    ServerWithPort(Ipv4Addr),
}
impl Ai {
    pub fn from_arg(arg: &str) -> Self {
        match arg {
            "random" => Self::RandomMove,
            "jroylance" => Self::JroylanceCustom,
            other => Self::ServerWithPort(Ipv4Addr::from_str(other).unwrap()),
        }
    }

    pub fn make_move(&self, spaces: &Spaces, player: usize) -> Option<Move> {
        match self {
            Ai::RandomMove => {
                let mut my_spaces = vec![];
                for x in 0..BOARD_SIZE {
                    for y in 0..BOARD_SIZE {
                        if spaces[x][y].owner() == Some(player) && spaces[x][y].get_units() > 0 {
                            my_spaces.push(Coordinate { x, y });
                        }
                    }
                }
                my_spaces.into_iter().choose(&mut thread_rng()).map(|from| {
                    let to = from
                        .surrounding()
                        .into_iter()
                        .filter(|to| spaces[to.x][to.y] != Space::Mountain)
                        .choose(&mut thread_rng())
                        .expect("Should always be a path out of a space");

                    Move {
                        owner: player,
                        units: spaces[from.x][from.y].get_units(),
                        from,
                        to,
                    }
                })
            }
            Ai::JroylanceCustom => {
                let mut my_spaces = vec![];
                for x in 0..BOARD_SIZE {
                    for y in 0..BOARD_SIZE {
                        if spaces[x][y].owner() == Some(player) && spaces[x][y].get_units() > 0 {
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
                                spaces[to.x][to.y] != Space::Mountain
                                    && spaces[to.x][to.y].owner() != Some(player)
                                    && spaces[to.x][to.y].get_units() + 2
                                        < spaces[from.x][from.y].get_units()
                            })
                            .map(|to| (*from, to))
                    })
                    .choose(&mut thread_rng())
                {
                    Some(Move {
                        owner: player,
                        units: spaces[from.x][from.y].get_units(),
                        from,
                        to,
                    })
                } else if let Ok((from, to)) = my_spaces
                    .iter()
                    .flat_map(|from| {
                        let mut possible_tos = from
                            .surrounding()
                            .into_iter()
                            .filter(|to| spaces[to.x][to.y].owner() == Some(player))
                            .map(|to| (*from, to))
                            .collect::<Vec<_>>();

                        // Shuffle to prevent moving in a loop
                        possible_tos.shuffle(&mut thread_rng());

                        possible_tos.into_iter()
                    })
                    .collect::<Vec<_>>()
                    .choose_weighted(&mut thread_rng(), |(from, _)| {
                        spaces[from.x][from.y].get_units()
                    })
                {
                    Some(Move {
                        owner: player,
                        units: spaces[from.x][from.y].get_units(),
                        from: *from,
                        to: *to,
                    })
                } else {
                    None
                }
            }
            Ai::ServerWithPort(_) => unimplemented!(),
        }
    }
}
