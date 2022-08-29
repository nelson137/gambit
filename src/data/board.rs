use core::fmt;

use bevy::prelude::*;

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
