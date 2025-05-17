use bevy::{prelude::*, ui::UiSystem};
use bevy_startup_tree::{startup_tree, AddStartupTree};

use crate::utils::NoopExts;

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

pub struct GameMenuLogicPlugin;

impl Plugin for GameMenuLogicPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Resources
            .init_resource::<PopupState>()
            .init_resource::<GameOverTimer>()
            // States
            .init_state::<MenuState>()
            // Observers
            .add_observer(set_state_to_game_on_load)
            // Systems
            .add_systems(Startup, init_menu_state_from_cli)
            .add_systems(PostUpdate, menu_size.before(UiSystem::Layout))
            .add_systems(OnEnter(MenuState::FenInput), on_enter_menu_state_fen_input)
            .add_systems(OnEnter(MenuState::Menu), on_enter_menu_state_menu)
            .add_systems(OnEnter(MenuState::Game), on_enter_menu_state_game)
            .add_systems(OnEnter(MenuState::DoGameOver), on_enter_menu_state_do_game_over)
            .add_systems(Update, fen_menu.run_if(in_state(MenuState::FenInput)))
            .add_systems(Update, game_menu_elements_sizes.run_if(in_state(MenuState::Menu)))
            .add_systems(Update, game_over.run_if(in_state(MenuState::DoGameOver)))
            .noop();
    }
}

#[cfg(test)]
pub mod test {
    use bevy::prelude::*;

    use crate::utils::NoopExts;

    use super::*;

    pub struct TestMenuStateInGamePlugin;

    impl Plugin for TestMenuStateInGamePlugin {
        fn build(&self, app: &mut App) {
            app.noop()
                .init_state::<MenuState>()
                .add_systems(PreStartup, |mut s: ResMut<NextState<_>>| s.set(MenuState::Game))
                .noop();
        }
    }
}
