use bevy::prelude::*;

use crate::game::{board::Tile, selection::SelectionEvent};

use super::position::{MouseBoardLocation, MouseWorldPosition};

pub(super) fn mouse_handler(
    mouse_buttons: Res<Input<MouseButton>>,
    mouse_loc: Res<MouseBoardLocation>,
    mut event_writer: EventWriter<SelectionEvent>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(square) = **mouse_loc {
            event_writer.send(SelectionEvent::MouseDown(square));
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(square) = **mouse_loc {
            event_writer.send(SelectionEvent::MouseUp(square));
        }
    }
}

#[derive(Component)]
pub struct DragContainer;

pub(super) fn update_drag_container(
    mouse_world_pos: Res<MouseWorldPosition>,
    q_tiles: Query<&Node, With<Tile>>,
    mut q_container: Query<&mut Style, With<DragContainer>>,
) {
    let Vec2 { x: width, y: height } = q_tiles.iter().next().unwrap().size();
    let mut style = q_container.single_mut();
    style.size.width = Val::Px(width);
    style.size.height = Val::Px(height);
    style.position.top = Val::Px(mouse_world_pos.y - height / 2.0);
    style.position.left = Val::Px(mouse_world_pos.x - width / 2.0);
}
