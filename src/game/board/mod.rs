use bevy::{prelude::*, ui::UiSystem};
use bevy_startup_tree::{AddStartupTree, startup_tree};

use crate::utils::NoopExts;

pub use self::{
    captures::*, highlight_tile::*, hints::*, icons::*, moves::*, pieces::*, promoter::*,
    selection::*, square::*, state::*, tile::*, ui::*,
};

use super::ui::spawn_ui;

mod captures;
mod highlight_tile;
mod hints;
mod icons;
mod moves;
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
            // Observers
            .add_observer(set_board_on_load_game)
            .add_observer(spawn_pieces_on_load_game)
            .add_observer(hide_end_game_icons_on_load_game)
            // Systems
            .add_startup_tree(startup_tree! {
                spawn_board.after(spawn_ui) => {
                    spawn_tiles => {
                        spawn_highlight_tiles,
                        spawn_hints,
                        spawn_promoters,
                        spawn_end_game_icons,
                    },
                },
            })
            .add_systems(PostUpdate, end_game_icon_size.before(UiSystem::Layout))
            .noop();
    }
}
