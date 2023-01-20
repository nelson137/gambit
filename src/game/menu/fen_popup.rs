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
        pos2, Align2, Context, Key, Rect, Response, Sense, TextEdit, TextStyle, Ui, Vec2, Window,
    };

    use crate::utils::UiSetTextStyleSize;

    use super::{FenPopupData, FenPopupInteraction};

    pub(super) fn fen_window(ctx: &Context, data: &mut FenPopupData) -> FenPopupInteraction {
        let mut interaction = FenPopupInteraction::None;

        Window::new("FEN Popup")
            .resizable(false)
            .title_bar(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|mut ui| {
                    if cancel_button(ui).clicked() {
                        interaction = FenPopupInteraction::Cancel;
                    }

                    ui.set_text_style_size(&TextStyle::Body, 24.0);
                    ui.set_text_style_size(&TextStyle::Heading, 32.0);

                    ui.heading("Load Game");

                    ui.separator();

                    const MARGIN: f32 = 12.0;
                    let contents_rect = ui.available_rect_before_wrap().shrink(MARGIN);
                    ui.allocate_ui_at_rect(contents_rect, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("FEN:");

                            let text_edit = TextEdit::singleline(&mut data.fen);
                            let response = ui.add_sized(ui.available_size(), text_edit);
                            if response.lost_focus() && ui.input().key_pressed(Key::Enter) {
                                interaction = FenPopupInteraction::Submit;
                            }
                        });
                    });
                    ui.add_space(MARGIN);
                });
            });

        interaction
    }

    fn cancel_button(ui: &mut Ui) -> Response {
        let rect = ui.available_rect_before_wrap();

        const PAD: f32 = 4.0;
        let cancel_size = Vec2::splat(ui.spacing().icon_width - PAD);
        let cancel_rect = Rect::from_min_size(
            pos2(rect.right() - cancel_size.x - PAD, rect.top() + PAD),
            cancel_size,
        );

        let cancel_id = ui.auto_id_with("cancel_button");
        let response = ui.interact(cancel_rect, cancel_id, Sense::click());

        let painter = ui.painter();
        let stroke = ui.style().interact(&response).fg_stroke;
        painter.line_segment([cancel_rect.left_top(), cancel_rect.right_bottom()], stroke);
        painter.line_segment([cancel_rect.left_bottom(), cancel_rect.right_top()], stroke);

        response
    }
}
