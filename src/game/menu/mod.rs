use bevy::{prelude::*, ui::UiSystem};
use bevy_startup_tree::{startup_tree, AddStartupTree};

use crate::utils::AppNoop;

use super::board::{
    is_promoting_piece, promotion_buttons, promotion_cancel_click_handler, promotion_event_handler,
    promotion_ui_sizes, start_promotion,
};

use self::fen_popup::*;

pub use self::{game_menu::*, state::*};

mod fen_popup;
mod game_menu;
mod state;

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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct PromoterSystem;

pub struct GameMenuLogicPlugin;

impl Plugin for GameMenuLogicPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Resources
            .init_resource::<FenPopupData>()
            .init_resource::<GameOverTimer>()
            // States
            .add_state::<MenuState>()
            // Systems
            .add_systems(Startup, init_menu_state_from_cli)
            .add_systems(PostUpdate, menu_size.before(UiSystem::Layout))
            .add_systems(OnEnter(MenuState::FenInput), on_enter_menu_state)
            .add_systems(OnEnter(MenuState::Menu), on_enter_menu_state)
            .add_systems(OnEnter(MenuState::Game), on_enter_menu_state)
            .add_systems(OnEnter(MenuState::DoGameOver), on_enter_menu_state)
            .add_systems(Update, fen_menu.run_if(in_state(MenuState::FenInput)))
            .add_systems(
                Update,
                (game_menu_buttons, game_menu_elements_sizes).run_if(in_state(MenuState::Menu)),
            )
            .add_systems(Update, game_over.run_if(in_state(MenuState::DoGameOver)))
            .add_systems(Update, start_promotion)
            .add_systems(Update, promotion_ui_sizes.run_if(is_promoting_piece))
            .add_systems(
                PreUpdate,
                (promotion_buttons, promotion_cancel_click_handler, promotion_event_handler)
                    .chain()
                    .in_set(PromoterSystem)
                    .run_if(is_promoting_piece)
                    .after(UiSystem::Focus),
            )
            .noop();
    }
}

#[cfg(test)]
pub mod test {
    use bevy::prelude::*;

    use crate::utils::AppNoop;

    use super::*;

    pub struct TestMenuStateInGamePlugin;

    impl Plugin for TestMenuStateInGamePlugin {
        fn build(&self, app: &mut App) {
            app.noop()
                .add_state::<MenuState>()
                .add_systems(PreStartup, |mut s: ResMut<NextState<_>>| s.set(MenuState::Game))
                .noop();
        }
    }
}
