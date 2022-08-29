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

#[derive(Component)]
pub struct Piece;

#[derive(Component)]
pub struct Location {
    pub file: u8,
    pub rank: u8,
    pub z: f32,
}

impl Location {
    pub const fn new(file: u8, rank: u8, z: f32) -> Self {
        Self { file, rank, z }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let file = (b'a' + self.file) as char;
        let rank = self.rank + 1;
        f.write_fmt(format_args!("{file}{rank}"))
    }
}
