use std::collections::{HashMap, hash_map::Entry};

use bevy::prelude::*;

use crate::{debug_name_f, game::consts::Z_MOVE_HINT};

use super::{Square, TilesMap};

#[derive(Component, Clone, Copy)]
#[require(Visibility::Hidden, GlobalZIndex(Z_MOVE_HINT))]
pub enum Hint {
    Move,
    Capture,
}

#[derive(Clone, Copy)]
pub struct HintDescriptor {
    hint: Hint,
    square: Square,
}

impl HintDescriptor {
    fn new(hint: Hint, square: Square) -> Self {
        Self { hint, square }
    }

    pub fn r#move(square: Square) -> Self {
        Self::new(Hint::Move, square)
    }

    pub fn capture(square: Square) -> Self {
        Self::new(Hint::Capture, square)
    }
}

#[derive(Debug)]
pub struct TileHints {
    pub move_entity: Entity,
    pub capture_entity: Entity,
}

impl TileHints {
    fn entity(&self, hint: Hint) -> Entity {
        match hint {
            Hint::Move => self.move_entity,
            Hint::Capture => self.capture_entity,
        }
    }
}

pub fn spawn_hints(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiles_map: Res<TilesMap>,
    mut hints_map: ResMut<TileHintsMap>,
) {
    let move_hint_texture = asset_server.load("images/hints/move.png");
    let capture_hint_texture = asset_server.load("images/hints/capture.png");

    for square in chess::ALL_SQUARES {
        let square = Square::new(square);

        // Move hint
        let move_entity = commands
            .spawn((
                debug_name_f!("Move Hint ({square})"),
                Hint::Move,
                square,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    ImageNode::new(move_hint_texture.clone()),
                    Node {
                        width: Val::Percent(100.0 / 3.0),
                        height: Val::Percent(100.0 / 3.0),
                        ..default()
                    },
                )],
            ))
            .id();

        // Capture hint
        let capture_entity = commands
            .spawn((
                debug_name_f!("Capture Hint ({square})"),
                Hint::Capture,
                square,
                ImageNode::new(capture_hint_texture.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ))
            .id();

        hints_map.set(square, TileHints { capture_entity, move_entity });
        commands.entity(tiles_map.at(square)).add_children(&[move_entity, capture_entity]);
    }
}

#[derive(Resource)]
pub struct TileHintsMap(HashMap<Square, TileHints>);

impl Default for TileHintsMap {
    fn default() -> Self {
        Self(HashMap::with_capacity(64))
    }
}

impl TileHintsMap {
    pub fn set(&mut self, square: Square, hints: TileHints) {
        match self.0.entry(square) {
            Entry::Occupied(_) => panic!("move hints already in the state at {square}"),
            Entry::Vacant(e) => e.insert(hints),
        };
    }

    pub fn at(&self, square: Square) -> &TileHints {
        self.0.get(&square).unwrap_or_else(|| panic!("no move hints at {square}"))
    }

    pub fn many_from_descriptors(&self, descriptors: &[HintDescriptor]) -> Vec<Entity> {
        descriptors.iter().map(|d| self.at(d.square).entity(d.hint)).collect()
    }
}
