use serde::{Deserialize, Serialize};

pub const BOARD_SIZE: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}
impl Coordinate {
    /**
     * Excludes spaces outside of bounds
     */
    pub fn surrounding(&self) -> Vec<Self> {
        let mut surrounding = vec![];
        if self.x > 0 {
            surrounding.push(Coordinate {
                x: self.x - 1,
                y: self.y,
            })
        }
        if self.y > 0 {
            surrounding.push(Coordinate {
                x: self.x,
                y: self.y - 1,
            })
        }
        if self.x < BOARD_SIZE - 1 {
            surrounding.push(Coordinate {
                x: self.x + 1,
                y: self.y,
            })
        }
        if self.y < BOARD_SIZE - 1 {
            surrounding.push(Coordinate {
                x: self.x,
                y: self.y + 1,
            })
        }
        surrounding
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Space {
    PlayerCapital { owner: usize, units: usize },
    PlayerTown { owner: usize, units: usize },
    NeutralTown { units: usize },
    PlayerEmpty { owner: usize, units: usize },
    Empty,
    Mountain,
}
impl Space {
    pub fn get_units(&self) -> usize {
        match self {
            Space::PlayerCapital { owner: _, units } => *units,
            Space::PlayerTown { owner: _, units } => *units,
            Space::NeutralTown { units } => *units,
            Space::PlayerEmpty { owner: _, units } => *units,
            Space::Empty | Space::Mountain => 0,
        }
    }
    pub fn unsafe_set_units(&mut self, new_units: usize) {
        match self {
            Space::PlayerCapital { owner: _, units } => *units = new_units,
            Space::PlayerTown { owner: _, units } => *units = new_units,
            Space::NeutralTown { units } => *units = new_units,
            Space::PlayerEmpty { owner: _, units } => *units = new_units,
            Space::Empty | Space::Mountain => {
                panic!("Tried to set units on invalid space type")
            }
        }
    }
    pub fn owner(&self) -> Option<usize> {
        match self {
            Space::PlayerCapital { owner, units: _ } => Some(*owner),
            Space::PlayerTown { owner, units: _ } => Some(*owner),
            Space::PlayerEmpty { owner, units: _ } => Some(*owner),
            Space::NeutralTown { .. } | Space::Empty | Space::Mountain => None,
        }
    }
}

pub type Spaces = [[Space; BOARD_SIZE]; BOARD_SIZE];

#[derive(Debug, Serialize, Deserialize)]
pub struct TurnRequest {
    pub turn: usize,
    pub player: usize,
    pub spaces: Spaces,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TurnResponse {
    pub from: Coordinate,
    pub to: Coordinate,
}
