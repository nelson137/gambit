use bevy::{
    ecs::{observer::TriggerEvent, world::Command},
    prelude::*,
};
use chess::File;

use crate::{
    game::{audio::PlayGameAudio, board::AnimatePiece, game_over::GameOver, mouse::Dragging},
    utils::NoopExts,
};

use super::{
    BoardState, Captured, PieceColor, PieceMeta, PieceType, PromotingPiece, SelectionEvent, Square,
};

#[derive(Debug)]
pub struct MovePlugin;

impl Plugin for MovePlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Events
            .add_event::<MovePiece>()
            .add_event::<MovePieceCompleted>()
            // Observers
            .observe(move_piece)
            .noop();
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct MovePiece {
    from_sq: Square,
    to_sq: Square,
    promotion: Option<PieceType>,
    animate: bool,
}

impl MovePiece {
    pub fn new(
        from_sq: Square,
        to_sq: Square,
        promotion: Option<PieceType>,
        animate: bool,
    ) -> Self {
        Self { from_sq, to_sq, promotion, animate }
    }
}

pub fn move_piece(
    trigger: Trigger<MovePiece>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_state: Res<BoardState>,
    mut q_info: Query<(&PieceMeta, &mut UiImage)>,
) {
    let entity = trigger.entity();
    let &MovePiece { from_sq, to_sq, promotion, animate } = trigger.event();
    let Ok((&PieceMeta { color, typ }, mut image)) = q_info.get_mut(entity) else { return };

    trace!(?color, ?typ, %from_sq, %to_sq, ?promotion, "Move piece");

    if let Some(promo_typ) = promotion {
        // Update the piece texture
        let new_asset_path = PieceMeta::new(color, promo_typ).asset_path();
        image.texture = asset_server.load(new_asset_path);
    } else if typ == PieceType::PAWN && to_sq.get_rank() == color.to_their_backrank() {
        // Start promotion
        commands.entity(entity).insert(PromotingPiece::new(from_sq, to_sq));
        return;
    }

    // Clear selection & hints, update last move highlights
    commands.trigger(SelectionEvent::Unselect);
    commands.trigger(SelectionEvent::UpdateLastMove(from_sq, to_sq));

    // Update piece maps
    commands.add(UpdatePieceState::new(color, from_sq, to_sq));

    // Move UI piece
    commands.entity(entity).remove::<Dragging>();
    commands.add(MoveUiPiece::new(entity, from_sq, to_sq, animate));

    let is_capture = board_state.has_piece_at(to_sq);
    let is_en_passant = board_state.move_is_en_passant(color, to_sq);

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
            commands.add(UpdatePieceState::new(color, from_sq, to_sq));
            commands.add(MoveUiPiece::new(entity, from_sq, to_sq, true));
            is_castle = true;
        } else if castle_rights.has_queenside() && to_sq == queenside_sq {
            let from_sq = Square::from_coords(back_rank, File::A);
            let to_sq = Square::from_coords(back_rank, File::D);
            let entity = board_state.piece(from_sq);
            commands.add(UpdatePieceState::new(color, from_sq, to_sq));
            commands.add(MoveUiPiece::new(entity, from_sq, to_sq, true));
            is_castle = true;
        }
    }

    commands.add(UpdateBoardState::new(from_sq, to_sq, color, typ, promotion, is_capture));

    // Play audio
    commands.add(if promotion.is_some() {
        PlayGameAudio::Promote
    } else if is_capture || is_en_passant {
        PlayGameAudio::Capture
    } else if is_castle {
        PlayGameAudio::Castle
    } else {
        match color {
            PieceColor::BLACK => PlayGameAudio::MoveOpponent,
            PieceColor::WHITE => PlayGameAudio::MoveSelf,
        }
    });

    commands.add(|world: &mut World| {
        TriggerEvent { event: MovePieceCompleted, targets: () }.apply(world);
    });
}

pub struct MoveUiPiece {
    entity: Entity,
    from_sq: Square,
    to_sq: Square,
    animate: bool,
}

impl MoveUiPiece {
    pub fn new(entity: Entity, from_sq: Square, to_sq: Square, animate: bool) -> Self {
        Self { entity, from_sq, to_sq, animate }
    }
}

impl Command for MoveUiPiece {
    fn apply(self, world: &mut World) {
        let Self { entity, from_sq, to_sq, animate } = self;
        trace!(to_sq = %to_sq, "Move UI piece");

        if let Some(mut square) = world.entity_mut(entity).get_mut::<Square>() {
            *square = to_sq;
        }

        if animate {
            AnimatePiece::new(entity, from_sq, to_sq).apply(world);
        } else {
            let to_tile_entity = world.resource::<BoardState>().tile(to_sq);
            world.entity_mut(to_tile_entity).push_children(&[entity]);
        }
    }
}

struct UpdatePieceState {
    color: PieceColor,
    from_sq: Square,
    to_sq: Square,
}

impl UpdatePieceState {
    fn new(color: PieceColor, from_sq: Square, to_sq: Square) -> Self {
        Self { color, from_sq, to_sq }
    }
}

impl Command for UpdatePieceState {
    fn apply(self, world: &mut World) {
        let Self { color, from_sq, to_sq } = self;
        let captured = world.resource_mut::<BoardState>().update_piece(color, from_sq, to_sq);
        if let Some(piece) = captured {
            world.trigger_targets(Captured, piece);
        }
    }
}

struct UpdateBoardState {
    from_sq: Square,
    to_sq: Square,
    color: PieceColor,
    typ: PieceType,
    promotion: Option<PieceType>,
    is_capture: bool,
}

impl UpdateBoardState {
    fn new(
        from_sq: Square,
        to_sq: Square,
        color: PieceColor,
        typ: PieceType,
        promotion: Option<PieceType>,
        is_capture: bool,
    ) -> Self {
        Self { from_sq, to_sq, color, typ, promotion, is_capture }
    }
}

impl Command for UpdateBoardState {
    fn apply(self, world: &mut World) {
        let Self { from_sq, to_sq, color, typ, promotion, is_capture } = self;
        let mut board_state = world.resource_mut::<BoardState>();

        // Update `chess::Board`
        board_state.make_board_move(from_sq, to_sq, promotion);

        if typ == PieceType::PAWN || is_capture {
            board_state.reset_half_move_clock();
        } else {
            board_state.inc_half_move_clock();
        }

        if color == PieceColor::BLACK {
            board_state.inc_full_move_count();
        }

        board_state.sync_status();

        if board_state.is_game_over() {
            GameOver.apply(world);
        }
    }
}

#[derive(Event)]
pub struct MovePieceCompleted;
