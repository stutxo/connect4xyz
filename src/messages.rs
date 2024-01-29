// use nostr_sdk::secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    NewGame(Option<String>),
    JoinGame(Players),
    Input(usize),
    Replay,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Players {
    pub player1: Option<String>,
    pub player2: Option<String>,
}

impl Players {
    pub fn new(player1: Option<String>, player2: Option<String>) -> Self {
        Self { player1, player2 }
    }
}
