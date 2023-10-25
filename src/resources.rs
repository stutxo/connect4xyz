use bevy::prelude::Resource;
use futures::channel::mpsc::{Receiver, Sender};
use nostr_sdk::Keys;

use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct Board {
    pub moves: Vec<PlayerMove>,
    pub player_turn: usize,
    pub winner: Option<usize>,
    pub in_progress: bool,
}

impl Board {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            player_turn: 1,
            winner: None,
            in_progress: false,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
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

#[derive(Resource)]
pub struct NostrStuff {
    pub local_keys: Keys,
}

impl NostrStuff {
    pub fn new() -> Self {
        Self {
            local_keys: Keys::generate(),
        }
    }
}

#[derive(Resource)]
pub struct NetworkStuff {
    pub read: Option<Receiver<String>>,
}

impl NetworkStuff {
    pub fn new() -> Self {
        Self { read: None }
    }
}

#[derive(Resource, Clone)]
pub struct SendNetMsg {
    pub send: Option<Sender<String>>,
    pub start: bool,
    pub local_player: usize,
    pub created_game: bool,
}

impl SendNetMsg {
    pub fn new() -> Self {
        Self {
            send: None,
            start: false,
            local_player: 0,
            created_game: true,
        }
    }
}
