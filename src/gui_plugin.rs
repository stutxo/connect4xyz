use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use crate::{
    components::{CoinMove, CoinSlot, TopRow},
    resources::{Board, PlayerMove},
};

const COIN_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const COLUMNS: usize = 7;
const ROWS: usize = 7;
const SPACING: f32 = 5.0;
pub struct Connect4GuiPlugin;

impl Plugin for Connect4GuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::new())
            .add_systems(Startup, setup)
            .add_systems(Update, (place, move_coin));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
        },
        ..Default::default()
    });

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
                    .insert(TopRow());
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn place(
    touches: Res<Touches>,
    mouse: Res<Input<MouseButton>>,
    mut board_pos: Query<(&mut CoinSlot, &mut Sprite, &Transform, &mut Visibility)>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut update_sprite: Query<&mut Handle<Image>, With<TopRow>>,
    mut board: ResMut<Board>,
) {
    // if board.turn == 0 {
    //     return;
    // }
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

    for (coin, mut sprite, _, mut visibility) in board_pos.iter_mut() {
        if Some(coin.c) == hovered_column {
            if coin.r == 6 {
                *visibility = Visibility::Visible;
                if board.player_turn == 1 {
                    for mut handle in &mut update_sprite.iter_mut() {
                        *handle = asset_server.load("red_circle.png");
                    }
                } else {
                    for mut handle in &mut update_sprite.iter_mut() {
                        *handle = asset_server.load("yellow_circle.png");
                    }
                }
            } else {
                sprite.color = Color::rgb(0.9, 0.9, 0.9);
            }

            if mouse.just_pressed(MouseButton::Left)
                || mouse.just_pressed(MouseButton::Right)
                || touches.iter_just_pressed().any(|_| true)
            {
                let coin_location = *board.column_state.get(&coin.c).unwrap_or(&0);

                if coin_location <= 5 {
                    board.player_turn = if board.player_turn == 1 { 2 } else { 1 };
                    let next_player_turn = board.player_turn;

                    let player_move = PlayerMove::new(next_player_turn, coin.c, coin_location);
                    board.moves.push(player_move);

                    let new_coin_location = coin_location + 1;

                    board.column_state.insert(coin.c, new_coin_location);

                    let offset_x = -COIN_SIZE.x * (COLUMNS as f32) / 2.0;
                    let offset_y = -COIN_SIZE.y * (ROWS as f32) / 2.0;

                    if board.player_turn != 1 {
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
    mut coin_query: Query<(&CoinMove, &mut Transform)>,
    board_pos: Query<(&CoinSlot, &Transform), Without<CoinMove>>,
) {
    for (coin, mut coin_transform) in coin_query.iter_mut() {
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
                let mut target_reached = false;
                while !target_reached {
                    if current.y > target.y {
                        current.y -= 1.0;
                    } else {
                        target_reached = true;
                    }
                    coin_transform.translation = current;
                }
            }
        }
    }
}

// fn check_game(board: ResMut<Board>) {}
