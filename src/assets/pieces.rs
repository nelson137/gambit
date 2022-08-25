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

/// A 2D array of algebraic coordinates for the starting location of all chess pieces. Each
/// coordinate is a tuple of `(File, Rank)`.
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
pub const PIECE_ASSET_COORDS: [&[(u8, u8)]; 2] = [
    // Black
    &[
        // Rank 8
        (0, 7),
        (1, 7),
        (2, 7),
        (3, 7),
        (4, 7),
        (5, 7),
        (6, 7),
        (7, 7),
        // Rank 7
        (0, 6),
        (1, 6),
        (2, 6),
        (3, 6),
        (4, 6),
        (5, 6),
        (6, 6),
        (7, 6),
    ],
    // White
    &[
        // Rank 2
        (0, 1),
        (1, 1),
        (2, 1),
        (3, 1),
        (4, 1),
        (5, 1),
        (6, 1),
        (7, 1),
        // Rank 1
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0),
        (5, 0),
        (6, 0),
        (7, 0),
    ],
];
