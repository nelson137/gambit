use bevy::{ecs::system::Command, prelude::*};
use chess::File;

use crate::game::{audio::PlayGameAudio, game_over::GameOver};

use super::{
    BoardState, Captured, PieceColor, PieceMeta, PieceType, PromotingPiece, SelectionEvent, Square,
};

#[derive(Debug)]
pub struct MovePlugin;

impl Plugin for MovePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (start_move, move_piece));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
pub struct StartMove {
    from_sq: Square,
    to_sq: Square,
}

impl StartMove {
    pub fn new(from_sq: Square, to_sq: Square) -> Self {
        Self { from_sq, to_sq }
    }
}

pub fn start_move(
    mut commands: Commands,
    q_added: Query<(Entity, &PieceMeta, &StartMove), Added<StartMove>>,
) {
    for (entity, &PieceMeta { color, typ }, &StartMove { from_sq, to_sq }) in &q_added {
        trace!(?color, ?typ, %from_sq, %to_sq, "Start move");

        let mut entity_cmds = commands.entity(entity);
        entity_cmds.remove::<StartMove>();

        if typ == PieceType::PAWN && to_sq.get_rank() == color.to_their_backrank() {
            entity_cmds.insert(PromotingPiece::new(from_sq, to_sq));
        } else {
            entity_cmds.insert(MovePiece::new(from_sq, to_sq, None));
        }
    }
}

#[derive(Component)]
pub struct MovePiece {
    from_sq: Square,
    to_sq: Square,
    promotion: Option<PieceType>,
}

impl MovePiece {
    pub fn new(from_sq: Square, to_sq: Square, promotion: Option<PieceType>) -> Self {
        Self { from_sq, to_sq, promotion }
    }
}

pub fn move_piece(
    mut commands: Commands,
    mut board_state: ResMut<BoardState>,
    mut selection_events: EventWriter<SelectionEvent>,
    q_added: Query<(Entity, &PieceMeta, &MovePiece), Added<MovePiece>>,
) {
    for (entity, &PieceMeta { color, typ }, &MovePiece { from_sq, to_sq, promotion }) in &q_added {
        trace!(?color, ?typ, %from_sq, %to_sq, ?promotion, "Move piece");

        commands.entity(entity).remove::<MovePiece>();

        // Move UI piece
        commands.add(MoveUiPiece::new(entity, color, from_sq, to_sq));

        let is_capture = board_state.has_piece_at(to_sq);

        let mut is_castle = false;
        if typ == PieceType::KING {
            let castle_rights = board_state.my_castle_rights();
            let back_rank = color.to_my_backrank();
            let kingside_sq = Square::from_coords(back_rank, File::G);
            let queenside_sq = Square::from_coords(back_rank, File::C);

            // Move UI rook
            if castle_rights.has_kingside() && to_sq == kingside_sq {
                let from_sq = Square::from_coords(back_rank, File::H);
                let to_sq = Square::from_coords(back_rank, File::F);
                let entity = board_state.piece(from_sq);
                commands.add(MoveUiPiece::new(entity, color, from_sq, to_sq));
                is_castle = true;
            } else if castle_rights.has_queenside() && to_sq == queenside_sq {
                let from_sq = Square::from_coords(back_rank, File::A);
                let to_sq = Square::from_coords(back_rank, File::D);
                let entity = board_state.piece(from_sq);
                commands.add(MoveUiPiece::new(entity, color, from_sq, to_sq));
                is_castle = true;
            }
        }

        // Play audio
        commands.add(if promotion.is_some() {
            PlayGameAudio::Promote
        } else if is_capture {
            PlayGameAudio::Capture
        } else if is_castle {
            PlayGameAudio::Castle
        } else {
            match color {
                PieceColor::BLACK => PlayGameAudio::MoveOpponent,
                PieceColor::WHITE => PlayGameAudio::MoveSelf,
            }
        });

        // Clear selection & hints, update last move highlights
        selection_events
            .send_batch([SelectionEvent::Unselect, SelectionEvent::UpdateLastMove(from_sq, to_sq)]);

        // Update `chess::Board`
        board_state.make_board_move(from_sq, to_sq, promotion);

        if board_state.is_game_over() {
            commands.add(GameOver);
        }
    }
}

pub struct MoveUiPiece {
    entity: Entity,
    color: PieceColor,
    from_sq: Square,
    to_sq: Square,
}

impl MoveUiPiece {
    pub fn new(entity: Entity, color: PieceColor, from_sq: Square, to_sq: Square) -> Self {
        Self { entity, color, from_sq, to_sq }
    }
}

impl Command for MoveUiPiece {
    fn apply(self, world: &mut World) {
        let Self { entity, color, from_sq, to_sq } = self;
        trace!(to_sq = %to_sq, "Move UI piece");

        if let Some(mut square) = world.entity_mut(entity).get_mut::<Square>() {
            square.move_to(to_sq);
        }

        let mut board_state = world.resource_mut::<BoardState>();

        let to_tile_entity = board_state.tile(to_sq);

        // Update piece maps
        if let Some(captured_piece) = board_state.update_piece(color, from_sq, to_sq) {
            world.entity_mut(captured_piece).insert(Captured);
        }

        world.entity_mut(to_tile_entity).push_children(&[entity]);
    }
}
