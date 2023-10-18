use bevy::prelude::Component;

#[derive(Component)]
pub struct CoinPos {
    pub c: usize,
    pub r: usize,
}

impl CoinPos {
    pub fn new(c: usize, r: usize) -> Self {
        Self { c, r }
    }
}

#[derive(Component)]
pub struct Player {
    pub player: i32,
}

impl Player {
    pub fn new(player: i32) -> Self {
        Self { player }
    }
}

#[derive(Component)]
pub struct Coin {
    pub location: (usize, usize, usize),
}

impl Coin {
    pub fn new(player: usize, column: usize, row: usize) -> Self {
        Self {
            location: (player, column, row),
        }
    }
}

#[derive(Component)]
pub struct TopRow();
