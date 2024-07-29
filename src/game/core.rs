use bevy::{
    app::{App, Plugin},
    prelude::AssetApp,
    utils::default,
    window::{Window, WindowResolution},
};

use crate::{
    game::consts::{INIT_WIN_HEIGHT, INIT_WIN_WIDTH},
    utils::NoopExts,
    utils::SortableChildrenPlugin,
};

pub struct GameHeadlessPlugin;

impl Plugin for GameHeadlessPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_plugins(bevy::core::TaskPoolPlugin::default())
            .add_plugins(bevy::core::TypeRegistrationPlugin)
            .add_plugins(bevy::time::TimePlugin)
            .add_plugins(bevy::transform::TransformPlugin)
            .add_plugins(bevy::hierarchy::HierarchyPlugin)
            .add_plugins(bevy::input::InputPlugin)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .add_plugins(bevy::audio::AudioPlugin::default())
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(SortableChildrenPlugin)
            .noop();
    }
}

pub struct GameHeadPlugin;

impl Plugin for GameHeadPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_plugins(bevy::core::FrameCountPlugin)
            .add_plugins(bevy::window::WindowPlugin {
                primary_window: Some(Window {
                    title: "Gambit".into(),
                    resolution: WindowResolution::new(INIT_WIN_WIDTH, INIT_WIN_HEIGHT),
                    resizable: true,
                    ..default()
                }),
                ..default()
            })
            .add_plugins(bevy::a11y::AccessibilityPlugin)
            .add_plugins(bevy::winit::WinitPlugin::<bevy::winit::WakeUp>::default())
            .add_plugins(bevy::render::RenderPlugin::default())
            .add_plugins(bevy::render::texture::ImagePlugin::default())
            .add_plugins(bevy::render::pipelined_rendering::PipelinedRenderingPlugin)
            .add_plugins(bevy::core_pipeline::CorePipelinePlugin)
            .add_plugins(bevy::sprite::SpritePlugin)
            .add_plugins(bevy::text::TextPlugin)
            .add_plugins(bevy::ui::UiPlugin)
            .noop();
    }
}

pub struct GameTestPlugin;

impl Plugin for GameTestPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .init_asset::<bevy::render::texture::Image>()
            .init_asset::<bevy::text::Font>()
            .noop();
    }
}
