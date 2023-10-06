use eframe::{
    egui,
    epaint::ahash::{HashMap, HashMapExt},
};

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Box::new(Connect4Xyz::new(cc))),
    );
}

#[derive(Default)]
struct Connect4Xyz {
    board_state: Vec<(i32, i32, i32)>,
    player_turn: i32,
    column_state: HashMap<i32, i32>,
}

impl Connect4Xyz {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut column_state = HashMap::new();

        for col in 0..7 {
            column_state.insert(col, 5);
        }

        Self {
            board_state: Vec::new(),
            player_turn: 1,
            column_state,
        }
    }
}

impl eframe::App for Connect4Xyz {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let empty_button = "○";
            let p1_button = "⏺";
            let p2_button = "⊗";

            let num_rows = 6;
            let num_columns = 7;

            for row in 0..num_rows {
                ui.horizontal(|ui| {
                    for column in 0..num_columns {
                        if self.board_state.contains(&(1, column, row)) {
                            let _ = ui.button(p1_button);
                        } else if self.board_state.contains(&(2, column, row)) {
                            let _ = ui.button(p2_button);
                        } else if ui.button(empty_button).clicked() {
                            let coin_location =
                                *self.column_state.get(&column).unwrap_or(&(&num_rows - 1));
                            if coin_location < num_rows {
                                self.player_turn = if self.player_turn == 1 { 2 } else { 1 };

                                self.board_state
                                    .push((self.player_turn, column, coin_location));

                                self.column_state.insert(column, coin_location - 1);
                            }
                        }
                    }
                });
            }
        });
    }
}
