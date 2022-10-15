use chess::{Color, File, Piece, Rank};

use crate::data::{PieceColor, PieceType};

/// build_piece_fn!("{COLOR}", "{PIECE}") -> "assets/pieces/{COLOR}-{PIECE}.png"
macro_rules! build_piece_fn {
    ($color:literal, $piece:literal) => {
        concat!("pieces/", $color, "-", $piece, ".png")
        // concat!($color, "-", $piece)
    };
}

macro_rules! build_piece_filenames {
    ($color:literal foreach [$piece:literal $(,$restPieces:literal)+ $(,)?]) => {
        build_piece_filenames!(
            @buildFilenames $color
            foreach [$($restPieces),*]
            into [build_piece_fn!($color, $piece)]
        )
    };

    // @buildFilenames [...$colors]
    // foreach [...$pieces]
    // into [...$acc]

    (@buildFilenames $color:literal foreach [$piece:literal $(,$restPieces:tt)* $(,)?] into [$($acc:tt)+]) => {
        build_piece_filenames!(
            @buildFilenames $color
            foreach [$($restPieces),*]
            // into [$($acc)+, build_piece_filenames!(@join $color and $piece)]
            into [$($acc)+, build_piece_fn!($color, $piece)]
        )
    };
    (@buildFilenames $color:literal foreach [END $(,$restPieces:tt)* $(,)?] into [$($acc:tt)+]) => {
        build_piece_filenames!(
            @buildFilenames [$($restColors),*]
            foreach [$($restPieces),*]
            into [$($acc)+]
        )
    };
    (@buildFilenames $_color:literal foreach [] into [$($acc:tt)+]) => {
        [$($acc)+]
    };
}

/// The width and height dimension of all piece asset images.
pub const PIECE_ASSET_SIZE: f32 = 150.0;

pub const PIECE_ASSET_PATHS: [&[&str]; 2] = [
    &build_piece_filenames![
        "black"
        foreach [
            "rook", "knight", "bishop", "queen", "king", "bishop", "knight", "rook",
            "pawn", "pawn", "pawn", "pawn", "pawn", "pawn", "pawn", "pawn"
        ]
    ],
    &build_piece_filenames![
        "white"
        foreach [
            "pawn", "pawn", "pawn", "pawn", "pawn", "pawn", "pawn", "pawn",
            "rook", "knight", "bishop", "queen", "king", "bishop", "knight", "rook"
        ]
    ],
];

pub const PIECE_COLORS_TYPES: [&[(PieceColor, PieceType)]; 2] = [
    &[
        // Rank 8
        (PieceColor(Color::Black), PieceType(Piece::Rook)),
        (PieceColor(Color::Black), PieceType(Piece::Knight)),
        (PieceColor(Color::Black), PieceType(Piece::Bishop)),
        (PieceColor(Color::Black), PieceType(Piece::Queen)),
        (PieceColor(Color::Black), PieceType(Piece::King)),
        (PieceColor(Color::Black), PieceType(Piece::Bishop)),
        (PieceColor(Color::Black), PieceType(Piece::Knight)),
        (PieceColor(Color::Black), PieceType(Piece::Rook)),
        // Rank 7
        (PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (PieceColor(Color::Black), PieceType(Piece::Pawn)),
    ],
    &[
        // Rank 2
        (PieceColor(Color::White), PieceType(Piece::Pawn)),
        (PieceColor(Color::White), PieceType(Piece::Pawn)),
        (PieceColor(Color::White), PieceType(Piece::Pawn)),
        (PieceColor(Color::White), PieceType(Piece::Pawn)),
        (PieceColor(Color::White), PieceType(Piece::Pawn)),
        (PieceColor(Color::White), PieceType(Piece::Pawn)),
        (PieceColor(Color::White), PieceType(Piece::Pawn)),
        (PieceColor(Color::White), PieceType(Piece::Pawn)),
        // Rank 1
        (PieceColor(Color::White), PieceType(Piece::Rook)),
        (PieceColor(Color::White), PieceType(Piece::Knight)),
        (PieceColor(Color::White), PieceType(Piece::Bishop)),
        (PieceColor(Color::White), PieceType(Piece::Queen)),
        (PieceColor(Color::White), PieceType(Piece::King)),
        (PieceColor(Color::White), PieceType(Piece::Bishop)),
        (PieceColor(Color::White), PieceType(Piece::Knight)),
        (PieceColor(Color::White), PieceType(Piece::Rook)),
    ],
];

/// A 2D array of algebraic coordinates for the starting square of all chess pieces. Each coordinate
/// is a tuple of `(File, Rank)`.
///
/// The array contains 2 sub-arrays containing the coordinates for a player's set of pieces. The
/// first sub-array is for black, whose pieces start in ranks 7 and 8. The second sub-array is for
/// white, whose pieces start in ranks 1 and 2.
///
/// `(0, 0)` is a1 and `(7, 7)` is h8.
///
/// ```
///
///               Black
/// +---+---+---+---+---+---+---+---+
/// | r | n | b | q | k | b | n | r | 8
/// +---+---+---+---+---+---+---+---+
/// | p | p | p | p | p | p | p | p | 7
/// +---+---+---+---+---+---+---+---+
/// |   |   |   |   |   |   |   |   | 6
/// +---+---+---+---+---+---+---+---+
/// |   |   |   |   |   |   |   |   | 5
/// +---+---+---+---+---+---+---+---+
/// |   |   |   |   |   |   |   |   | 4
/// +---+---+---+---+---+---+---+---+
/// |   |   |   |   |   |   |   |   | 3
/// +---+---+---+---+---+---+---+---+
/// | P | P | P | P | P | P | P | P | 2
/// +---+---+---+---+---+---+---+---+
/// | R | N | B | K | Q | B | N | R | 1
/// +---+---+---+---+---+---+---+---+
///   a   b   c   d   e   f   g   h
///               White
///
/// ```
pub const PIECE_ASSET_COORDS: [&[(Rank, File)]; 2] = [
    // Black
    &[
        // Rank 8
        (Rank::Eighth, File::A),
        (Rank::Eighth, File::B),
        (Rank::Eighth, File::C),
        (Rank::Eighth, File::D),
        (Rank::Eighth, File::E),
        (Rank::Eighth, File::F),
        (Rank::Eighth, File::G),
        (Rank::Eighth, File::H),
        // Rank 7
        (Rank::Seventh, File::A),
        (Rank::Seventh, File::B),
        (Rank::Seventh, File::C),
        (Rank::Seventh, File::D),
        (Rank::Seventh, File::E),
        (Rank::Seventh, File::F),
        (Rank::Seventh, File::G),
        (Rank::Seventh, File::H),
    ],
    // White
    &[
        // Rank 2
        (Rank::Second, File::A),
        (Rank::Second, File::B),
        (Rank::Second, File::C),
        (Rank::Second, File::D),
        (Rank::Second, File::E),
        (Rank::Second, File::F),
        (Rank::Second, File::G),
        (Rank::Second, File::H),
        // Rank 1
        (Rank::First, File::A),
        (Rank::First, File::B),
        (Rank::First, File::C),
        (Rank::First, File::D),
        (Rank::First, File::E),
        (Rank::First, File::F),
        (Rank::First, File::G),
        (Rank::First, File::H),
    ],
];
