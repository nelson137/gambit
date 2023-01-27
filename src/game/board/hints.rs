use bevy::{ecs::system::Command, prelude::*};

use crate::{debug_name, game::consts::Z_MOVE_HINT};

use super::{BoardLocation, BoardState, MoveHints};

#[derive(Default)]
pub struct ShowHints(pub Vec<Entity>);

impl Command for ShowHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                vis.is_visible = true;
            }
        }
    }
}

#[derive(Default)]
pub struct HideHints(pub Vec<Entity>);

impl Command for HideHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                vis.is_visible = false;
            }
        }
    }
}

pub fn spawn_hints(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
) {
    let move_hint_texture = asset_server.load("images/hints/move.png");
    let capture_hint_texture = asset_server.load("images/hints/capture.png");
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };

    for square in chess::ALL_SQUARES {
        let location = BoardLocation::new(square);

        // Move hint
        let move_entity = commands
            .spawn((
                debug_name!("Move Hint ({square})"),
                location,
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: pos_top_left,
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    visibility: Visibility::INVISIBLE,
                    z_index: ZIndex::Local(Z_MOVE_HINT),
                    ..default()
                },
            ))
            .with_children(|cmds| {
                cmds.spawn(ImageBundle {
                    image: UiImage(move_hint_texture.clone()),
                    style: Style {
                        size: Size::new(Val::Percent(100.0 / 3.0), Val::Percent(100.0 / 3.0)),
                        ..default()
                    },
                    ..default()
                });
            })
            .id();

        // Capture hint
        let capture_entity = commands
            .spawn((
                debug_name!("Capture Hint ({square})"),
                location,
                ImageBundle {
                    image: UiImage(capture_hint_texture.clone()),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: pos_top_left,
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..default()
                    },
                    visibility: Visibility::INVISIBLE,
                    z_index: ZIndex::Local(Z_MOVE_HINT),
                    ..default()
                },
            ))
            .id();

        board_state.set_move_hints(square, MoveHints { capture_entity, move_entity });
        commands.entity(board_state.tile(square)).push_children(&[move_entity, capture_entity]);
    }
}
