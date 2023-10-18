use std::collections::HashMap;

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct BoardState {
    pub animate_column: Option<usize>,
}

impl BoardState {
    pub fn new(turn: i32) -> Self {
        Self {
            animate_column: None,
        }
    }
}

#[derive(Resource)]
pub struct Board {
    pub moves: Vec<(usize, usize, usize)>,
    pub column_state: HashMap<usize, usize>,
    pub player_turn: usize,
}

impl Board {
    pub fn new() -> Self {
        let mut column_state = HashMap::new();

        for col in 0..7 {
            column_state.insert(col, 5);
        }

        Self {
            moves: Vec::new(),
            column_state,
            player_turn: 1,
        }
    }
}
