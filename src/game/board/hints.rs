use bevy::{ecs::system::Command, prelude::*};

use crate::{debug_name_f, game::consts::Z_MOVE_HINT};

use super::{BoardState, MoveHints, Square};

#[derive(Default)]
pub struct ShowHints(pub Vec<Entity>);

impl Command for ShowHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                *vis = Visibility::Visible;
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
                *vis = Visibility::Hidden;
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
        let square = Square::new(square);

        // Move hint
        let move_entity = commands
            .spawn((
                debug_name_f!("Move Hint ({square})"),
                square,
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: pos_top_left,
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    z_index: ZIndex::Global(Z_MOVE_HINT),
                    ..default()
                },
            ))
            .with_children(|cmds| {
                cmds.spawn(ImageBundle {
                    image: UiImage::new(move_hint_texture.clone()),
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
                debug_name_f!("Capture Hint ({square})"),
                square,
                ImageBundle {
                    image: UiImage::new(capture_hint_texture.clone()),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: pos_top_left,
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    z_index: ZIndex::Global(Z_MOVE_HINT),
                    ..default()
                },
            ))
            .id();

        board_state.set_move_hints(square, MoveHints { capture_entity, move_entity });
        commands.entity(board_state.tile(square)).push_children(&[move_entity, capture_entity]);
    }
}
