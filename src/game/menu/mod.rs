use bevy::prelude::*;

use crate::utils::AppPushOrderedStartupStages;

mod game_menu;
mod state;

#[allow(unused_imports)]
pub use self::{game_menu::*, state::*};

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .add_state(MenuState::default())
            // Startup
            .push_ordered_startup_stages([
                (
                    SpawnStage::Phase1,
                    SystemStage::parallel()
                        .with_system(spawn_menu_dim_layer)
                        .with_system(spawn_menu),
                ),
                (SpawnStage::Phase2, SystemStage::single(spawn_menu_elements)),
            ])
            // Systems
            .add_system_set(SystemSet::on_enter(MenuState::Menu).with_system(on_enter_menu_state))
            .add_system_set(SystemSet::on_enter(MenuState::Game).with_system(on_enter_menu_state))
            .add_system(start_game_button);
    }
}

#[derive(Clone, StageLabel)]
enum SpawnStage {
    Phase1,
    Phase2,
}
