use bevy::prelude::*;
use gui_plugin::Connect4GuiPlugin;
use nostr_plugin::NostrPlugin;

mod components;
mod gui_plugin;
mod messages;
mod nostr_plugin;
mod resources;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "connect4.xyz".to_string(),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            Connect4GuiPlugin,
            NostrPlugin,
        ))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}
