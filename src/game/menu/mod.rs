use bevy::{prelude::*, ui::UiSystem};
use bevy_startup_tree::{startup_tree, AddStartupTree};

use super::board::{
    is_promoting_piece, promotion_buttons, promotion_cancel_click_handler, promotion_event_handler,
    promotion_ui_sizes,
};

mod fen_popup;
mod game_menu;
mod state;

#[allow(unused_imports)]
pub use self::{fen_popup::*, game_menu::*, state::*};

pub struct GameMenuUiPlugin;

impl Plugin for GameMenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_tree(startup_tree! {
            spawn_menu_dim_layer => {
                spawn_menu => {
                    spawn_menu_elements => spawn_menu_buttons,
                },
            },
        });
    }
}

pub struct GameMenuLogicPlugin;

impl Plugin for GameMenuLogicPlugin {
    fn build(&self, app: &mut App) {
        let menu_state = MenuState::from_world(&mut app.world);

        app
            // Resources
            .init_resource::<FenPopupData>()
            .init_resource::<GameOverTimer>()
            // States
            .add_state(menu_state)
            // Systems
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new().before(UiSystem::Flex).with_system(menu_size),
            )
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
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(is_promoting_piece)
                    .with_system(promotion_ui_sizes)
                    .with_system(promotion_buttons)
                    .with_system(promotion_cancel_click_handler.after(promotion_buttons))
                    .with_system(promotion_event_handler.after(promotion_cancel_click_handler)),
            )
            .add_system_set(SystemSet::on_update(MenuState::DoGameOver).with_system(game_over));
    }
}
