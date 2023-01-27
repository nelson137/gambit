use bevy::{ecs::system::Command, prelude::*};

#[derive(Default)]
pub struct ShowHints(pub Vec<Entity>);

impl Command for ShowHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                vis.is_visible = true;
            }
        }
    }
}

#[derive(Default)]
pub struct HideHints(pub Vec<Entity>);

impl Command for HideHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                vis.is_visible = false;
            }
        }
    }
}
