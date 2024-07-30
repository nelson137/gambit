use bevy::{ecs::component::ComponentHooks, prelude::*, utils::HashSet};

pub trait NoopExts {
    fn noop(&mut self) -> &mut Self {
        self
    }
}

impl NoopExts for App {}
impl NoopExts for World {}
impl NoopExts for ComponentHooks {}

#[macro_export]
macro_rules! __hook {
    ($hook_system:path) => {
        |mut __world: ::bevy::ecs::world::DeferredWorld,
         __entity: ::bevy::ecs::entity::Entity,
         __cid: ::bevy::ecs::component::ComponentId| {
            __world.commands().add(move |__world: &mut ::bevy::ecs::world::World| {
                static __SYSID: ::std::sync::OnceLock<
                    ::bevy::ecs::system::SystemId<::bevy::ecs::entity::Entity>,
                > = std::sync::OnceLock::new();
                let __sysid = *__SYSID.get_or_init(|| {
                    ::bevy::log::debug!(system = %stringify!($hook_system), "register hook");
                    __world.register_system($hook_system)
                });
                __world.run_system_with_input(__sysid, __entity).expect("run hook");
            });
        }
    };

    ($component:path => $hook_system:path) => {
        |mut __world: ::bevy::ecs::world::DeferredWorld,
         __entity: ::bevy::ecs::entity::Entity,
         __cid: ::bevy::ecs::component::ComponentId| {
            let __component = __world.get::<$component>(__entity)
                .expect("entity has hook component")
                .clone();
            __world.commands().add(move |__world: &mut ::bevy::ecs::world::World| {
                static __SYSID: ::std::sync::OnceLock<
                    ::bevy::ecs::system::SystemId<(::bevy::ecs::entity::Entity, $component)>,
                > = std::sync::OnceLock::new();
                let __sysid = *__SYSID.get_or_init(|| {
                    ::bevy::log::debug!(system = %stringify!($hook_system), "register hook");
                    __world.register_system($hook_system)
                });
                __world.run_system_with_input(__sysid, (__entity, __component)).expect("run hook");
            });
        }
    };
}
pub use crate::__hook as hook;

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
