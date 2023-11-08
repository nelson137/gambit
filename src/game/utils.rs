use bevy::{
    ecs::system::{Command, CommandQueue},
    prelude::*,
};

#[derive(Default)]
pub struct GameCommandList(CommandQueue);

impl GameCommandList {
    pub fn add<C: Command>(&mut self, command: C) {
        self.0.push(command);
    }
}

impl Command for GameCommandList {
    fn apply(mut self, world: &mut World) {
        self.0.apply(world);
    }
}
