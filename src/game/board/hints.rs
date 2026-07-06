use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::consts::z;

use super::Square;

#[derive(Component, Clone, Copy, Default)]
pub enum Hint {
    Move,
    #[default]
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

impl Default for TileHints {
    fn default() -> Self {
        Self { move_entity: Entity::PLACEHOLDER, capture_entity: Entity::PLACEHOLDER }
    }
}

impl TileHints {
    fn entity(&self, hint: Hint) -> Entity {
        match hint {
            Hint::Move => self.move_entity,
            Hint::Capture => self.capture_entity,
        }
    }
}

pub(super) fn hints(square: Square) -> impl SceneList {
    fn move_hint_texture() -> impl Scene {
        template(|ctx| {
            let asset_server = ctx.resource::<AssetServer>();
            let image = asset_server.load::<Image>("images/hints/move.png");
            Ok(ImageNode::new(image))
        })
    }

    fn capture_hint_texture() -> impl Scene {
        template(|ctx| {
            let asset_server = ctx.resource::<AssetServer>();
            let image = asset_server.load::<Image>("images/hints/capture.png");
            Ok(ImageNode::new(image))
        })
    }

    let scenes: Vec<Box<dyn Scene>> = vec![
        Box::new(bsn! {
            Name::new(format!("Move Hint ({square})"))
            template_value(Hint::Move)
            template_value(square)
            Visibility::Hidden
            ZIndex(z::MOVE_HINT)
            Node {
                position_type: PositionType::Absolute,
                top: px(0.0),
                left: px(0.0),
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            on(super::EnableHint::handle)
            on(super::ShowingIndicator::on_add)
            on(super::ShowingIndicator::on_remove)
            Children [
                (
                    move_hint_texture()
                    Node { width: percent(100.0 / 3.0), height: percent(100.0 / 3.0) }
                )
            ]
        }),
        Box::new(bsn! {
            Name::new(format!("Capture Hint ({square})"))
            template_value(Hint::Capture)
            template_value(square)
            Visibility::Hidden
            ZIndex(z::MOVE_HINT)
            capture_hint_texture()
            Node {
                position_type: PositionType::Absolute,
                top: px(0.0),
                left: px(0.0),
                width: percent(100.0),
                height: percent(100.0),
            }
            on(super::EnableHint::handle)
            on(super::ShowingIndicator::on_add)
            on(super::ShowingIndicator::on_remove)
        }),
    ];

    scenes
}

pub(super) fn load_hint_map(
    q_highlights: Query<(Entity, &Square, &Hint)>,
    mut hints_map: ResMut<TileHintsMap>,
) {
    fn update(entry: &mut TileHints, hint: &Hint, entity: Entity) {
        match *hint {
            Hint::Move => entry.move_entity = entity,
            Hint::Capture => entry.capture_entity = entity,
        }
    }
    for (entity, square, hint) in &q_highlights {
        let entry = hints_map.0.entry(*square);
        update(entry.or_default(), hint, entity);
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
    pub fn at(&self, square: Square) -> &TileHints {
        self.0.get(&square).unwrap_or_else(|| panic!("no move hints at {square}"))
    }

    pub fn many_from_descriptors(&self, descriptors: &[HintDescriptor]) -> Vec<Entity> {
        descriptors.iter().map(|d| self.at(d.square).entity(d.hint)).collect()
    }
}
