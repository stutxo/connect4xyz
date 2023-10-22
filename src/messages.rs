use crate::resources::PlayerMove;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    Lfg,
    StartGame,
    Input(PlayerMove),
}
