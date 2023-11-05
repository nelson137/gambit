use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use bevy::prelude::*;
use bevy_egui::{egui::Ui, EguiContext};
use chess::{Board, BoardBuilder, CastleRights};

use crate::{
    game::{board::PieceColor, load::LoadGame},
    utils::StateExts,
};

use super::MenuState;

const DEFAULT_BOARD_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn compute_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[derive(Resource)]
pub(super) struct FenPopupData {
    fen: String,
    fen_hash: u64,
    controls: FenPopupDataControls,
    controls_hash: u64,
    invalid_fen: bool,
}

impl Default for FenPopupData {
    fn default() -> Self {
        let controls = FenPopupDataControls::default();
        let fen = String::new();
        Self {
            controls_hash: compute_hash(&controls),
            controls,
            fen_hash: compute_hash(&fen),
            fen,
            invalid_fen: false,
        }
    }
}

const EN_PASSANT_LABELS: &[&str] = &[
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
];

fn en_passant_label(black_to_move: bool, file: chess::File) -> &'static str {
    let rank = if black_to_move { 0b0000 } else { 0b1000 };
    EN_PASSANT_LABELS[file.to_index() | rank]
}

impl FenPopupData {
    fn en_passant_label(&self) -> &'static str {
        en_passant_label(self.black_to_move, self.en_passant_target_file)
    }

    fn en_passant_selectable(&mut self, ui: &mut Ui, file: chess::File) {
        let label = en_passant_label(self.black_to_move, file);
        ui.selectable_value(&mut self.en_passant_target_file, file, label);
    }

    fn sync(&mut self) {
        let mut fen_hash = compute_hash(&self.fen);
        let mut controls_hash = compute_hash(&self.controls);

        if fen_hash != self.fen_hash {
            // Update the controls

            let board = Board::from_str(&self.fen);
            self.invalid_fen = board.is_err();

            if let Ok(board) = board {
                self.black_to_move = board.side_to_move() == PieceColor::BLACK;

                let white_castle_rights = board.castle_rights(chess::Color::White);
                self.castle_rights_white_kingside = white_castle_rights.has_kingside();
                self.castle_rights_white_queenside = white_castle_rights.has_queenside();

                let black_castle_rights = board.castle_rights(chess::Color::Black);
                self.castle_rights_black_kingside = black_castle_rights.has_kingside();
                self.castle_rights_black_queenside = black_castle_rights.has_queenside();

                match board.en_passant() {
                    Some(square) => {
                        self.has_en_passant_target = true;
                        self.en_passant_target_file = square.get_file();
                    }
                    None => self.has_en_passant_target = false,
                }

                controls_hash = compute_hash(&self.controls);
            }
        } else if controls_hash != self.controls_hash {
            // Update the FEN

            if self.fen.is_empty() {
                self.fen.replace_range(.., DEFAULT_BOARD_FEN);
            }

            let board = BoardBuilder::from_str(&self.fen);
            self.invalid_fen = board.is_err();
            if let Ok(mut board) = board {
                let fen_black_to_move = board.get_side_to_move() == chess::Color::Black;
                if fen_black_to_move != self.black_to_move {
                    board.side_to_move(!board.get_side_to_move());
                }

                board.castle_rights(chess::Color::White, self.white_castle_rights());
                board.castle_rights(chess::Color::Black, self.black_castle_rights());

                let en_passant = self.has_en_passant_target.then_some(self.en_passant_target_file);
                board.en_passant(en_passant);

                self.fen = board.to_string();
                fen_hash = compute_hash(&self.fen);
            }
        }

        self.fen_hash = fen_hash;
        self.controls_hash = controls_hash;
    }
}

impl Deref for FenPopupData {
    type Target = FenPopupDataControls;

    fn deref(&self) -> &Self::Target {
        &self.controls
    }
}

impl DerefMut for FenPopupData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.controls
    }
}

impl FenPopupData {
    pub fn reset(&mut self) {
        self.fen.clear();
    }
}

#[derive(Hash)]
pub(super) struct FenPopupDataControls {
    black_to_move: bool,
    castle_rights_white_kingside: bool,
    castle_rights_white_queenside: bool,
    castle_rights_black_kingside: bool,
    castle_rights_black_queenside: bool,
    has_en_passant_target: bool,
    en_passant_target_file: chess::File,
}

