use bevy::{
    asset::{embedded_path, io::embedded::EmbeddedAssetRegistry},
    prelude::*,
};

macro_rules! asset_data {
    ($asset_path:literal) => {
        (
            embedded_path!(concat!("assets/", $asset_path)),
            include_bytes!(concat!("../assets/", $asset_path)),
        )
    };
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        let embedded = app.world_mut().resource_mut::<EmbeddedAssetRegistry>();

        #[allow(clippy::single_element_loop)]
        for (virtual_path, bytes) in [asset_data!("fonts/Montserrat.ttf")] {
            embedded.insert_asset(Default::default(), &virtual_path, bytes);
        }
    }
}
