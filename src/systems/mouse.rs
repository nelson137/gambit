use bevy::prelude::*;

use crate::{
    assets::TILE_ASSET_SIZE,
    data::{Location, MainCamera, MouseWorldPosition, Tile},
};

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn mouse_screen_position_to_world(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
) {
    let win = windows.primary();

    if let Some(screen_pos) = win.cursor_position() {
        let (camera, camera_transf) = q_camera.single();

        let window_size = Vec2::new(win.width() as f32, win.height() as f32);

        // Convert mouse position on screen [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = 2.0 * (screen_pos / window_size) - Vec2::ONE;

        // Matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transf.compute_matrix() * camera.projection_matrix().inverse();

        // Convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        mouse_world_pos.0 = world_pos.truncate();
    }
}

pub fn click_handler(
    mouse_buttons: Res<Input<MouseButton>>,
    mouse_world_pos: Res<MouseWorldPosition>,
    q_tiles: Query<(&Location, &Transform), With<Tile>>,
) {
    if mouse_buttons.just_released(MouseButton::Left) {
        let pos = mouse_world_pos.0.extend(0.0);
        for (loc, transf) in &q_tiles {
            let collision = bevy::sprite::collide_aabb::collide(
                pos,
                Vec2::ZERO,
                transf.translation,
                Vec2::splat(TILE_ASSET_SIZE),
            );
            if collision.is_some() {
                eprintln!("CLICK: {loc}");
                break;
            }
        }
    }
}
