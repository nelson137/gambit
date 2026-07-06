use std::borrow::Borrow;

use bevy::{ecs, prelude::*, state};

use crate::game::consts;

pub trait NoopExts {
    fn noop(&mut self) -> &mut Self {
        self
    }
}

impl NoopExts for App {}
impl NoopExts for World {}

pub struct TextFontBuilder {
    path: &'static str,
    font_size: FontSize,
}

impl TextFontBuilder {
    pub fn new(path: &'static str) -> Self {
        Self { path, font_size: FontSize::Px(12.) }
    }

    pub fn template(self) -> impl Scene {
        template(move |ctx| {
            let asset_server = ctx.resource::<AssetServer>();
            let font = FontSource::Handle(asset_server.load(self.path));
            Ok(TextFont { font, font_size: self.font_size, ..default() })
        })
    }

    pub fn font_size_px(mut self, font_size: f32) -> Self {
        self.font_size = FontSize::Px(font_size);
        self
    }
}

pub fn text_font(path: &'static str, font_size: f32) -> impl Scene {
    TextFontBuilder::new(path).font_size_px(font_size).template()
}

pub fn text_default_font(font_size: f32) -> impl Scene {
    TextFontBuilder::new(consts::FONT_PATH).font_size_px(font_size).template()
}

pub trait CommandsTriggerMany {
    #[allow(unused)]
    fn trigger_many<'trigger, Evt>(&mut self, entities: impl IntoIterator<Item: Borrow<Entity>>)
    where
        Evt: From<Entity> + Event,
        Evt::Trigger<'trigger>: ecs::event::Trigger<Evt> + Default;

    #[allow(unused)]
    fn trigger_many_with<'trigger, Evt, F>(
        &mut self,
        entities: impl IntoIterator<Item: Borrow<Entity>>,
        f: F,
    ) where
        Evt: Event,
        Evt::Trigger<'trigger>: ecs::event::Trigger<Evt> + Default,
        F: Fn(Entity) -> Evt;
}

impl<'w, 's> CommandsTriggerMany for Commands<'w, 's> {
    fn trigger_many<'trigger, Evt>(&mut self, entities: impl IntoIterator<Item: Borrow<Entity>>)
    where
        Evt: From<Entity> + Event,
        Evt::Trigger<'trigger>: ecs::event::Trigger<Evt> + Default,
    {
        for e in entities {
            self.trigger(<Evt as From<Entity>>::from(*e.borrow()));
        }
    }

    fn trigger_many_with<'trigger, Evt, F>(
        &mut self,
        entities: impl IntoIterator<Item: Borrow<Entity>>,
        f: F,
    ) where
        Evt: Event,
        Evt::Trigger<'trigger>: ecs::event::Trigger<Evt> + Default,
        F: Fn(Entity) -> Evt,
    {
        for e in entities {
            self.trigger(f(*e.borrow()));
        }
    }
}

pub fn recolor_on<const AUTO_PROPAGATE: bool, E, T>(
    color: Color,
) -> impl ecs::system::ObserverSystem<E, (), ()> + Clone
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
) -> impl ecs::system::ObserverSystem<E, (), ()> + Clone
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
