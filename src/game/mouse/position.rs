use bevy::prelude::*;
use chess::Square;

use crate::data::{MainCamera, Tile, UiSquare};

#[derive(Default, Deref, DerefMut, Resource)]
pub(super) struct MouseWorldPosition(pub Vec2);

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub(super) fn mouse_screen_position_to_world(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
) {
    let win = if let Some(w) = windows.get_primary() { w } else { return };

    if let Some(screen_pos) = win.cursor_position() {
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
}

#[derive(Default, Deref, DerefMut, Resource)]
pub(super) struct MouseSquare(pub Option<Square>);

pub(super) fn mouse_world_position_to_square(
    mouse_world_pos: Res<MouseWorldPosition>,
    mut mouse_square: ResMut<MouseSquare>,
    q_tiles: Query<(&UiSquare, &GlobalTransform, &Node), With<Tile>>,
) {
    let mouse_pos = mouse_world_pos.extend(0.0);

    for (square, global_transf, node) in &q_tiles {
        let collision = bevy::sprite::collide_aabb::collide(
            mouse_pos,
            Vec2::ZERO,
            global_transf.translation(), // The z component doesn't matter, it is truncated away
            node.size(),
        );
        if collision.is_some() {
            **mouse_square = Some(**square);
            return;
        }
    }

    **mouse_square = None;
}
