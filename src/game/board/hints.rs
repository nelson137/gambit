use bevy::prelude::*;

use crate::{debug_name_f, game::consts::Z_MOVE_HINT};

use super::{BoardState, Square};

#[derive(Component)]
pub struct Hint;

#[derive(Debug)]
pub struct TileHints {
    pub move_entity: Entity,
    pub capture_entity: Entity,
}

pub fn spawn_hints(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
) {
    let move_hint_texture = asset_server.load("images/hints/move.png");
    let capture_hint_texture = asset_server.load("images/hints/capture.png");

    for square in chess::ALL_SQUARES {
        let square = Square::new(square);

        // Move hint
        let move_entity = commands
            .spawn((
                Hint,
                debug_name_f!("Move Hint ({square})"),
                square,
                NodeBundle {
                    node: Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    ..default()
                },
                GlobalZIndex(Z_MOVE_HINT),
            ))
            .with_children(|cmds| {
                cmds.spawn(ImageBundle {
                    image: ImageNode::new(move_hint_texture.clone()),
                    node: Node {
                        width: Val::Percent(100.0 / 3.0),
                        height: Val::Percent(100.0 / 3.0),
                        ..default()
                    },
                    ..default()
                });
            })
            .id();

        // Capture hint
        let capture_entity = commands
            .spawn((
                Hint,
                debug_name_f!("Capture Hint ({square})"),
                square,
                ImageBundle {
                    image: ImageNode::new(capture_hint_texture.clone()),
                    node: Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    ..default()
                },
                GlobalZIndex(Z_MOVE_HINT),
            ))
            .id();

        board_state.set_tile_hints(square, TileHints { capture_entity, move_entity });
        commands.entity(board_state.tile(square)).add_children(&[move_entity, capture_entity]);
    }
}
