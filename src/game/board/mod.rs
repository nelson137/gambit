use bevy::{prelude::*, ui::UiSystem};
use bevy_startup_tree::{startup_tree, AddStartupTree};

use crate::utils::AppNoop;

pub use self::{
    highlight_tile::*, hints::*, icons::*, pieces::*, promoter::*, selection::*, square::*,
    state::*, tile::*, ui::*,
};

use super::ui::spawn_ui;

mod highlight_tile;
mod hints;
mod icons;
mod pieces;
mod promoter;
mod selection;
mod square;
mod state;
mod tile;
mod ui;

#[derive(Debug)]
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Resources
            .init_resource::<BoardState>()
            // Systems
            .add_startup_tree(startup_tree! {
                spawn_board.after(spawn_ui) => {
                    spawn_tiles => {
                        spawn_highlight_tiles,
                        spawn_hints,
                        spawn_pieces,
                        spawn_promoters,
                        spawn_end_game_icons,
                    },
                },
            })
            .add_systems(PostUpdate, (board_size, end_game_icon_size).before(UiSystem::Layout))
            .noop();
    }
}
