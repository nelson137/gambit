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
