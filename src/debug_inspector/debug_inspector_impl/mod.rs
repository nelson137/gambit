use bevy::{app::prelude::*, ecs::prelude::*, prelude::Deref};
use bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext};
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
            .add_plugins(DefaultInspectorConfigPlugin)
            .init_resource::<InspectorState>()
            .init_resource::<DebugInspectorIsUsingMouse>()
            .add_systems(PreStartup, (gizmos::spawn_gizmo_camera, gizmos::configure_gizmos))
            .add_systems(EguiPrimaryContextPass, inspector_ui)
            .add_systems(Update, self::gizmos::draw_entity_hover_gizmo)
            .noop();
    }
}

fn inspector_ui(world: &mut World) {
    let mut q = world.query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>();
    let Ok(context) = q.single_mut(world) else { return };
    let mut context = EguiContext::clone(&context);

    world
        .resource_scope::<InspectorState, _>(|world, mut state| state.ui(world, context.get_mut()));

    let ctx = context.get_mut();
    world.resource_mut::<DebugInspectorIsUsingMouse>().0 =
        ctx.is_pointer_over_area() || ctx.wants_pointer_input();
}

#[derive(Default, Deref, Resource)]
pub struct DebugInspectorIsUsingMouse(bool);
