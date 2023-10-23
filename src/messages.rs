use serde::{Deserialize, Serialize};

use crate::resources::PlayerMove;

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    Lfg,
    StartGame,
    Input(usize),
    Replay,
    Spectate(Vec<PlayerMove>),
}
