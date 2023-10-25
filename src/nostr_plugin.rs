use std::sync::{Arc, Mutex};

use nanoid::nanoid;

use wasm_bindgen::JsValue;
use web_sys::{window, History};

use bevy::prelude::*;
use futures::StreamExt;
use nostr_sdk::{
    serde_json, Client, ClientMessage, EventBuilder, Filter, RelayPoolNotification, Tag, Timestamp,
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    components::{CoinMove, ReplayButton},
    gui_plugin::check_player_connection_and_hide_button,
    messages::NetworkMessage,
    resources::{Board, NetworkStuff, NostrStuff, PlayerMove, SendNetMsg},
    AppState,
};

const COIN_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const COLUMNS: usize = 7;
const ROWS: usize = 7;
const SPACING: f32 = 5.0;

pub struct NostrPlugin;

impl Plugin for NostrPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NostrStuff::new())
            .insert_resource(NetworkStuff::new())
            .insert_resource(SendNetMsg::new())
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(Update, handle_net_msg.run_if(in_state(AppState::InGame)));
    }
}

fn setup(
    nostr_stuff: Res<NostrStuff>,
    mut network_stuff: ResMut<NetworkStuff>,
    mut send_net_msg: ResMut<SendNetMsg>,
) {
    let (send_tx, send_rx) = futures::channel::mpsc::channel::<String>(1000);
    let (input_tx, mut input_rx) = futures::channel::mpsc::channel::<String>(1000);

    let input_tx_clone = input_tx.clone();
    let send_net_msg_clone = send_net_msg.clone();

    network_stuff.read = Some(send_rx);
    send_net_msg.send = Some(input_tx);

    info!("nostr pubkey {:?}", nostr_stuff.local_keys.public_key());
    let nostr_keys = Arc::new(Mutex::new(nostr_stuff.local_keys));

    spawn_local(async move {
        let client = Client::new(&nostr_keys.lock().unwrap());

        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://relay.damus.io").await.unwrap();
        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://relay.snort.social").await.unwrap();
        #[cfg(target_arch = "wasm32")]
        client.add_relay("wss://nostr.lu.ke").await.unwrap();
        client.connect().await;

        let location = web_sys::window().unwrap().location();

        let tag = location.pathname().unwrap().to_string();

        let client_clone = client.clone();
        let tag_clone = tag.clone();
        let nostr_keys_clone: Arc<Mutex<nostr_sdk::Keys>> = nostr_keys.clone();

        spawn_local(async move {
            while let Some(message) = input_rx.next().await {
                info!("sending {:?}", message);

                let input = ClientMessage::new_event(
                    EventBuilder::new_text_note(message, &[Tag::Hashtag(tag_clone.clone())])
                        .to_event(&nostr_keys_clone.lock().unwrap())
                        .unwrap(),
                );
                client_clone.clone().send_msg(input).await.unwrap();
            }
        });

        let subscription = Filter::new().hashtag(tag.clone());

        client.subscribe(vec![subscription]).await;

        if send_net_msg_clone.created_game {
            let lfg_msg = NetworkMessage::Lfg;
            let serialized_message = serde_json::to_string(&lfg_msg).unwrap();

            match input_tx_clone.clone().try_send(serialized_message) {
                Ok(()) => {}
                Err(e) => error!("Error sending message: {} CHANNEL FULL???", e),
            };
        }

        client
            .handle_notifications(|notification| async {
                if let RelayPoolNotification::Event(_url, event) = notification {
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
                    NetworkMessage::Lfg => {
                        if !send_net_msg.start {
                            send_net_msg.local_player = 1;
                            info!("received lfg");
                            let start_game_msg = NetworkMessage::StartGame;
                            let serialized_message =
                                serde_json::to_string(&start_game_msg).unwrap();

                            match send_net_msg
                                .send
                                .as_mut()
                                .unwrap()
                                .try_send(serialized_message)
                            {
                                Ok(()) => {}
                                Err(e) => error!("Error sending message: {} CHANNEL FULL???", e),
                            };
                            check_player_connection_and_hide_button(true);
                            send_net_msg.start = true;
                        } else {
                            let spectate_msg = NetworkMessage::Spectate(board.moves.clone());
                            let serialized_message = serde_json::to_string(&spectate_msg).unwrap();

                            match send_net_msg
                                .send
                                .as_mut()
                                .unwrap()
                                .try_send(serialized_message)
                            {
                                Ok(()) => {}
                                Err(e) => error!("Error sending message: {} CHANNEL FULL???", e),
                            };
                        }
                    }
                    NetworkMessage::StartGame => {
                        if send_net_msg.start {
                            return;
                        }
                        info!("received start game");

                        send_net_msg.local_player = 2;
                        let start_game_msg = NetworkMessage::StartGame;
                        let serialized_message = serde_json::to_string(&start_game_msg).unwrap();

                        match send_net_msg
                            .send
                            .as_mut()
                            .unwrap()
                            .try_send(serialized_message)
                        {
                            Ok(()) => {}
                            Err(e) => error!("Error sending message: {} CHANNEL FULL???", e),
                        };
                        check_player_connection_and_hide_button(true);
                        send_net_msg.start = true;
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
                    NetworkMessage::Spectate(board_moves) => {
                        if send_net_msg.start {
                            return;
                        }
                        send_net_msg.local_player = 3;
                        board.moves = board_moves;
                    }
                    NetworkMessage::Input(new_input) => {
                        info!("received input {:?}", new_input);

                        let row_pos = board.moves.iter().filter(|m| m.column == new_input).count();
                        if row_pos <= 5 {
                            let player_move =
                                PlayerMove::new(board.player_turn, new_input, row_pos);

                            info!("receiv player move {:?}", player_move);

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

                            break;
                        }
                    }
                },
                Err(e) => {
                    info!("Failed to deserialize message: {:?}", e);
                }
            }
        }
    }
}
