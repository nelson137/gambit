use bevy::prelude::*;
use chess::File;

use crate::{
    game::{board::AnimatePiece, game_over::GameOver, mouse::Dragging},
    utils::NoopExts,
};

use super::{
    BoardState, Captured, GameStatus, PieceColor, PieceMeta, PieceType, PiecesMap, PromotingPiece,
    Square, TilesMap,
};

#[derive(Debug)]
pub struct MovePlugin;

impl Plugin for MovePlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Observers
            .add_observer(move_piece)
            .noop();
    }
}

#[derive(Event, Debug, PartialEq, Eq)]
pub struct MovePiece {
    pub from_sq: Square,
    pub to_sq: Square,
    pub promotion: Option<PieceType>,
    pub animate: bool,
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
    pieces_map: Res<PiecesMap>,
    board_state: Res<BoardState>,
    mut q_info: Query<(&PieceMeta, &mut ImageNode)>,
) {
    let entity = trigger.target();
    let &MovePiece { from_sq, to_sq, promotion, animate } = trigger.event();
    let Ok((&PieceMeta { color, typ }, mut image)) = q_info.get_mut(entity) else { return };

    trace!(?color, ?typ, %from_sq, %to_sq, ?promotion, "Move piece");

    if let Some(promo_typ) = promotion {
        // Update the piece texture
        let new_asset_path = PieceMeta::new(color, promo_typ).asset_path();
        image.image = asset_server.load(new_asset_path);
    } else if typ == PieceType::PAWN && to_sq.get_rank() == color.to_their_backrank() {
        // Start promotion
        commands.entity(entity).insert(PromotingPiece::new(from_sq, to_sq));
        return;
    }

    // Update piece maps
    commands.queue(UpdatePieceState::new(color, from_sq, to_sq));

    // Move UI piece
    commands.entity(entity).remove::<Dragging>();
    commands.queue(MoveUiPiece::new(entity, from_sq, to_sq, animate));

    let is_capture = pieces_map.has(to_sq);
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
            let entity = pieces_map.at(from_sq);
            commands.queue(UpdatePieceState::new(color, from_sq, to_sq));
            commands.queue(MoveUiPiece::new(entity, from_sq, to_sq, true));
            is_castle = true;
        } else if castle_rights.has_queenside() && to_sq == queenside_sq {
            let from_sq = Square::from_coords(back_rank, File::A);
            let to_sq = Square::from_coords(back_rank, File::D);
            let entity = pieces_map.at(from_sq);
            commands.queue(UpdatePieceState::new(color, from_sq, to_sq));
            commands.queue(MoveUiPiece::new(entity, from_sq, to_sq, true));
            is_castle = true;
        }
    }

    commands.queue(UpdateBoardState::new(from_sq, to_sq, color, typ, promotion, is_capture));

    commands.trigger(MovePieceCompleted {
        is_promotion: promotion.is_some(),
        is_capture: is_capture || is_en_passant,
        is_castle,
        color,
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
            let to_tile_entity = world.resource::<TilesMap>().at(to_sq);
            world.entity_mut(to_tile_entity).add_children(&[entity]);
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
        world.resource_scope(|world, mut pieces_map: Mut<PiecesMap>| {
            let mut board_state = world.resource_mut::<BoardState>();
            let captured = board_state.update_piece(&mut pieces_map, color, from_sq, to_sq);
            if let Some(piece) = captured {
                world.trigger_targets(Captured, piece);
            }
        });
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

        let game_status = board_state.game_status();
        *world.resource_mut::<GameStatus>() = game_status;

        if game_status.is_game_over() {
            world.trigger(GameOver);
        }
    }
}

#[derive(Event)]
pub struct MovePieceCompleted {
    pub is_promotion: bool,
    pub is_capture: bool,
    pub is_castle: bool,
    pub color: PieceColor,
}
