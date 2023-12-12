use bevy::{
    app::prelude::*, ecs::prelude::*, prelude::Deref, reflect::TypeRegistry, utils::default,
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{
        vec2, Align, Context, FontFamily, FontId, Key, Label, Layout, RichText, ScrollArea,
        TextEdit, Ui,
    },
    EguiContext, EguiSet,
};
use bevy_inspector_egui::bevy_inspector::{
    by_type_id::ui_for_resource,
    hierarchy::{hierarchy_ui, SelectedEntities},
    ui_for_entities_shared_components, ui_for_entity,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

use crate::{
    game::stockfish::{SfCommand, SfCommunications, SfMessage, Stockfish},
    utils::AppNoop,
};

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
            .init_resource::<DebugInspectorIsUsingMouse>()
            .add_systems(Update, debug_inspector_update.after(EguiSet::BeginFrame))
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

#[derive(Resource)]
struct InspectorState {
    left_state: SplitPanelState<Pane>,
    right_state: SplitPanelState<Pane>,
    selected_entities: SelectedEntities,
    sf_text_edit_model: String,
}

impl Default for InspectorState {
    fn default() -> Self {
        let left_state = SplitPanelState::equally_sized([Pane::Hierarchy, Pane::EntityComponents]);
        let right_state = SplitPanelState::equally_sized([Pane::Resources, Pane::Stockfish]);
        Self {
            left_state,
            right_state,
            selected_entities: default(),
            sf_text_edit_model: default(),
        }
    }
}

impl InspectorState {
    fn ui(&mut self, world: &mut World, ctx: &mut Context) {
        let type_registry = world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        let mut panel_viewer = InspectorPaneViewer {
            world,
            type_registry: &type_registry,
            selected_entities: &mut self.selected_entities,
            sf_text_edit_model: &mut self.sf_text_edit_model,
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
    sf_text_edit_model: &'a mut String,
}

impl<'a> PaneViewer for InspectorPaneViewer<'a> {
    type Pane = Pane;

    fn ui(&mut self, ui: &mut Ui, pane: &mut Self::Pane) {
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
        ui.separator();

        let sf_comms = self.world.resource::<SfCommunications>();

        let mut user_command: Option<SfCommand> = None;

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            // Fix layout bug that adds scrollbar to whole pane
            ui.add_space(1.0);

            ui.horizontal(|ui| {
                let output = TextEdit::singleline(self.sf_text_edit_model)
                    .min_size(vec2(ui.available_width(), 0.0))
                    .hint_text("Enter a stockfish command")
                    .show(ui);
                if output.response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    user_command = Some(SfCommand::Custom({
                        let cmd_str = self.sf_text_edit_model.trim();
                        let mut s = String::with_capacity(cmd_str.len() + 1);
                        s.push_str(cmd_str);
                        s.push('\n');
                        s
                    }));
                    self.sf_text_edit_model.clear();
                }
            });

            ui.add_space(4.0);

            let layout = Layout::top_down(Align::Min);
            let ui = &mut ui.child_ui(ui.available_rect_before_wrap(), layout);

            let font_id = FontId::new(10.0, FontFamily::Monospace);
            let text_height = ui.fonts(|f| f.row_height(&font_id));

            let scroll_area = ScrollArea::both().auto_shrink([false; 2]).stick_to_bottom(true);
            scroll_area.show_rows(ui, text_height, sf_comms.len(), |ui, range| {
                for i in range {
                    let text = match &sf_comms[i] {
                        SfMessage::Command(cmd) => RichText::new(cmd.to_str().trim()).strong(),
                        SfMessage::Response(res) => RichText::new(res.trim()),
                    };
                    let text = text.font(font_id.clone());
                    ui.add(Label::new(text).wrap(false));
                }
            });
        });

        if let Some(command) = user_command {
            self.world.resource_mut::<Stockfish>().push_cmd(command);
        }
    }
}
