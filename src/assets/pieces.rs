use crate::game::board::{PieceColor, PieceType};

macro_rules! asset_path {
    ($color:literal, $type:literal) => {
        concat!("images/pieces/", $color, "-", $type, ".png")
    };
}

pub trait PieceColorAndTypeAssetPath {
    fn asset_path(self) -> &'static str;
}

impl PieceColorAndTypeAssetPath for (PieceColor, PieceType) {
    fn asset_path(self) -> &'static str {
        match (self.0, self.1) {
            (PieceColor::BLACK, PieceType::BISHOP) => asset_path!("black", "bishop"),
            (PieceColor::BLACK, PieceType::KING) => asset_path!("black", "king"),
            (PieceColor::BLACK, PieceType::KNIGHT) => asset_path!("black", "knight"),
            (PieceColor::BLACK, PieceType::PAWN) => asset_path!("black", "pawn"),
            (PieceColor::BLACK, PieceType::QUEEN) => asset_path!("black", "queen"),
            (PieceColor::BLACK, PieceType::ROOK) => asset_path!("black", "rook"),
            (PieceColor::WHITE, PieceType::BISHOP) => asset_path!("white", "bishop"),
            (PieceColor::WHITE, PieceType::KING) => asset_path!("white", "king"),
            (PieceColor::WHITE, PieceType::KNIGHT) => asset_path!("white", "knight"),
            (PieceColor::WHITE, PieceType::PAWN) => asset_path!("white", "pawn"),
            (PieceColor::WHITE, PieceType::QUEEN) => asset_path!("white", "queen"),
            (PieceColor::WHITE, PieceType::ROOK) => asset_path!("white", "rook"),
        }
    }
}
