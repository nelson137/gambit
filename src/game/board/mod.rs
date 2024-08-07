use bevy::{ecs::system::SystemId, prelude::*, ui::UiSystem};
use bevy_startup_tree::{startup_tree, AddStartupTree};

use crate::utils::NoopExts;

pub use self::{
    captures::*, highlight_tile::*, hints::*, icons::*, moves::*, pieces::*, promoter::*,
    selection::*, square::*, state::*, tile::*, ui::*,
};

use super::{ui::spawn_ui, LoadGame};

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
            .observe(set_board_on_load_game)
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
            .add_systems(PostUpdate, end_game_icon_size.before(UiSystem::Layout))
            .noop();
    }
}

fn set_board_on_load_game(
    _trigger: Trigger<LoadGame>,
    mut spawn_pieces_id: Local<Option<SystemId>>,
    mut commands: Commands,
    mut board_state: ResMut<BoardState>,
    q_pieces: Query<Entity, With<PieceMeta>>,
) {
    board_state.clear_pieces();
    q_pieces.iter().for_each(|e| commands.entity(e).despawn_recursive());

    let spawn_pieces_id =
        spawn_pieces_id.get_or_insert_with(|| commands.register_one_shot_system(spawn_pieces));
    commands.run_system(*spawn_pieces_id);
}
