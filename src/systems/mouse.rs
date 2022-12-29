use bevy::prelude::*;

use crate::data::{Dragging, MainCamera, MouseSquare, MouseWorldPosition, Tile, UiPiece, UiSquare};

pub struct MousePositionPlugin;

impl Plugin for MousePositionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseWorldPosition>()
            .init_resource::<MouseSquare>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new().with_system(mouse_screen_position_to_world).with_system(
                    mouse_world_position_to_square.after(mouse_screen_position_to_world),
                ),
            );
    }
}

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
fn mouse_screen_position_to_world(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
    mut q_dragging: Query<&mut Transform, (With<Dragging>, With<UiPiece>)>,
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

        for mut transf in &mut q_dragging {
            transf.translation.x = world_pos.x;
            transf.translation.y = world_pos.y;
        }
    }
}

fn mouse_world_position_to_square(
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
