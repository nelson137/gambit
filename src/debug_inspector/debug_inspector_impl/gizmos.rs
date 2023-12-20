use bevy::{
    core_pipeline::clear_color::ClearColorConfig, math::vec2, prelude::*,
    render::view::RenderLayers, window::PrimaryWindow,
};

use super::state::InspectorState;

pub(super) fn spawn_gizmo_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera { order: 1, ..default() },
            camera_2d: Camera2d { clear_color: ClearColorConfig::None },
            ..default()
        },
        UiCameraConfig { show_ui: false },
        RenderLayers::layer(1),
    ));
}

pub(super) fn configure_gizmos(mut config: ResMut<GizmoConfig>) {
    config.render_layers = RenderLayers::layer(1);
    config.line_width = 4.0;
}

pub(super) fn draw_entity_hover_gizmo(
    mut inspector_state: ResMut<InspectorState>,
    q_win: Query<&Window, With<PrimaryWindow>>,
    q_global_transf: Query<&GlobalTransform>,
    q_node: Query<&Node>,
    mut gizmos: Gizmos,
) {
    let Some(entity) = inspector_state.panes_state.hierarchy_hover.take() else { return };

    let Ok(win) = q_win.get_single() else { return };
    let half_res = vec2(win.resolution.width(), win.resolution.height()) / 2.0;

    let Ok(global_transf) = q_global_transf.get(entity) else { return };
    let transl = global_transf.compute_transform().translation;
    let pos = vec2(transl.x - half_res.x, half_res.y - transl.y);

    let Ok(node) = q_node.get(entity) else { return };
    let size = node.size();

    const GIZMO_COLOR: Color = Color::RED;

    if size == Vec2::ZERO {
        gizmos.circle_2d(pos, 1.0, GIZMO_COLOR);
    } else {
        gizmos.rect_2d(pos, 0.0, size, GIZMO_COLOR);
    }
}
