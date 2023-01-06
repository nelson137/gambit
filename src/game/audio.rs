use bevy::{ecs::system::Command, prelude::*};
use chess::BitBoard;

use crate::game::board::BoardState;

#[derive(Resource)]
pub struct GameAudioHandles {
    pub capture: Handle<AudioSource>,
    pub castle: Handle<AudioSource>,
    pub move_check: Handle<AudioSource>,
    pub move_opponent: Handle<AudioSource>,
    pub move_self: Handle<AudioSource>,
}

impl FromWorld for GameAudioHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            capture: asset_server.load("audio/capture.flac"),
            castle: asset_server.load("audio/castle.flac"),
            move_check: asset_server.load("audio/move-check.flac"),
            move_opponent: asset_server.load("audio/move-opponent.flac"),
            move_self: asset_server.load("audio/move-self.flac"),
        }
    }
}

/// A `MoveCheck` variant is intentionally absent as the check audio should never be explicitly
/// played. The variant for the the move type should be used; the board will be examined and if
/// there are any checkers than the check audio will be played instead of the one specified.
#[derive(Debug)]
pub enum PlayGameAudio {
    Capture,
    Castle,
    // MoveCheck,
    MoveOpponent,
    MoveSelf,
}

impl Command for PlayGameAudio {
    fn write(self, world: &mut World) {
        let checkers = *world.resource::<BoardState>().board().checkers();

        let audio_handles = world.resource::<GameAudioHandles>();
        let handle = match self {
            _ if checkers != BitBoard::new(0) => &audio_handles.move_check,
            Self::Capture => &audio_handles.capture,
            Self::Castle => &audio_handles.castle,
            Self::MoveOpponent => &audio_handles.move_opponent,
            Self::MoveSelf => &audio_handles.move_self,
        }
        .clone_weak();

        world.resource::<Audio>().play(handle);
    }
}
