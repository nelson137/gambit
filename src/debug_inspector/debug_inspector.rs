use bevy::{ecs::system::SystemState, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{egui, EguiContexts},
    bevy_inspector::ui_for_world,
    DefaultInspectorConfigPlugin,
};

use crate::utils::AppNoop;

use super::DebugInspectorPlugin;

impl Plugin for DebugInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_systems(Update, world_inspector_ui)
            .noop();
    }
}

const DEFAULT_SIZE: (f32, f32) = (300.0, 200.0);

fn world_inspector_ui(world: &mut World) {
    let mut egui_state = SystemState::<EguiContexts>::new(world);
    let ctx = egui_state.get_mut(world).ctx_mut().clone();

    egui::Window::new("World Inspector").default_size(DEFAULT_SIZE).show(&ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui_for_world(world, ui);
            ui.allocate_space(ui.available_size());
        });
    });
}
