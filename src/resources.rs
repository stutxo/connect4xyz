use bevy::{
    prelude::{error, info, Resource},
    time::Time,
};
use futures::channel::mpsc::{Receiver, Sender};
use nostr_sdk::{
    prelude::kind, secp256k1::XOnlyPublicKey, serde_json, ClientMessage, Event, EventBuilder, Keys,
    Kind, Tag, Timestamp,
};

use serde::{Deserialize, Serialize};

use crate::messages::{NetworkMessage, Players};

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
    pub send: Option<Sender<ClientMessage>>,
    pub start: bool,
    pub local_player: XOnlyPublicKey,
    pub created_game: bool,
    pub nostr_keys: Keys,
    pub game_tag: Tag,
    pub player_type: usize,
}

impl SendNetMsg {
    pub fn new() -> Self {
        let nostr_keys = Keys::generate();
        let local_player = nostr_keys.public_key();

        Self {
            send: None,
            start: false,
            local_player,
            created_game: true,
            nostr_keys,
            game_tag: Tag::Hashtag("".to_string()),
            player_type: 0,
        }
    }

    pub fn new_game(self) {
        let msg = NetworkMessage::NewGame;
        let serialized_message = serde_json::to_string(&msg).unwrap();

        let nostr_msg = ClientMessage::new_event(
            EventBuilder::new(
                Kind::Replaceable(11111),
                serialized_message,
                &[self.game_tag, Tag::Hashtag("new_game".to_string())],
            )
            .to_event(&self.nostr_keys)
            .unwrap(),
        );

        info!("sending new game msg {:?}", nostr_msg);

        match self.send.clone().unwrap().try_send(nostr_msg) {
            Ok(()) => {}
            Err(e) => error!("Error sending new_game message: {}", e),
        };
    }

    pub fn join_game(self) {
        let msg = NetworkMessage::JoinGame(self.local_player);
        let serialized_message = serde_json::to_string(&msg).unwrap();

        //use nip5 instead?
        // some relays i have to add nip40 or the message doesnt get cleared from the relay
        //nip40 Expiration Timestamp https://github.com/nostr-protocol/nips/blob/master/40.md

        let expire = Tag::Expiration(Timestamp::now() + 5_i64);

        let nostr_msg = ClientMessage::new_event(
            EventBuilder::new(
                Kind::Ephemeral(21000),
                serialized_message,
                &[self.game_tag, expire, Tag::Hashtag("join_game".to_string())],
            )
            .to_event(&self.nostr_keys)
            .unwrap(),
        );

        info!("sending join game msg {:?}", nostr_msg);

        match self.send.clone().unwrap().try_send(nostr_msg) {
            Ok(()) => {}
            Err(e) => error!("Error sending join_game message: {}", e),
        };
    }

    pub fn start_game(self, players: Players) {
        let msg = NetworkMessage::StartGame(players);
        let serialized_message = serde_json::to_string(&msg).unwrap();

        let nostr_msg = ClientMessage::new_event(
            EventBuilder::new(
                Kind::Replaceable(11111),
                serialized_message,
                &[self.game_tag],
            )
            .to_event(&self.nostr_keys)
            .unwrap(),
        );

        info!("sending start game msg {:?}", nostr_msg);

        match self.send.clone().unwrap().try_send(nostr_msg) {
            Ok(()) => {}
            Err(e) => error!("Error sending start_game message: {}", e),
        };
    }

    pub fn send_input(self, input: usize) {
        let msg = NetworkMessage::Input(input);
        let serialized_message = serde_json::to_string(&msg).unwrap();

        let nostr_msg = ClientMessage::new_event(
            EventBuilder::new_text_note(serialized_message, &[self.game_tag])
                .to_event(&self.nostr_keys)
                .unwrap(),
        );

        info!("sending input game msg {:?}", nostr_msg);

        match self.send.clone().unwrap().try_send(nostr_msg) {
            Ok(()) => {}
            Err(e) => error!("Error sending send_input message: {}", e),
        };
    }

    pub fn send_replay(self) {
        let msg = NetworkMessage::Replay;
        let serialized_message = serde_json::to_string(&msg).unwrap();

        let nostr_msg = ClientMessage::new_event(
            EventBuilder::new_text_note(serialized_message, &[self.game_tag])
                .to_event(&self.nostr_keys)
                .unwrap(),
        );

        info!("sending replay game msg {:?}", nostr_msg);

        match self.send.clone().unwrap().try_send(nostr_msg) {
            Ok(()) => {}
            Err(e) => error!("Error sending new_game message: {}", e),
        };
    }
}
