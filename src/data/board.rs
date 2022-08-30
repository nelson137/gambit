use core::fmt;

use bevy::prelude::*;

#[derive(Component)]
pub struct Board;

#[derive(Component)]
pub struct Tile;

/// The "black" bord color.
///
/// `#769656`
pub const COLOR_BLACK: Color = Color::rgb(
    0x76 as f32 / u8::MAX as f32,
    0x96 as f32 / u8::MAX as f32,
    0x56 as f32 / u8::MAX as f32,
);

/// The "white" bord color.
///
/// `#eeeed2`
pub const COLOR_WHITE: Color = Color::rgb(
    0xee as f32 / u8::MAX as f32,
    0xee as f32 / u8::MAX as f32,
    0xd2 as f32 / u8::MAX as f32,
);

pub const BOARD_TEXT_FONT_SIZE: f32 = 28.0;

const _BOARD_LOCATION_TEXT_OFFSET: f32 = 60.0;

pub const BOARD_FILE_TEXT_OFFSET_X: f32 = _BOARD_LOCATION_TEXT_OFFSET;
pub const BOARD_FILE_TEXT_OFFSET_Y: f32 = -BOARD_FILE_TEXT_OFFSET_X;

pub const BOARD_RANK_TEXT_OFFSET_X: f32 = -BOARD_RANK_TEXT_OFFSET_Y;
pub const BOARD_RANK_TEXT_OFFSET_Y: f32 = _BOARD_LOCATION_TEXT_OFFSET;

#[derive(Component)]
pub struct Piece;

#[derive(Component, Clone, Copy)]
pub struct Location {
    pub file: u8,
    pub rank: u8,
    pub z: f32,
}

impl Location {
    pub const fn new(file: u8, rank: u8, z: f32) -> Self {
        Self { file, rank, z }
    }

    pub fn file_char(&self) -> char {
        (b'a' + self.file) as char
    }

    pub fn rank_char(&self) -> char {
        (b'0' + self.rank + 1) as char
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{}{}", self.file_char(), self.rank_char()))
    }
}
