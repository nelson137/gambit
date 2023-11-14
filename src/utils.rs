use bevy::prelude::*;
use bevy_egui::egui::{TextStyle, Ui};

pub trait AppNoop {
    fn noop(&mut self) -> &mut Self;
}

impl AppNoop for App {
    fn noop(&mut self) -> &mut Self {
        self
    }
}

pub struct DebugBevyInspectorPlugin;

impl Plugin for DebugBevyInspectorPlugin {
    #[cfg(feature = "debug-inspector")]
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin)
            .add_systems(Update, Self::world_inspector_ui);
    }

    #[cfg(not(feature = "debug-inspector"))]
    fn build(&self, _app: &mut App) {}
}

impl DebugBevyInspectorPlugin {
    #[cfg(feature = "debug-inspector")]
    fn world_inspector_ui(world: &mut World) {
        use bevy::ecs::system::SystemState;
        use bevy_inspector_egui::{
            bevy_egui::{egui, EguiContexts},
            bevy_inspector::ui_for_world,
        };

        const DEFAULT_SIZE: (f32, f32) = (300.0, 200.0);

        let mut egui_state = SystemState::<EguiContexts>::new(world);
        let ctx = egui_state.get_mut(world).ctx_mut().clone();
        egui::Window::new("World Inspector").default_size(DEFAULT_SIZE).show(&ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui_for_world(world, ui);
                ui.allocate_space(ui.available_size());
            });
        });
    }
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug_name {
    ($name:literal) => {
        Name::new($name)
    };
    ($name:expr) => {
        Name::new($name)
    };
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug_name {
    ($($args:tt),* $(,)?) => {
        ()
    };
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug_name_f {
    ($name_fmt:literal $(, $name_args:expr)* $(,)?) => {
        Name::new(format!($name_fmt $(, $name_args)*))
    };
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug_name_f {
    ($($args:tt),* $(,)?) => {
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

pub trait RoundToNearest {
    fn round_to_nearest(self, step: Self) -> Self;
}

impl RoundToNearest for u32 {
    fn round_to_nearest(self, step: Self) -> Self {
        ((self + (step / 2 as Self)) / step) * step
    }
}
