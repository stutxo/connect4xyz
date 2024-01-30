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
use web_sys::window;

use crate::{
    components::{CoinMove, ReplayButton},
    messages::{NetworkMessage, Players},
    resources::{Board, NetworkStuff, PlayerMove, SendNetMsg},
    AppState,
};

const COIN_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const COLUMNS: usize = 7;
const ROWS: usize = 7;
const SPACING: f32 = 5.0;

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
    let window = window().expect("no global `window` exists");
    let local_storage = window
        .local_storage()
        .expect("no local storage")
        .expect("local storage is not available");

    if let Ok(Some(username)) = local_storage.get_item("username") {
        send_net_msg.local_ln_address = Some(username.clone());
        info!("username found in local storage {:?}", username)
    } else {
        info!("no username found in local storage")
    }

    let (send_tx, send_rx) = futures::channel::mpsc::channel::<String>(1000);
    let (nostr_msg_tx, mut nostr_msg_rx) = futures::channel::mpsc::channel::<ClientMessage>(1000);

    let nostr_msg_tx_clone = nostr_msg_tx.clone();

    let location = web_sys::window().unwrap().location();
    let game_id = location.pathname().unwrap().to_string();
    let tag = format!("connect4.xyz game_id = {}", game_id);
    send_net_msg.game_tag = Tag::Hashtag(tag.clone());

    let send_net_msg_clone = send_net_msg.clone();
    let send_net_msg_clone_2 = send_net_msg.clone();

    network_stuff.read = Some(send_rx);
    send_net_msg.send = Some(nostr_msg_tx);

    spawn_local(async move {
        let nostr_keys = &send_net_msg_clone.nostr_keys;
        let client = Client::new(nostr_keys);

        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://relay.nostrss.re").await.unwrap();
        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://nostr.lu.ke").await.unwrap();

        client.connect().await;

        let client_clone = client.clone();

        spawn_local(async move {
            while let Some(msg) = nostr_msg_rx.next().await {
                info!("sent event: {:?}", msg);
                client_clone.clone().send_msg(msg).await.unwrap();
            }
        });

        let filter = Filter::new().kind(Kind::Regular(4444)).hashtag(tag.clone());

        client.subscribe(vec![filter.clone()]).await;

        let mut events: Vec<NostrEvent> = client
            .get_events_of(vec![filter], Some(Duration::new(10, 0)))
            .await
            .unwrap();

        let mut spectator = false;
        events.reverse();

        if let Some(last_event) = events.last() {
            match serde_json::from_str::<NetworkMessage>(&last_event.content) {
                Ok(NetworkMessage::NewGame(player)) => {
                    info!("current tip: {:?}", last_event.content);

                    let players = if send_net_msg_clone_2.local_ln_address.is_none() {
                        Players::new(player, None)
                    } else {
                        Players::new(player, send_net_msg_clone_2.local_ln_address.clone())
                    };

                    let msg = NetworkMessage::JoinGame(players);
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

                    match nostr_msg_tx_clone.clone().try_send(nostr_msg) {
                        Ok(()) => {}
                        Err(e) => {
                            error!("Error sending join_game message: {}", e)
                        }
                    };
                }
                Ok(_) => {
                    //this means you are player 3
                    info!("current tip: {:?}", last_event.content);
                    info!("spectate mode");
                    spectator = true;
                }
                Err(error) => {
                    info!("error: {:?}", error);
                    spectator = true;
                }
            }
        } else {
            info!("current tip: no events");
            let msg = if send_net_msg_clone_2.local_ln_address.is_none() {
                NetworkMessage::NewGame(None)
            } else {
                NetworkMessage::NewGame(send_net_msg_clone_2.local_ln_address.clone())
            };

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

            match nostr_msg_tx_clone.clone().try_send(nostr_msg) {
                Ok(()) => {}
                Err(e) => {
                    error!("Error sending join_game message: {}", e)
                }
            };
        };

        for event in events.drain(..) {
            if event.pubkey == nostr_keys.public_key() {
                info!("skipping own event");
                continue;
            }
            if event.content.contains("NewGame") && spectator
                || event.content.contains("JoinGame") && spectator
            {
                info!("skipping event");
                continue;
            }
            if event.content.contains("NewGame") {
                //sub to messages only from the other players pubkey to stop spam
                let new_subscription = Filter::new()
                    .author(event.pubkey)
                    .kind(Kind::Regular(4444))
                    .since(Timestamp::now())
                    .hashtag(tag.clone());

                info!("sub to player 1 events only {:?}", event.pubkey);

                client.subscribe(vec![new_subscription]).await;
            }

            info!("processing stored event: {:?}", event);

            match send_tx.clone().try_send(event.content.clone()) {
                Ok(()) => {}
                Err(e) => {
                    error!("Error sending message: {} CHANNEL FULL???", e)
                }
            };
        }

        client
            .handle_notifications(|notification| async {
                if let RelayPoolNotification::Event { relay_url, event } = notification {
                    if event.pubkey != nostr_keys.public_key() {
                        info!("received event: {:?}", event);
                        if event.content.contains("JoinGame") && !spectator {
                            let new_subscription = Filter::new()
                                .author(event.pubkey)
                                .kind(Kind::Regular(4444))
                                .since(Timestamp::now())
                                .hashtag(tag.clone());

                            info!("sub to player 2 events only {:?}", event.pubkey);

                            client.subscribe(vec![new_subscription]).await;
                        }

                        match send_tx.clone().try_send(event.content.clone()) {
                            Ok(()) => {}
                            Err(e) => {
                                error!("Error sending message: {} CHANNEL FULL???", e)
                            }
                        };
                    }
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
    if let Some(ref mut receive_rx) = network_stuff.read {
        while let Ok(Some(message)) = receive_rx.try_next() {
            match serde_json::from_str::<NetworkMessage>(&message) {
                Ok(network_message) => match network_message {
                    NetworkMessage::Input(new_input) => {
                        if send_net_msg.player_type == 0 {
                            send_net_msg.player_type = 3;
                        }
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
                    NetworkMessage::JoinGame(game_info) => {
                        if send_net_msg.start {
                            continue;
                        }

                        send_net_msg.p2_ln_address = game_info.player2;
                        //recevied message from p2 so you must be p1
                        send_net_msg.player_type = 1;
                        info!("player type: 1");
                        send_net_msg.start = true;
                    }
                    NetworkMessage::NewGame(player1) => {
                        if send_net_msg.start {
                            continue;
                        }

                        send_net_msg.p2_ln_address = player1;
                        //recevied message from p1 so you must be p2
                        send_net_msg.player_type = 2;
                        info!("player type: 2");
                        send_net_msg.start = true;
                    }
                },

                Err(e) => {
                    info!("Failed to deserialize message: {:?}", e);
                }
            }
        }
    }
}
