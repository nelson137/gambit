use core::fmt;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::{Range, RangeInclusive},
};

use bevy::prelude::*;
use grid::Grid;

pub const Z_PIECE_SELECTED: f32 = 1.5;

pub const Z_PIECE: f32 = 1.0;

pub const Z_HIGHLIGHT_TILE: f32 = 0.3;

pub const Z_MOVE_HINT: f32 = 0.2;

pub const Z_NOTATION_TEXT: f32 = 0.1;

pub const Z_TILE: f32 = 0.0;

#[derive(Component)]
pub struct Board;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::rgba(1.0, 1.0, 0.0, 0.5);

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct HighlightTile;

/// The "black" bord color.
///
/// `#769656`
pub const COLOR_BLACK: Color = Color::rgb(
    0x76 as f32 / u8::MAX as f32,
    0x96 as f32 / u8::MAX as f32,
    0x56 as f32 / u8::MAX as f32,
);

/// The "white" bord color.
///
/// `#eeeed2`
pub const COLOR_WHITE: Color = Color::rgb(
    0xee as f32 / u8::MAX as f32,
    0xee as f32 / u8::MAX as f32,
    0xd2 as f32 / u8::MAX as f32,
);

pub const BOARD_TEXT_FONT_SIZE: f32 = 28.0;

const _BOARD_LOCATION_TEXT_OFFSET: f32 = 60.0;

pub const BOARD_FILE_TEXT_OFFSET_X: f32 = _BOARD_LOCATION_TEXT_OFFSET;
pub const BOARD_FILE_TEXT_OFFSET_Y: f32 = -BOARD_FILE_TEXT_OFFSET_X;

pub const BOARD_RANK_TEXT_OFFSET_X: f32 = -BOARD_RANK_TEXT_OFFSET_Y;
pub const BOARD_RANK_TEXT_OFFSET_Y: f32 = _BOARD_LOCATION_TEXT_OFFSET;

#[derive(Default)]
pub struct ShowingMovesFor(pub Option<Location>);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ShowHint;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HideHint;

#[derive(Component)]
pub struct Piece;

#[derive(Clone, Copy, Component, PartialEq, Eq, Debug)]
pub enum PieceColor {
    Black,
    White,
}

#[derive(Clone, Copy, Component, Debug, Eq)]
pub enum PieceType {
    Bishop,
    King { been_in_check: bool },
    Knight,
    Pawn,
    Queen,
    Rook,
}

impl PartialEq for PieceType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Hash for PieceType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Location {
    file: u8,
    rank: u8,
    pub z: f32,
    pub snap: bool,
}

impl Location {
    pub const fn file_to_char(file: u8) -> char {
        (b'a' + file) as char
    }

    pub const fn rank_to_char(rank: u8) -> char {
        (b'0' + rank + 1) as char
    }

    pub const fn new_with_z(file: u8, rank: u8, z: f32) -> Self {
        Self { file, rank, z, snap: true }
    }

    pub const fn new(file: u8, rank: u8) -> Self {
        Self::new_with_z(file, rank, 0.0)
    }

    pub fn with_file(mut self, file: u8) -> Self {
        self.file = file;
        self
    }

    pub fn with_rank(mut self, rank: u8) -> Self {
        self.rank = rank;
        self
    }

    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn file(&self) -> u8 {
        self.file
    }

    pub fn rank(&self) -> u8 {
        self.rank
    }

    pub fn file_char(&self) -> char {
        Self::file_to_char(self.file)
    }

    pub fn rank_char(&self) -> char {
        Self::rank_to_char(self.rank)
    }

    pub fn move_to(&mut self, location: Location) {
        self.file = location.file;
        self.rank = location.rank;
    }

    pub fn try_offset(&self, file_offset: i8, rank_offset: i8) -> Option<Location> {
        let file = self.file as i8 + file_offset;
        let rank = self.rank as i8 + rank_offset;
        const RANGE: Range<i8> = 0..8;
        if RANGE.contains(&file) && RANGE.contains(&rank) {
            Some(Self { file: file as u8, rank: rank as u8, z: self.z, snap: self.snap })
        } else {
            None
        }
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file && self.rank == other.rank
    }
}

impl Eq for Location {}

impl Hash for Location {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Self::file_to_char(self.file).hash(state);
        Self::rank_to_char(self.rank).hash(state);
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}{}",
            Self::file_to_char(self.file),
            Self::rank_to_char(self.rank)
        ))
    }
}

