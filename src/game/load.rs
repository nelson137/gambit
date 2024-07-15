use bevy::{
    ecs::{system::SystemState, world::Command},
    prelude::*,
};

use crate::game::menu::MenuState;

use super::{
    board::{spawn_pieces, LoadCaptureState, PieceMeta},
    BoardState,
};

pub struct DespawnPieces;

impl Command for DespawnPieces {
    fn apply(self, world: &mut World) {
        let mut state = SystemState::<(Commands, Query<Entity, With<PieceMeta>>)>::new(world);
        let (mut commands, q_pieces) = state.get_mut(world);
        q_pieces.iter().for_each(|e| commands.entity(e).despawn_recursive());
        state.apply(world);
    }
}

pub struct LoadGame(pub chess::Board);

impl Command for LoadGame {
    fn apply(self, world: &mut World) {
        let mut board_state = world.resource_mut::<BoardState>();
        board_state.clear_pieces();
        board_state.set_board(&self.0);

        DespawnPieces.apply(world);

        let mut system_state =
            SystemState::<(Commands, Res<AssetServer>, ResMut<BoardState>)>::new(world);
        spawn_pieces.run((), system_state.get_mut(world));
        system_state.apply(world);

        LoadCaptureState.apply(world);

        world.resource_mut::<NextState<MenuState>>().set(MenuState::Game);
    }
}
