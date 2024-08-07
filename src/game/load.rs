use bevy::{
    ecs::{system::SystemState, world::Command},
    prelude::*,
};

use super::board::PieceMeta;

pub struct DespawnPieces;

impl Command for DespawnPieces {
    fn apply(self, world: &mut World) {
        let mut state = SystemState::<(Commands, Query<Entity, With<PieceMeta>>)>::new(world);
        let (mut commands, q_pieces) = state.get_mut(world);
        q_pieces.iter().for_each(|e| commands.entity(e).despawn_recursive());
        state.apply(world);
    }
}
