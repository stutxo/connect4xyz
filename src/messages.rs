use nostr_sdk::secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    NewGame(Option<String>),
    JoinGame(Players),
    Input(usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Players {
    pub p1_name: Option<String>,
    pub p2_name: Option<String>,
    pub p1_pubkey: XOnlyPublicKey,
    pub p2_pubkey: XOnlyPublicKey,
}

impl Players {
    pub fn new(
        p1_name: Option<String>,
        p2_name: Option<String>,
        p1_pubkey: XOnlyPublicKey,
        p2_pubkey: XOnlyPublicKey,
    ) -> Self {
        Self {
            p1_name,
            p2_name,
            p1_pubkey,
            p2_pubkey,
        }
    }
}
