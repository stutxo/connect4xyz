use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    NewGame,
    JoinGame,
    StartGame,
    Input(usize),
    Replay,
}
