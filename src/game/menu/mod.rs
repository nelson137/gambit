use bevy::prelude::*;
use bevy_startup_tree::{startup_tree, AddStartupTree};

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
            // Startup
            .add_startup_tree(startup_tree! {
                spawn_menu_dim_layer,
                spawn_menu => {
                    spawn_menu_elements => spawn_menu_buttons,
                },
            })
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
            .add_system_set(SystemSet::on_update(MenuState::Menu).with_system(game_menu_buttons))
            .add_system_set(SystemSet::on_update(MenuState::DoGameOver).with_system(game_over));
    }
}
