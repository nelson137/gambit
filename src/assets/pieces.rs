use crate::game::board::{PieceColor, PieceType};

pub trait PieceColorAndTypeAssetPath {
    fn asset_path(self) -> &'static str;
}

impl PieceColorAndTypeAssetPath for (PieceColor, PieceType) {
    fn asset_path(self) -> &'static str {
        match (self.0, self.1) {
            (PieceColor::BLACK, PieceType::BISHOP) => "images/pieces/black-bishop.png",
            (PieceColor::BLACK, PieceType::KING) => "images/pieces/black-king.png",
            (PieceColor::BLACK, PieceType::KNIGHT) => "images/pieces/black-knight.png",
            (PieceColor::BLACK, PieceType::PAWN) => "images/pieces/black-pawn.png",
            (PieceColor::BLACK, PieceType::QUEEN) => "images/pieces/black-queen.png",
            (PieceColor::BLACK, PieceType::ROOK) => "images/pieces/black-rook.png",
            (PieceColor::WHITE, PieceType::BISHOP) => "images/pieces/white-bishop.png",
            (PieceColor::WHITE, PieceType::KING) => "images/pieces/white-king.png",
            (PieceColor::WHITE, PieceType::KNIGHT) => "images/pieces/white-knight.png",
            (PieceColor::WHITE, PieceType::PAWN) => "images/pieces/white-pawn.png",
            (PieceColor::WHITE, PieceType::QUEEN) => "images/pieces/white-queen.png",
            (PieceColor::WHITE, PieceType::ROOK) => "images/pieces/white-rook.png",
        }
    }
}
