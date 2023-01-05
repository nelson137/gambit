use bevy::prelude::*;

#[derive(Resource)]
pub struct GameAudioHandles {
    pub capture: Handle<AudioSource>,
    pub castle: Handle<AudioSource>,
    pub move_opponent: Handle<AudioSource>,
    pub move_self: Handle<AudioSource>,
}

impl FromWorld for GameAudioHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            capture: asset_server.load("audio/capture.flac"),
            castle: asset_server.load("audio/castle.flac"),
            move_opponent: asset_server.load("audio/move-opponent.flac"),
            move_self: asset_server.load("audio/move-self.flac"),
        }
    }
}
