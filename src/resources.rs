use std::collections::HashMap;

use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Board {
    pub moves: Vec<PlayerMove>,
    pub column_state: HashMap<usize, usize>,
    pub player_turn: usize,
    pub winner: Option<usize>,
    pub in_progress: bool,
}

impl Board {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            column_state: HashMap::new(),
            player_turn: 1,
            winner: None,
            in_progress: false,
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
    pub fn is_winner(&self, moves: &[PlayerMove]) -> bool {
        [(0, 1), (1, 0), (1, 1), (1, -1)]
            .iter()
            .any(|&(column_direction, row_direction)| {
                self.check_direction(moves, column_direction, row_direction)
                    + self.check_direction(moves, -column_direction, -row_direction)
                    + 1
                    >= 4
            })
    }

    pub fn check_direction(
        &self,
        moves: &[PlayerMove],
        column_direction: isize,
        row_direction: isize,
    ) -> usize {
        let mut count = 0;
        let mut current_column = self.column as isize + column_direction;
        let mut current_row = self.row as isize + row_direction;

        while current_column >= 0
            && current_row >= 0
            && current_column < 7
            && current_row < 6
            && moves.iter().any(|m| {
                m.player == self.player
                    && m.column == current_column as usize
                    && m.row == current_row as usize
            })
        {
            count += 1;
            current_column += column_direction;
            current_row += row_direction;
        }

        count
    }
}
