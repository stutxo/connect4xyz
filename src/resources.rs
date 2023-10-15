use bevy::prelude::Resource;

#[derive(Resource)]
pub struct GameState {
    pub player_move: [[i32; 7]; 6],
    pub turn: i32,
    pub winner: i32,
    pub game_over: bool,
}
