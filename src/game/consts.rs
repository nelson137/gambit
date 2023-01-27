use bevy::prelude::*;

/// The game background color.
///
/// `#302e2b`
pub const COLOR_BG: Color = Color::rgb(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
);

pub const Z_GAME_MENU: i32 = 21;

pub const Z_GAME_MENU_DIM_LAYER: i32 = 20;

pub const Z_CHECKMATE_ICONS: i32 = 15;

pub const Z_PIECE_SELECTED: i32 = 11;

pub const Z_PIECE: i32 = 10;

pub const Z_HIGHLIGHT_TILE: i32 = 4;

pub const Z_MOVE_HINT: i32 = 3;

pub const Z_NOTATION_TEXT: i32 = 2;

pub const Z_TILE: i32 = 1;
