use std::collections::{HashMap, hash_map::Entry};

use bevy::prelude::*;

use crate::game::consts::z;

use super::Square;

#[derive(Component, Clone, Default)]
pub struct HighlightTile;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::srgba(1.0, 1.0, 0.0, 0.5);

pub fn highlight_tile(square: Square) -> impl Scene {
    bsn! {
        Name::new(format!("Highlight Tile ({square})"))
        HighlightTile
        template_value(square)
        BackgroundColor(COLOR_HIGHLIGHT)
        Visibility::Hidden
        ZIndex(z::HIGHLIGHT_TILE)
        Node {
            position_type: PositionType::Absolute,
            top: px(0.0),
            left: px(0.0),
            width: percent(100.0),
            height: percent(100.0),
        }
        on(super::EnableSelectionHighlight::handle)
        on(super::ShowingIndicator::on_add)
        on(super::ShowingIndicator::on_remove)
    }
}

pub(super) fn load_highlight_map(
    q_highlights: Query<(Entity, &Square), With<HighlightTile>>,
    mut highlights_map: ResMut<HighlightTilesMap>,
) {
    for (entity, square) in &q_highlights {
        highlights_map.set(*square, entity);
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
}
