use bevy::{ecs::prelude::*, reflect::TypeRegistryArc};
use bevy_egui::egui::{
    vec2, Align, FontFamily, FontId, Key, Label, Layout, RichText, ScrollArea, TextEdit, Ui,
};
use bevy_inspector_egui::bevy_inspector::{
    by_type_id::ui_for_resource,
    hierarchy::{hierarchy_ui, SelectedEntities},
    ui_for_entities_shared_components, ui_for_entity,
};

use crate::game::stockfish::{SfCommand, SfCommunications, SfMessage, Stockfish};

use super::split_panel::PaneViewer;

#[derive(Clone, Copy)]
pub(super) enum Pane {
    Hierarchy,
    EntityComponents,
    Resources,
    Stockfish,
}

#[derive(Default)]
pub(super) struct PanesState {
    selected_entities: SelectedEntities,
    sf_text_edit: String,
}

pub(super) struct InspectorPaneViewer<'a> {
    pub(super) world: &'a mut World,
    pub(super) state: &'a mut PanesState,
}

impl InspectorPaneViewer<'_> {
    fn get_type_registry(&self) -> TypeRegistryArc {
        self.world.resource::<AppTypeRegistry>().0.clone()
    }
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
        hierarchy_ui(self.world, ui, &mut self.state.selected_entities);
    }

    fn show_entity_components(&mut self, ui: &mut Ui) {
        ui.heading("Entity Components");
        ui.separator();

        match self.state.selected_entities.as_slice() {
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

        let type_registry = self.get_type_registry();
        let type_registry = type_registry.read();

        let mut resources = type_registry
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
                ui_for_resource(self.world, type_id, ui, name, &type_registry);
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
                let output = TextEdit::singleline(&mut self.state.sf_text_edit)
                    .min_size(vec2(ui.available_width(), 0.0))
                    .hint_text("Enter a stockfish command")
                    .show(ui);
                if output.response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    user_command = Some(SfCommand::Custom({
                        let cmd_str = self.state.sf_text_edit.trim();
                        let mut s = String::with_capacity(cmd_str.len() + 1);
                        s.push_str(cmd_str);
                        s.push('\n');
                        s
                    }));
                    self.state.sf_text_edit.clear();
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
