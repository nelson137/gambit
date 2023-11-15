use bevy::{ecs::system::Command, prelude::*};
use chess::File;

use super::{
    audio::PlayGameAudio,
    board::{BoardState, PieceColor, PieceType, PromotingPiece, SelectionEvent, Square, UiPiece},
    captures::Captured,
    game_over::GameOver,
};

#[derive(Component)]
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
    q_added: Query<(Entity, &UiPiece, &StartMove), Added<StartMove>>,
) {
    for (entity, &UiPiece { color, typ }, &StartMove { from_sq, to_sq }) in &q_added {
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
    q_added: Query<(Entity, &UiPiece, &MovePiece), Added<MovePiece>>,
    q_pieces: Query<Entity, With<UiPiece>>,
) {
    for (entity, &UiPiece { color, typ }, &MovePiece { from_sq, to_sq, promotion }) in &q_added {
        trace!(?color, ?typ, %from_sq, %to_sq, ?promotion, "Move piece");

        // Move UI piece
        commands.add(MoveUiPiece::new(entity, to_sq));

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
                commands.add(MoveUiPiece::new(entity, to_sq));
                is_castle = true;
            } else if castle_rights.has_queenside() && to_sq == queenside_sq {
                let from_sq = Square::from_coords(back_rank, File::A);
                let to_sq = Square::from_coords(back_rank, File::D);
                let entity = board_state.piece(from_sq);
                commands.add(MoveUiPiece::new(entity, to_sq));
                is_castle = true;
            }
        }

        // Update piece maps
        let captured_piece = board_state.update_piece(color, from_sq, to_sq);
        if let Some(entity) = captured_piece.and_then(|entity| q_pieces.get(entity).ok()) {
            commands.entity(entity).insert(Captured);
        }

        // Play audio
        commands.add(if promotion.is_some() {
            PlayGameAudio::Promote
        } else if captured_piece.is_some() {
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
    to_sq: Square,
}

impl MoveUiPiece {
    pub fn new(entity: Entity, to_sq: Square) -> Self {
        Self { entity, to_sq }
    }
}

impl Command for MoveUiPiece {
    fn apply(self, world: &mut World) {
        trace!(to_sq = %self.to_sq, "Move UI piece");

        if let Some(mut square) = world.entity_mut(self.entity).get_mut::<Square>() {
            square.move_to(self.to_sq);
        }

        let board_state = world.resource_mut::<BoardState>();
        let to_tile_entity = board_state.tile(self.to_sq);

        world.entity_mut(to_tile_entity).push_children(&[self.entity]);
    }
}
