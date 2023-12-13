use bevy::{ecs::prelude::*, utils::default};
use bevy_egui::egui::Context;
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;

use super::{
    panes::{InspectorPaneViewer, Pane},
    split_panel::{SplitPanel, SplitPanelState},
};

#[derive(Resource)]
pub(super) struct InspectorState {
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
    pub(super) fn ui(&mut self, world: &mut World, ctx: &mut Context) {
        let mut panel_viewer = InspectorPaneViewer {
            world,
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
