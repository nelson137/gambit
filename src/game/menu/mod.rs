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
        app
            // Resources
            .init_resource::<FenPopupData>()
            .init_resource::<GameOverTimer>()
            // States
            .add_state::<MenuState>()
            // Systems
            .add_startup_system(init_menu_state_from_cli)
            .add_system(menu_size.in_base_set(CoreSet::PostUpdate).before(UiSystem::Flex))
            .add_system(on_enter_menu_state.in_schedule(OnEnter(MenuState::FenInput)))
            .add_system(on_enter_menu_state.in_schedule(OnEnter(MenuState::Menu)))
            .add_system(on_enter_menu_state.in_schedule(OnEnter(MenuState::Game)))
            .add_system(on_enter_menu_state.in_schedule(OnEnter(MenuState::DoGameOver)))
            .add_system(fen_menu.in_set(OnUpdate(MenuState::FenInput)))
            .add_systems(
                (game_menu_buttons, game_menu_elements_sizes).in_set(OnUpdate(MenuState::Menu)),
            )
            .add_system(game_over.in_set(OnUpdate(MenuState::DoGameOver)))
            .add_systems(
                (
                    promotion_ui_sizes,
                    promotion_buttons,
                    promotion_cancel_click_handler.after(promotion_buttons),
                    promotion_event_handler.after(promotion_cancel_click_handler),
                )
                    .distributive_run_if(is_promoting_piece),
            );
    }
}
