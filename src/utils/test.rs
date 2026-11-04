use bevy::{
    app::{App, Plugin},
    asset::AssetApp,
};

use crate::utils::NoopExts;

pub mod random_values {
    use rand::random;

    pub fn color() -> chess::Color {
        if random::<bool>() { chess::Color::White } else { chess::Color::Black }
    }

    pub fn file() -> chess::File {
        chess::File::from_index((random::<f32>() * chess::NUM_FILES as f32) as usize)
    }

    pub fn castle_rights() -> chess::CastleRights {
        let king = random::<bool>() as usize * chess::CastleRights::KingSide.to_index();
        let queen = random::<bool>() as usize * chess::CastleRights::QueenSide.to_index();
        chess::CastleRights::from_index(king | queen)
    }
}

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
            .add_plugins(bevy::scene::ScenePlugin)
            .add_plugins(bevy::input_focus::InputFocusPlugin)
            .add_plugins(bevy::image::ImagePlugin::default())
            .init_asset::<bevy::shader::Shader>()
            .init_asset::<bevy::text::Font>()
            .init_resource::<crate::game::menu::FenInputMenuState>()
            .noop();
    }
}
