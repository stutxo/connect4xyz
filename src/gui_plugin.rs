use std::sync::atomic::{AtomicBool, Ordering};

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use nostr_sdk::serde_json;
use serde::Serialize;

extern crate js_sys;

use crate::{
    components::{CoinMove, CoinSlot, DisplayTurn, TextChanges, TopRow},
    resources::{Board, GameState, PlayerMove},
    AppState,
};

use nanoid::nanoid;

use wasm_bindgen::prelude::*;
use web_sys::{window, History};

const COIN_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const COLUMNS: usize = 7;
const ROWS: usize = 7;
const SPACING: f32 = 5.0;

static CEATE_GAME_CALLED: AtomicBool = AtomicBool::new(false);
static JOIN_GAME_CALLED: AtomicBool = AtomicBool::new(false);

#[derive(Serialize)]
struct ShareData {
    msg: String,
    moves: Vec<PlayerMove>,
}

pub struct Connect4GuiPlugin;

impl Plugin for Connect4GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>()
            .insert_resource(Board::new())
            .add_systems(Startup, (setup, setup_game))
            .add_systems(
                Update,
                (check_new_game_system.run_if(in_state(AppState::Menu)),),
            )
            .add_systems(
                Update,
                (place, move_coin, update_text).run_if(in_state(AppState::InGame)),
            );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
        },
        ..Default::default()
    });
}

