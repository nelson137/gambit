use bevy::{ecs, platform::collections::HashSet, prelude::*, state};

pub trait NoopExts {
    fn noop(&mut self) -> &mut Self {
        self
    }
}

impl NoopExts for App {}
impl NoopExts for World {}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug_name {
    ($name:literal) => {
        Name::new($name)
    };
    ($name:expr) => {
        Name::new($name)
    };
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug_name {
    ($($args:tt),* $(,)?) => {
        ()
    };
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug_name_f {
    ($name_fmt:literal $(, $name_args:expr)* $(,)?) => {
        Name::new(format!($name_fmt $(, $name_args)*))
    };
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug_name_f {
    ($name_fmt:literal $(, $args:expr)* $(,)?) => {
        ()
    };
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
        self.queue(move |world: &mut World| {
            let parent = world.query_filtered::<Entity, With<Tag>>().single(world).unwrap();
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
    q_sortable_parents: Query<&ChildOf, With<SortIndex>>,
) {
    for parent in &q_sortable_parents {
        sortable_parents.0.insert(parent.parent());
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

pub fn recolor_on<const AUTO_PROPAGATE: bool, E, T>(
    color: Color,
) -> impl ecs::system::ObserverSystem<E, (), ()>
where
    E: Event
        + EntityEvent
        + for<'a> Event<Trigger<'a> = ecs::event::PropagateEntityTrigger<AUTO_PROPAGATE, E, T>>
        + 'static,
    T: ecs::traversal::Traversal<E> + 'static,
{
    let system = move |mut event: On<E>, mut commands: Commands| {
        event.propagate(false);
        let entity = event.event_target();
        commands.entity(entity).insert(BackgroundColor(color));
    };
    ecs::system::IntoObserverSystem::into_system(system)
}

pub fn set_state_on<'w, 't, const AUTO_PROPAGATE: bool, S, E, T>(
    state: S,
) -> impl ecs::system::ObserverSystem<E, (), ()>
where
    S: state::state::FreelyMutableState,
    E: EntityEvent
        + for<'a> Event<Trigger<'a> = ecs::event::PropagateEntityTrigger<AUTO_PROPAGATE, E, T>>,
    T: ecs::traversal::Traversal<E> + 'static,
{
    let system = move |mut event: On<E>, mut next_state: ResMut<NextState<S>>| {
        event.propagate(false);
        next_state.set(state.clone());
    };
    ecs::system::IntoObserverSystem::into_system(system)
}
