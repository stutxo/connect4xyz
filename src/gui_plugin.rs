use bevy::{core_pipeline::clear_color::ClearColorConfig, log, prelude::*, ui::update};

use crate::{
    components::{Coin, CoinPos, TopRow},
    resources::{Board, BoardState},
};

const COIN_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const COLUMNS: usize = 7;
const ROWS: usize = 7;
const SPACING: f32 = 5.0;
pub struct Connect4GuiPlugin;

impl Plugin for Connect4GuiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoardState::new(1))
            .insert_resource(Board::new())
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
                    .insert(CoinPos::new(column, row));
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
                    .insert(CoinPos::new(column, row))
                    .insert(TopRow());
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn place(
    touches: Res<Touches>,
    mouse: Res<Input<MouseButton>>,
    mut board_pos: Query<(&mut CoinPos, &mut Sprite, &Transform, &mut Visibility)>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut board_state: ResMut<BoardState>,
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

    for (coin, mut sprite, transform, mut visibility) in board_pos.iter_mut() {
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
                let coin_location = *board.column_state.get(&coin.c).unwrap_or(&(6 - 1));
                info!("coin_location: {}", coin_location);

                board.player_turn = if board.player_turn == 1 { 2 } else { 1 };
                let next_player_turn = board.player_turn;

                if coin_location != 0 || !board.moves.iter().any(|&(_, _, loc)| loc == 0) {
                    board.moves.push((next_player_turn, coin.c, coin_location));

                    let new_coin_location = if coin_location > 0 {
                        coin_location - 1
                    } else {
                        coin_location
                    };
                    board.column_state.insert(coin.c, new_coin_location);
                }

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
                        .insert(Coin::new(next_player_turn, coin.c, coin_location));
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
                        .insert(Coin::new(next_player_turn, coin.c, coin_location));
                }

                break;
            }
        } else if coin.r == 6 {
            *visibility = Visibility::Hidden;
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

fn move_coin(
    mut coin_query: Query<(&mut Coin, &mut Transform)>,
    board: Res<Board>,
    mut board_pos: Query<(&mut CoinPos, &mut Sprite, &Transform, &mut Visibility), Without<Coin>>,
) {
    for (coin_comp, mut coin_transform) in coin_query.iter_mut() {
        for (player, column, row) in board.moves.iter() {
            // If the coin's location matches the move
            if coin_comp.location == (*player, *column, *row) {
                for (pos, _, column_transform, _) in board_pos.iter() {
                    if pos.c == *column
                        && column_transform
                            .translation
                            .distance(coin_transform.translation)
                            > 10.0
                    {
                        info!("column transform: {}", column_transform.translation);
                        info!("coin transform: {}", coin_transform.translation);
                        coin_transform.translation += Vec3::new(0.0, -1.0, 0.0);
                    }
                }
            }
        }
    }
}
