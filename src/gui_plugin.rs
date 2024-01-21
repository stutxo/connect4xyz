use std::{
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
};

use bevy::{asset::AssetMetaCheck, core_pipeline::clear_color::ClearColorConfig, prelude::*};
use js_sys::Uint8Array;
use nostr_sdk::{secp256k1::XOnlyPublicKey, serde_json, Keys};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
extern crate js_sys;

use crate::{
    components::{CoinMove, CoinSlot, DisplayTurn, ReplayButton, TextChanges, TopRow},
    resources::{Board, PlayerMove, SendNetMsg},
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
static LOGIN_NOSTR_CALLED: AtomicBool = AtomicBool::new(false);
static LOGIN_GUEST_CALLED: AtomicBool = AtomicBool::new(false);
// static RESET_CALLED: AtomicBool = AtomicBool::new(false);

pub struct Connect4GuiPlugin;

impl Plugin for Connect4GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>()
            .insert_resource(Board::new())
            .add_systems(Startup, (setup, setup_game))
            .add_systems(Update, nostr_keys.run_if(in_state(AppState::LogIn)))
            .add_systems(
                Update,
                (
                    nostr_keys,
                    check_new_game_system.run_if(in_state(AppState::Menu)),
                ),
            )
            .add_systems(
                Update,
                (place, move_coin, update_text).run_if(in_state(AppState::InGame)),
            );
    }
}

fn nostr_keys(mut send_net_msg: ResMut<SendNetMsg>, mut next_state: ResMut<NextState<AppState>>) {
    if send_net_msg.created_game {
        if LOGIN_NOSTR_CALLED.load(Ordering::SeqCst) {
            LOGIN_NOSTR_CALLED.store(false, Ordering::SeqCst);

            let pub_key = send_net_msg.nostr_public_key.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut pub_key = pub_key.lock().await;
                *pub_key = get_pub_key().await;
            });
            let window = web_sys::window().unwrap();
            let event = web_sys::CustomEvent::new("loggedIn").unwrap();
            window.dispatch_event(&event).unwrap();
            next_state.set(AppState::Menu);
        }

        if LOGIN_GUEST_CALLED.load(Ordering::SeqCst) {
            LOGIN_GUEST_CALLED.store(false, Ordering::SeqCst);

            let window = web_sys::window().unwrap();
            let event = web_sys::CustomEvent::new("loggedIn").unwrap();
            window.dispatch_event(&event).unwrap();
            next_state.set(AppState::Menu);
        }
    } else {
        if LOGIN_NOSTR_CALLED.load(Ordering::SeqCst) {
            LOGIN_NOSTR_CALLED.store(false, Ordering::SeqCst);
            let pub_key = send_net_msg.nostr_public_key.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let mut pub_key = pub_key.lock().await;
                *pub_key = get_pub_key().await;
            });
            let window = web_sys::window().unwrap();
            let event = web_sys::CustomEvent::new("loggedIn").unwrap();
            window.dispatch_event(&event).unwrap();
            next_state.set(AppState::InGame);
        }

        if LOGIN_GUEST_CALLED.load(Ordering::SeqCst) {
            LOGIN_GUEST_CALLED.store(false, Ordering::SeqCst);
            let window = web_sys::window().unwrap();
            let event = web_sys::CustomEvent::new("loggedIn").unwrap();
            window.dispatch_event(&event).unwrap();
            next_state.set(AppState::InGame);
        }
    }
}

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
        next_state.set(AppState::LogIn);

        send_net_msg.created_game = false;
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

