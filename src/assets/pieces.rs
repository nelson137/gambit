use chess::{Color, Piece};

pub trait PieceColorAndTypeAssetPath {
    fn asset_path(self) -> &'static str;
}

impl PieceColorAndTypeAssetPath for (chess::Color, chess::Piece) {
    fn asset_path(self) -> &'static str {
        match self {
            (Color::Black, Piece::Bishop) => "images/pieces/black-bishop.png",
            (Color::Black, Piece::King) => "images/pieces/black-king.png",
            (Color::Black, Piece::Knight) => "images/pieces/black-knight.png",
            (Color::Black, Piece::Pawn) => "images/pieces/black-pawn.png",
            (Color::Black, Piece::Queen) => "images/pieces/black-queen.png",
            (Color::Black, Piece::Rook) => "images/pieces/black-rook.png",
            (Color::White, Piece::Bishop) => "images/pieces/white-bishop.png",
            (Color::White, Piece::King) => "images/pieces/white-king.png",
            (Color::White, Piece::Knight) => "images/pieces/white-knight.png",
            (Color::White, Piece::Pawn) => "images/pieces/white-pawn.png",
            (Color::White, Piece::Queen) => "images/pieces/white-queen.png",
            (Color::White, Piece::Rook) => "images/pieces/white-rook.png",
        }
    }
}
