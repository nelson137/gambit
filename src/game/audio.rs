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

#[derive(Event, Debug)]
enum PlayGameAudio {
    Capture,
    Castle,
    MoveCheck,
    MoveOpponent,
    MoveSelf,
    Promote,
}

impl PlayGameAudio {
    fn on_piece_move_completed(
        move_piece_completed: On<MovePieceCompleted>,
        mut commands: Commands,
        board_state: Res<BoardState>,
    ) {
        let checkers = *board_state.board().checkers();

        commands.trigger(if checkers != chess::EMPTY {
            PlayGameAudio::MoveCheck
        } else if move_piece_completed.is_promotion {
            PlayGameAudio::Promote
        } else if move_piece_completed.is_capture {
            PlayGameAudio::Capture
        } else if move_piece_completed.is_castle {
            PlayGameAudio::Castle
        } else {
            match move_piece_completed.color {
                PieceColor::BLACK => PlayGameAudio::MoveOpponent,
                PieceColor::WHITE => PlayGameAudio::MoveSelf,
            }
        });
    }

    fn handle(play_game_audio: On<Self>, mut commands: Commands, asset_server: Res<AssetServer>) {
        trace!(action = ?*play_game_audio, "Play audio");

        let source = asset_server.load::<AudioSource>(match *play_game_audio {
            Self::Capture => "audio/capture.flac",
            Self::Castle => "audio/castle.flac",
            Self::MoveCheck => "audio/move-check.flac",
            Self::MoveOpponent => "audio/move-opponent.flac",
            Self::MoveSelf => "audio/move-self.flac",
            Self::Promote => "audio/promote.flac",
        });

        commands.spawn((AudioPlayer(source), PlaybackSettings::DESPAWN));
    }
}
