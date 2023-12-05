use bevy::{prelude::*, reflect::TypeRegistry, window::PrimaryWindow};
use bevy_egui::egui::{RichText, Ui};
use bevy_inspector_egui::bevy_inspector::{
    by_type_id::ui_for_resource,
    hierarchy::{hierarchy_ui, SelectedEntities},
    ui_for_entities_shared_components, ui_for_entity,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiSet},
    egui, DefaultInspectorConfigPlugin,
};

use crate::utils::AppNoop;

use super::{
    split_panel::{PaneViewer, SplitPanel, SplitPanelState},
    DebugInspectorPlugin,
};

impl Plugin for DebugInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .register_type::<bevy::pbr::AmbientLight>()
            .register_type::<bevy::pbr::ClusterConfig>()
            .register_type::<bevy::pbr::DirectionalLight>()
            .register_type::<bevy::pbr::PointLight>()
            .register_type::<bevy::pbr::StandardMaterial>()
            .add_plugins(DefaultInspectorConfigPlugin)
            .init_resource::<InspectorState>()
            .add_systems(Update, debug_inspector_update.after(EguiSet::BeginFrame))
            .noop();
    }
}

fn debug_inspector_update(world: &mut World) {
    let mut q = world.query_filtered::<&mut EguiContext, With<PrimaryWindow>>();
    let Ok(context) = q.get_single_mut(world) else { return };
    let mut context = context.clone();
    world
        .resource_scope::<InspectorState, _>(|world, mut state| state.ui(world, context.get_mut()));
}

#[derive(Resource)]
struct InspectorState {
    left_state: SplitPanelState<Pane>,
    right_state: SplitPanelState<Pane>,
    selected_entities: SelectedEntities,
}

impl Default for InspectorState {
    fn default() -> Self {
        let left_state = SplitPanelState::equally_sized([Pane::Hierarchy, Pane::EntityComponents]);
        let right_state = SplitPanelState::equally_sized([Pane::Resources, Pane::Stockfish]);
        Self { left_state, right_state, selected_entities: default() }
    }
}

impl InspectorState {
    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let type_registry = world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        let mut panel_viewer = InspectorPaneViewer {
            world,
            type_registry: &type_registry,
            selected_entities: &mut self.selected_entities,
        };

        const DEFAULT_WIDTH: f32 = 318.0;

        SplitPanel::left("Left Panel", &mut self.left_state)
            .default_width(DEFAULT_WIDTH)
            .show(ctx, &mut panel_viewer);

        SplitPanel::right("Right Panel", &mut self.right_state)
            .default_width(DEFAULT_WIDTH)
            .show(ctx, &mut panel_viewer);
    }
}

#[derive(Clone, Copy)]
enum Pane {
    Hierarchy,
    EntityComponents,
    Resources,
    Stockfish,
}

struct InspectorPaneViewer<'a> {
    world: &'a mut World,
    type_registry: &'a TypeRegistry,
    selected_entities: &'a mut SelectedEntities,
}

impl<'a> PaneViewer for InspectorPaneViewer<'a> {
    type Pane = Pane;

    fn ui(&mut self, ui: &mut egui::Ui, pane: &mut Self::Pane) {
        match *pane {
            Pane::Hierarchy => self.show_hierarchy(ui),
            Pane::EntityComponents => self.show_entity_components(ui),
            Pane::Resources => self.show_resources(ui),
            Pane::Stockfish => self.show_stockfish(ui),
        }
    }
}

impl<'a> InspectorPaneViewer<'a> {
    fn show_hierarchy(&mut self, ui: &mut Ui) {
        ui.heading("Hierarchy");
        ui.separator();
        hierarchy_ui(self.world, ui, self.selected_entities);
    }

    fn show_entity_components(&mut self, ui: &mut Ui) {
        ui.heading("Entity Components");
        ui.separator();

        match self.selected_entities.as_slice() {
            &[] => {
                ui.label(RichText::new("No entities selected").italics());
            }
            &[entity] => {
                ui_for_entity(self.world, entity, ui);
            }
            entities => {
                ui.label("Shared components");
                ui_for_entities_shared_components(self.world, entities, ui);
            }
        }
    }

    fn show_resources(&mut self, ui: &mut Ui) {
        ui.heading("Resources");
        ui.separator();

        let mut resources = self
            .type_registry
            .iter()
            .filter(|&reg| reg.data::<ReflectResource>().is_some())
            .map(|reg| {
                let info = reg.type_info();
                (info.type_path_table().short_path(), info.type_id())
            })
            .filter(|&(_, type_id)| self.world.components().get_resource_id(type_id).is_some())
            .collect::<Vec<_>>();
        resources.sort_by_key(|(name, _)| *name);

        for (name, type_id) in resources {
            ui.collapsing(name, |ui| {
                ui_for_resource(self.world, type_id, ui, name, self.type_registry);
            });
        }
    }

    fn show_stockfish(&mut self, ui: &mut Ui) {
        ui.heading("Stockfish");
    }
}
