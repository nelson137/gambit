use bevy::{ecs::system::Command, prelude::*};

#[derive(Component)]
pub struct HighlightTile;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::rgba(1.0, 1.0, 0.0, 0.5);

#[derive(Deref, DerefMut)]
pub struct ShowHighlight(pub Entity);

impl Command for ShowHighlight {
    fn write(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            vis.is_visible = true;
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct HideHighlight(pub Entity);

impl Command for HideHighlight {
    fn write(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            vis.is_visible = false;
        }
    }
}