impl Default for FenPopupDataControls {
    fn default() -> Self {
        Self {
            black_to_move: false,
            castle_rights_white_kingside: true,
            castle_rights_white_queenside: true,
            castle_rights_black_kingside: true,
            castle_rights_black_queenside: true,
            has_en_passant_target: false,
            en_passant_target_file: chess::File::A,
        }
    }
}

impl FenPopupDataControls {
    fn white_castle_rights(&self) -> CastleRights {
        castle_rights(self.castle_rights_white_kingside, self.castle_rights_white_queenside)
    }

    fn black_castle_rights(&self) -> CastleRights {
        castle_rights(self.castle_rights_black_kingside, self.castle_rights_black_queenside)
    }
}

fn castle_rights(kingside: bool, queenside: bool) -> CastleRights {
    match (kingside, queenside) {
        (true, true) => CastleRights::Both,
        (true, false) => CastleRights::KingSide,
        (false, true) => CastleRights::QueenSide,
        (false, false) => CastleRights::NoRights,
    }
}

enum FenPopupInteraction {
    None,
    Cancel,
    Submit,
}

pub(super) fn fen_menu(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    mut menu_state: ResMut<State<MenuState>>,
    mut data: ResMut<FenPopupData>,
) {
    match egui::fen_window(egui_context.ctx_mut(), &mut data) {
        FenPopupInteraction::Cancel => {
            menu_state.transition(MenuState::Menu);
        }
        FenPopupInteraction::Submit => match chess::Board::from_str(&data.fen) {
            Ok(board) => commands.add(LoadGame(board)),
            Err(err) => error!("{err}"),
        },
        FenPopupInteraction::None => (),
    }
}

mod egui {
    use bevy_egui::egui::{
        lerp, pos2, vec2, Align, Align2, Button, Color32, ComboBox, Context, Frame, Key, Layout,
        Response, RichText, Sense, Stroke, TextBuffer, TextEdit, TextStyle, Ui, Vec2, WidgetInfo,
        WidgetType, Window,
    };
    use egui_extras::{Size, StripBuilder};

    use crate::utils::UiSetTextStyleSize;

    use super::{FenPopupData, FenPopupInteraction, DEFAULT_BOARD_FEN};

    /// `#7fa650`
    const LOAD_BUTTON_COLOR: Color32 = Color32::from_rgb(0x7f, 0xa6, 0x50);

    const LOAD_BUTTON_TEXT_COLOR: Color32 = Color32::WHITE;

    const CANCEL_BUTTON_TEXT_COLOR: Color32 = Color32::WHITE;

    /// `#ba2929`
    const CANCEL_BUTTON_COLOR: Color32 = Color32::from_rgb(0xba, 0x29, 0x29);

    pub(super) fn fen_window(ctx: &Context, data: &mut FenPopupData) -> FenPopupInteraction {
        let mut interaction = FenPopupInteraction::None;

        Window::new("FEN Popup")
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                if ui.input().key_pressed(Key::Escape) {
                    interaction = FenPopupInteraction::Cancel;
                    return;
                }

                ui.vertical_centered_justified(|mut ui| {
                    ui.set_text_style_size(&TextStyle::Body, 20.0);
                    ui.set_text_style_size(&TextStyle::Button, 24.0);
                    ui.set_text_style_size(&TextStyle::Heading, 32.0);

                    ui.heading("Load Game");
                    ui.separator();

                    ui.set_min_size(vec2(768.0, 0.0));
                    Frame::none().outer_margin(12.0).show(ui, |ui| {
                        const SPACING: f32 = 24.0;

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("FEN:").underline());

                            let response = fen_text_edit(ui, &mut data.fen);
                            if response.lost_focus() && ui.input().key_pressed(Key::Enter) {
                                interaction = FenPopupInteraction::Submit;
                            }
                        });

                        ui.add_space(SPACING);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Side to move:").underline());
                            players_turn_label_white(ui, data.black_to_move);
                            players_turn_toggle(ui, &mut data.black_to_move);
                            players_turn_label_black(ui, data.black_to_move);
                        });

                        ui.add_space(SPACING);

                        ui.horizontal(|ui| {
                            castle_rights_check_boxes(ui, data);
                        });