#[derive(Debug)]
pub struct BoardPiece {
    pub color: PieceColor,
    pub typ: PieceType,
    pub did_move: bool,
}

impl BoardPiece {
    pub fn new(color: PieceColor, typ: PieceType) -> Self {
        Self { color, typ, did_move: false }
    }
}

#[derive(Debug)]
pub struct MoveHints {
    pub entity_move: Entity,
    pub entity_capture: Entity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceMoveType {
    Move,
    Capture,
    Castle,
}

#[derive(Clone, Copy)]
pub struct ValidMove {
    location: Location,
    typ: PieceMoveType,
}

impl PartialEq for ValidMove {
    fn eq(&self, other: &Self) -> bool {
        self.location.eq(&other.location)
    }
}

impl Eq for ValidMove {}

impl Hash for ValidMove {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
    }
}

impl ValidMove {
    pub fn new(location: Location, typ: PieceMoveType) -> Self {
        Self { location, typ }
    }
}

#[derive(Eq)]
pub struct PieceMove {
    location: Location,
    can_capture: bool,
}

impl PartialEq for PieceMove {
    fn eq(&self, other: &Self) -> bool {
        self.location.eq(&other.location)
    }
}

impl Hash for PieceMove {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.location.hash(state);
    }
}

impl PieceMove {
    fn new(location: Location, can_capture: bool) -> Self {
        Self { location, can_capture }
    }
}

pub struct LocationOffset {
    file: i8,
    rank: i8,
}

impl LocationOffset {
    fn new(file: i8, rank: i8) -> Self {
        Self { file, rank }
    }
}

/**
 * Store all possible moves for a piece
 */
pub struct BoardState {
    pub move_count: u32,
    pub pieces: HashMap<Location, BoardPiece>,
    pub move_hints: HashMap<Location, MoveHints>,
    selected_valid_moves_cache: HashMap<Location, HashSet<ValidMove>>,
    // location_to_possible_captures: HashMap<Location, Vec<Location>>,
    piece_possible_moves: HashMap<Location, Vec<Vec<PieceMove>>>,
    valid_moves: HashMap<Location, Vec<Location>>,
    global_possible_captures: Grid<u8>,
    // piece_type_move_sets: HashMap<PieceType, Vec<Vec<LocationOffset>>>,
}

impl Default for BoardState {
    fn default() -> Self {
        /*
        let mut piece_type_move_sets = HashMap::with_capacity(32);

        // Pawn
        let pawn_move_set = vec![
            vec![LocationOffset::new(0, 1), LocationOffset::new(0, 1)],
            vec![LocationOffset::new(-1, 1)],
            vec![LocationOffset::new(1, 1)],
        ];
        piece_type_move_sets.insert(PieceType::Pawn, pawn_move_set);

        // Rook
        let rook_move_set = vec![
            (1..8).map(|r| LocationOffset::new(0, r)).collect(), // N
            (1..8).map(|f| LocationOffset::new(f, 0)).collect(), // E
            (1..8).map(|r| LocationOffset::new(0, -r)).collect(), // S
            (1..8).map(|f| LocationOffset::new(-f, 0)).collect(), // W
        ];
        piece_type_move_sets.insert(PieceType::Rook, rook_move_set);

        // Knight
        let knight_move_set = vec![
            vec![LocationOffset::new(1, 2)],
            vec![LocationOffset::new(2, 1)],
            vec![LocationOffset::new(2, -1)],
            vec![LocationOffset::new(1, -2)],
            vec![LocationOffset::new(-1, -2)],
            vec![LocationOffset::new(-2, -1)],
            vec![LocationOffset::new(-2, 1)],
            vec![LocationOffset::new(-1, 2)],
        ];
        piece_type_move_sets.insert(PieceType::Knight, knight_move_set);

        // Bishop
        let bishop_move_set = vec![
            (1..8).map(|o| LocationOffset::new(o, o)).collect(), // NE
            (1..8).map(|o| LocationOffset::new(o, -o)).collect(), // SE
            (1..8).map(|o| LocationOffset::new(-o, -o)).collect(), // SW
            (1..8).map(|o| LocationOffset::new(-o, o)).collect(), // NW
        ];
        piece_type_move_sets.insert(PieceType::Bishop, bishop_move_set);

        // Queen
        let queen_move_set = vec![
            (1..8).map(|r| LocationOffset::new(0, r)).collect(), // N
            (1..8).map(|o| LocationOffset::new(o, o)).collect(), // NE
            (1..8).map(|f| LocationOffset::new(f, 0)).collect(), // E
            (1..8).map(|o| LocationOffset::new(o, -o)).collect(), // SE
            (1..8).map(|r| LocationOffset::new(0, -r)).collect(), // S
            (1..8).map(|o| LocationOffset::new(-o, -o)).collect(), // SW
            (1..8).map(|f| LocationOffset::new(-f, 0)).collect(), // W
            (1..8).map(|o| LocationOffset::new(-o, o)).collect(), // NW
        ];
        piece_type_move_sets.insert(PieceType::Queen, queen_move_set);

        // King
        let king_move_set = vec![
            vec![LocationOffset::new(0, 1)],   // N
            vec![LocationOffset::new(1, 1)],   // NE
            vec![LocationOffset::new(1, 0)],   // E
            vec![LocationOffset::new(1, -1)],  // SE
            vec![LocationOffset::new(0, -1)],  // S
            vec![LocationOffset::new(-1, -1)], // SW
            vec![LocationOffset::new(-1, 0)],  // W
            vec![LocationOffset::new(-1, 1)],  // NW
        ];
        piece_type_move_sets.insert(PieceType::King { been_in_check: false }, king_move_set);
        */

        Self {
            move_count: 0,
            pieces: HashMap::with_capacity(32),
            move_hints: HashMap::with_capacity(64),
            selected_valid_moves_cache: HashMap::with_capacity(64),
            // location_to_possible_captures: HashMap::with_capacity(64),
            piece_possible_moves: HashMap::with_capacity(64),
            valid_moves: HashMap::with_capacity(64),
            global_possible_captures: Grid::new(8, 8),
            // piece_type_move_sets,
        }
    }
}

