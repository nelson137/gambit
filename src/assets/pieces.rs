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

/// A 2D array of algebraic coordinates, the color, and piece type of the starting square of all
/// chess pieces.
///
/// The array contains 2 sub-arrays containing the coordinates for a player's set of pieces. The
/// first sub-array is for black and the second sub-array is for white. The sub-arrays list the
/// player's pieces rank-wise starting from the top left of their two ranks and going to the bottom
/// right. Thus, the list is as follows:
///
/// ```
/// [
///     [r, n, b, q, k, b, n, r, p, p, p, p, p, p, p, p],
///     [R, N, B, Q, K, B, N, R, P, P, P, P, P, P, P, P],
/// ]
/// ```
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
pub const PIECE_COLORS_TYPES: [[(Rank, File, PieceColor, PieceType); 16]; 2] = [
    // Black
    [
        // Rank 8
        (Rank::Eighth, File::A, PieceColor(Color::Black), PieceType(Piece::Rook)),
        (Rank::Eighth, File::B, PieceColor(Color::Black), PieceType(Piece::Knight)),
        (Rank::Eighth, File::C, PieceColor(Color::Black), PieceType(Piece::Bishop)),
        (Rank::Eighth, File::D, PieceColor(Color::Black), PieceType(Piece::Queen)),
        (Rank::Eighth, File::E, PieceColor(Color::Black), PieceType(Piece::King)),
        (Rank::Eighth, File::F, PieceColor(Color::Black), PieceType(Piece::Bishop)),
        (Rank::Eighth, File::G, PieceColor(Color::Black), PieceType(Piece::Knight)),
        (Rank::Eighth, File::H, PieceColor(Color::Black), PieceType(Piece::Rook)),
        // Rank 7
        (Rank::Seventh, File::A, PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (Rank::Seventh, File::B, PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (Rank::Seventh, File::C, PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (Rank::Seventh, File::D, PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (Rank::Seventh, File::E, PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (Rank::Seventh, File::F, PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (Rank::Seventh, File::G, PieceColor(Color::Black), PieceType(Piece::Pawn)),
        (Rank::Seventh, File::H, PieceColor(Color::Black), PieceType(Piece::Pawn)),
    ],
    // White
    [
        // Rank 2
        (Rank::Second, File::A, PieceColor(Color::White), PieceType(Piece::Pawn)),
        (Rank::Second, File::B, PieceColor(Color::White), PieceType(Piece::Pawn)),
        (Rank::Second, File::C, PieceColor(Color::White), PieceType(Piece::Pawn)),
        (Rank::Second, File::D, PieceColor(Color::White), PieceType(Piece::Pawn)),
        (Rank::Second, File::E, PieceColor(Color::White), PieceType(Piece::Pawn)),
        (Rank::Second, File::F, PieceColor(Color::White), PieceType(Piece::Pawn)),
        (Rank::Second, File::G, PieceColor(Color::White), PieceType(Piece::Pawn)),
        (Rank::Second, File::H, PieceColor(Color::White), PieceType(Piece::Pawn)),
        // Rank 1
        (Rank::First, File::A, PieceColor(Color::White), PieceType(Piece::Rook)),
        (Rank::First, File::B, PieceColor(Color::White), PieceType(Piece::Knight)),
        (Rank::First, File::C, PieceColor(Color::White), PieceType(Piece::Bishop)),
        (Rank::First, File::D, PieceColor(Color::White), PieceType(Piece::Queen)),
        (Rank::First, File::E, PieceColor(Color::White), PieceType(Piece::King)),
        (Rank::First, File::F, PieceColor(Color::White), PieceType(Piece::Bishop)),
        (Rank::First, File::G, PieceColor(Color::White), PieceType(Piece::Knight)),
        (Rank::First, File::H, PieceColor(Color::White), PieceType(Piece::Rook)),
    ],
];
