use std::{
    collections::hash_map::DefaultHasher,
    fmt::Write,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    str::FromStr,
    sync::Arc,
};

use bevy::prelude::*;
use bevy_egui::{
    egui::{FontId, RichText, Ui},
    EguiContexts,
};
use chess::{Board, BoardBuilder, CastleRights};

use crate::game::{board::PieceColor, load::LoadGame};

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
    focus_fen: bool,
    controls: FenPopupDataControls,
    controls_hash: u64,
    invalid_fen: bool,
    fonts: FenPopupFonts,
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
            focus_fen: true,
            invalid_fen: false,
            fonts: FenPopupFonts::default(),
        }
    }
}

impl FenPopupData {
    pub fn reset(&mut self) {
        self.fen.clear();
        self.fen_hash = compute_hash(&self.fen);
        self.focus_fen = true;
        self.controls = default();
        self.controls_hash = compute_hash(&self.controls);
        self.invalid_fen = false;
    }
}

impl FenPopupData {
    fn en_passant_label(&self) -> RichText {
        en_passant_label(self.black_to_move, self.en_passant_target_file, self.fonts.sublabel())
    }

    fn en_passant_selectable(&mut self, ui: &mut Ui, file: chess::File) {
        let label = en_passant_label(self.black_to_move, file, self.fonts.sublabel());
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

                let mut fen_iter = self.fen.split(' ').skip(4);
                let half_move = fen_iter.next().and_then(|s| s.parse().ok());
                let full_move = fen_iter.next().and_then(|s| s.parse().ok());
                if let Some(half_move) = half_move {
                    self.halfmove_clock = half_move;
                }
                if let Some(full_move) = full_move {
                    self.fullmove_count = full_move;
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

                self.fen.truncate(self.fen.len() - 3);
                let half_move = self.halfmove_clock;
                let full_move = self.fullmove_count;
                write!(&mut self.fen, "{half_move} {full_move}")
                    .expect("Write halfmove clock & fullmove count to FEN");

                fen_hash = compute_hash(&self.fen);
            }
        }

        self.fen_hash = fen_hash;
        self.controls_hash = controls_hash;
    }
}

const EN_PASSANT_LABELS: &[&str] = &[
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
];

