use bevy::{app::prelude::*, ecs::prelude::*, prelude::Deref, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiPreUpdateSet};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

use crate::utils::NoopExts;

use super::DebugInspectorPlugin;

use self::state::InspectorState;

mod gizmos;
mod panes;
mod split_panel;
mod state;

impl Plugin for DebugInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .register_type::<bevy::pbr::AmbientLight>()
            .register_type::<bevy::pbr::ClusterConfig>()
            .register_type::<bevy::pbr::DirectionalLight>()
            .register_type::<bevy::pbr::PointLight>()
            .register_type::<bevy::pbr::StandardMaterial>()
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(bevy::pbr::PbrPlugin::default())
            .add_plugins(bevy::gizmos::GizmoPlugin)
            .init_resource::<InspectorState>()
            .init_resource::<DebugInspectorIsUsingMouse>()
            .add_systems(
                PreStartup,
                (self::gizmos::spawn_gizmo_camera, self::gizmos::configure_gizmos),
            )
            .add_systems(
                Update,
                (
                    debug_inspector_update.after(EguiPreUpdateSet::BeginPass),
                    self::gizmos::draw_entity_hover_gizmo,
                )
                    .chain(),
            )
            .noop();
    }
}

fn debug_inspector_update(world: &mut World) {
    let mut q = world.query_filtered::<&mut EguiContext, With<PrimaryWindow>>();
    let Ok(context) = q.get_single_mut(world) else { return };
    let mut context = EguiContext::clone(&context);

    world
        .resource_scope::<InspectorState, _>(|world, mut state| state.ui(world, context.get_mut()));

    let ctx = context.get_mut();
    world.resource_mut::<DebugInspectorIsUsingMouse>().0 =
        ctx.is_pointer_over_area() || ctx.wants_pointer_input();
}

#[derive(Default, Deref, Resource)]
pub struct DebugInspectorIsUsingMouse(bool);
