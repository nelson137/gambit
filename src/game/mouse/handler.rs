use bevy::prelude::*;

use crate::{
    debug_name,
    game::{
        board::{MouseSelectionEvent, Square, Tile, TilesMap},
        consts::Z_PIECE_SELECTED,
    },
};

use super::position::{MouseBoardSquare, MouseWorldPosition};

pub(super) fn mouse_handler(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_sq: Res<MouseBoardSquare>,
    mut event_writer: EventWriter<MouseSelectionEvent>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left)
        && let Some(square) = **mouse_sq
    {
        event_writer.write(MouseSelectionEvent::MouseDown(square));
    }

    if mouse_buttons.just_released(MouseButton::Left)
        && let Some(square) = **mouse_sq
    {
        event_writer.write(MouseSelectionEvent::MouseUp(square));
    }
}

#[derive(Component)]
#[require(GlobalZIndex(Z_PIECE_SELECTED))]
pub struct DragContainer;

pub(super) fn spawn_drag_container(mut commands: Commands) {
    commands.spawn((debug_name!("Drag Container"), DragContainer, Node::DEFAULT));
}

pub(super) fn update_drag_container(
    mouse_world_pos: Res<MouseWorldPosition>,
    q_tiles: Query<&ComputedNode, With<Tile>>,
    mut q_container: Query<&mut Node, With<DragContainer>>,
) {
    let Some(tile_computed_node) = q_tiles.iter().next() else { return };
    let inverse_scale_factor = tile_computed_node.inverse_scale_factor();
    let Vec2 { x: width, y: height } = tile_computed_node.size() * inverse_scale_factor;
    if let Ok(mut node) = q_container.single_mut() {
        node.width = Val::Px(width);
        node.height = Val::Px(height);
        node.top = Val::Px(mouse_world_pos.y - height / 2.0);
        node.left = Val::Px(mouse_world_pos.x - width / 2.0);
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(storage = "SparseSet")]
pub struct Dragging {
    pub original_square: Square,
}

pub fn on_add_dragging_piece(
    trigger: Trigger<OnAdd, Dragging>,
    mut commands: Commands,
    q_drag_container: Query<Entity, With<DragContainer>>,
) {
    let piece = trigger.target();
    commands.entity(q_drag_container.single().unwrap()).add_child(piece);
}

pub fn on_remove_dragging_piece(
    trigger: Trigger<OnRemove, Dragging>,
    mut commands: Commands,
    tiles_map: Res<TilesMap>,
    q_dragging: Query<&Dragging>,
) {
    let piece = trigger.target();
    let Ok(dragging) = q_dragging.get(piece) else { return };
    let original_tile = tiles_map.at(dragging.original_square);
    commands.entity(original_tile).add_child(piece);
}