fn en_passant_label(black_to_move: bool, file: chess::File, font: FontId) -> RichText {
    let rank = if black_to_move { 0b0000 } else { 0b1000 };
    let text = EN_PASSANT_LABELS[file.to_index() | rank];
    RichText::new(text).font(font)
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

#[derive(Hash)]
pub(super) struct FenPopupDataControls {
    black_to_move: bool,
    castle_rights_white_kingside: bool,
    castle_rights_white_queenside: bool,
    castle_rights_black_kingside: bool,
    castle_rights_black_queenside: bool,
    has_en_passant_target: bool,
    en_passant_target_file: chess::File,
    halfmove_clock: u8,
    fullmove_count: u16,
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
            halfmove_clock: 0,
            fullmove_count: 1,
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

struct FenPopupFonts {
    title: FontId,
    label: FontId,
    sublabel: FontId,
    button: FontId,
    editable: FontId,
    drag_value: bevy_egui::egui::TextStyle,
}

impl Default for FenPopupFonts {
    fn default() -> Self {
        Self {
            title: FontId::new(32.0, default()),
            label: FontId::new(18.0, default()),
            sublabel: FontId::new(16.0, default()),
            button: FontId::new(20.0, default()),
            editable: FontId::new(14.5, default()),
            drag_value: bevy_egui::egui::TextStyle::Name(Arc::from("gambit-drag-value")),
        }
    }
}

macro_rules! handle_getters {
    ($( $handle:ident : $ty:ty ; )+) => {
        $( fn $handle(&self) -> $ty { self.$handle.clone() } )+
    }
}

impl FenPopupFonts {
    handle_getters! {
        title: FontId;
        label: FontId;
        sublabel: FontId;
        button: FontId;
        editable: FontId;
        drag_value: bevy_egui::egui::TextStyle;
    }
}

pub(super) fn fen_menu_style(data: Res<FenPopupData>, mut egui_contexts: EguiContexts) {
    egui_contexts.ctx_mut().style_mut(|style| {
        let drag_value_style = FontId::new(16.0, default());
        style.text_styles.entry(data.fonts.drag_value()).or_insert(drag_value_style);
        style.drag_value_text_style = data.fonts.drag_value();

        style.spacing.interact_size = bevy_egui::egui::vec2(16.0, 16.0);
        style.spacing.icon_width = 16.0;
    });
}

enum FenPopupInteraction {
    None,
    Cancel,
    Submit,
}

pub(super) fn fen_menu(
    mut commands: Commands,
    mut egui_contexts: EguiContexts,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut data: ResMut<FenPopupData>,
) {
    match egui::fen_window(egui_contexts.ctx_mut(), &mut data) {
        FenPopupInteraction::Cancel => {
            next_menu_state.set(MenuState::Menu);
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
        lerp, pos2,
        text::{CCursor, CCursorRange},
        text_edit::TextEditOutput,
        vec2, Align, Align2, Button, Color32, ComboBox, Context, DragValue, FontId, Frame, Key,
        Layout, Response, RichText, Sense, Stroke, TextEdit, Ui, Vec2, WidgetInfo, WidgetType,
        Window,
    };
    use chess::ALL_FILES;
    use egui_extras::{Size, StripBuilder};

    use super::{FenPopupData, FenPopupInteraction, DEFAULT_BOARD_FEN};

    pub(super) fn fen_window(ctx: &Context, data: &mut FenPopupData) -> FenPopupInteraction {
        let mut interaction = FenPopupInteraction::None;

        Window::new("FEN Popup")
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                if ui.input(|i| i.key_pressed(Key::Escape)) {
                    interaction = FenPopupInteraction::Cancel;
                    return;
                }

                ui.vertical_centered_justified(|ui| {
                    ui.heading(RichText::new("Load Game").font(data.fonts.title()));
                    ui.separator();

                    ui.set_min_size(vec2(626.0, 0.0));
                    Frame::none().outer_margin(12.0).show(ui, |ui| {
                        const SPACING: f32 = 24.0;
                        ui.horizontal(|ui| fen_control(ui, data, &mut interaction));
                        ui.add_space(SPACING);
                        ui.horizontal(|ui| side_to_move_control(ui, data));
                        ui.add_space(SPACING);
                        ui.horizontal(|ui| en_passant_target_control(ui, data));
                        ui.add_space(SPACING);
                        ui.horizontal(|ui| castle_rights_check_boxes(ui, data));
                        ui.add_space(SPACING);
                        ui.horizontal(|ui| move_tracker_controls(ui, data));
                        ui.add_space(SPACING);
                        ui.horizontal(|ui| action_buttons(ui, data, &mut interaction));
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

    fn fen_control(ui: &mut Ui, data: &mut FenPopupData, interaction: &mut FenPopupInteraction) {
        ui.label(RichText::new("FEN:").font(data.fonts.label()));

        let TextEditOutput { response, mut state, .. } = TextEdit::singleline(&mut data.fen)
            .hint_text(DEFAULT_BOARD_FEN)
            .font(data.fonts.editable())
            .desired_width(f32::INFINITY)
            .show(ui);

        if data.focus_fen {
            data.focus_fen = false;
            ui.memory_mut(move |mem| mem.request_focus(response.id));
        }

        if response.clicked() {
            let text_len = data.fen.len();
            if text_len > 0 {
                state.set_ccursor_range(Some(CCursorRange::two(
                    CCursor::default(),
                    CCursor::new(text_len),
                )));
                state.store(ui.ctx(), response.id);
            }
        }

        if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            *interaction = FenPopupInteraction::Submit;
        }
    }

    fn side_to_move_control(ui: &mut Ui, data: &mut FenPopupData) {
        ui.label(RichText::new("Side to move:").font(data.fonts.label()));
        players_turn_label(ui, data, "White", !data.black_to_move);
        players_turn_toggle(ui, &mut data.black_to_move);
        players_turn_label(ui, data, "Black", data.black_to_move);
    }

    fn players_turn_label(ui: &mut Ui, data: &FenPopupData, text: &str, selected: bool) {
        let text = RichText::new(text).font(data.fonts.sublabel());
        ui.label(if selected { text.strong() } else { text });
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

    fn en_passant_target_control(ui: &mut Ui, data: &mut FenPopupData) {
        let layout = Layout::left_to_right(Align::Center).with_cross_justify(true);
        ui.with_layout(layout, |ui| {
            ui.label(RichText::new("En Passant Target:").font(data.fonts.label()));

            ui.checkbox(&mut data.has_en_passant_target, "");

            ui.add_visible_ui(data.has_en_passant_target, |ui| {
                let text = data.en_passant_label();
                ComboBox::from_label("").selected_text(text).show_ui(ui, |ui| {
                    for file in ALL_FILES {
                        data.en_passant_selectable(ui, file);
                    }
                });
            });
        });
    }

    fn castle_rights_check_boxes(ui: &mut Ui, data: &mut FenPopupData) {
        let label = |text: &str, font: FontId| RichText::new(text).font(font);

        let white_header = label("White Castle Rights:", data.fonts.label());
        let black_header = label("Black Castle Rights:", data.fonts.label());
        let king_label = label("Kingside (O-O)", data.fonts.sublabel());
        let queen_label = label("Queenside (O-O-O)", data.fonts.sublabel());

        let strip_builder = StripBuilder::new(ui).sizes(Size::initial(224.0), 2);
        strip_builder.horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.vertical(|ui| {
                    ui.label(white_header);
                    ui.checkbox(&mut data.castle_rights_white_kingside, king_label.clone());
                    ui.checkbox(&mut data.castle_rights_white_queenside, queen_label.clone());
                });
            });
            strip.cell(|ui| {
                ui.vertical(|ui| {
                    ui.label(black_header);
                    ui.checkbox(&mut data.castle_rights_black_kingside, king_label);
                    ui.checkbox(&mut data.castle_rights_black_queenside, queen_label);
                });
            });
        });
    }

    fn move_tracker_controls(ui: &mut Ui, data: &mut FenPopupData) {
        let strip_builder = StripBuilder::new(ui).sizes(Size::initial(224.0), 2);
        strip_builder.horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Halfmove Clock:").font(data.fonts.label()));
                    ui.add(DragValue::new(&mut data.halfmove_clock).clamp_range(0..=99));
                });
            });
            strip.cell(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Fullmove Count:").font(data.fonts.label()));
                    ui.add(DragValue::new(&mut data.fullmove_count).clamp_range(0..=u16::MAX));
                });
            });
        });
    }

    fn action_buttons(ui: &mut Ui, data: &FenPopupData, interaction: &mut FenPopupInteraction) {
        let size = ui.available_size_before_wrap();
        let layout = Layout::right_to_left(Align::Center);
        ui.allocate_ui_with_layout(size, layout, |ui| {
            if load_button(ui, data.fonts.button()).clicked() {
                *interaction = FenPopupInteraction::Submit;
            }

            ui.add_space(8.0);

            if cancel_button(ui, data.fonts.button()).clicked() {
                *interaction = FenPopupInteraction::Cancel;
            }
        });
    }

    fn cancel_button(ui: &mut Ui, font: FontId) -> Response {
        let text = RichText::new("Cancel").font(font).color(Color32::WHITE);
        ui.add(Button::new(text).fill(Color32::from_rgb(0xba, 0x29, 0x29))) // #ba2929
    }

    fn load_button(ui: &mut Ui, font: FontId) -> Response {
        let text = RichText::new("Load").font(font).color(Color32::WHITE);
        ui.add(Button::new(text).fill(Color32::from_rgb(0x7f, 0xa6, 0x50))) // #7fa650
    }
}
