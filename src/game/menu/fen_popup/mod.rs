use std::str::FromStr;

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::game::LoadGame;

use super::MenuState;

pub use self::state::PopupState;

mod state;
mod view;

enum FenPopupInteraction {
    None,
    Cancel,
    Submit,
}

pub(super) fn fen_menu(
    mut commands: Commands,
    mut egui_contexts: EguiContexts,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut state: ResMut<PopupState>,
) {
    let ctx = egui_contexts.ctx_mut();

    if !state.fonts.is_initialized() {
        state.fonts.initialize(ctx)
    }

    match view::fen_window(ctx, &mut state) {
        FenPopupInteraction::Cancel => {
            next_menu_state.set(MenuState::Menu);
        }
        FenPopupInteraction::Submit => match chess::Board::from_str(&state.fen) {
            Ok(board) => commands.trigger(LoadGame::in_game(board)),
            Err(err) => error!("{err}"),
        },
        FenPopupInteraction::None => {}
    }
}
