use std::collections::{HashMap, hash_map::Entry};

use bevy::prelude::*;
use chess::{File, Rank};

use crate::utils::text_default_font;

use super::Square;

#[derive(Component, Clone, Default)]
pub struct Tile;

/// The "black" bord color.
///
/// `#769656`
pub const COLOR_BLACK: Color = Color::srgb(
    0x76 as f32 / u8::MAX as f32,
    0x96 as f32 / u8::MAX as f32,
    0x56 as f32 / u8::MAX as f32,
);

/// The "white" bord color.
///
/// `#eeeed2`
pub const COLOR_WHITE: Color = Color::srgb(
    0xee as f32 / u8::MAX as f32,
    0xee as f32 / u8::MAX as f32,
    0xd2 as f32 / u8::MAX as f32,
);

pub(super) fn tiles() -> impl SceneList {
    let mut scenes = Vec::with_capacity(chess::ALL_SQUARES.len());

    for square in chess::ALL_SQUARES.map(Square::new) {
        let file = square.get_file();
        let rank = square.get_rank();

        let file_rank_sum = rank.to_index() + file.to_index();
        let color = if file_rank_sum.is_multiple_of(2) { COLOR_BLACK } else { COLOR_WHITE };

        let mut labels: Vec<Box<dyn Scene>> = Vec::new();
        let text_color =
            TextColor(if file_rank_sum.is_multiple_of(2) { COLOR_WHITE } else { COLOR_BLACK });

        // File markers
        if square.get_rank() == Rank::First {
            labels.push(Box::new(bsn! {
                Text({ square.file_char().to_string() })
                text_default_font(16.)
                template_value(text_color)
                Node {
                    position_type: PositionType::Absolute,
                    bottom: percent(3.5),
                    right: percent(6.0),
                }
            }));
        }

        // Rank markers
        if square.get_file() == File::A {
            labels.push(Box::new(bsn! {
                Text({ square.rank_char().to_string() })
                text_default_font(16.)
                template_value(text_color)
                Node {
                    position_type: PositionType::Absolute,
                    top: percent(1.0),
                    left: percent(4.5),
                }
            }));
        }

        scenes.push(bsn! {
            Name::new(format!("Tile ({square})"))
            Tile
            template_value(square)
            Node {
                position_type: PositionType::Relative,
                width: percent(100.0 / 8.0),
                height: percent(100.0 / 8.0),
            }
            BackgroundColor(color)
            Children [
                { labels },
                super::highlight_tile(square),
                { super::hints(square) },
            ]
        });
    }

    scenes
}

pub(super) fn load_tile_map(
    q_tiles: Query<(Entity, &Square), With<Tile>>,
    mut tiles_map: ResMut<TilesMap>,
) {
    for (entity, square) in &q_tiles {
        tiles_map.set(*square, entity);
    }
}

#[derive(Resource)]
pub struct TilesMap(HashMap<Square, Entity>);

impl Default for TilesMap {
    fn default() -> Self {
        Self(HashMap::with_capacity(64))
    }
}

impl TilesMap {
    fn set(&mut self, square: Square, entity: Entity) {
        match self.0.entry(square) {
            Entry::Occupied(_) => panic!("tile already in the state at {square}"),
            Entry::Vacant(e) => e.insert(entity),
        };
    }

    pub fn at(&self, square: Square) -> Entity {
        self.0.get(&square).copied().unwrap_or_else(|| panic!("no tile at {square}"))
    }
}
