use std::collections::{HashMap, hash_map::Entry};

use bevy::prelude::*;

use crate::{debug_name_f, game::consts::Z_HIGHLIGHT_TILE};

use super::{Square, TilesMap};

#[derive(Component)]
#[require(Visibility::Hidden, BackgroundColor(COLOR_HIGHLIGHT), GlobalZIndex(Z_HIGHLIGHT_TILE))]
pub struct HighlightTile;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::srgba(1.0, 1.0, 0.0, 0.5);

pub fn spawn_highlight_tiles(
    mut commands: Commands,
    tiles_map: Res<TilesMap>,
    mut highlights_map: ResMut<HighlightTilesMap>,
) {
    let top = Val::Px(0.0);
    let left = Val::Px(0.0);

    for square in chess::ALL_SQUARES {
        let square = Square::new(square);

        let hl_tile_entity = commands
            .spawn((
                debug_name_f!("Highlight Tile ({square})"),
                HighlightTile,
                square,
                Node {
                    position_type: PositionType::Absolute,
                    top,
                    left,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ))
            .id();

        highlights_map.set(square, hl_tile_entity);
        commands.entity(tiles_map.at(square)).add_child(hl_tile_entity);
    }
}

#[derive(Resource)]
pub struct HighlightTilesMap(HashMap<Square, Entity>);

impl Default for HighlightTilesMap {
    fn default() -> Self {
        Self(HashMap::with_capacity(64))
    }
}

impl HighlightTilesMap {
    fn set(&mut self, square: Square, entity: Entity) {
        match self.0.entry(square) {
            Entry::Occupied(_) => panic!("highlight already in the state at {square}"),
            Entry::Vacant(e) => e.insert(entity),
        };
    }

    pub fn at(&self, square: Square) -> Entity {
        self.0.get(&square).copied().unwrap_or_else(|| panic!("no highlight at {square}"))
    }

    pub fn many<const N: usize>(&self, squares: [Square; N]) -> [Entity; N] {
        squares.map(|s| self.at(s))
    }
}
