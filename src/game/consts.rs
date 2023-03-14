use bevy::prelude::*;

/// The game background color.
///
/// `#302e2b`
pub const COLOR_BG: Color = Color::rgb(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
);

pub const FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";

//==================================================
// Sizes
//==================================================

pub const INIT_WIN_WIDTH: f32 = 670.0;
pub const INIT_WIN_HEIGHT: f32 = 750.0;

pub const UI_GAP: f32 = 8.0;
pub const UI_GAP_VAL: Val = Val::Px(UI_GAP);

pub const CAPTURES_PANEL_HEIGHT: f32 = 32.0;

const INIT_BOARD_SIZE: f32 = INIT_WIN_HEIGHT - 2.0 * CAPTURES_PANEL_HEIGHT;

pub const MENU_WIDTH: f32 = 82.0;
pub const MENU_HEIGHT: f32 = 60.0;

pub const INIT_MENU_WIDTH: f32 = INIT_BOARD_SIZE * MENU_WIDTH / 100.0;

pub const INIT_MENU_TITLE_SIZE: f32 = 128.0;

pub const INIT_MENU_BUTTON_TEXT_SIZE: f32 = 48.0;

pub const MIN_BOARD_SIZE: f32 = 256.0;

//==================================================
// Z-Values
//==================================================

pub const Z_GAME_MENU: i32 = 21;

pub const Z_GAME_MENU_DIM_LAYER: i32 = 20;

pub const Z_END_GAME_ICONS: i32 = 15;

pub const Z_PIECE_SELECTED: i32 = 11;

pub const Z_PIECE: i32 = 10;

pub const Z_MOVE_HINT: i32 = 4;

pub const Z_HIGHLIGHT_TILE: i32 = 3;

pub const Z_NOTATION_TEXT: i32 = 2;

pub const Z_TILE: i32 = 1;
