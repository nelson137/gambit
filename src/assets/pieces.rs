use chess::{Color, Piece, Square};

pub trait SquareStartingPieceInfo {
    fn starting_piece_info(self) -> Option<(&'static str, Color, Piece)>;
}

impl SquareStartingPieceInfo for Square {
    fn starting_piece_info(self) -> Option<(&'static str, Color, Piece)> {
        match self {
            Square::A1 => Some(("images/pieces/white-rook.png", Color::White, Piece::Rook)),
            Square::B1 => Some(("images/pieces/white-knight.png", Color::White, Piece::Knight)),
            Square::C1 => Some(("images/pieces/white-bishop.png", Color::White, Piece::Bishop)),
            Square::D1 => Some(("images/pieces/white-queen.png", Color::White, Piece::Queen)),
            Square::E1 => Some(("images/pieces/white-king.png", Color::White, Piece::King)),
            Square::F1 => Some(("images/pieces/white-bishop.png", Color::White, Piece::Bishop)),
            Square::G1 => Some(("images/pieces/white-knight.png", Color::White, Piece::Knight)),
            Square::H1 => Some(("images/pieces/white-rook.png", Color::White, Piece::Rook)),
            Square::A2 => Some(("images/pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::B2 => Some(("images/pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::C2 => Some(("images/pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::D2 => Some(("images/pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::E2 => Some(("images/pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::F2 => Some(("images/pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::G2 => Some(("images/pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::H2 => Some(("images/pieces/white-pawn.png", Color::White, Piece::Pawn)),
            Square::A7 => Some(("images/pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::B7 => Some(("images/pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::C7 => Some(("images/pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::D7 => Some(("images/pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::E7 => Some(("images/pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::F7 => Some(("images/pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::G7 => Some(("images/pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::H7 => Some(("images/pieces/black-pawn.png", Color::Black, Piece::Pawn)),
            Square::A8 => Some(("images/pieces/black-rook.png", Color::Black, Piece::Rook)),
            Square::B8 => Some(("images/pieces/black-knight.png", Color::Black, Piece::Knight)),
            Square::C8 => Some(("images/pieces/black-bishop.png", Color::Black, Piece::Bishop)),
            Square::D8 => Some(("images/pieces/black-queen.png", Color::Black, Piece::Queen)),
            Square::E8 => Some(("images/pieces/black-king.png", Color::Black, Piece::King)),
            Square::F8 => Some(("images/pieces/black-bishop.png", Color::Black, Piece::Bishop)),
            Square::G8 => Some(("images/pieces/black-knight.png", Color::Black, Piece::Knight)),
            Square::H8 => Some(("images/pieces/black-rook.png", Color::Black, Piece::Rook)),
            _ => None,
        }
    }
}
