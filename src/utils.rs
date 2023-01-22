use std::fmt;

use bevy::{ecs::schedule::StateData, prelude::*};
use bevy_egui::egui::{TextStyle, Ui};

pub trait StateExts<S> {
    fn transition_overwrite(&mut self, state: S);
    fn transition(&mut self, state: S);
    fn transition_push(&mut self, state: S);
    fn transition_pop(&mut self);
    fn transition_replace(&mut self, state: S);
}

impl<S: StateData + Copy + fmt::Display> StateExts<S> for State<S> {
    fn transition_overwrite(&mut self, state: S) {
        self.overwrite_set(state).unwrap_or_else(|e| panic!("Failed to set state {state}: {e}"));
    }

    fn transition(&mut self, state: S) {
        self.set(state).unwrap_or_else(|e| panic!("Failed to set state {state}: {e}"));
    }

    fn transition_push(&mut self, state: S) {
        self.push(state).unwrap_or_else(|e| panic!("Failed to push state {state}: {e}"));
    }

    fn transition_pop(&mut self) {
        self.pop().unwrap_or_else(|e| panic!("Failed to pop state: {e}"));
    }

    fn transition_replace(&mut self, state: S) {
        self.replace(state).unwrap_or_else(|e| panic!("Failed to replace state {state}: {e}"));
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
        app.add_plugin(bevy_inspector_egui::DefaultInspectorConfigPlugin)
            .add_system(Self::world_inspector_ui);
    }

    #[cfg(not(feature = "bevy-inspector-egui"))]
    fn build(&self, _app: &mut App) {}
}

impl DebugBevyInspectorPlugin {
    #[cfg(feature = "bevy-inspector-egui")]
    fn world_inspector_ui(world: &mut World) {
        use bevy_inspector_egui::{
            bevy_egui::{egui, EguiContext},
            bevy_inspector::ui_for_world,
        };

        const DEFAULT_SIZE: (f32, f32) = (300.0, 200.0);

        let ctx = world.resource_mut::<EguiContext>().ctx_mut().clone();
        egui::Window::new("World Inspector").default_size(DEFAULT_SIZE).show(&ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
    }
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

pub trait UiSetTextStyleSize {
    fn set_text_style_size(&mut self, style: &TextStyle, size: f32);
}

impl UiSetTextStyleSize for &mut Ui {
    fn set_text_style_size(&mut self, style: &TextStyle, size: f32) {
        if let Some(text_id) = self.style_mut().text_styles.get_mut(style) {
            text_id.size = size;
        }
    }
}
