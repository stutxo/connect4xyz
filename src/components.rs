use bevy::prelude::Component;

use crate::resources::PlayerMove;

#[derive(Component)]
pub struct CoinSlot {
    pub c: usize,
    pub r: usize,
}

impl CoinSlot {
    pub fn new(c: usize, r: usize) -> Self {
        Self { c, r }
    }
}

#[derive(Component)]
pub struct CoinMove {
    pub player_move: PlayerMove,
    pub reached_target: bool,
}

impl CoinMove {
    pub fn new(player_move: PlayerMove) -> Self {
        Self {
            player_move,
            reached_target: false,
        }
    }
}

#[derive(Component)]
pub struct TopRow;

#[derive(Component)]
pub struct TextChanges;

#[derive(Component)]
pub struct DisplayTurn;
#[derive(Component)]
pub struct ReplayButton;
