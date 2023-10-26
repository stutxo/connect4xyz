use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use nostr_sdk::serde_json;

use crate::{
    components::{CoinMove, CoinSlot, DisplayTurn, ReplayButton, TextChanges, TopRow},
    messages::NetworkMessage,
    resources::{Board, PlayerMove, SendNetMsg},
    AppState,
};

use nanoid::nanoid;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::{window, History};

const COIN_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const COLUMNS: usize = 7;
const ROWS: usize = 7;
const SPACING: f32 = 5.0;

pub struct Connect4GuiPlugin;

impl Plugin for Connect4GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>()
            .insert_resource(Board::new())
            .add_systems(Startup, setup)
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), cleanup_menu)
            .add_systems(OnEnter(AppState::InGame), setup_game)
            .add_systems(
                Update,
                (place, move_coin.after(place), update_text.after(move_coin))
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Resource)]
struct MenuData {
    button_entity: Entity,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.20, 0.20, 0.20);

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut send_net_msg: ResMut<SendNetMsg>,
) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
        },
        ..Default::default()
    });

    #[cfg(target_arch = "wasm32")]
    if is_game_id_present() {
        send_net_msg.created_game = false;
        next_state.set(AppState::InGame);
    }
}

pub fn is_game_id_present() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(location) = window().and_then(|w| w.location().pathname().ok()) {
            return !location.is_empty() && location != "/";
        }
    }
    false
}

fn setup_menu(mut commands: Commands) {
    let button_entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Create New Game",
                        TextStyle {
                            font_size: 15.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        })
        .id();
    commands.insert_resource(MenuData { button_entity });
}

fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                let alphabet: [char; 52] = [
                    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F',
                    'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
                    'W', 'X', 'Y', 'Z',
                ];

                let game_id = nanoid!(8, &alphabet);

                let location = web_sys::window().unwrap().location();

                let host = location.host().unwrap();

                let protocol = location.protocol().unwrap();

                let full_url = format!("{protocol}//{host}/{game_id}");

                let history: History = window().unwrap().history().unwrap();
                history
                    .push_state_with_url(&JsValue::from_str("Create New Game"), "", Some(&full_url))
                    .expect("pushState failed");

                let window = web_sys::window().unwrap();
                let event = web_sys::CustomEvent::new("urlChanged").unwrap();
                window.dispatch_event(&event).unwrap();

                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    let offset_x = -COIN_SIZE.x * (COLUMNS as f32) / 2.0;
    let offset_y = -COIN_SIZE.y * (ROWS as f32) / 2.0;

    for column in 0..COLUMNS {
        for row in 0..ROWS {
            if row != 6 {
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(COIN_SIZE),
                            ..default()
                        },
                        texture: asset_server.load("white_circle.png"),
                        transform: Transform::from_xyz(
                            offset_x + column as f32 * (COIN_SIZE.x + SPACING),
                            offset_y + row as f32 * (COIN_SIZE.y + SPACING),
                            0.0,
                        ),
                        ..default()
                    })
                    .insert(CoinSlot::new(column, row));
            } else {
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(COIN_SIZE),
                            ..default()
                        },
                        texture: asset_server.load("white_circle.png"),
                        transform: Transform::from_xyz(
                            offset_x + column as f32 * (COIN_SIZE.x + SPACING),
                            offset_y + row as f32 * (COIN_SIZE.y + SPACING),
                            0.0,
                        ),
                        ..default()
                    })
                    .insert(Visibility::Hidden)
                    .insert(CoinSlot::new(column, row))
                    .insert(TopRow);
            }
        }
    }

    let text = Text::from_sections([TextSection::new(
        String::new(),
        TextStyle {
            color: Color::BLACK,
            font_size: 18.0,
            ..Default::default()
        },
    )]);

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            texture: asset_server.load("red_circle.png"),
            transform: Transform::from_xyz(0.0, 180.0, 1.0),
            ..default()
        })
        .insert(DisplayTurn)
        .with_children(|parent| {
            parent
                .spawn(Text2dBundle {
                    text: text.with_alignment(TextAlignment::Center),
                    transform: Transform {
                        translation: Vec3::new(0., -20.0, 1.0),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(TextChanges);
        });

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            texture: asset_server.load("repeat.png"),
            transform: Transform::from_xyz(30.0, 180., 1.0),
            ..default()
        })
        .insert(Visibility::Hidden)
        .insert(ReplayButton);
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn place(
    touches: Res<Touches>,
    mouse: Res<Input<MouseButton>>,
    mut board_pos: Query<(&CoinSlot, &mut Sprite, &Transform, &mut Visibility)>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut update_sprite: Query<&mut Handle<Image>, (With<TopRow>, Without<DisplayTurn>)>,
    mut board: ResMut<Board>,
    coin_query: Query<Entity, With<CoinMove>>,
    mut replay_button: Query<(&mut ReplayButton, &Transform, &mut Visibility), Without<CoinSlot>>,
    mut send_net_msg: ResMut<SendNetMsg>,
) {
    let (camera, camera_transform) = camera_query.single();

    let get_position = |cursor_position: Vec2, window: &Window| {
        let screen_size = Vec2::new(window.width(), window.height());
        let screen_position = Vec2::new(
            cursor_position.x / screen_size.x,
            1.0 - (cursor_position.y / screen_size.y),
        );

        let clip_position = (screen_position - Vec2::new(0.5, 0.5)) * 2.0;
        let mut position = camera
            .projection_matrix()
            .inverse()
            .project_point3(clip_position.extend(0.0));
        position = *camera_transform * position;
        position.truncate()
    };

    let mut hovered_column: Option<usize> = None;

    if let Some(window) = windows.iter().next() {
        if let Some(cursor) = window.cursor_position() {
            let position = get_position(cursor, window);

            for (coin, _, transform, _) in board_pos.iter() {
                if position.distance(transform.translation.truncate()) < 20.0 {
                    hovered_column = Some(coin.c);
                    break;
                }
            }
        }
    }

    for touch in touches.iter() {
        if let Some(window) = windows.iter().next() {
            let position = get_position(touch.position(), window);
            for (coin, _, transform, _) in board_pos.iter() {
                if position.distance(transform.translation.truncate()) < 20.0 {
                    hovered_column = Some(coin.c);
                    break;
                }
            }
        }
    }

    if board.winner.is_some() && send_net_msg.start {
        for (_, transform, mut visibility) in replay_button.iter_mut() {
            *visibility = Visibility::Visible;
            if mouse.just_pressed(MouseButton::Left)
                || mouse.just_pressed(MouseButton::Right)
                || touches.iter_just_pressed().any(|_| true)
            {
                if let Some(window) = windows.iter().next() {
                    if let Some(cursor) = window.cursor_position() {
                        let position = get_position(cursor, window);

                        if position.distance(transform.translation.truncate()) < 20.0 {
                            *board = Board::new();
                            for entity in coin_query.iter() {
                                commands.entity(entity).despawn();
                            }
                            *visibility = Visibility::Hidden;
                            send_net_msg.clone().send_replay();
                            break;
                        }
                    }
                }
                for touch in touches.iter() {
                    if let Some(window) = windows.iter().next() {
                        let position = get_position(touch.position(), window);
                        if position.distance(transform.translation.truncate()) < 20.0 {
                            *board = Board::new();
                            for entity in coin_query.iter() {
                                commands.entity(entity).despawn();
                            }
                            *visibility = Visibility::Hidden;
                            send_net_msg.clone().send_replay();
                            break;
                        }
                    }
                }
            }
        }
    }

    for (coin, mut sprite, _, mut visibility) in board_pos.iter_mut() {
        if Some(coin.c) == hovered_column && board.winner.is_none() {
            if coin.r == 6 && !board.in_progress {
                *visibility = Visibility::Visible;

                if send_net_msg.player_type == 1 {
                    for mut handle in &mut update_sprite.iter_mut() {
                        *handle = asset_server.load("red_circle.png");
                    }
                } else {
                    for mut handle in &mut update_sprite.iter_mut() {
                        *handle = asset_server.load("yellow_circle.png");
                    }
                }
            } else if coin.r == 6 {
                *visibility = Visibility::Hidden;
            } else {
                sprite.color = Color::rgb(0.9, 0.9, 0.9);
            }

            if board.in_progress {
                continue;
            }
            if board.player_turn == send_net_msg.player_type
                && (mouse.just_pressed(MouseButton::Left)
                    || mouse.just_pressed(MouseButton::Right)
                    || touches.iter_just_pressed().any(|_| true))
            {
                let row_pos = board.moves.iter().filter(|m| m.column == coin.c).count();
                if row_pos <= 5 {
                    let player_move = PlayerMove::new(send_net_msg.player_type, coin.c, row_pos);
                    board.moves.push(player_move);

                    send_net_msg.clone().send_input(coin.c);

                    let offset_x = -COIN_SIZE.x * (COLUMNS as f32) / 2.0;
                    let offset_y = -COIN_SIZE.y * (ROWS as f32) / 2.0;

                    if board.player_turn == 1 {
                        commands
                            .spawn(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(COIN_SIZE),
                                    ..Default::default()
                                },
                                texture: asset_server.load("red_circle.png"),
                                transform: Transform::from_xyz(
                                    offset_x + coin.c as f32 * (COIN_SIZE.x + SPACING),
                                    offset_y + 6_f32 * (COIN_SIZE.y + SPACING),
                                    1.0,
                                ),
                                ..Default::default()
                            })
                            .insert(CoinMove::new(player_move));
                    } else {
                        commands
                            .spawn(SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(COIN_SIZE),
                                    ..Default::default()
                                },
                                texture: asset_server.load("yellow_circle.png"),
                                transform: Transform::from_xyz(
                                    offset_x + coin.c as f32 * (COIN_SIZE.x + SPACING),
                                    offset_y + 6_f32 * (COIN_SIZE.y + SPACING),
                                    1.0,
                                ),
                                ..Default::default()
                            })
                            .insert(CoinMove::new(player_move));
                    }

                    break;
                }
            }
        } else if coin.r == 6 {
            *visibility = Visibility::Hidden;
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

fn move_coin(
    mut coin_query: Query<(&mut CoinMove, &mut Transform)>,
    board_pos: Query<(&CoinSlot, &Transform), Without<CoinMove>>,
    mut board: ResMut<Board>,
    time: Res<Time>,
) {
    for (mut coin, mut coin_transform) in coin_query.iter_mut() {
        for (coin_pos, board_transform) in board_pos.iter() {
            if coin.player_move.column == coin_pos.c && coin.player_move.row == coin_pos.r {
                let target = Vec3::new(
                    board_transform.translation.x,
                    board_transform.translation.y,
                    1.,
                );
                let mut current = Vec3::new(
                    coin_transform.translation.x,
                    coin_transform.translation.y,
                    1.,
                );

                if current.y > target.y {
                    current.y -= 1.0 * 250.0 * time.delta_seconds();
                    board.in_progress = true;
                } else if !coin.reached_target {
                    check_win(&mut board);

                    current.y = target.y;
                    board.in_progress = false;
                    board.player_turn = if board.player_turn == 1 { 2 } else { 1 };
                    coin.reached_target = true;
                }

                coin_transform.translation = current;
            }
        }
    }
}

fn check_win(board: &mut ResMut<Board>) {
    if has_winning_move(&board.moves) {
        board.winner = board.player_turn.into();
    }
}

fn update_text(
    mut display_turn: Query<&mut Handle<Image>, With<DisplayTurn>>,
    asset_server: Res<AssetServer>,
    mut text: Query<&mut Text, With<TextChanges>>,
    board: Res<Board>,
    send_net_msg: Res<SendNetMsg>,
) {
    if send_net_msg.start {
        check_player_connection_and_hide_button();
    }
    if send_net_msg.player_type == 0 {
        if !send_net_msg.start {
            for mut text in &mut text {
                text.sections[0].value = "waiting for player to join...".to_string();
            }
        }
    } else if send_net_msg.player_type == 3 {
        for mut text in &mut text {
            text.sections[0].value = "spectating(doesnt work yet)".to_string();
        }
    } else if board.player_turn == send_net_msg.player_type {
        for mut text in &mut text {
            text.sections[0].value = "your turn".to_string();
        }
    } else {
        for mut text in &mut text {
            text.sections[0].value = "waiting for player 2...".to_string();
        }
    }

    if send_net_msg.start {
        if board.player_turn == 1 {
            for mut handle in &mut display_turn.iter_mut() {
                *handle = asset_server.load("red_circle.png");
            }
        } else {
            for mut handle in &mut display_turn.iter_mut() {
                *handle = asset_server.load("yellow_circle.png");
            }
        }
    } else if send_net_msg.player_type == 3 {
        for mut handle in &mut display_turn.iter_mut() {
            *handle = asset_server.load("spec.png");
        }
    } else {
        for mut handle in &mut display_turn.iter_mut() {
            *handle = asset_server.load("connecting.png");
        }
    }
    if board.winner.is_some() {
        if send_net_msg.start {
            if board.winner == send_net_msg.player_type.into() {
                for mut text in &mut text {
                    text.sections[0].value = "you win!!".to_string();
                }
            } else {
                for mut text in &mut text {
                    text.sections[0].value = "lol loser".to_string();
                }
            }
        } else {
            for mut text in &mut text {
                text.sections[0].value = "game over".to_string();
            }
        }
        if send_net_msg.start {
            if send_net_msg.player_type == 1 {
                for mut handle in &mut display_turn.iter_mut() {
                    *handle = asset_server.load("red_circle.png");
                }
            } else {
                for mut handle in &mut display_turn.iter_mut() {
                    *handle = asset_server.load("yellow_circle.png");
                }
            }
        } else {
            for mut handle in &mut display_turn.iter_mut() {
                *handle = asset_server.load("spec.png");
            }
        }
    }
}

fn has_winning_move(moves: &[PlayerMove]) -> bool {
    moves.iter().any(|move_| move_.is_winner(moves))
}

#[wasm_bindgen]
extern "C" {
    fn hideCopyButton();
}

#[wasm_bindgen]
pub fn check_player_connection_and_hide_button() {
    hideCopyButton();
}
