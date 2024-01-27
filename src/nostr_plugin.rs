use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use bevy::prelude::*;
use futures::{lock::Mutex, StreamExt};
use nostr_sdk::{
    serde_json, Client, ClientMessage, Event as NostrEvent, EventBuilder, Filter, Keys, Kind,
    RelayPoolNotification, Tag, Timestamp,
};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;

use crate::{
    components::{CoinMove, ReplayButton},
    gui_plugin::LOGIN_PUBKEY,
    messages::NetworkMessage,
    resources::{Board, NetworkStuff, PlayerMove, SendNetMsg},
    AppState,
};

const COIN_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const COLUMNS: usize = 7;
const ROWS: usize = 7;
const SPACING: f32 = 5.0;

#[derive(Debug)]
enum Mode {
    ListGameMode,
    SpectateMode,
    JoinGameMode,
}

pub struct NostrPlugin;

impl Plugin for NostrPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkStuff::new())
            .insert_resource(SendNetMsg::new())
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(Update, handle_net_msg.run_if(in_state(AppState::InGame)));
    }
}

fn setup(mut network_stuff: ResMut<NetworkStuff>, mut send_net_msg: ResMut<SendNetMsg>) {
    if let Some(pubkey) = LOGIN_PUBKEY.lock().expect("Mutex is poisoned").as_ref() {
        send_net_msg.local_ln_address = Some(pubkey.clone());
    }

    let (send_tx, send_rx) = futures::channel::mpsc::channel::<String>(1000);
    let (nostr_msg_tx, mut nostr_msg_rx) = futures::channel::mpsc::channel::<ClientMessage>(1000);

    let mut nostr_msg_tx_clone = nostr_msg_tx.clone();

    let location = web_sys::window().unwrap().location();
    let tag = location.pathname().unwrap().to_string();
    send_net_msg.game_tag = Tag::Hashtag(tag.clone());

    let send_net_msg_clone = send_net_msg.clone();

    network_stuff.read = Some(send_rx);
    send_net_msg.send = Some(nostr_msg_tx);

    spawn_local(async move {
        let nostr_keys = &send_net_msg_clone.nostr_keys;
        let client = Client::new(nostr_keys);

        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://relay.nostrss.re").await.unwrap();

        client.connect().await;

        let client_clone = client.clone();

        spawn_local(async move {
            while let Some(msg) = nostr_msg_rx.next().await {
                info!("sending game event: {:?}", msg);
                client_clone.clone().send_msg(msg).await.unwrap();
            }
        });

        let filter = Filter::new()
            // .kinds(vec![Kind::Replaceable(11111), Kind::Ephemeral(21000)])
            // .until(Timestamp::now())
            .hashtag(tag.clone());

        let events: Vec<NostrEvent> = client
            .get_events_of(vec![filter.clone()], Some(Duration::new(10, 0)))
            .await
            .unwrap();

        if let Some(last_event) = events.last() {
            info!("{:#?}", last_event);
            match serde_json::from_str::<NetworkMessage>(&last_event.content) {
                Ok(NetworkMessage::NewGame) => {
                    info!("current tip: new game, sending join game");
                    let msg = NetworkMessage::JoinGame;
                    let serialized_message = serde_json::to_string(&msg).unwrap();

                    let nostr_msg = ClientMessage::event(
                        EventBuilder::new(
                            Kind::Regular(4444),
                            serialized_message,
                            [Tag::Hashtag(tag.clone())],
                        )
                        .to_event(&nostr_keys)
                        .unwrap(),
                    );

                    match nostr_msg_tx_clone.try_send(nostr_msg) {
                        Ok(()) => {}
                        Err(e) => error!("Error sending join_game message: {}", e),
                    };
                }
                Ok(msg) => {
                    info!("current tip: {:?}", msg);
                    info!("spectate mode");
                }
                Err(error) => {
                    info!("error: {:?}", error);
                    info!("spectate mode");
                }
            }
        } else {
            info!("current tip: no events, sending new game");
            let msg = NetworkMessage::NewGame;
            let serialized_message = serde_json::to_string(&msg).unwrap();

            let nostr_msg = ClientMessage::event(
                EventBuilder::new(
                    Kind::Regular(4444),
                    serialized_message,
                    [Tag::Hashtag(tag.clone())],
                )
                .to_event(&nostr_keys)
                .unwrap(),
            );

            match nostr_msg_tx_clone.try_send(nostr_msg) {
                Ok(()) => {}
                Err(e) => error!("Error sending join_game message: {}", e),
            };
        };

        //we match over the buffer and catch up depending on what mode we are in.

        //we then sub to new messages

        client.subscribe(vec![filter]).await;
        info!("hello?   ");

        // if top is input then set to spectate mode
        // if top is list_game then set to join game mode
        // if top is join_game then set to spectate mode
        // if top is zero then set to list game mode

        client
            .handle_notifications(|notification| async {
                if let RelayPoolNotification::Event { event, relay_url } = notification {
                    // match serde_json::from_str::<NetworkMessage>(&event.content) {
                    //     Ok(NetworkMessage::StartGame(players)) => {
                    //         let new_subscription = Filter::new()
                    //             .authors(vec![players.player1, players.player2])
                    //             .kind(Kind::Regular(4444))
                    //             .hashtag(tag.clone());

                    //         info!("sub to {:?}", players);

                    //         client.subscribe(vec![new_subscription]).await;
                    //     }
                    //     Ok(_) => {}
                    //     Err(_) => {}
                    // }

                    info!("received game event: {:?}", event);

                    match send_tx.clone().try_send(event.content.clone()) {
                        Ok(()) => {}
                        Err(e) => error!("Error sending message: {} CHANNEL FULL???", e),
                    };
                }

                Ok(false)
            })
            .await
            .unwrap();
    });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_net_msg(
    mut network_stuff: ResMut<NetworkStuff>,
    mut send_net_msg: ResMut<SendNetMsg>,
    mut board: ResMut<Board>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    coin_query: Query<Entity, With<CoinMove>>,
    mut replay_button: Query<(&mut ReplayButton, &mut Visibility)>,
) {
    // if send_net_msg.created_game {
    //     send_net_msg.clone().new_game();
    //     send_net_msg.created_game = false;
    // }

    if let Some(ref mut receive_rx) = network_stuff.read {
        while let Ok(Some(message)) = receive_rx.try_next() {
            match serde_json::from_str::<NetworkMessage>(&message) {
                Ok(network_message) => match network_message {
                    // NetworkMessage::NewGame => {
                    //     if send_net_msg.start {
                    //         return;
                    //     }
                    //     send_net_msg.clone().join_game();
                    // }
                    // NetworkMessage::JoinGame => {
                    // if send_net_msg.start {
                    //     return;
                    // }

                    // let players = Players::new(
                    //     send_net_msg.nostr_keys.clone().public_key(),
                    //     other_player,
                    // );
                    // send_net_msg.start = true;
                    // send_net_msg.clone().start_game(players);

                    // send_net_msg.player_type = 1;
                    // }
                    // NetworkMessage::StartGame(players) => {
                    //     if send_net_msg.start {
                    //         return;
                    //     }

                    //     if send_net_msg.nostr_keys.clone().public_key() != players.player1
                    //         && send_net_msg.nostr_keys.clone().public_key() != players.player2
                    //     {
                    //         send_net_msg.player_type = 3;

                    //         send_net_msg.start = true;
                    //         return;
                    //     }

                    //     send_net_msg.clone().start_game(players);

                    //     send_net_msg.start = true;
                    //     send_net_msg.player_type = 2;
                    // }
                    NetworkMessage::Input(new_input) => {
                        let row_pos = board.moves.iter().filter(|m| m.column == new_input).count();
                        if row_pos <= 5 {
                            let player_move =
                                PlayerMove::new(board.player_turn, new_input, row_pos);

                            board.moves.push(player_move);

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
                                            offset_x + new_input as f32 * (COIN_SIZE.x + SPACING),
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
                                            offset_x + new_input as f32 * (COIN_SIZE.x + SPACING),
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
                    NetworkMessage::Replay => {
                        *board = Board::new();
                        for entity in coin_query.iter() {
                            commands.entity(entity).despawn();
                        }
                        for (_, mut visibility) in replay_button.iter_mut() {
                            *visibility = Visibility::Hidden;
                        }
                    }
                    _ => {}
                },

                Err(e) => {
                    info!("Failed to deserialize message: {:?}", e);
                }
            }
        }
    }
}
