use bevy::prelude::*;

use crate::components::Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_empty().insert(Player::new(1));
    commands.spawn_empty().insert(Player::new(2));
}
