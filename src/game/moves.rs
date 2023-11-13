use bevy::{ecs::system::Command, prelude::*};
use chess::File;

use crate::game::utils::WorldExts;

use super::{
    audio::PlayGameAudio,
    board::{BoardState, PieceColor, PieceType, SelectionEvent, Square, StartPromotion, UiPiece},
    captures::Captured,
    game_over::GameOver,
    utils::GameCommandList,
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

        commands.entity(entity).remove::<StartMove>();

        if typ == PieceType::PAWN && to_sq.get_rank() == color.to_their_backrank() {
            commands.add(StartPromotion::new(entity, color, from_sq, to_sq));
        } else {
            commands.add(MovePiece::new(entity, color, typ, from_sq, to_sq, None));
        }
    }
}

pub struct MovePiece {
    entity: Entity,
    color: PieceColor,
    typ: PieceType,
    from_sq: Square,
    to_sq: Square,
    promotion: Option<PieceType>,
}

impl MovePiece {
    pub fn new(
        entity: Entity,
        color: PieceColor,
        typ: PieceType,
        from_sq: Square,
        to_sq: Square,
        promotion: Option<PieceType>,
    ) -> Self {
        Self { entity, color, typ, from_sq, to_sq, promotion }
    }
}

impl Command for MovePiece {
    fn apply(self, world: &mut World) {
        let Self { entity, color, typ, from_sq, to_sq, promotion } = self;
        trace!(?color, ?typ, %from_sq, %to_sq, ?promotion, "Move piece");

        let mut board_state = world.resource_mut::<BoardState>();
        let mut cmd_list = GameCommandList::default();

        // Move UI piece
        cmd_list.add(MoveUiPiece::new(entity, to_sq));

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
                cmd_list.add(MoveUiPiece::new(entity, to_sq));
                is_castle = true;
            } else if castle_rights.has_queenside() && to_sq == queenside_sq {
                let from_sq = Square::from_coords(back_rank, File::A);
                let to_sq = Square::from_coords(back_rank, File::D);
                let entity = board_state.piece(from_sq);
                cmd_list.add(MoveUiPiece::new(entity, to_sq));
                is_castle = true;
            }
        }

        // Update piece maps
        let captured_piece = board_state.update_piece(color, from_sq, to_sq);
        if let Some(entity) = captured_piece {
            let UiPiece { color, typ } = world.entity_piece_info(entity);
            cmd_list.add(Captured::new(entity, color, typ));
        }

        // Play audio
        cmd_list.add(if promotion.is_some() {
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
        world.send_event_batch([
            SelectionEvent::Unselect,
            SelectionEvent::UpdateLastMove(from_sq, to_sq),
        ]);

        let mut board_state = world.resource_mut::<BoardState>();

        // Update `chess::Board`
        board_state.make_board_move(from_sq, to_sq, promotion);

        if board_state.is_game_over() {
            cmd_list.add(GameOver);
        }

        cmd_list.apply(world);
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
