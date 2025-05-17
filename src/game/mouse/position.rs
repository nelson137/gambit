use bevy::{prelude::*, window::PrimaryWindow};
use chess::{File, Rank};

use crate::game::{
    board::{Square, UiBoard},
    camera::MainCamera,
};

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseWorldPosition(pub Vec2);

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub(super) fn mouse_screen_position_to_world(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
) {
    let Ok(win) = q_window.get_single() else { return };
    let Some(screen_pos) = win.cursor_position() else { return };

    let (camera, camera_transf) = q_camera.single();

    let window_size = Vec2::new(win.width(), win.height());

    // Convert mouse position on screen [0..resolution] to ndc [0..2] (gpu coordinates)
    let ndc = 2.0 * screen_pos / window_size;

    if let Some(ndc_to_world) = camera.ndc_to_world(camera_transf, ndc.extend(0.)) {
        **mouse_world_pos = ndc_to_world.truncate();
    };
}

#[derive(Default, Deref, DerefMut, Resource)]
pub(super) struct MouseBoardSquare(pub Option<Square>);

pub(super) fn mouse_world_position_to_square(
    q_board: Query<(&GlobalTransform, &ComputedNode), With<UiBoard>>,
    mouse_world_pos: Res<MouseWorldPosition>,
    mut mouse_sq: ResMut<MouseBoardSquare>,
) {
    let mouse_pos = **mouse_world_pos;

    let Ok((board_global_transf, board_computed_node)) = q_board.get_single() else { return };
    let inverse_scale_factor = board_computed_node.inverse_scale_factor();
    let board_pos = board_global_transf.translation().truncate() * inverse_scale_factor;
    let board_size = board_computed_node.size() * inverse_scale_factor;
    let tile_size = board_size / 8.0;

    let board_top_left = board_pos - (board_size / 2.0);
    let mouse_board_pos = mouse_pos - board_top_left;

    let mouse_in_board = 0.0 < mouse_board_pos.x
        && mouse_board_pos.x < board_size.x
        && 0.0 < mouse_board_pos.y
        && mouse_board_pos.y < board_size.y;

    **mouse_sq = if mouse_in_board {
        let mouse_board_pos_a1 = Vec2::new(mouse_board_pos.x, board_size.y - mouse_board_pos.y);
        let mouse_square_index = mouse_board_pos_a1 / tile_size;
        let file = File::from_index(mouse_square_index.x as usize);
        let rank = Rank::from_index(mouse_square_index.y as usize);
        Some(Square::from_coords(rank, file))
    } else {
        None
    };
}
