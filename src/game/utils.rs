use bevy::{ecs::system::SystemState, prelude::*, utils::HashSet};

use super::board::UiPiece;

pub trait WorldExts {
    fn entity_piece_info(&mut self, entity: Entity) -> UiPiece;
}

impl WorldExts for World {
    fn entity_piece_info(&mut self, entity: Entity) -> UiPiece {
        *SystemState::<Query<&UiPiece>>::new(self).get(self).component::<UiPiece>(entity)
    }
}

pub trait ReparentInTag {
    fn reparent_in_tag<Tag: Component>(
        &mut self,
        entities: impl IntoIterator<Item = Entity> + Send + 'static,
    );
}

impl ReparentInTag for Commands<'_, '_> {
    fn reparent_in_tag<Tag: Component>(
        &mut self,
        entities: impl IntoIterator<Item = Entity> + Send + 'static,
    ) {
        self.add(move |world: &mut World| {
            let parent = world.query_filtered::<Entity, With<Tag>>().single(world);
            let mut parent = world.entity_mut(parent);
            for entity in entities {
                parent.add_child(entity);
            }
        });
    }
}

pub struct SortableChildrenPlugin;

impl Plugin for SortableChildrenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SortableEntities>().add_systems(
            PostStartup,
            (collect_sortable_parents, tag_sortable_entities, sort_sortable_entities).chain(),
        );
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Component)]
pub struct SortIndex(pub usize);

const CHILD_INDEX_MIN: SortIndex = SortIndex(usize::MIN);

#[derive(Default, Resource)]
struct SortableEntities(HashSet<Entity>);

#[derive(Component)]
struct Sortable;

fn collect_sortable_parents(
    mut sortable_parents: ResMut<SortableEntities>,
    q_sortable_parents: Query<&Parent, With<SortIndex>>,
) {
    for parent in &q_sortable_parents {
        sortable_parents.0.insert(parent.get());
    }
}

fn tag_sortable_entities(world: &mut World) {
    let Some(sortable_entities) = world.remove_resource::<SortableEntities>() else { return };
    for entity in sortable_entities.0 {
        let mut entity = world.entity_mut(entity);
        if !entity.contains::<Sortable>() {
            entity.insert(Sortable);
        }
    }
}

fn sort_sortable_entities(
    mut q_sortable: Query<&mut Children, With<Sortable>>,
    q_index: Query<&SortIndex>,
) {
    for mut children in &mut q_sortable {
        children.sort_by_key(|entity| q_index.get(*entity).unwrap_or(&CHILD_INDEX_MIN))
    }
}