impl BoardState {
    pub fn is_colors_turn_at(&self, location: Location) -> bool {
        let color = self.pieces.get(&location).expect("TODO").color;
        match color {
            PieceColor::Black => self.move_count % 2 == 1,
            PieceColor::White => self.move_count % 2 == 0,
        }
    }

    fn get_hints(&self, location: Location) -> &MoveHints {
        self.move_hints.get(&location).expect("Failed to get hints: none at location")
    }

    fn get_capture_count(&mut self, Location { file, rank, .. }: Location) -> &mut u8 {
        self.global_possible_captures
            .get_mut(rank as usize, file as usize)
            .expect("failed to get capture count from grid, invalid location")
    }

    fn old_possible_piece_moves(&self, location: Location) -> HashSet<ValidMove> {
        let &BoardPiece { color, typ, did_move } = self.pieces.get(&location).expect(
            "Failed to calculate possible moves for piece: no such piece exists at this location",
        );

        let mut moves = HashSet::new();
        let mut safe_insert = |move_: ValidMove| {
            if !moves.insert(move_) {
                panic!(
                    "failed to insert possible move into hash set, already exists: {:?} {}",
                    move_.typ, move_.location
                );
            }
        };

        // Return whether a directional piece (pawn, rook, bishop, queen) could move past loc
        let mut push_directional_move = |loc: Location| match self.pieces.get(&loc) {
            Some(piece) if piece.color != color => {
                safe_insert(ValidMove::new(loc, PieceMoveType::Capture));
                false
            }
            None => {
                safe_insert(ValidMove::new(loc, PieceMoveType::Move));
                true
            }
            _ => false,
        };
        let mut push_directional_offset = |file_o, rank_o| {
            location.try_offset(file_o, rank_o).map(&mut push_directional_move).unwrap_or(false)
        };

        match typ {
            PieceType::Pawn => {
                let (direction, start_rank): (i8, u8) = match color {
                    PieceColor::Black => (-1, 6),
                    PieceColor::White => (1, 1),
                };

                if let Some(loc1) = location.try_offset(0, direction) {
                    // safe_insert(PieceMove::new(loc1, false));
                    if !self.pieces.contains_key(&loc1) {
                        safe_insert(ValidMove::new(loc1, PieceMoveType::Move));
                        if location.rank == start_rank {
                            let loc2 = loc1.with_rank((loc1.rank as i8 + direction) as u8);
                            if !self.pieces.contains_key(&loc2) {
                                safe_insert(ValidMove::new(loc2, PieceMoveType::Move));
                            }
                        }
                    }
                }

                #[inline(always)]
                fn loc_capturable(
                    pieces: &HashMap<Location, BoardPiece>,
                    location: Location,
                    color: PieceColor,
                ) -> bool {
                    pieces.get(&location).map(|p| p.color != color).unwrap_or(false)
                }
                if let Some(loc) = location.try_offset(1, direction) {
                    if loc_capturable(&self.pieces, loc, color) {
                        safe_insert(ValidMove::new(loc, PieceMoveType::Capture));
                    }
                }
                if let Some(loc) = location.try_offset(-1, direction) {
                    if loc_capturable(&self.pieces, loc, color) {
                        safe_insert(ValidMove::new(loc, PieceMoveType::Capture));
                    }
                }
            }

            PieceType::Rook => {
                let (mut n, mut e, mut s, mut w) = (true, true, true, true);
                for i in 1..8 {
                    n = n && push_directional_offset(0, i);
                    e = e && push_directional_offset(i, 0);
                    s = s && push_directional_offset(0, -i);
                    w = w && push_directional_offset(-i, 0);
                }
            }

            PieceType::Knight => {
                const KNIGHT_MOVES: [(i8, i8); 8] =
                    [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)];
                for (file_o, rank_o) in KNIGHT_MOVES {
                    push_directional_offset(file_o, rank_o);
                }
            }

            PieceType::Bishop => {
                let (mut ne, mut se, mut sw, mut nw) = (true, true, true, true);
                for o in 1..8 {
                    ne = ne && push_directional_offset(o, o);
                    se = se && push_directional_offset(o, -o);
                    sw = sw && push_directional_offset(-o, -o);
                    nw = nw && push_directional_offset(-o, o);
                }
            }

            PieceType::Queen => {
                let (mut n, mut ne, mut e, mut se, mut s, mut sw, mut w, mut nw) =
                    (true, true, true, true, true, true, true, true);
                for o in 1..8 {
                    n = n && push_directional_offset(0, o);
                    ne = ne && push_directional_offset(o, o);
                    e = e && push_directional_offset(o, 0);
                    se = se && push_directional_offset(o, -o);
                    s = s && push_directional_offset(0, -o);
                    sw = sw && push_directional_offset(-o, -o);
                    w = w && push_directional_offset(-o, 0);
                    nw = nw && push_directional_offset(-o, o);
                }
            }

            PieceType::King { been_in_check } => {
                const KING_MOVES: [(i8, i8); 8] =
                    [(0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)];
                for (file_o, rank_o) in KING_MOVES {
                    push_directional_offset(file_o, rank_o);
                }

                #[inline(always)]
                fn castle_path_is_clear(
                    pieces: &HashMap<Location, BoardPiece>,
                    location: Location,
                    mut file_range: RangeInclusive<u8>,
                ) -> bool {
                    file_range.all(|f| !pieces.contains_key(&location.with_file(f)))
                }
                if !did_move && !been_in_check {
                    let rank = match color {
                        PieceColor::White => 0,
                        PieceColor::Black => 7,
                    };
                    const CASTLE_PARAMS: [(u8, RangeInclusive<u8>, u8); 2] =
                        [(0, 1..=3, 2), (7, 5..=6, 6)];
                    for (rook_file, castle_path_range, castle_file) in CASTLE_PARAMS {
                        let rook_loc = Location::new(rook_file, rank);
                        #[allow(clippy::collapsible_if)]
                        if self.pieces.get(&rook_loc).map_or(false, |rook| !rook.did_move) {
                            if castle_path_is_clear(&self.pieces, location, castle_path_range) {
                                // TODO: Only push moves that will take the king *out* of check.
                                // TODO:
                                // TODO: This probably means we will have to keep track of *all*
                                // TODO: possible moves for *all* pieces at *all* times rather than
                                // TODO: only keeping track of the possible move set for the
                                // TODO: selected piece.
                                let castle_loc = rook_loc.with_file(castle_file);
                                // safe_insert(PossibleMove::new(castle_loc, PieceMoveType::Castle));
                                safe_insert(ValidMove::new(castle_loc, PieceMoveType::Move));
                            }
                        }
                    }
                }
            }
        }
        moves
    }

    fn possible_piece_moves(&self, location: Location) -> Vec<Vec<PieceMove>> {
        let &BoardPiece { color, typ, did_move } = self.pieces.get(&location).expect(
            "Failed to calculate possible moves for piece: no such piece exists at this location",
        );

        let push_offset_to =
            |file_o: i8, rank_o: i8, can_capture: bool, moves: &mut Vec<PieceMove>| {
                if let Some(loc) = location.try_offset(file_o, rank_o) {
                    moves.push(PieceMove::new(loc, can_capture));
                }
            };

        fn to_move(loc: Location) -> PieceMove {
            PieceMove::new(loc, true)
        }

        match typ {
            PieceType::Pawn => {
                let (direction, start_rank): (i8, u8) = match color {
                    PieceColor::Black => (-1, 6),
                    PieceColor::White => (1, 1),
                };

                let mut forward_moves = Vec::with_capacity(2);
                if let Some(loc1) = location.try_offset(0, direction) {
                    if !self.pieces.contains_key(&loc1) {
                        forward_moves.push(PieceMove::new(loc1, false));
                        if location.rank == start_rank {
                            if let Some(loc2) = location.try_offset(0, 2 * direction) {
                                if !self.pieces.contains_key(&loc2) {
                                    forward_moves.push(PieceMove::new(loc2, false));
                                }
                            }
                        }
                    }
                }

                let mut cap_left = Vec::with_capacity(1);
                push_offset_to(-1, direction, true, &mut cap_left);

                let mut cap_right = Vec::with_capacity(1);
                push_offset_to(1, direction, true, &mut cap_right);

                vec![forward_moves, cap_left, cap_right]
            }

            PieceType::Rook => {
                const R: Range<i8> = 1..8;
                vec![
                    R.filter_map(|r| location.try_offset(0, r)).map(to_move).collect(), // N
                    R.filter_map(|f| location.try_offset(f, 0)).map(to_move).collect(), // E
                    R.filter_map(|r| location.try_offset(0, -r)).map(to_move).collect(), // S
                    R.filter_map(|f| location.try_offset(-f, 0)).map(to_move).collect(), // W
                ]
            }

            PieceType::Knight => {
                [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)]
                    .into_iter()
                    .filter_map(|(f, r)| location.try_offset(f, r))
                    .map(|loc| vec![to_move(loc)])
                    .collect()
            }

            PieceType::Bishop => {
                const R: Range<i8> = 1..8;
                vec![
                    R.filter_map(|o| location.try_offset(o, o)).map(to_move).collect(), // NE
                    R.filter_map(|o| location.try_offset(o, -o)).map(to_move).collect(), // SE
                    R.filter_map(|o| location.try_offset(-o, -o)).map(to_move).collect(), // SW
                    R.filter_map(|o| location.try_offset(-o, o)).map(to_move).collect(), // NW
                ]
            }

            PieceType::Queen => {
                const R: Range<i8> = 1..8;
                vec![
                    R.filter_map(|r| location.try_offset(0, r)).map(to_move).collect(), // N
                    R.filter_map(|o| location.try_offset(o, o)).map(to_move).collect(), // NE
                    R.filter_map(|f| location.try_offset(f, 0)).map(to_move).collect(), // E
                    R.filter_map(|o| location.try_offset(o, -o)).map(to_move).collect(), // SE
                    R.filter_map(|r| location.try_offset(0, -r)).map(to_move).collect(), // S
                    R.filter_map(|o| location.try_offset(-o, -o)).map(to_move).collect(), // SW
                    R.filter_map(|f| location.try_offset(-f, 0)).map(to_move).collect(), // W
                    R.filter_map(|o| location.try_offset(-o, o)).map(to_move).collect(), // NW
                ]
            }

            PieceType::King { been_in_check } => {
                let mut moves = Vec::with_capacity(10);

                let steps = [(0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)]
                    .into_iter()
                    .filter_map(|(f, r)| location.try_offset(f, r))
                    .map(|loc| vec![to_move(loc)]);
                moves.extend(steps);

                if !did_move && !been_in_check {
                    let rank = match color {
                        PieceColor::White => 0,
                        PieceColor::Black => 7,
                    };

                    #[inline(always)]
                    fn castle_path_is_clear(
                        pieces: &HashMap<Location, BoardPiece>,
                        location: Location,
                        mut file_range: RangeInclusive<u8>,
                    ) -> bool {
                        file_range.all(|f| !pieces.contains_key(&location.with_file(f)))
                    }

                    const CASTLE_PARAMS: [(u8, RangeInclusive<u8>, u8); 2] =
                        [(0, 1..=3, 2), (7, 5..=6, 6)];
                    for (rook_file, path, castle_file) in CASTLE_PARAMS {
                        let rook_loc = Location::new(rook_file, rank);
                        #[allow(clippy::collapsible_if)]
                        if self.pieces.get(&rook_loc).map_or(false, |rook| !rook.did_move) {
                            if castle_path_is_clear(&self.pieces, location, path) {
                                let castle_loc = rook_loc.with_file(castle_file);
                                moves.push(vec![PieceMove::new(castle_loc, false)]);
                            }
                        }
                    }
                }

                moves
            }
        }
    }

    // fn calculate_and_cache_piece_moves(&mut self, location: Location) {
    //     self.selected_valid_moves_cache.insert(location, self.possible_piece_moves(location));
    // }

    // pub fn get_piece_moves(&self, location: &Location) -> &HashSet<ValidMove> {
    //     self.selected_valid_moves_cache
    //         .get(location)
    //         .expect("Failed to get possible moves: not cached")
    // }

    pub fn show_piece_move_hints(&mut self, commands: &mut Commands, location: Location) {
        let possible_moves = self.possible_piece_moves(location);
        let mut valid_moves = Vec::new();

        let BoardPiece { color, .. } =
            self.pieces.get(&location).expect("no such piece at location");

        for move_series in &possible_moves {
            for PieceMove { location: move_loc, .. } in move_series {
                match self.pieces.get(move_loc) {
                    Some(BoardPiece { color: move_color, .. }) if move_color == color => break,
                    piece => {
                        let loc_hints = self.get_hints(*move_loc);
                        let hint_entity = if piece.is_some() {
                            loc_hints.entity_capture
                        } else {
                            loc_hints.entity_move
                        };
                        commands.entity(hint_entity).insert(ShowHint);
                        valid_moves.push(*move_loc);
                        let count = self
                            .global_possible_captures
                            .get_mut(move_loc.rank as usize, move_loc.file as usize)
                            .expect("invalid piece move location");
                        *count += 1;
                        if piece.is_some() {
                            break;
                        }
                    }
                }
            }
        }

        self.piece_possible_moves.insert(location, possible_moves);
        self.valid_moves.insert(location, valid_moves);

        // self.calculate_and_cache_piece_moves(location);
        // for ValidMove { location, typ } in self.get_piece_moves(&location) {
        //     let loc_hints = self.get_hints(location);
        //     let entity = match typ {
        //         PieceMoveType::Move | PieceMoveType::Castle => loc_hints.entity_move,
        //         PieceMoveType::Capture => loc_hints.entity_capture,
        //     };
        //     commands.entity(entity).insert(ShowHint);
        // }
    }

    pub fn hide_piece_move_hints(&mut self, commands: &mut Commands, location: Location) {
        if let Some(valid_moves) = self.valid_moves.remove(&location) {
            for v_move in valid_moves {
                let loc_hints = self.get_hints(v_move);
                commands.entity(loc_hints.entity_move).insert(HideHint);
                commands.entity(loc_hints.entity_capture).insert(HideHint);

                let count = self.get_capture_count(v_move);
                *count = count.saturating_sub(1);
            }
        }

        self.piece_possible_moves.remove(&location);

        // for ValidMove { location, typ } in self.get_piece_moves(location) {
        //     let loc_hints = self.get_hints(location);
        //     let entity = match typ {
        //         PieceMoveType::Move | PieceMoveType::Castle => loc_hints.entity_move,
        //         PieceMoveType::Capture => loc_hints.entity_capture,
        //     };
        //     commands.entity(entity).insert(HideHint);
        // }
    }

    pub fn move_piece(&mut self, from: Location, to: Location) {
        let (_old_loc, mut piece) = self
            .pieces
            .remove_entry(&from)
            .expect("Failed to move board state piece: no piece found at source location");
        piece.did_move = true;
        match self.pieces.entry(to) {
            Entry::Occupied(entry) => {
                panic!(
                    "Failed to move board state piece: piece already at destination location {}",
                    entry.key()
                )
            }
            Entry::Vacant(entry) => {
                entry.insert(piece);
            }
        }
    }
}
