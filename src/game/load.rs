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

        world.resource_mut::<State<MenuState>>().transition_replace(MenuState::Game);
    }
}

pub(super) fn load_capture_state(world: &mut World) {
    let board_state = world.get_resource::<BoardState>().unwrap_or_else(|| panic!("TODO"));
    let board = board_state.board();

    let capture_counts = update_capture_counts(&board);

    for color in ALL_COLORS {
        let color = PieceColor(color);
        for typ in ALL_PIECES {
            if typ == chess::Piece::King {
                continue;
            }
            let typ = PieceType(typ);
            let count = capture_counts[color][typ];
            let diff = CapStateDiff::Set(count);
            CapStateUpdate::new(color, typ, diff).write(world);
        }
    }
}

const PAWNS_COUNT: u8 = 8;
const KNIGHTS_COUNT: u8 = 2;
const BISHOPS_COUNT: u8 = 2;
const ROOKS_COUNT: u8 = 2;
const QUEENS_COUNT: u8 = 1;

fn update_capture_counts(board: &chess::Board) -> GameCaptures<u8> {
    use chess::{Color, Piece};

    let mut capture_counts = GameCaptures::<u8>::default();
    let kings_bb = board.pieces(Piece::King);

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
    fn inverse_count(counts: &mut GameCaptures<u8>, color: Color, typ: Piece, max_count: u8) {
        let color = PieceColor(color);
        let typ = PieceType(typ);
        counts[color][typ] = max_count - counts[color][typ];
    }

    for color in ALL_COLORS {
        inverse_count(&mut capture_counts, color, Piece::Pawn, PAWNS_COUNT);
        inverse_count(&mut capture_counts, color, Piece::Knight, KNIGHTS_COUNT);
        inverse_count(&mut capture_counts, color, Piece::Bishop, BISHOPS_COUNT);
        inverse_count(&mut capture_counts, color, Piece::Rook, ROOKS_COUNT);
        inverse_count(&mut capture_counts, color, Piece::Queen, QUEENS_COUNT);
    }

    capture_counts
}
