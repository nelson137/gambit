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
    use bevy_egui::egui::{pos2, Context, Key, Rect, Response, Sense, Ui, Vec2, Window};

    use super::{FenPopupData, FenPopupInteraction};

    pub(super) fn fen_window(ctx: &Context, data: &mut FenPopupData) -> FenPopupInteraction {
        let mut interaction = FenPopupInteraction::None;

        Window::new("FEN Popup").resizable(false).title_bar(false).show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                if cancel_button(ui).clicked() {
                    interaction = FenPopupInteraction::Cancel;
                }

                ui.heading("Input FEN");

                let response = ui.text_edit_singleline(&mut data.fen);
                if response.lost_focus() && ui.input().key_pressed(Key::Enter) {
                    interaction = FenPopupInteraction::Submit;
                }
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
