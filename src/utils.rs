use std::fmt;

use bevy::{ecs::schedule::StateData, prelude::*};

pub trait StateExts<S> {
    fn transition_overwrite(&mut self, state: S);
    fn transition(&mut self, state: S);
}

impl<S: StateData + Copy + fmt::Display> StateExts<S> for State<S> {
    fn transition_overwrite(&mut self, state: S) {
        self.overwrite_set(state).unwrap_or_else(|e| panic!("Failed to set state {state}: {e}"));
    }

    fn transition(&mut self, state: S) {
        self.set(state).unwrap_or_else(|e| panic!("Failed to set state {state}: {e}"));
    }
}

pub trait AppPushOrderedStartupStages {
    fn push_ordered_startup_stages<I: IntoIterator<Item = (impl StageLabel + Clone, SystemStage)>>(
        &mut self,
        labels_and_stages: I,
    ) -> &mut Self;
}

impl AppPushOrderedStartupStages for App {
    fn push_ordered_startup_stages<
        I: IntoIterator<Item = (impl StageLabel + Clone, SystemStage)>,
    >(
        &mut self,
        labels_and_stages: I,
    ) -> &mut Self {
        let mut labels_and_stages = labels_and_stages.into_iter();
        let first = labels_and_stages.next().expect("list is empty");
        self.add_startup_stage_after(StartupStage::Startup, first.0.clone(), first.1);
        labels_and_stages.into_iter().fold(first.0, |last_label, (label, stage)| {
            self.add_startup_stage_after(last_label, label.clone(), stage);
            label
        });
        self
    }
}

pub struct DebugBevyInspectorPlugin;

impl Plugin for DebugBevyInspectorPlugin {
    #[cfg(feature = "bevy-inspector-egui")]
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin);
    }

    #[cfg(not(feature = "bevy-inspector-egui"))]
    fn build(&self, _app: &mut App) {}
}

#[cfg(feature = "bevy-inspector-egui")]
#[macro_export]
macro_rules! debug_name {
    ($($name_args:expr),+) => {
        Name::new(format!($($name_args),+))
    };
}

#[cfg(not(feature = "bevy-inspector-egui"))]
#[macro_export]
macro_rules! debug_name {
    ($name_args:tt) => {
        ()
    };
}
