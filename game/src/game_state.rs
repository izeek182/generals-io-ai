use std::{
    borrow::BorrowMut,
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt::{Display, Write},
};

use model::{Coordinate, Space, Spaces};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

pub const BOARD_SIZE: usize = 20;
const NUM_TOWNS: usize = 10;
const NUM_MOUNTAINS: usize = 100;
const CAPITAL_STARTING_UNITS: usize = 5;
const NEUTRAL_TOWN_STARTING_UNITS: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub owner: String,
    pub units: usize,
    pub from: Coordinate,
    pub to: Coordinate,
}

#[derive(Debug, Serialize, Clone)]
pub struct GameState {
    pub game_id: String,
    pub spaces: Spaces,
    pub turn: usize,
}
impl GameState {
    pub fn new(game_id: String, player_ids: Vec<String>) -> Self {
        let mut spaces: Spaces = std::array::from_fn(|_| std::array::from_fn(|_| Space::Empty));

        fn random_unoccupied_space(spaces: &Spaces) -> Coordinate {
            loop {
                let x = thread_rng().gen_range(0..BOARD_SIZE);
                let y = thread_rng().gen_range(0..BOARD_SIZE);
                if spaces[x][y] == Space::Empty {
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

        for player_id in player_ids {
            let capital_coord = random_unoccupied_space(&spaces);
            spaces[capital_coord.x][capital_coord.y] = Space::PlayerCapital {
                owner: player_id,
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
                spaces[coord.x][coord.y] = Space::Empty;
            }
        }

        GameState {
            game_id,
            spaces,
            turn: 0,
        }
    }

    pub fn handle_moves(&mut self, mut moves: Vec<Move>) {
        // Subtract units from all "from" spaces
        for m in &moves {
            self.spaces[m.from.x][m.from.y].unsafe_set_units(0);
        }

        // Handle "meet in the middle" - delete moves that lose that encounter
        for i in 0..moves.len() {
            for j in 0..moves.len() {
                if i != j && moves[i].from == moves[j].to && moves[i].to == moves[j].from {
                    let min_units = moves[i].units.min(moves[j].units);
                    moves[i].units -= min_units;
                    moves[j].units -= min_units;
                }
            }
        }
        moves.retain(|m| m.units > 0);

        // Quick pass to handle moves from someone to their own territory
        for m in &mut moves {
            if self.spaces[m.from.x][m.from.y].owner() == self.spaces[m.to.x][m.to.y].owner() {
                self.spaces[m.to.x][m.to.y]
                    .unsafe_set_units(self.spaces[m.to.x][m.to.y].get_units() + m.units);
                m.units = 0;
            }
        }
        moves.retain(|m| m.units > 0);

        // Create mapping from destination to (owner, unit)
        let moves_with_unit_counts = {
            let mut new_moves = BTreeMap::new();

            for m in moves {
                new_moves
                    .entry(m.to)
                    .or_insert(vec![])
                    .push((m.owner, m.units));
            }

            new_moves
        };

        let mut takeovers: Vec<(String, String)> = Vec::new();
        for (dest, mut moves) in moves_with_unit_counts.into_iter() {
            // Progressively eliminate attacking armies by subtracting the total units of the weakest attacker, and removing armies with 0 units
            while moves.len() > 1 {
                let weakest_army_units = moves
                    .iter()
                    .map(|(_, units)| *units)
                    .min()
                    .expect("Just checked that length > 1");

                for (_, units) in moves.iter_mut() {
                    *units -= weakest_army_units;
                }

                moves.retain(|(_, units)| *units > 0);
            }
            // Only need to worry about the case where there's one person moving to the space
            // If 0, we don't do anything.
            if let Some((owner, source_units)) = moves.first() {
                let defending_units = self.spaces[dest.x][dest.y].get_units();

                if defending_units < *source_units {
                    let remaining_units = source_units - (defending_units);
                    // Attacker wins
                    self.spaces[dest.x][dest.y] = match &self.spaces[dest.x][dest.y] {
                        Space::PlayerCapital {
                            owner: looser,
                            units: _,
                        } => {
                            takeovers.push((owner.to_owned(), looser.to_owned()));
                            Space::PlayerTown {
                                owner: owner.clone(),
                                units: remaining_units,
                            }
                        }
                        Space::PlayerTown { .. } => Space::PlayerTown {
                            owner: owner.clone(),
                            units: remaining_units,
                        },
                        Space::NeutralTown { .. } => Space::PlayerTown {
                            owner: owner.clone(),
                            units: remaining_units,
                        },
                        Space::PlayerEmpty { .. } => Space::PlayerEmpty {
                            owner: owner.clone(),
                            units: remaining_units,
                        },
                        Space::Empty => Space::PlayerEmpty {
                            owner: owner.clone(),
                            units: remaining_units,
                        },
                        Space::Mountain => panic!("Tried to conquer a mountain"),
                    }
                } else {
                    // Defender wins
                    if self.spaces[dest.x][dest.y] != Space::Empty {
                        self.spaces[dest.x][dest.y]
                            .unsafe_set_units(defending_units.saturating_sub(*source_units))
                    }
                }
            }
        }

        for (winner, looser) in takeovers {
            self.takeover(&winner, &looser);
        }
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
                    Space::PlayerEmpty { owner: _, units } => {
                        if self.turn % 25 == 0 {
                            *units += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn remaining_players(&self) -> BTreeSet<String> {
        let mut remaining_players = BTreeSet::new();

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                self.spaces[x][y]
                    .owner()
                    .map(|owner| remaining_players.insert(owner.to_string()));
            }
        }
        remaining_players
    }

    fn takeover(&mut self, winner_id: &String, looser_id: &String) {
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let space = &mut self.spaces[x][y];
                if space.owner().is_some() && space.owner().unwrap() == looser_id {
                    space.unsafe_set_owner(winner_id.to_owned());
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Delta {
    coord: Coordinate,
    prev: Space,
    next: Space,
}

#[derive(Debug, Serialize, Clone)]
pub struct DeltafiedGameState {
    pub initial_game_state: GameState,
    pub deltas: Vec<Vec<Delta>>,
}

impl DeltafiedGameState {
    pub fn new(state: &GameState) -> DeltafiedGameState {
        return DeltafiedGameState {
            initial_game_state: state.clone(),
            deltas: Vec::new(),
        };
    }

    pub fn add_delta(&mut self, prev_state: &GameState, next_state: &GameState) {
        let mut new_delta: Vec<Delta> = Vec::new();
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let prev_space = &prev_state.spaces[x][y];
                let next_space = &next_state.spaces[x][y];
                if prev_space != next_space {
                    new_delta.push(Delta {
                        coord: Coordinate { x, y },
                        prev: prev_space.clone(),
                        next: next_space.clone(),
                    })
                }
            }
        }
        self.deltas.push(new_delta);
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
                    Space::PlayerEmpty { .. } => 'p',
                    Space::Empty => ' ',
                    Space::Mountain => '^',
                };
                f.write_char(char)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}
