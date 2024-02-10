use bevy::{log::error, prelude::Resource};
use futures::channel::mpsc::{Receiver, Sender};

use nostr_sdk::{serde_json, ClientMessage, EventBuilder, FromBech32, Keys, Kind, Tag, ToBech32};
use serde::{Deserialize, Serialize};
use web_sys::window;

use crate::messages::NetworkMessage;

#[derive(Resource)]
pub struct Board {
    pub moves: Vec<PlayerMove>,
    pub player_turn: usize,
    pub winner: Option<usize>,
    pub in_progress: bool,
    pub draw: bool,
}

impl Board {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            player_turn: 1,
            winner: None,
            in_progress: false,
            draw: false,
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
pub struct NetworkStuff {
    pub read: Option<Receiver<String>>,
}

impl NetworkStuff {
    pub fn new() -> Self {
        Self { read: None }
    }
}

#[derive(Resource, Clone)]
pub struct GameState {
    pub send: Option<Sender<ClientMessage>>,
    pub start: bool,
    pub nostr_keys: Keys,
    pub game_tag: Tag,
    pub player_type: usize,
    pub local_ln_address: Option<String>,
    pub p2_ln_address: Option<String>,
}

impl GameState {
    pub fn new() -> Self {
        let window = window().expect("no global `window` exists");
        let local_storage = window
            .local_storage()
            .expect("no local storage")
            .expect("local storage is not available");

        let nostr_keys = if let Ok(Some(nostr_keys)) = local_storage.get_item("nostr_key") {
            let secret_key = nostr_sdk::key::SecretKey::from_bech32(&nostr_keys).unwrap();
            let keys = Keys::new(secret_key);
            keys
        } else {
            let nostr_keys = Keys::generate();
            let secret_key =
                nostr_sdk::key::SecretKey::to_bech32(&nostr_keys.secret_key().unwrap());
            local_storage
                .set_item("nostr_key", &secret_key.unwrap())
                .expect("Error setting nostr_key in local storage");
            nostr_keys
        };

        Self {
            send: None,
            start: false,
            nostr_keys,
            game_tag: Tag::Hashtag("".to_string()),
            player_type: 0,
            local_ln_address: None,
            p2_ln_address: None,
        }
    }

    pub fn send_input(self, input: usize) {
        let msg = NetworkMessage::Input(input);
        let serialized_message = serde_json::to_string(&msg).unwrap();

        let nostr_msg = ClientMessage::event(
            EventBuilder::new(Kind::Regular(4444), serialized_message, [self.game_tag])
                .to_event(&self.nostr_keys)
                .unwrap(),
        );

        match self.send.clone().unwrap().try_send(nostr_msg) {
            Ok(()) => {}
            Err(e) => error!("Error sending send_input message: {}", e),
        };
    }
}
