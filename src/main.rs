use bevy::{audio::AudioPlugin, prelude::*};

fn main() {
    App::new()
        // Plugins
        .add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<AudioPlugin>())
        // Run
        .run();
}
