use bevy_egui::egui::{
    lerp, pos2,
    text::{CCursor, CCursorRange},
    text_edit::TextEditOutput,
    vec2, Align, Align2, Button, Color32, ComboBox, Context, DragValue, FontId, Frame, Key, Layout,
    Response, RichText, Sense, Stroke, TextEdit, Ui, Vec2, WidgetInfo, WidgetType, Window,
};
use chess::ALL_FILES;
use egui_extras::{Size, StripBuilder};

use super::{state::PopupState, FenPopupInteraction, DEFAULT_BOARD_FEN};

pub(super) fn fen_window(ctx: &Context, state: &mut PopupState) -> FenPopupInteraction {
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
                ui.heading(RichText::new("Load Game").font(state.fonts.title()));
                ui.separator();

                ui.set_min_size(vec2(626.0, 0.0));
                Frame::none().outer_margin(12.0).show(ui, |ui| {
                    const SPACING: f32 = 24.0;
                    ui.horizontal(|ui| state.fen_control(ui, &mut interaction));
                    ui.add_space(SPACING);
                    ui.horizontal(|ui| state.side_to_move_section(ui));
                    ui.add_space(SPACING);
                    ui.horizontal(|ui| state.en_passant_target_section(ui));
                    ui.add_space(SPACING);
                    ui.horizontal(|ui| state.castle_rights_section(ui));
                    ui.add_space(SPACING);
                    ui.horizontal(|ui| state.move_trackers_section(ui));
                    ui.add_space(SPACING);
                    ui.horizontal(|ui| state.action_buttons_section(ui, &mut interaction));
                });
            });
        });

    state.sync();

    match interaction {
        FenPopupInteraction::Submit if state.fen.is_empty() => {
            state.fen.replace_range(.., DEFAULT_BOARD_FEN)
        }
        _ => {}
    }

    interaction
}

const EN_PASSANT_LABELS: &[&str] = &[
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
];

fn en_passant_label(black_to_move: bool, file: chess::File, font: FontId) -> RichText {
    let rank = if black_to_move { 0b0000 } else { 0b1000 };
    let text = EN_PASSANT_LABELS[file.to_index() | rank];
    RichText::new(text).font(font)
}

impl PopupState {
    fn fen_control(&mut self, ui: &mut Ui, interaction: &mut FenPopupInteraction) {
        ui.label(RichText::new("FEN:").font(self.fonts.label()));

        let font = self.fonts.editable();
        let TextEditOutput { response, mut state, .. } = TextEdit::singleline(&mut self.fen)
            .hint_text(DEFAULT_BOARD_FEN)
            .font(font)
            .desired_width(f32::INFINITY)
            .show(ui);

        if self.focus_fen {
            self.focus_fen = false;
            ui.memory_mut(move |mem| mem.request_focus(response.id));
        }

        if response.clicked() {
            let text_len = self.fen.len();
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

    fn side_to_move_section(&mut self, ui: &mut Ui) {
        ui.label(RichText::new("Side to move:").font(self.fonts.label()));
        players_turn_label(ui, "White", self.fonts.sublabel(), !self.black_to_move);
        players_turn_toggle(ui, &mut self.black_to_move);
        players_turn_label(ui, "Black", self.fonts.sublabel(), self.black_to_move);
    }

    fn en_passant_target_section(&mut self, ui: &mut Ui) {
        let layout = Layout::left_to_right(Align::Center).with_cross_justify(true);
        ui.with_layout(layout, |ui| {
            ui.label(RichText::new("En Passant Target:").font(self.fonts.label()));

            ui.checkbox(&mut self.has_en_passant_target, "");

            ui.add_visible_ui(self.has_en_passant_target, |ui| {
                let text = en_passant_label(
                    self.black_to_move,
                    self.en_passant_target_file,
                    self.fonts.sublabel(),
                );
                ComboBox::from_label("").selected_text(text).show_ui(ui, |ui| {
                    for file in ALL_FILES {
                        let label =
                            en_passant_label(self.black_to_move, file, self.fonts.sublabel());
                        ui.selectable_value(&mut self.en_passant_target_file, file, label);
                    }
                });
            });
        });
    }

    fn castle_rights_section(&mut self, ui: &mut Ui) {
        let label = |text: &str, font: FontId| RichText::new(text).font(font);

        let white_header = label("White Castle Rights:", self.fonts.label());
        let black_header = label("Black Castle Rights:", self.fonts.label());
        let king_label = label("Kingside (O-O)", self.fonts.sublabel());
        let queen_label = label("Queenside (O-O-O)", self.fonts.sublabel());

        let strip_builder = StripBuilder::new(ui).sizes(Size::initial(224.0), 2);
        strip_builder.horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.vertical(|ui| {
                    ui.label(white_header);
                    ui.checkbox(&mut self.castle_rights_white_kingside, king_label.clone());
                    ui.checkbox(&mut self.castle_rights_white_queenside, queen_label.clone());
                });
            });
            strip.cell(|ui| {
                ui.vertical(|ui| {
                    ui.label(black_header);
                    ui.checkbox(&mut self.castle_rights_black_kingside, king_label);
                    ui.checkbox(&mut self.castle_rights_black_queenside, queen_label);
                });
            });
        });
    }

    fn move_trackers_section(&mut self, ui: &mut Ui) {
        let strip_builder = StripBuilder::new(ui).sizes(Size::initial(224.0), 2);
        strip_builder.horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Halfmove Clock:").font(self.fonts.label()));
                    ui.add(DragValue::new(&mut self.halfmove_clock).clamp_range(0..=99));
                });
            });
            strip.cell(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Fullmove Count:").font(self.fonts.label()));
                    ui.add(DragValue::new(&mut self.fullmove_count).clamp_range(0..=u16::MAX));
                });
            });
        });
    }

    fn action_buttons_section(&self, ui: &mut Ui, interaction: &mut FenPopupInteraction) {
        let size = ui.available_size_before_wrap();
        let layout = Layout::right_to_left(Align::Center);
        ui.allocate_ui_with_layout(size, layout, |ui| {
            if load_button(ui, self.fonts.button()).clicked() {
                *interaction = FenPopupInteraction::Submit;
            }

            ui.add_space(8.0);

            if cancel_button(ui, self.fonts.button()).clicked() {
                *interaction = FenPopupInteraction::Cancel;
            }
        });
    }
}

fn players_turn_label(ui: &mut Ui, text: &str, font: FontId, selected: bool) {
    let text = RichText::new(text).font(font);
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

fn cancel_button(ui: &mut Ui, font: FontId) -> Response {
    let text = RichText::new("Cancel").font(font).color(Color32::WHITE);
    ui.add(Button::new(text).fill(Color32::from_rgb(0xba, 0x29, 0x29))) // #ba2929
}

fn load_button(ui: &mut Ui, font: FontId) -> Response {
    let text = RichText::new("Load").font(font).color(Color32::WHITE);
    ui.add(Button::new(text).fill(Color32::from_rgb(0x7f, 0xa6, 0x50))) // #7fa650
}
