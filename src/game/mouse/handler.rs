use bevy::{
    ecs::component::{ComponentHooks, Mutable, StorageType},
    prelude::*,
};

use crate::{
    debug_name,
    game::{
        board::{BoardState, MouseSelectionEvent, Square, Tile},
        consts::Z_PIECE_SELECTED,
    },
    utils::{NoopExts, hook},
};

use super::position::{MouseBoardSquare, MouseWorldPosition};

pub(super) fn mouse_handler(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_sq: Res<MouseBoardSquare>,
    mut event_writer: EventWriter<MouseSelectionEvent>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(square) = **mouse_sq {
            event_writer.write(MouseSelectionEvent::MouseDown(square));
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(square) = **mouse_sq {
            event_writer.write(MouseSelectionEvent::MouseUp(square));
        }
    }
}

#[derive(Component)]
pub struct DragContainer;

pub(super) fn spawn_drag_container(mut commands: Commands) {
    commands.spawn((
        DragContainer,
        debug_name!("Drag Container"),
        Node::DEFAULT,
        GlobalZIndex(Z_PIECE_SELECTED),
    ));
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

#[derive(Clone)]
pub struct Dragging {
    pub original_square: Square,
}

impl Component for Dragging {
    type Mutability = Mutable;
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .noop()
            .on_add(hook!(on_add_dragging_piece))
            .on_remove(hook!(Dragging => on_remove_dragging_piece))
            .noop();
    }
}

fn on_add_dragging_piece(
    In(piece): In<Entity>,
    mut commands: Commands,
    q_drag_container: Query<Entity, With<DragContainer>>,
) {
    commands.entity(piece).insert(ChildOf(q_drag_container.single().unwrap()));
}

fn on_remove_dragging_piece(
    In((piece, dragging)): In<(Entity, Dragging)>,
    mut commands: Commands,
    board_state: Res<BoardState>,
) {
    let original_tile = board_state.tile(dragging.original_square);
    commands.entity(piece).insert(ChildOf(original_tile));
}
