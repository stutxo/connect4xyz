use std::collections::HashMap;

use egui::FontFamily::Proportional;
use egui::{FontId, TextStyle::*};
use log::info;

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct Connect4App {
    board_state: Vec<(i32, i32, i32)>,
    player_turn: i32,
    column_state: HashMap<i32, i32>,
    game_start: bool,
}

impl Connect4App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        let mut column_state = HashMap::new();

        for col in 0..7 {
            column_state.insert(col, 5);
        }

        Self {
            board_state: Vec::new(),
            player_turn: 1,
            column_state,
            game_start: false,
        }
    }
}

impl eframe::App for Connect4App {
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let info = frame.info();

        let url = &info.web_info.location.url;
        info!("URL: {}", url);

        if !self.game_start {
            share_link(ctx, self);
        } else {
            game_board(ctx, self);
        };
    }
}

fn share_link(ctx: &egui::Context, game: &mut Connect4App) {
    egui::Window::new("connect4.xyz")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let url = "https://connect4.xyz/324234";
                ui.label(url);
                if ui.button("📋").clicked() {
                    ui.output_mut(|o| o.copied_text = url.to_string());
                    // game.game_start = true;
                };
            });
            ui.spacing();
            ui.spacing();
            ui.label("waiting for player to connect...");
        });
}

fn game_board(ctx: &egui::Context, game: &mut Connect4App) {
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

                                game.board_state
                                    .push((game.player_turn, column, coin_location));

                                game.column_state.insert(column, coin_location - 1);
                            }
                        }
                    }
                });
            }
        });
}
