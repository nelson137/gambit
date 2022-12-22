use bevy::prelude::*;

use crate::{
    assets::TILE_ASSET_SIZE,
    data::{Dragging, MainCamera, MouseSquare, MouseWorldPosition, Tile, UiPiece, UiSquare},
};

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn mouse_screen_position_to_world(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
    mut q_dragging: Query<&mut Transform, (With<Dragging>, With<UiPiece>)>,
) {
    let win = if let Some(w) = windows.get_primary() { w } else { return };

    if let Some(screen_pos) = win.cursor_position() {
        let (camera, camera_transf) = q_camera.single();

        let window_size = Vec2::new(win.width(), win.height());

        // Convert mouse position on screen [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = 2.0 * (screen_pos / window_size) - Vec2::ONE;

        // Matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transf.compute_matrix() * camera.projection_matrix().inverse();

        // Convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        **mouse_world_pos = world_pos.truncate();

        for mut transf in &mut q_dragging {
            transf.translation.x = world_pos.x;
            transf.translation.y = world_pos.y;
        }
    }
}

pub fn mouse_world_position_to_square(
    mouse_world_pos: Res<MouseWorldPosition>,
    mut mouse_square: ResMut<MouseSquare>,
    q_tiles: Query<(&UiSquare, &Transform), With<Tile>>,
) {
    let mouse_pos = mouse_world_pos.extend(0.0);

    for (square, transf) in &q_tiles {
        let collision = bevy::sprite::collide_aabb::collide(
            mouse_pos,
            Vec2::ZERO,
            transf.translation, // The z component doesn't matter, it is truncated away
            Vec2::splat(TILE_ASSET_SIZE) * transf.scale.truncate(),
        );
        if collision.is_some() {
            **mouse_square = Some(**square);
            return;
        }
    }

    **mouse_square = None;
}
