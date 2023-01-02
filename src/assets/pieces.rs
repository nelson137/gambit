use chess::{Color, Piece, Square};

pub trait SquareStartingPieceInfo {
    fn starting_piece_info(self) -> Option<(&'static str, Color, Piece)>;
}

impl SquareStartingPieceInfo for Square {
    fn starting_piece_info(self) -> Option<(&'static str, Color, Piece)> {
        match self {
            Square::A1 => Some(("pieces/white-rook.png", Color::White, Piece::Rook)),
            Square::B1 => Some(("pieces/white-knight.png", Color::White, Piece::Knight)),
            Square::C1 => Some(("pieces/white-bishop.png", Color::White, Piece::Bishop)),
            Square::D1 => Some(("pieces/white-queen.png", Color::White, Piece::Queen)),
            Square::E1 => Some(("pieces/white-king.png", Color::White, Piece::King)),
            Square::F1 => Some(("pieces/white-bishop.png", Color::White, Piece::Bishop)),
            Square::G1 => Some(("pieces/white-knight.png", Color::White, Piece::Knight)),
            Square::H1 => Some(("pieces/white-rook.png", Color::White, Piece::Rook)),
            Square::A2 => Some(("pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::B2 => Some(("pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::C2 => Some(("pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::D2 => Some(("pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::E2 => Some(("pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::F2 => Some(("pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::G2 => Some(("pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::H2 => Some(("pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::A7 => Some(("pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::B7 => Some(("pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::C7 => Some(("pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::D7 => Some(("pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::E7 => Some(("pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::F7 => Some(("pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::G7 => Some(("pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::H7 => Some(("pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::A8 => Some(("pieces/black-rook.png", Color::Black, Piece::Rook)),
            Square::B8 => Some(("pieces/black-knight.png", Color::Black, Piece::Knight)),
            Square::C8 => Some(("pieces/black-bishop.png", Color::Black, Piece::Bishop)),
            Square::D8 => Some(("pieces/black-queen.png", Color::Black, Piece::Queen)),
            Square::E8 => Some(("pieces/black-king.png", Color::Black, Piece::King)),
            Square::F8 => Some(("pieces/black-bishop.png", Color::Black, Piece::Bishop)),
            Square::G8 => Some(("pieces/black-knight.png", Color::Black, Piece::Knight)),
            Square::H8 => Some(("pieces/black-rook.png", Color::Black, Piece::Rook)),
            _ => None,
        }
    }
}
