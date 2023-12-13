use bevy::{ecs::prelude::*, utils::default};
use bevy_egui::egui::Context;

use super::{
    panes::{InspectorPaneViewer, Pane, PanesState},
    split_panel::{SplitPanel, SplitPanelState},
};

#[derive(Resource)]
pub(super) struct InspectorState {
    left_state: SplitPanelState<Pane>,
    right_state: SplitPanelState<Pane>,
    panes_state: PanesState,
}

impl Default for InspectorState {
    fn default() -> Self {
        let left_state = SplitPanelState::equally_sized([Pane::Hierarchy, Pane::EntityComponents]);
        let right_state = SplitPanelState::equally_sized([Pane::Resources, Pane::Stockfish]);
        Self { left_state, right_state, panes_state: default() }
    }
}

impl InspectorState {
    pub(super) fn ui(&mut self, world: &mut World, ctx: &mut Context) {
        let mut panel_viewer = InspectorPaneViewer { world, state: &mut self.panes_state };

        const DEFAULT_WIDTH: f32 = 318.0;

        SplitPanel::left("Left Panel", &mut self.left_state)
            .default_width(DEFAULT_WIDTH)
            .show(ctx, &mut panel_viewer);

        SplitPanel::right("Right Panel", &mut self.right_state)
            .default_width(DEFAULT_WIDTH)
            .show(ctx, &mut panel_viewer);
    }
}
