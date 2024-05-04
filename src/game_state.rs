use std::{
    borrow::BorrowMut,
    collections::VecDeque,
    fmt::{Display, Write},
};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

const BOARD_SIZE: usize = 20;
const NUM_TOWNS: usize = 10;
const NUM_MOUNTAINS: usize = 100;
const CAPITAL_STARTING_UNITS: usize = 5;
const NEUTRAL_TOWN_STARTING_UNITS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Coordinate {
    x: usize,
    y: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Move {
    from: Coordinate,
    to: Coordinate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum Space {
    PlayerCapital { owner: usize, units: usize },
    PlayerTown { owner: usize, units: usize },
    NeutralTown { units: usize },
    PlayerEmptySpace { owner: usize, units: usize },
    EmptySpace,
    Mountain,
}
impl Space {
    pub fn expect_units(&self) -> usize {
        match self {
            Space::PlayerCapital { owner: _, units } => *units,
            Space::PlayerTown { owner: _, units } => *units,
            Space::NeutralTown { units } => *units,
            Space::PlayerEmptySpace { owner: _, units } => *units,
            _ => panic!("Tried to get unit count from space without units"),
        }
    }
}

pub type Spaces = [[Space; BOARD_SIZE]; BOARD_SIZE];

#[derive(Debug, Serialize, Clone)]
pub struct GameState {
    pub spaces: Spaces,
    pub turn: usize,
}
impl GameState {
    pub fn new(num_players: usize) -> Self {
        let mut spaces = [[Space::EmptySpace; BOARD_SIZE]; BOARD_SIZE];

        fn random_unoccupied_space(spaces: &Spaces) -> Coordinate {
            loop {
                let x = thread_rng().gen_range(0..BOARD_SIZE);
                let y = thread_rng().gen_range(0..BOARD_SIZE);
                if spaces[x][y] == Space::EmptySpace {
                    return Coordinate { x, y };
                }
            }
        }
        fn still_connected(spaces: &Spaces) -> bool {
            let mut visited = [[false; BOARD_SIZE]; BOARD_SIZE];

            let mut visit_queue = VecDeque::new();
            visit_queue.push_back(random_unoccupied_space(spaces));

            while let Some(space) = visit_queue.pop_back() {
                let mut potential_next_spaces = Vec::new();
                if space.x > 0 {
                    potential_next_spaces.push(Coordinate {
                        x: space.x - 1,
                        y: space.y,
                    });
                }
                if space.y > 0 {
                    potential_next_spaces.push(Coordinate {
                        x: space.x,
                        y: space.y - 1,
                    });
                }
                if space.x < BOARD_SIZE - 1 {
                    potential_next_spaces.push(Coordinate {
                        x: space.x + 1,
                        y: space.y,
                    });
                }
                if space.y < BOARD_SIZE - 1 {
                    potential_next_spaces.push(Coordinate {
                        x: space.x,
                        y: space.y + 1,
                    });
                }

                for next_space in potential_next_spaces {
                    if spaces[next_space.x][next_space.y] != Space::Mountain
                        && !visited[next_space.x][next_space.y]
                    {
                        visited[next_space.x][next_space.y] = true;
                        visit_queue.push_back(next_space);
                    }
                }
            }

            for x in 0..BOARD_SIZE {
                for y in 0..BOARD_SIZE {
                    if spaces[x][y] != Space::Mountain && !visited[x][y] {
                        return false;
                    }
                }
            }
            true
        }

        for player_index in 0..num_players {
            let capital_coord = random_unoccupied_space(&spaces);
            spaces[capital_coord.x][capital_coord.y] = Space::PlayerCapital {
                owner: player_index,
                units: CAPITAL_STARTING_UNITS,
            };
        }
        for _ in 0..NUM_TOWNS {
            let coord = random_unoccupied_space(&spaces);
            spaces[coord.x][coord.y] = Space::NeutralTown {
                units: NEUTRAL_TOWN_STARTING_UNITS,
            };
        }
        let mut num_mountains_remaining = NUM_MOUNTAINS;
        while num_mountains_remaining > 0 {
            let coord = random_unoccupied_space(&spaces);
            spaces[coord.x][coord.y] = Space::Mountain;
            if still_connected(&spaces) {
                num_mountains_remaining -= 1;
            } else {
                spaces[coord.x][coord.y] = Space::EmptySpace;
            }
        }

        GameState { spaces, turn: 0 }
    }

    pub fn handle_moves(&mut self, moves: Vec<Move>) {
        let moves_with_unit_counts = moves
            .into_iter()
            .map(|m| (m, self.spaces[m.from.x][m.from.y].expect_units()))
            .collect::<Vec<_>>();
        unimplemented!()
    }

    pub fn populate_spaces(&mut self) {
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                match self.spaces[x][y].borrow_mut() {
                    Space::PlayerCapital { owner: _, units } => *units += 1,
                    Space::PlayerTown { owner: _, units } => {
                        if self.turn % 2 == 0 {
                            *units += 1;
                        }
                    }
                    Space::PlayerEmptySpace { owner: _, units } => {
                        if self.turn % 10 == 0 {
                            *units += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                let char = match self.spaces[x][y] {
                    Space::PlayerCapital { .. } => 'P',
                    Space::PlayerTown { .. } => 'p',
                    Space::NeutralTown { .. } => 'n',
                    Space::PlayerEmptySpace { .. } => 'p',
                    Space::EmptySpace => ' ',
                    Space::Mountain => '^',
                };
                f.write_char(char)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
