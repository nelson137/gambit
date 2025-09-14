use bevy::{log::Level, prelude::*};

pub const LOG_LEVEL: Level = Level::INFO;

#[cfg(all(not(debug_assertions), not(feature = "trace-logs")))]
pub const LOG_FILTER: &str = "wgpu=error";
#[cfg(all(debug_assertions, not(feature = "trace-logs")))]
pub const LOG_FILTER: &str = "wgpu=error,gambit=debug";
#[cfg(feature = "trace-logs")]
pub const LOG_FILTER: &str = "wgpu=error,gambit=trace";

/// The game background color.
///
/// `#302e2b`
pub const COLOR_BG: Color = Color::srgb(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
);

pub const FONT_PATH: &str = "fonts/montserrat-700.otf";

pub const TITLE_FONT_PATH: &str = "fonts/montserrat-800.otf";

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

//==================================================
// Sizes
//==================================================

#[cfg(not(feature = "debug"))]
pub const INIT_WIN_WIDTH: f32 = 698.0;
#[cfg(not(feature = "debug"))]
pub const INIT_WIN_HEIGHT: f32 = 750.0;

#[cfg(feature = "debug")]
pub const INIT_WIN_WIDTH: f32 = 698.0 * 2.0;
#[cfg(feature = "debug")]
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

/// The z-index values for elements of the game UI.
///
/// Constants prefixed with `G_` indicate a `GlobalZIndex` value, those without the prefix indicate
/// a (local) `ZIndex` value.
pub mod z {
    // Global --------------------

    pub const G_MENU: i32 = 50;

    /// The z-index for the `DragContainer` and piece `AnimationLayer`.
    pub const G_PIECE_MOVE: i32 = 10;

    // Board --------------------

    pub const G_PROMOTER: i32 = 1;

    /// The z-index for the end-game icons: checkmate, checkmate loser, and draw.
    pub const END_GAME_ICONS: i32 = 15;

    pub const PIECE: i32 = 10;

    /// The z-index for move and capture hints.
    pub const MOVE_HINT: i32 = 3;

    pub const HIGHLIGHT_TILE: i32 = 2;
}
