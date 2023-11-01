use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
};
use chess::{ALL_COLORS, ALL_PIECES};

use crate::{game::menu::MenuState, utils::StateExts};

use super::{
    board::{PieceColor, PieceType, UiPiece},
    captures::{CapStateDiff, CapStateUpdate, GameCaptures},
    spawn_pieces, BoardState,
};

pub struct DespawnPieces;

impl Command for DespawnPieces {
    fn write(self, world: &mut World) {
        let mut state = SystemState::<(Commands, Query<Entity, With<UiPiece>>)>::new(world);
        let (mut commands, q_pieces) = state.get_mut(world);
        q_pieces.for_each(|e| commands.entity(e).despawn_recursive());
        state.apply(world);
    }
}

pub struct LoadGame(pub chess::Board);

impl Command for LoadGame {
    fn write(self, world: &mut World) {
        let mut board_state = world.resource_mut::<BoardState>();
        board_state.clear_pieces();
        board_state.set_board(&self.0);

        DespawnPieces.write(world);

        let mut system_state =
            SystemState::<(Commands, Res<AssetServer>, ResMut<BoardState>)>::new(world);
        spawn_pieces.run((), system_state.get_mut(world));
        system_state.apply(world);

        let mut system_state = SystemState::<(Commands, Res<BoardState>)>::new(world);
        load_capture_state.run((), system_state.get_mut(world));
        system_state.apply(world);

        world.resource_mut::<State<MenuState>>().transition_replace(MenuState::Game);
    }
}

fn load_capture_state(mut commands: Commands, board_state: Res<BoardState>) {
    let board = board_state.board();

    let capture_counts = update_capture_counts(board);

    for color in ALL_COLORS {
        let color = PieceColor(color);
        for typ in ALL_PIECES {
            if typ == PieceType::KING {
                continue;
            }
            let typ = PieceType(typ);
            let count = capture_counts[color][typ];
            let diff = CapStateDiff::Set(count);
            commands.add(CapStateUpdate::new(color, typ, diff));
        }
    }
}

const PAWNS_COUNT: u8 = 8;
const KNIGHTS_COUNT: u8 = 2;
const BISHOPS_COUNT: u8 = 2;
const ROOKS_COUNT: u8 = 2;
const QUEENS_COUNT: u8 = 1;

fn update_capture_counts(board: &chess::Board) -> GameCaptures<u8> {
    let mut capture_counts = GameCaptures::<u8>::default();
    let kings_bb = board.pieces(chess::Piece::King);

    for color in ALL_COLORS {
        let color_pieces_bb = *board.color_combined(color) & !kings_bb;
        let color = !color;
        for square in color_pieces_bb {
            match board.piece_on(square) {
                Some(typ) => capture_counts[PieceColor(color)][PieceType(typ)] += 1,
                _ => error!("Failed to get piece at {square} on new board"),
            }
        }
    }

    #[inline]
    fn inverse_count(counts: &mut GameCaptures<u8>, color: PieceColor, typ: PieceType, max: u8) {
        counts[color][typ] = max - counts[color][typ];
    }

    for color in ALL_COLORS.map(PieceColor) {
        inverse_count(&mut capture_counts, color, PieceType::PAWN, PAWNS_COUNT);
        inverse_count(&mut capture_counts, color, PieceType::KNIGHT, KNIGHTS_COUNT);
        inverse_count(&mut capture_counts, color, PieceType::BISHOP, BISHOPS_COUNT);
        inverse_count(&mut capture_counts, color, PieceType::ROOK, ROOKS_COUNT);
        inverse_count(&mut capture_counts, color, PieceType::QUEEN, QUEENS_COUNT);
    }

    capture_counts
}
