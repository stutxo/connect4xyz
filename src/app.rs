use std::collections::HashMap;

use std::sync::Arc;
use std::time::Duration;

use egui::mutex::RwLock;
use egui::FontFamily::Proportional;
use egui::{FontId, OpenUrl, TextEdit, TextStyle::*};

use futures::channel::mpsc::{Receiver, Sender};
use futures::lock::Mutex;
use futures::StreamExt;
use log::{info, warn};
use nanoid::nanoid;
use nostr_sdk::prelude::{FromBech32, ToBech32};
use nostr_sdk::secp256k1::XOnlyPublicKey;
use nostr_sdk::{
    serde_json, Client, ClientMessage, EventBuilder, Filter, Keys, RelayPoolNotification, Tag,
    Timestamp,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
//
#[cfg(target_arch = "wasm32")]
use web_sys::{window, History};

// #[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct Connect4App {
    board_state: Vec<(i32, i32, i32)>,
    player_turn: i32,
    column_state: HashMap<i32, i32>,
    game_start: Arc<Mutex<bool>>,
    connect_nostr: bool,
    sub_nostr: bool,
    game_tx: Arc<Mutex<Sender<String>>>,
    game_rx: Arc<Mutex<Receiver<String>>>,
    nostr_keys: Arc<Mutex<Keys>>,
}

impl Connect4App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        let nostr_keys = Keys::generate();

        let mut column_state = HashMap::new();

        for col in 0..7 {
            column_state.insert(col, 5);
        }

        let (game_tx, mut game_rx) = futures::channel::mpsc::channel::<String>(1000);

        Self {
            board_state: Vec::new(),
            player_turn: 1,
            column_state,
            game_start: Arc::new(Mutex::new(false)),
            connect_nostr: false,
            sub_nostr: false,
            game_tx: Arc::new(Mutex::new(game_tx)),
            game_rx: Arc::new(Mutex::new(game_rx)),
            nostr_keys: Arc::new(Mutex::new(nostr_keys)),
        }
    }
}

impl eframe::App for Connect4App {
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.game_start.try_lock().unwrap().clone() {
            #[cfg(target_arch = "wasm32")]
            if !is_game_id_present() {
                let game_id = nanoid!(8);

                let location = web_sys::window().unwrap().location();

                let host = location.host().unwrap();

                let protocol = location.protocol().unwrap();

                let full_url = format!("{protocol}//{host}/{game_id}");

                // Changing the URL without reloading
                let history: History = window().unwrap().history().unwrap();
                history
                    .push_state_with_url(&JsValue::from_str("New Game"), "", Some(&full_url))
                    .expect("pushState failed");
            };

            if !self.connect_nostr {
                self.connect_nostr = true;

                let nostr_keys = self.nostr_keys.clone();

                spawn_local(async move {
                    let location = web_sys::window().unwrap().location();

                    let tag = location.pathname().unwrap().to_string();

                    let relay = "wss://relay.damus.io".to_string();

                    let new_game = serde_json::to_string(&tag).expect("serializing request");

                    let broadcast_peer = ClientMessage::new_event(
                        EventBuilder::new_text_note(new_game, &[Tag::Hashtag(tag.clone())])
                            .to_event(&*nostr_keys.lock().await)
                            .unwrap(),
                    );

                    warn!("LIST GAME {:?}", broadcast_peer);

                    let client = Client::new(&*nostr_keys.lock().await);
                    #[cfg(target_arch = "wasm32")]
                    client.add_relay(relay).await.unwrap();

                    client.connect().await;

                    client.send_msg(broadcast_peer).await.unwrap();
                });
            }
            share_link(ctx, self);
        };
        game_board(ctx, self);
    }
}

