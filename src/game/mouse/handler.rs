use bevy::prelude::*;

use crate::game::{
    board::{MouseSelectionEvent, Square, Tile, TilesMap},
    consts::z,
};

use super::position::{MouseBoardSquare, MouseWorldPosition};

pub(super) fn mouse_handler(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_sq: Res<MouseBoardSquare>,
    mut message_writer: MessageWriter<MouseSelectionEvent>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left)
        && let Some(square) = **mouse_sq
    {
        message_writer.write(MouseSelectionEvent::MouseDown(square));
    }

    if mouse_buttons.just_released(MouseButton::Left)
        && let Some(square) = **mouse_sq
    {
        message_writer.write(MouseSelectionEvent::MouseUp(square));
    }
}

#[derive(Component, Clone, Default)]
pub struct DragContainer;

pub(super) fn drag_container() -> impl Scene {
    bsn! {
        #DragContainer
        DragContainer
        Node
        GlobalZIndex(z::G_PIECE_MOVE)
    }
}

pub(super) fn update_drag_container(
    mouse_world_pos: Res<MouseWorldPosition>,
    q_tiles: Query<&ComputedNode, With<Tile>>,
    mut q_container: Single<&mut Node, With<DragContainer>>,
) {
    let Some(tile_computed_node) = q_tiles.iter().next() else { return };
    let inverse_scale_factor = tile_computed_node.inverse_scale_factor();
    let Vec2 { x: width, y: height } = tile_computed_node.size() * inverse_scale_factor;
    let node = &mut **q_container;
    node.width = px(width);
    node.height = px(height);
    node.top = px(mouse_world_pos.y - height / 2.0);
    node.left = px(mouse_world_pos.x - width / 2.0);
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(storage = "SparseSet")]
pub struct Dragging {
    pub original_square: Square,
}

impl Dragging {
    pub fn on_add(
        add: On<Add, Self>,
        mut commands: Commands,
        drag_container: Single<Entity, With<DragContainer>>,
    ) {
        commands.entity(*drag_container).add_child(add.entity);
    }

    pub fn on_remove(
        remove: On<Remove, Self>,
        mut commands: Commands,
        tiles_map: Res<TilesMap>,
        q_dragging: Query<&Self>,
    ) {
        let Ok(dragging) = q_dragging.get(remove.entity) else { return };
        let original_tile = tiles_map.at(dragging.original_square);
        commands.entity(original_tile).add_child(remove.entity);
    }
}
