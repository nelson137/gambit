use bevy::prelude::*;
use bevy_egui::EguiContext;

use crate::utils::StateExts;

use super::MenuState;

#[derive(Resource, Default)]
pub(super) struct FenPopupData {
    fen: String,
}

impl FenPopupData {
    pub fn reset(&mut self) {
        self.fen.clear();
    }
}

enum FenPopupInteraction {
    None,
    Cancel,
    Submit,
}

pub(super) fn fen_menu(
    mut egui_context: ResMut<EguiContext>,
    mut menu_state: ResMut<State<MenuState>>,
    mut data: ResMut<FenPopupData>,
) {
    match egui::fen_window(egui_context.ctx_mut(), &mut data) {
        FenPopupInteraction::Cancel => {
            menu_state.transition_pop();
        }
        FenPopupInteraction::Submit => {
            info!(fen = ?data.fen);
            menu_state.transition_pop();
        }
        FenPopupInteraction::None => (),
    }
}

mod egui {
    use bevy_egui::egui::{
        Align, Align2, Button, Color32, Context, Key, Layout, Response, RichText, TextBuffer,
        TextEdit, TextStyle, Ui, Vec2, Window,
    };

    use crate::utils::UiSetTextStyleSize;

    use super::{FenPopupData, FenPopupInteraction};

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
                ui.vertical_centered_justified(|mut ui| {
                    ui.set_text_style_size(&TextStyle::Body, 24.0);
                    ui.set_text_style_size(&TextStyle::Button, 24.0);
                    ui.set_text_style_size(&TextStyle::Heading, 32.0);

                    ui.heading("Load Game");

                    ui.separator();

                    const MARGIN: f32 = 12.0;

                    let contents_rect = ui.available_rect_before_wrap().shrink(MARGIN);
                    ui.allocate_ui_at_rect(contents_rect, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("FEN:");

                            let response = fen_text_edit(ui, &mut data.fen);
                            if response.lost_focus() && ui.input().key_pressed(Key::Enter) {
                                interaction = FenPopupInteraction::Submit;
                            }
                        });

                        ui.add_space(MARGIN);

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
                    ui.add_space(MARGIN);
                });
            });

        interaction
    }

    fn fen_text_edit(ui: &mut Ui, model: &mut dyn TextBuffer) -> Response {
        ui.add_sized(ui.available_size(), TextEdit::singleline(model))
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
