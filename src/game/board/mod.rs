use bevy::{prelude::*, ui::UiSystems};

use crate::utils::NoopExts;

pub use self::{
    captures::*, highlight_tile::*, hints::*, icons::*, moves::*, pieces::*, promoter::*,
    selection::*, square::*, state::*, tile::*, ui::*,
};

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
            .init_resource::<TilesMap>()
            .init_resource::<HighlightTilesMap>()
            .init_resource::<TileHintsMap>()
            .init_resource::<PiecesMap>()
            .init_resource::<GameStatus>()
            .init_resource::<BoardState>()
            // Observers
            .add_observer(set_board_on_load_game)
            .add_observer(spawn_pieces_on_load_game)
            .add_observer(ShowEndGameIcon::handle)
            .add_observer(ShowCheckmateIcons::handle)
            .add_observer(ShowDrawIcons::handle)
            .add_observer(hide_end_game_icons_on_load_game)
            // Systems
            .add_systems(Startup, (promoters.spawn(), end_game_icons.spawn()))
            .add_systems(
                PostStartup,
                (load_tile_map, load_highlight_map, load_hint_map, load_end_game_icons),
            )
            .add_systems(PostUpdate, end_game_icon_size.before(UiSystems::Layout))
            .noop();
    }
}
