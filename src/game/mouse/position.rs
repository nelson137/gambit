use bevy::{prelude::*, sprite::collide_aabb::collide};
use chess::{File, Rank, Square};

use crate::game::{board::UiBoard, camera::MainCamera};

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseWorldPosition(pub Vec2);

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub(super) fn mouse_screen_position_to_world(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
) {
    let Some(win) = windows.get_primary() else { return };
    let Some(screen_pos) = win.cursor_position() else { return };

    let (camera, camera_transf) = q_camera.single();

    let window_size = Vec2::new(win.width(), win.height());

    // Convert mouse position on screen [0..resolution] to ndc [0..2] (gpu coordinates)
    let ndc = 2.0 * (Vec2::new(0.0, 1.0) - (screen_pos / window_size)).abs();

    // Matrix for undoing the projection and camera transform
    let ndc_to_world = camera_transf.compute_matrix() * camera.projection_matrix().inverse();

    // Convert ndc to world-space coordinates
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

    **mouse_world_pos = world_pos.truncate();
}

#[derive(Default, Deref, DerefMut, Resource)]
pub(super) struct MouseBoardLocation(pub Option<Square>);

pub(super) fn mouse_world_position_to_square(
    q_board: Query<(&GlobalTransform, &Node), With<UiBoard>>,
    mouse_world_pos: Res<MouseWorldPosition>,
    mut mouse_loc: ResMut<MouseBoardLocation>,
) {
    let mouse_pos = **mouse_world_pos;

    let (board_global_transf, board_node) = q_board.single();
    let board_pos = board_global_transf.translation();
    let board_size = board_node.size();
    let tile_size = board_size / 8.0;

    let mouse_in_board =
        collide(mouse_pos.extend(0.0), Vec2::ZERO, board_pos, board_size).is_some();

    **mouse_loc = if mouse_in_board {
        let board_top_left = board_pos.truncate() - (board_size / 2.0);
        let mouse_board_pos = mouse_pos - board_top_left;
        let mouse_board_pos_a1 = Vec2::new(mouse_board_pos.x, board_size.y - mouse_board_pos.y);
        let mouse_square_index = mouse_board_pos_a1 / tile_size;
        let file = File::from_index(mouse_square_index.x as usize);
        let rank = Rank::from_index(mouse_square_index.y as usize);
        Some(Square::make_square(rank, file))
    } else {
        None
    };
}
