use bevy::prelude::*;
use chess::Square;

use crate::{
    assets::PIECE_ASSET_SIZE,
    data::{DoMove, DoUpdatePieceSquare, Dragging, UiSquare},
};

fn square_to_coords(square: Square, transl: &mut Vec3, win_size: f32) {
    let tile_size = win_size / 8.0;
    let origin_x = -(win_size / 2.0);
    let origin_y = origin_x;
    let file_i = square.get_file().to_index();
    let rank_i = square.get_rank().to_index();
    transl.x = origin_x + file_i as f32 * tile_size + tile_size / 2.0;
    transl.y = origin_y + rank_i as f32 * tile_size + tile_size / 2.0;
}

pub fn update_translation_for_square(
    windows: Res<Windows>,
    mut squares: Query<
        (&UiSquare, &mut Transform),
        (Without<Dragging>, Without<DoMove>, Without<DoUpdatePieceSquare>),
    >,
) {
    let win = if let Some(w) = windows.get_primary() { w } else { return };
    let win_size = win.width().min(win.height());
    let tile_scale = win_size / 8.0 / PIECE_ASSET_SIZE;

    for (square, mut transf) in &mut squares {
        transf.scale = Vec3::splat(tile_scale);
        square_to_coords(**square, &mut transf.translation, win_size);
    }
}
