use bevy::prelude::*;

use super::board::promotion_ui_sizes;

mod fen_popup;
mod game_menu;
mod state;

#[allow(unused_imports)]
pub use self::{fen_popup::*, game_menu::*, state::*};

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        let menu_state = MenuState::from_world(&mut app.world);

        app
            // Resources
            .init_resource::<FenPopupData>()
            .init_resource::<GameOverTimer>()
            // States
            .add_state(menu_state)
            // Systems
            .add_system_set(
                SystemSet::on_enter(MenuState::FenInput).with_system(on_enter_menu_state),
            )
            .add_system_set(SystemSet::on_enter(MenuState::Menu).with_system(on_enter_menu_state))
            .add_system_set(SystemSet::on_enter(MenuState::Game).with_system(on_enter_menu_state))
            .add_system_set(
                SystemSet::on_enter(MenuState::DoGameOver).with_system(on_enter_menu_state),
            )
            .add_system_set(SystemSet::on_update(MenuState::FenInput).with_system(fen_menu))
            .add_system_set(
                SystemSet::on_update(MenuState::Menu)
                    .with_system(game_menu_buttons)
                    .with_system(game_menu_elements_sizes),
            )
            .add_system(promotion_ui_sizes)
            .add_system_set(SystemSet::on_update(MenuState::DoGameOver).with_system(game_over));
    }
}
