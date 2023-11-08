use bevy::{log::Level, prelude::*};

pub const LOG_LEVEL: Level = Level::INFO;

#[cfg(all(not(debug_assertions), not(feature = "trace_logs")))]
pub const LOG_FILTER: &str = "wgpu=error";
#[cfg(all(debug_assertions, not(feature = "trace_logs")))]
pub const LOG_FILTER: &str = "wgpu=error,gambit=debug";
#[cfg(feature = "trace_logs")]
pub const LOG_FILTER: &str = "wgpu=error,gambit=trace";

/// The game background color.
///
/// `#302e2b`
pub const COLOR_BG: Color = Color::rgb(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
);

pub const FONT_PATH: &str = "fonts/montserrat-700.otf";

pub const TITLE_FONT_PATH: &str = "fonts/montserrat-800.otf";

//==================================================
// Sizes
//==================================================

pub const INIT_WIN_WIDTH: f32 = 670.0;
pub const INIT_WIN_HEIGHT: f32 = 750.0;

pub const UI_GAP: f32 = 8.0;
pub const UI_GAP_VAL: Val = Val::Px(UI_GAP);

pub const CAPTURES_PANEL_HEIGHT: f32 = 32.0;

pub const MENU_WIDTH_RATIO: f32 = 544.0 / INIT_WIN_WIDTH;
pub const MENU_HEIGHT_RATIO: f32 = 448.0 / INIT_WIN_HEIGHT;

pub const INIT_MENU_WIDTH: f32 = MENU_WIDTH_RATIO * INIT_WIN_WIDTH;
pub const INIT_MENU_HEIGHT: f32 = MENU_HEIGHT_RATIO * INIT_WIN_HEIGHT;

pub const INIT_MENU_TITLE_SIZE: f32 = 128.0;

pub const INIT_MENU_BUTTON_TEXT_SIZE: f32 = 48.0;

pub const MIN_BOARD_SIZE: Val = Val::Px(256.0);

//==================================================
// Z-Values
//==================================================

pub const Z_GAME_MENU: i32 = 21;

pub const Z_GAME_MENU_DIM_LAYER: i32 = 20;

pub const Z_PROMOTER: i32 = 16;

pub const Z_END_GAME_ICONS: i32 = 15;

pub const Z_PIECE_SELECTED: i32 = 11;

pub const Z_PIECE: i32 = 10;

pub const Z_MOVE_HINT: i32 = 4;

pub const Z_HIGHLIGHT_TILE: i32 = 3;

pub const Z_NOTATION_TEXT: i32 = 2;

pub const Z_TILE: i32 = 1;
