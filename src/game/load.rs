use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
};

use crate::{game::menu::MenuState, utils::StateExts};

use super::{board::UiPiece, spawn_pieces, BoardState};

struct DespawnPieces;

impl Command for DespawnPieces {
    fn write(self, world: &mut World) {
        let mut state = SystemState::<(Commands, Query<Entity, With<UiPiece>>)>::new(world);
        let (mut commands, q_pieces) = state.get_mut(world);
        q_pieces.for_each(|e| commands.entity(e).despawn_recursive());
        state.apply(world);
    }
}

pub struct LoadGame(pub chess::Board);

impl Command for LoadGame {
    fn write(self, world: &mut World) {
        let mut board_state = world.resource_mut::<BoardState>();
        board_state.clear_pieces();
        board_state.set_board(&self.0);

        DespawnPieces.write(world);

        let mut system_state =
            SystemState::<(Commands, Res<AssetServer>, ResMut<BoardState>)>::new(world);
        spawn_pieces.run((), system_state.get_mut(world));
        system_state.apply(world);

        world.resource_mut::<State<MenuState>>().transition_replace(MenuState::Game);
    }
}
