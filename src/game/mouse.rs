use bevy::prelude::*;

use crate::data::{Dragging, Dropped, MouseSquare, Z_PIECE, Z_PIECE_SELECTED};

use super::selection::SelectionEvent;

pub fn mouse_handler(
    mouse_buttons: Res<Input<MouseButton>>,
    mouse_square: Res<MouseSquare>,
    mut event_writer: EventWriter<SelectionEvent>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(mouse_square) = **mouse_square {
            event_writer.send(SelectionEvent::MouseDown(mouse_square));
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(mouse_square) = **mouse_square {
            event_writer.send(SelectionEvent::MouseUp(mouse_square));
        }
    }
}

pub fn start_drag(mut q_added_dragging: Query<&mut Transform, Added<Dragging>>) {
    for mut transf in &mut q_added_dragging {
        transf.translation.z = Z_PIECE_SELECTED;
    }
}

pub fn end_drag(
    mut commands: Commands,
    mut q_added_dragging: Query<(Entity, &mut Transform), Added<Dropped>>,
) {
    for (entity, mut transf) in &mut q_added_dragging {
        commands.entity(entity).remove::<(Dragging, Dropped)>();
        transf.translation.z = Z_PIECE;
    }
}