fn check_new_game_system(mut next_state: ResMut<NextState<AppState>>) {
    if CEATE_GAME_CALLED.load(Ordering::SeqCst) {
        let alphabet: [char; 31] = [
            '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j',
            'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];

        let game_id = nanoid!(6, &alphabet);

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

        CEATE_GAME_CALLED.store(false, Ordering::SeqCst);
    }

    if JOIN_GAME_CALLED.load(Ordering::SeqCst) {
        next_state.set(AppState::InGame);

        JOIN_GAME_CALLED.store(false, Ordering::SeqCst);
    }
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

    let game_text = Text::from_sections([TextSection::new(
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

            transform: Transform::from_xyz(0.0, 180.0, 1.0),
            ..default()
        })
        .insert(DisplayTurn)
        .with_children(|parent| {
            parent
                .spawn(Text2dBundle {
                    text: game_text.with_alignment(TextAlignment::Center),
                    transform: Transform {
                        translation: Vec3::new(0., -20.0, 1.0),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(TextChanges);
        });

    let window = web_sys::window().unwrap();
    let event = web_sys::CustomEvent::new("wasmLoaded").unwrap();
    window.dispatch_event(&event).unwrap();
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
    game_state: ResMut<GameState>,
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

    #[allow(clippy::collapsible_if)]
    if (board.winner.is_some() || board.draw) && game_state.player_type != 3 {
        let location = web_sys::window().unwrap().location();
        let full_url = location.href().unwrap();

        if board.winner == Some(game_state.player_type) {
            let msg = if let Some(ref address) = game_state.p2_ln_address {
                format!("I beat {} at #connect4\n\n\n{}\n\n", address, full_url)
            } else {
                format!(
                    "I beat an unknown player at #connect4\n\n\n{}\n\n",
                    full_url
                )
            };

            let share_data = ShareData {
                msg,
                moves: board.moves.clone(),
            };

            let send_board = serde_json::to_string(&share_data).unwrap();

            let mut event_init = web_sys::CustomEventInit::new();

            event_init.detail(&JsValue::from_str(&send_board));

            let event =
                web_sys::CustomEvent::new_with_event_init_dict("send_board", &event_init).unwrap();

            web_sys::window().unwrap().dispatch_event(&event).unwrap();
        } else {
            let msg = if let Some(ref address) = game_state.p2_ln_address {
                format!("I lost to {} at #connect4\n\n{}\n\n", address, full_url)
            } else {
                format!(
                    "I lost to an unknown player at #connect4\n\n{}\n\n",
                    full_url
                )
            };

            let share_data = ShareData {
                msg,
                moves: board.moves.clone(),
            };

            let send_board = serde_json::to_string(&share_data).unwrap();

            let mut event_init = web_sys::CustomEventInit::new();

            event_init.detail(&JsValue::from_str(&send_board));

            let event =
                web_sys::CustomEvent::new_with_event_init_dict("send_board", &event_init).unwrap();

            web_sys::window().unwrap().dispatch_event(&event).unwrap();
        }
    }

    for (coin, mut sprite, _, mut visibility) in board_pos.iter_mut() {
        if Some(coin.c) == hovered_column && board.winner.is_none() {
            if coin.r == 6 && !board.in_progress {
                *visibility = Visibility::Visible;

                if game_state.player_type == 1 {
                    for mut handle in &mut update_sprite.iter_mut() {
                        *handle = asset_server.load("red_circle.png");
                    }
                } else if game_state.player_type == 2 {
                    for mut handle in &mut update_sprite.iter_mut() {
                        *handle = asset_server.load("yellow_circle.png");
                    }
                } else {
                    for mut handle in &mut update_sprite.iter_mut() {
                        *handle = asset_server.load("white_circle.png");
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
            if board.player_turn == game_state.player_type
                && (mouse.just_pressed(MouseButton::Left)
                    || mouse.just_pressed(MouseButton::Right)
                    || touches.iter_just_pressed().any(|_| true))
            {
                let row_pos = board.moves.iter().filter(|m| m.column == coin.c).count();
                if row_pos <= 5 {
                    let player_move = PlayerMove::new(game_state.player_type, coin.c, row_pos);
                    board.moves.push(player_move);

                    game_state.clone().send_input(coin.c);

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
                    board.player_turn = if board.player_turn == 1 { 2 } else { 1 };

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
                    if board.winner.is_none() && is_draw(&board) {
                        board.draw = true;
                    }

                    current.y = target.y;
                    board.in_progress = false;

                    coin.reached_target = true;
                }

                coin_transform.translation = current;
            }
        }
    }
}

fn check_win(board: &mut ResMut<Board>) {
    if has_winning_move(&board.moves) {
        board.winner = if board.player_turn == 1 {
            Some(2)
        } else {
            Some(1)
        };
    }
}

fn is_draw(board: &Board) -> bool {
    board.moves.len() == 42
}

fn update_text(
    mut display_turn: Query<&mut Handle<Image>, With<DisplayTurn>>,
    asset_server: Res<AssetServer>,
    mut text: Query<&mut Text, With<TextChanges>>,
    board: Res<Board>,
    game_state: Res<GameState>,
) {
    if game_state.start {
        check_player_connection_and_hide_button();
    } else {
        hide_new_game_button();
    }

    let new_image: Option<&str>;
    let mut new_text_value: String;

    if board.winner.is_some() {
        if board.winner == Some(game_state.player_type) {
            let address_display = match &game_state.local_ln_address {
                Some(address) => address.clone(),
                None => "You".to_string(),
            };
            let enemy_display = match &game_state.p2_ln_address {
                Some(enemy) => enemy.clone(),
                None => "Player 2".to_string(),
            };
            new_text_value = format!("{} beat {}", address_display, enemy_display);
        } else {
            let address_display = match &game_state.local_ln_address {
                Some(address) => address.clone(),
                None => "You".to_string(),
            };
            let enemy_display = match &game_state.p2_ln_address {
                Some(enemy) => enemy.clone(),
                None => "Player 2".to_string(),
            };
            new_text_value = format!("{} lost to {}", address_display, enemy_display);
        }

        if game_state.player_type == 3 {
            new_text_value = "Game Over!!".to_string();
            new_image = match board.winner {
                Some(1) => Some("red_circle.png"),
                _ => Some("yellow_circle.png"),
            };
        } else {
            new_image = match game_state.player_type {
                1 => Some("red_circle.png"),
                2 => Some("yellow_circle.png"),
                _ => None,
            };
        }
    } else if board.draw {
        let address_display = match &game_state.local_ln_address {
            Some(address) => address.clone(),
            None => "You".to_string(),
        };
        let enemy_display = match &game_state.p2_ln_address {
            Some(enemy) => enemy.clone(),
            None => "Player 2".to_string(),
        };
        new_text_value = format!("{} drew against {}", address_display, enemy_display);
        new_image = None;
    } else if game_state.player_type == 0 {
        new_text_value = "Waiting for player to join...".to_string();
        new_image = None;
    } else {
        new_image = match board.player_turn {
            1 => Some("red_circle.png"),
            2 => Some("yellow_circle.png"),
            _ => None,
        };

        new_text_value = match game_state.player_type {
            3 => "Spectating".to_string(),
            _ if board.player_turn == game_state.player_type => {
                let address_display = match &game_state.local_ln_address {
                    Some(address) => address.clone(),
                    None => "".to_string(),
                };
                format!("Its your turn {}", address_display)
            }
            _ => {
                let address_display = match &game_state.p2_ln_address {
                    Some(address) => address.clone(),
                    None => "Player 2".to_string(),
                };
                format!("{}'s turn", address_display)
            }
        };
    }

    if let Some(image) = new_image {
        for mut handle in display_turn.iter_mut() {
            *handle = asset_server.load(image);
        }
    }

    for mut txt in text.iter_mut() {
        if txt.sections[0].value != new_text_value {
            txt.sections[0].value = new_text_value.to_string();
        }
    }
}

fn has_winning_move(moves: &[PlayerMove]) -> bool {
    moves.iter().any(|move_| move_.is_winner(moves))
}

#[wasm_bindgen]
extern "C" {
    fn hideNewGameButton();
}
#[wasm_bindgen]
extern "C" {
    fn hideCopyButton();
}

#[wasm_bindgen]
pub fn check_player_connection_and_hide_button() {
    hideCopyButton();
}

#[wasm_bindgen]
pub fn hide_new_game_button() {
    hideNewGameButton();
}

#[wasm_bindgen]
pub fn new_game() {
    CEATE_GAME_CALLED.store(true, Ordering::SeqCst);
}
#[wasm_bindgen]
pub fn join_game() {
    JOIN_GAME_CALLED.store(true, Ordering::SeqCst);
}
