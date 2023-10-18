use std::collections::HashMap;

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Board {
    pub moves: Vec<PlayerMove>,
    pub column_state: HashMap<usize, usize>,
    pub player_turn: usize,
    pub winner: Option<usize>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            column_state: HashMap::new(),
            player_turn: 1,
            winner: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PlayerMove {
    pub player: usize,
    pub column: usize,
    pub row: usize,
}

impl PlayerMove {
    pub fn new(player: usize, column: usize, row: usize) -> Self {
        Self {
            player,
            column,
            row,
        }
    }
}