                        ui.add_space(SPACING);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("En Passant Target:").underline());
                            en_passant_target(ui, data);
                        });

                        ui.add_space(SPACING);

                        ui.horizontal(|ui| {
                            if cancel_button(ui).clicked() {
                                interaction = FenPopupInteraction::Cancel;
                            }

                            let size = ui.available_rect_before_wrap().size();
                            let layout = Layout::right_to_left(Align::Center);
                            ui.allocate_ui_with_layout(size, layout, |ui| {
                                if load_button(ui).clicked() {
                                    interaction = FenPopupInteraction::Submit;
                                }
                            });
                        });
                    });
                });
            });

        data.sync();

        match interaction {
            FenPopupInteraction::Submit if data.fen.is_empty() => {
                data.fen.replace_range(.., DEFAULT_BOARD_FEN)
            }
            _ => {}
        }

        interaction
    }

    fn fen_text_edit(ui: &mut Ui, model: &mut dyn TextBuffer) -> Response {
        ui.add(
            TextEdit::singleline(model).hint_text(DEFAULT_BOARD_FEN).desired_width(f32::INFINITY),
        )
    }

    fn players_turn_label_white(ui: &mut Ui, black_to_move: bool) {
        let text = RichText::new("White");
        ui.label(if black_to_move { text } else { text.strong() });
    }

    fn players_turn_label_black(ui: &mut Ui, black_to_move: bool) {
        let text = RichText::new("Black");
        ui.label(if black_to_move { text.strong() } else { text });
    }

    // Taken from: https://github.com/emilk/egui/blob/041f2e64bac778c9095fbf4316dc1f7c7bceb670/crates/egui_demo_lib/src/demo/toggle_switch.rs
    fn players_turn_toggle(ui: &mut Ui, value: &mut bool) -> Response {
        let desired_size = ui.spacing().interact_size.y * vec2(2.5, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        if response.clicked() {
            *value = !*value;
            response.mark_changed();
        }
        response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, *value, ""));

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact_selectable(&response, *value);

            let fill_color = if *value { Color32::DARK_GRAY } else { Color32::LIGHT_GRAY };
            let stroke_color = if *value { Color32::GRAY } else { Color32::BLACK };
            let stroke = Stroke::new(visuals.fg_stroke.width, stroke_color);

            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui.painter().rect(rect, radius, fill_color, stroke);
            let how_on = ui.ctx().animate_bool(response.id, *value);
            let circle_x = lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = pos2(circle_x, rect.center().y);
            ui.painter().circle(center, 0.75 * radius, fill_color, stroke);
        }

        response
    }

    fn castle_rights_check_boxes(ui: &mut Ui, data: &mut FenPopupData) {
        let strip_builder = StripBuilder::new(ui).sizes(Size::initial(256.0), 2);
        strip_builder.horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("White Castle Rights").underline());
                    ui.checkbox(&mut data.castle_rights_white_kingside, "Kingside (O-O)");
                    ui.checkbox(&mut data.castle_rights_white_queenside, "Queenside (O-O-O)");
                });
            });
            strip.cell(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Black Castle Rights").underline());
                    ui.checkbox(&mut data.castle_rights_black_kingside, "Kingside (O-O)");
                    ui.checkbox(&mut data.castle_rights_black_queenside, "Queenside (O-O-O)");
                });
            });
        });
    }

    fn en_passant_target(ui: &mut Ui, data: &mut FenPopupData) {
        ui.checkbox(&mut data.has_en_passant_target, "");
        ui.add_enabled_ui(data.has_en_passant_target, |ui| {
            let text = data.en_passant_label();
            let output = ComboBox::from_label("").selected_text(text).show_ui(ui, |ui| {
                data.en_passant_selectable(ui, chess::File::A);
                data.en_passant_selectable(ui, chess::File::B);
                data.en_passant_selectable(ui, chess::File::C);
                data.en_passant_selectable(ui, chess::File::D);
                data.en_passant_selectable(ui, chess::File::E);
                data.en_passant_selectable(ui, chess::File::F);
                data.en_passant_selectable(ui, chess::File::G);
                data.en_passant_selectable(ui, chess::File::H);
            });
            output.response
        });
    }

    fn cancel_button(ui: &mut Ui) -> Response {
        let text = RichText::new("Cancel").color(CANCEL_BUTTON_TEXT_COLOR);
        ui.add(Button::new(text).fill(CANCEL_BUTTON_COLOR))
    }

    fn load_button(ui: &mut Ui) -> Response {
        let text = RichText::new("Load").color(LOAD_BUTTON_TEXT_COLOR);
        ui.add(Button::new(text).fill(LOAD_BUTTON_COLOR))
    }
}
