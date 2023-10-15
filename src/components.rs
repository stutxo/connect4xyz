use bevy::prelude::Component;

#[derive(Component)]
pub struct Coin {
    pub c: usize,
    pub r: usize,
}

impl Coin {
    pub fn new(c: usize, r: usize) -> Self {
        Self { c, r }
    }
}
