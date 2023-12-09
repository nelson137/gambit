use bevy::{prelude::*, reflect::TypeRegistry, window::PrimaryWindow};
use bevy_egui::{
    egui::{
        self, vec2, Align, Context, FontFamily, FontId, Key, Layout, RichText, ScrollArea,
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

        let communications = &[
            "uci",
            "isready",
            "Stockfish 15 by the Stockfish developers (see AUTHORS file)",
            "id name Stockfish 15",
            "id author the Stockfish developers (see AUTHORS file)",
            "option name Debug Log File type string default",
            "option name Threads type spin default 1 min 1 max 512",
            "option name Hash type spin default 16 min 1 max 33554432",
            "option name Clear Hash type button",
            "option name Ponder type check default false",
            "option name MultiPV type spin default 1 min 1 max 500",
            "option name Skill Level type spin default 20 min 0 max 20",
            "option name Move Overhead type spin default 10 min 0 max 5000",
            "option name Slow Mover type spin default 100 min 10 max 1000",
            "option name nodestime type spin default 0 min 0 max 10000",
            "uciok",
            "readyok",
            "ucinewgame",
            "isready",
            "position startpos moves d2d4 g8f6 e2e3 e7e6 g1f3 d7d5 f3e5 c7c5 d1h5",
            "readyok",
            "go infinite",
            "info string NNUE evaluation using nn-6877cd24400e.nnue enabled",
            "info depth 1 seldepth 1 multipv 1 score cp 1115 nodes 34 nps 34000 tbhits 0 time 1 pv f6h5",
            "info depth 2 seldepth 2 multipv 1 score cp 1265 nodes 66 nps 66000 tbhits 0 time 1 pv f6h5 d4c5 f8c5",
            "info depth 3 seldepth 3 multipv 1 score cp 1087 nodes 115 nps 57500 tbhits 0 time 2 pv f6h5 c2c3 f7f6",
            "info depth 4 seldepth 4 multipv 1 score cp 1165 nodes 182 nps 91000 tbhits 0 time 2 pv f6h5 d4c5 f8c5 c2c3",
            "info depth 5 seldepth 5 multipv 1 score cp 1165 nodes 429 nps 214500 tbhits 0 time 2 pv f6h5 b1d2 h5f6 d4c5",
            "info depth 6 seldepth 6 multipv 1 score cp 1158 nodes 1176 nps 294000 tbhits 0 time 4 pv f6h5 b1d2 h5f6 d4c5 f8c5 a2a4",
            "info depth 7 seldepth 7 multipv 1 score cp 1144 nodes 3851 nps 550142 tbhits 0 time 7 pv f6h5 f1b5 b8d7 e5d7 c8d7 b5d7 d8d7",
            "info depth 8 seldepth 10 multipv 1 score cp 1115 nodes 7408 nps 617333 tbhits 0 time 12 pv f6h5 f1b5 c8d7 e5d7 b8d7 e1g1 c5d4 e3d4 a7a6 b5d7 d8d7",
            "info depth 9 seldepth 14 multipv 1 score cp 1138 nodes 14906 nps 709809 tbhits 0 time 21 pv f6h5 f1b5 c8d7 e5d7 b8d7 e1g1 c5d4 e3d4 f8d6 b1d2 a7a6 b5d7 d8d7 f1e1 e8g8",
            "stop",
            "info depth 10 seldepth 17 multipv 1 score cp 1152 nodes 31118 nps 841027 tbhits 0 time 37 pv f6h5 f1b5 c8d7 e5d7 b8d7 e1g1 c5d4 b5d7 e8d7 g2g3 f8c5 a2a4",
            "bestmove f6h5 ponder f1b5",
        ];

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            // Fix layout bug that adds scrollbar to whole pane
            ui.add_space(1.0);

            ui.horizontal(|ui| {
                let output = TextEdit::singleline(self.sf_text_edit_model)
                    .min_size(vec2(ui.available_width(), 0.0))
                    .hint_text("Enter a stockfish command")
                    .show(ui);
                if output.response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    bevy::log::debug!("SUBMIT STOCKFISH COMMAND | {}", self.sf_text_edit_model);
                    self.sf_text_edit_model.clear();
                }
            });

            ui.add_space(4.0);

            let layout = Layout::top_down(Align::Min);
            let ui = &mut ui.child_ui(ui.available_rect_before_wrap(), layout);

            let font_id = FontId::new(10.0, FontFamily::Monospace);
            let row_height = ui.fonts(|f| f.row_height(&font_id));

            ScrollArea::both().show_rows(ui, row_height, communications.len(), |ui, range| {
                for i in range {
                    let text = RichText::new(communications[i].trim()).font(font_id.clone());
                    ui.add(egui::Label::new(text).wrap(false));
                }
            });
        });
    }
}
