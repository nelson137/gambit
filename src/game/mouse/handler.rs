use bevy::prelude::*;

use crate::{
    debug_name,
    game::{
        board::{SelectionEvent, Tile},
        consts::Z_PIECE_SELECTED,
    },
};

use super::position::{MouseBoardSquare, MouseWorldPosition};

pub(super) fn mouse_handler(
    mouse_buttons: Res<Input<MouseButton>>,
    mouse_sq: Res<MouseBoardSquare>,
    mut event_writer: EventWriter<SelectionEvent>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(square) = **mouse_sq {
            event_writer.send(SelectionEvent::MouseDown(square));
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(square) = **mouse_sq {
            event_writer.send(SelectionEvent::MouseUp(square));
        }
    }
}

#[derive(Component)]
pub struct DragContainer;

pub fn spawn_drag_container(mut commands: Commands) {
    commands.spawn((
        DragContainer,
        debug_name!("Drag Container"),
        NodeBundle { z_index: ZIndex::Global(Z_PIECE_SELECTED), ..default() },
    ));
}

pub(super) fn update_drag_container(
    mouse_world_pos: Res<MouseWorldPosition>,
    q_tiles: Query<&Node, With<Tile>>,
    mut q_container: Query<&mut Style, With<DragContainer>>,
) {
    let Some(tile_node) = q_tiles.iter().next() else { return };
    let Vec2 { x: width, y: height } = tile_node.size();
    let mut style = q_container.single_mut();
    style.width = Val::Px(width);
    style.height = Val::Px(height);
    style.top = Val::Px(mouse_world_pos.y - height / 2.0);
    style.left = Val::Px(mouse_world_pos.x - width / 2.0);
}
