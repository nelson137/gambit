use bevy::{
    math::vec2, prelude::*, render::camera::ClearColorConfig, utils::HashSet, window::PrimaryWindow,
};

use super::state::InspectorState;

pub(super) fn spawn_gizmo_camera(mut commands: Commands) {
    commands
        .spawn((Camera2d, Camera { order: 1, clear_color: ClearColorConfig::None, ..default() }));
}

pub(super) fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let config = config_store.config_mut::<DefaultGizmoConfigGroup>().0;
    config.line_width = 4.0;
}

pub(super) fn draw_entity_hover_gizmo(
    mut drawn_entities: Local<HashSet<Entity>>,
    mut inspector_state: ResMut<InspectorState>,
    q_win: Query<&Window, With<PrimaryWindow>>,
    q_global_transf: Query<&GlobalTransform>,
    q_computed_node: Query<&ComputedNode>,
    mut gizmos: Gizmos,
) {
    let Ok(win) = q_win.get_single() else { return };
    let half_res = vec2(win.resolution.width(), win.resolution.height());

    drawn_entities.extend(inspector_state.panes_state.selected_entities.iter());
    drawn_entities.extend(inspector_state.panes_state.hierarchy_hover.take());

    for entity in drawn_entities.drain() {
        let Ok(global_transf) = q_global_transf.get(entity) else { continue };
        let Ok(computed_node) = q_computed_node.get(entity) else { continue };

        let transl = global_transf.translation();
        let inverse_scale_factor = computed_node.inverse_scale_factor();
        let size = computed_node.size() * inverse_scale_factor;
        let pos = vec2(transl.x - half_res.x, half_res.y - transl.y) * inverse_scale_factor;

        const GIZMO_COLOR: Srgba = Srgba::RED;

        if size == Vec2::ZERO {
            gizmos.circle_2d(pos, 1.0, GIZMO_COLOR);
        } else {
            gizmos.rect_2d(pos, size, GIZMO_COLOR);
        }
    }
}
