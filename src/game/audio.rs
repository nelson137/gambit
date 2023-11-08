use bevy::{ecs::system::Command, prelude::*};
use chess::BitBoard;

use crate::game::board::BoardState;

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
    Promote,
}

impl Command for PlayGameAudio {
    fn apply(self, world: &mut World) {
        trace!(action = ?self, "Play audio");

        let checkers = *world.resource::<BoardState>().board().checkers();

        let source = world.resource::<AssetServer>().load(match self {
            _ if checkers != BitBoard::new(0) => "audio/move-check.flac",
            Self::Capture => "audio/capture.flac",
            Self::Castle => "audio/castle.flac",
            Self::MoveOpponent => "audio/move-opponent.flac",
            Self::MoveSelf => "audio/move-self.flac",
            Self::Promote => "audio/promote.flac",
        });
        world.spawn(AudioBundle { source, settings: PlaybackSettings::DESPAWN });
    }
}
