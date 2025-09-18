use bevy::prelude::*;

use crate::utils::NoopExts;

use super::board::{BoardState, MovePieceCompleted, PieceColor};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Observers
            .add_observer(PlayGameAudio::handle)
            .add_observer(PlayGameAudio::on_piece_move_completed)
            .noop();
    }
}

/// A `MoveCheck` variant is intentionally absent as the check audio should never be explicitly
/// played. The variant for the the move type should be used; the board will be examined and if
/// there are any checkers than the check audio will be played instead of the one specified.
#[derive(Event, Debug)]
enum PlayGameAudio {
    Capture,
    Castle,
    // MoveCheck,
    MoveOpponent,
    MoveSelf,
    Promote,
}

impl PlayGameAudio {
    fn on_piece_move_completed(trigger: Trigger<MovePieceCompleted>, mut commands: Commands) {
        commands.trigger(if trigger.is_promotion {
            PlayGameAudio::Promote
        } else if trigger.is_capture {
            PlayGameAudio::Capture
        } else if trigger.is_castle {
            PlayGameAudio::Castle
        } else {
            match trigger.color {
                PieceColor::BLACK => PlayGameAudio::MoveOpponent,
                PieceColor::WHITE => PlayGameAudio::MoveSelf,
            }
        });
    }

    fn handle(
        trigger: Trigger<Self>,
        mut commands: Commands,
        board_state: Res<BoardState>,
        asset_server: Res<AssetServer>,
    ) {
        trace!(action = ?*trigger, "Play audio");

        let checkers = *board_state.board().checkers();

        let path = if checkers != chess::EMPTY {
            "audio/move-check.flac"
        } else {
            match *trigger {
                Self::Capture => "audio/capture.flac",
                Self::Castle => "audio/castle.flac",
                Self::MoveOpponent => "audio/move-opponent.flac",
                Self::MoveSelf => "audio/move-self.flac",
                Self::Promote => "audio/promote.flac",
            }
        };

        let source = asset_server.load::<AudioSource>(path);
        commands.spawn((AudioPlayer(source), PlaybackSettings::DESPAWN));
    }
}
