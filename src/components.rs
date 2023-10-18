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
}

impl CoinMove {
    pub fn new(player_move: PlayerMove) -> Self {
        Self { player_move }
    }
}

#[derive(Component)]
pub struct TopRow();