fn share_link(ctx: &egui::Context, game: &mut Connect4App) {
    egui::Window::new("connect4.xyz")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label("Share this URL with your friend! ");
            ui.horizontal(|ui| {
                #[cfg(target_arch = "wasm32")]
                let location = window().unwrap().location();
                #[cfg(target_arch = "wasm32")]
                let url = location.href().unwrap();
                #[cfg(target_arch = "wasm32")]
                ui.label(url.clone());
                if ui.button("📋").clicked() {
                    // game.game_start = true;

                    // //must be run with RUSTFLAGS=--cfg=web_sys_unstable_apis for this to work
                    #[cfg(target_arch = "wasm32")]
                    if let Some(clipboard) = window().unwrap().navigator().clipboard() {
                        clipboard.write_text(&url);
                    }
                };
            });
            ui.spacing();
            ui.spacing();

            ui.label("waiting for player to connect...");

            if !game.sub_nostr {
                game.sub_nostr = true;

                let game_tx = game.game_tx.clone();

                let nostr_keys = game.nostr_keys.clone();

                spawn_local(async move {
                    let location = web_sys::window().unwrap().location();

                    let tag = location.pathname().unwrap().to_string();

                    let relay = "wss://relay.damus.io".to_string();

                    let client = Client::new(&*nostr_keys.lock().await);
                    #[cfg(target_arch = "wasm32")]
                    client.add_relay(relay).await.unwrap();

                    client.connect().await;

                    let subscription = Filter::new().since(Timestamp::now()).hashtag(tag.clone());
                    client.subscribe(vec![subscription]).await;

                    client
                        .handle_notifications(|notification| async {
                            if let RelayPoolNotification::Event(_url, event) = notification {
                                if event.pubkey != nostr_keys.lock().await.public_key() {
                                    info!("PLAYER 1 CONNECTED {:?}", event);
                                    let _ = game_tx
                                        .lock()
                                        .await
                                        .try_send("player 1 joined".to_string());

                                    let new_game = serde_json::to_string(&tag.clone()).expect("p1");

                                    let broadcast_peer = ClientMessage::new_event(
                                        EventBuilder::new_text_note(
                                            new_game,
                                            &[Tag::Hashtag(tag.clone())],
                                        )
                                        .to_event(&*nostr_keys.lock().await)
                                        .unwrap(),
                                    );

                                    client.send_msg(broadcast_peer).await.unwrap();
                                }
                            }
                            Ok(false)
                        })
                        .await
                        .unwrap();
                });
            };
        });
}

fn game_board(ctx: &egui::Context, game: &mut Connect4App) {
    let game_rx = game.game_rx.clone();

    let game_start = game.game_start.clone();
    spawn_local(async move {
        while let Some(message) = game_rx.lock().await.next().await {
            let mut start = game_start.lock().await;
            *start = true;
        }
    });

    if *game.game_start.try_lock().unwrap() {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(18.0, Proportional)),
            (Button, FontId::new(30.0, Proportional)),
        ]
        .into();

        ctx.set_style(style);

        egui::Window::new("connect4.xyz")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                let empty_button = "○";
                let p1_button = "⏺";
                let p2_button = "⊗";

                let num_rows = 6;
                let num_columns = 7;

                for row in 0..num_rows {
                    ui.horizontal(|ui| {
                        for column in 0..num_columns {
                            if game.board_state.contains(&(1, column, row)) {
                                let _ = ui.button(p1_button);
                            } else if game.board_state.contains(&(2, column, row)) {
                                let _ = ui.button(p2_button);
                            } else if ui.button(empty_button).clicked() {
                                let coin_location =
                                    *game.column_state.get(&column).unwrap_or(&(&num_rows - 1));
                                if coin_location < num_rows {
                                    game.player_turn = if game.player_turn == 1 { 2 } else { 1 };

                                    game.board_state.push((
                                        game.player_turn,
                                        column,
                                        coin_location,
                                    ));

                                    game.column_state.insert(column, coin_location - 1);
                                }
                            }
                        }
                    });
                }
            });
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
