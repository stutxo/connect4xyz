use nostr_sdk::secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    NewGame,
    JoinGame(XOnlyPublicKey),
    StartGame(Players),
    Input(usize),
    Replay,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Players {
    pub player1: XOnlyPublicKey,
    pub player2: XOnlyPublicKey,
}

impl Players {
    pub fn new(player1: XOnlyPublicKey, player2: XOnlyPublicKey) -> Self {
        Self { player1, player2 }
    }
}
