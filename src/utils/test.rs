use bevy::{
    app::{App, Plugin},
    asset::AssetApp,
};

use crate::utils::NoopExts;

pub struct GameTestPlugin;

impl Plugin for GameTestPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "debug")]
        app.add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::TRACE,
            ..Default::default()
        });

        app.noop()
            .add_plugins(bevy::app::TaskPoolPlugin::default())
            .add_plugins(bevy::asset::AssetPlugin::default())
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::image::ImagePlugin::default())
            .add_plugins(crate::utils::SortableChildrenPlugin)
            .init_asset::<bevy::text::Font>()
            .noop();
    }
}
