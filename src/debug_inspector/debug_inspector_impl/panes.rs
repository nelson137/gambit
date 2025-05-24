use bevy::{ecs::prelude::*, reflect::TypeRegistryArc};
use bevy_egui::egui::{
    Align, FontFamily, FontId, Key, Label, Layout, Response, RichText, ScrollArea, Shape, TextEdit,
    TextWrapMode, Ui, UiBuilder, collapsing_header::CollapsingState, emath::Rot2, remap, vec2,
};
use bevy_inspector_egui::bevy_inspector::{
    by_type_id::ui_for_resource,
    guess_entity_name,
    hierarchy::{SelectedEntities, SelectionMode},
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
    pub hierarchy_hover: Option<Entity>,
    pub selected_entities: SelectedEntities,
    pub sf_text_edit: String,
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

        let mut q = self.world.query_filtered::<Entity, Without<ChildOf>>();
        let mut entities: Vec<_> = q.iter(self.world).collect();
        entities.sort();

        for &entity in &entities {
            self.show_hierarchy_entity(ui, entity, &entities);
        }
    }

    fn show_hierarchy_entity(&mut self, ui: &mut Ui, entity: Entity, local_generation: &[Entity]) {
        let selected = self.state.selected_entities.contains(entity);

        let name = guess_entity_name(self.world, entity);

        let children =
            self.world.get::<Children>(entity).map(|c| c.to_vec()).filter(|c| !c.is_empty());
        let has_children = children.is_some();

        let heading_id = ui.make_persistent_id(("hierarchy-item", &name));
        let mut collapsing = CollapsingState::load_with_default_open(ui.ctx(), heading_id, false);

        let header_response = ui.horizontal(|ui| {
            let prev_item_spacing = ui.spacing_mut().item_spacing;
            ui.spacing_mut().item_spacing.x = 2.0;
            let collapser = collapsing.show_toggle_button(ui, move |ui, openness, response| {
                if has_children {
                    show_collapsing_icon(ui, openness, response);
                }
            });
            ui.spacing_mut().item_spacing = prev_item_spacing;

            let mut label = RichText::new(&name);
            if selected {
                label = label.strong();
            }
            let label = ui.selectable_label(selected, label);

            if label.hovered() {
                self.state.hierarchy_hover = Some(entity);
            }

            if label.clicked() {
                let mode = ui.input(|i| {
                    SelectionMode::from_ctrl_shift(i.modifiers.command, i.modifiers.shift)
                });
                self.state.selected_entities.select(mode, entity, |from, to| {
                    let is_boundary = |&e| e == from || e == to;
                    let Some(from_i) = local_generation.iter().position(is_boundary) else {
                        return [].iter().copied();
                    };
                    let Some(to_offset) =
                        local_generation[from_i + 1..].iter().position(is_boundary)
                    else {
                        return [].iter().copied();
                    };
                    let to_i = from_i + to_offset + 1;
                    local_generation[from_i..=to_i].iter().copied()
                });
            }

            collapser
        });

        if let Some(children) = children {
            collapsing.show_body_indented(&header_response.response, ui, |ui| {
                let prev_indent = ui.spacing_mut().indent;
                ui.spacing_mut().indent = 22.0;
                for &child in &children {
                    self.show_hierarchy_entity(ui, child, &children);
                }
                ui.spacing_mut().indent = prev_indent;
            });
        }
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
            let ui = &mut ui.new_child(
                UiBuilder::new().max_rect(ui.available_rect_before_wrap()).layout(layout),
            );

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
                    ui.add(Label::new(text).wrap_mode(TextWrapMode::Extend));
                }
            });
        });

        if let Some(command) = user_command {
            self.world.resource_mut::<Stockfish>().push_cmd(command);
        }
    }
}

/// Draw a pointy triangle arrow
fn show_collapsing_icon(ui: &mut Ui, openness: f32, response: &Response) {
    let visuals = ui.style().interact(response);

    let stroke = visuals.fg_stroke;

    let rect = response.rect;
    let rect = rect.expand2(rect.size() * -0.125).expand(visuals.expansion);

    use std::f32::consts::TAU;
    let rotation = Rot2::from_angle(remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    points.iter_mut().for_each(|p| *p = rect.center() + rotation * (*p - rect.center()));

    ui.painter().add(Shape::closed_line(points, stroke));
}