fn check_new_game_system(mut next_state: ResMut<NextState<AppState>>) {
    if CEATE_GAME_CALLED.load(Ordering::SeqCst) {
        let alphabet: [char; 52] = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
            'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y',
            'Z',
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
    coin_query: Query<Entity, With<CoinMove>>,
    mut end_game_buttons: Query<
        (&mut ReplayButton, &Transform, &mut Visibility),
        Without<CoinSlot>,
    >,
    send_net_msg: ResMut<SendNetMsg>,
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
    if (board.winner.is_some() || board.draw) && send_net_msg.player_type != 3 {
        if board.winner == Some(send_net_msg.player_type) {
            let send_board = board.moves.clone();
            let send_board = serde_json::to_string(&send_board).unwrap();

            let mut event_init = web_sys::CustomEventInit::new();

            event_init.detail(&JsValue::from_str(&send_board));

            let event =
                web_sys::CustomEvent::new_with_event_init_dict("send_board", &event_init).unwrap();

            web_sys::window().unwrap().dispatch_event(&event).unwrap();
        }

        //     for (_, transform, mut visibility) in end_game_buttons.iter_mut() {
        //         *visibility = Visibility::Visible;
        //         if mouse.just_pressed(MouseButton::Left)
        //             || mouse.just_pressed(MouseButton::Right)
        //             || touches.iter_just_pressed().any(|_| true)
        //         {
        //             if let Some(window) = windows.iter().next() {
        //                 if let Some(cursor) = window.cursor_position() {
        //                     let position = get_position(cursor, window);

        //                     if position.distance(transform.translation.truncate()) < 20.0 {
        //                         *board = Board::new();
        //                         for entity in coin_query.iter() {
        //                             commands.entity(entity).despawn();
        //                         }
        //                         *visibility = Visibility::Hidden;
        //                         send_net_msg.clone().send_replay();
        //                         hide_copy_board();
        //                         break;
        //                     }
        //                 }
        //             }
        //             for touch in touches.iter() {
        //                 if let Some(window) = windows.iter().next() {
        //                     let position = get_position(touch.position(), window);
        //                     if position.distance(transform.translation.truncate()) < 20.0 {
        //                         *board = Board::new();
        //                         for entity in coin_query.iter() {
        //                             commands.entity(entity).despawn();
        //                         }
        //                         *visibility = Visibility::Hidden;
        //                         send_net_msg.clone().send_replay();
        //                         hide_copy_board();
        //                         break;
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }
    }

    for (coin, mut sprite, _, mut visibility) in board_pos.iter_mut() {
        if Some(coin.c) == hovered_column && board.winner.is_none() {
            if coin.r == 6 && !board.in_progress {
                *visibility = Visibility::Visible;

                if send_net_msg.player_type == 1 {
                    for mut handle in &mut update_sprite.iter_mut() {
                        *handle = asset_server.load("red_circle.png");
                    }
                } else if send_net_msg.player_type == 2 {
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
    send_net_msg: Res<SendNetMsg>,
) {
    if send_net_msg.start {
        check_player_connection_and_hide_button();
    } else {
        hide_new_game_button();
    }

    let new_image: Option<&str>;
    let mut new_text_value: &str;

    if board.winner.is_some() {
        if board.winner == Some(send_net_msg.player_type) {
            new_text_value = "You win!!";
        } else {
            new_text_value = "You lose!!";
        }

        if send_net_msg.player_type == 3 {
            new_text_value = "Game Over!!";
            new_image = match board.winner {
                Some(1) => Some("red_circle.png"),
                _ => Some("yellow_circle.png"),
            };
        } else {
            new_image = match send_net_msg.player_type {
                1 => Some("red_circle.png"),
                2 => Some("yellow_circle.png"),
                _ => None,
            };
        }
    } else if board.draw {
        new_text_value = "Its a draw!!";
        new_image = None;
    } else if send_net_msg.player_type == 0 {
        new_text_value = "Waiting for player to join...";
        new_image = None;
    } else {
        new_image = match board.player_turn {
            1 => Some("red_circle.png"),
            2 => Some("yellow_circle.png"),
            _ => None,
        };

        new_text_value = match send_net_msg.player_type {
            3 => "Spectating",
            _ if board.player_turn == send_net_msg.player_type => "Its your turn",
            _ => "Player 2's turn...",
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
    fn hideCopyButton();
}

#[wasm_bindgen]
pub fn check_player_connection_and_hide_button() {
    hideCopyButton();
}

#[wasm_bindgen]
extern "C" {
    fn hideNewGameButton();
}

#[wasm_bindgen]
pub fn hide_new_game_button() {
    hideNewGameButton();
}

#[wasm_bindgen]
extern "C" {
    fn hideCopyBoardButton();
}

#[wasm_bindgen]
pub fn hide_copy_board() {
    hideCopyBoardButton();
}

#[wasm_bindgen]
pub fn new_game() {
    CEATE_GAME_CALLED.store(true, Ordering::SeqCst);
}

// #[wasm_bindgen]
// pub fn replay() {
//     info!("replay called");
//     RESET_CALLED.store(true, Ordering::SeqCst);
// }

// #[wasm_bindgen]
// pub fn nostr_login() {
//     LOGIN_NOSTR_CALLED.store(true, Ordering::SeqCst);
// }
#[wasm_bindgen]
pub fn nostr_login(pubkey: String) {
    info!("nostr login called {}", pubkey);
    LOGIN_NOSTR_CALLED.store(true, Ordering::SeqCst);
}

#[wasm_bindgen]
pub fn guest_login() {
    LOGIN_GUEST_CALLED.store(true, Ordering::SeqCst);
}

#[wasm_bindgen(inline_js = "
export async function pub_key() {
    if (typeof window.nostr !== 'undefined') {
        try {
            const encoder = new TextEncoder();
            const publicKey = await window.nostr.getPublicKey();
            const view = encoder.encode(publicKey);
            console.log(view);
            return view;
        } catch (error) {
            // Handle the error when the popup is closed or any other error
            console.error('Error occurred:', error);
            // Return null or handle it in a way that does not crash your app
            return null;
        }
    } else {
        console.error('window.nostr is not available');
        return null;
    }
}
")]

extern "C" {
    async fn pub_key() -> JsValue;
}

async fn get_pub_key() -> XOnlyPublicKey {
    info!("nostr login called");
    let get_pub_key = pub_key().await;

    let array = Uint8Array::new(&get_pub_key);
    let bytes: Vec<u8> = array.to_vec();
    let nostr_pub_key = std::str::from_utf8(&bytes).unwrap();
    XOnlyPublicKey::from_str(nostr_pub_key).unwrap()
}
