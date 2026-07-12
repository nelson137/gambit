use std::borrow::Borrow;

use bevy::{ecs, feathers, prelude::*, state};

use crate::game::consts;

pub trait NoopExts {
    fn noop(&mut self) -> &mut Self {
        self
    }
}

impl NoopExts for App {}
impl NoopExts for World {}

pub fn text_font(font_size: impl Into<FontSize>, game_font_weight: GameFontWeight) -> impl Scene {
    let font_size = font_size.into();
    template(move |ctx| {
        let asset_server = ctx.resource::<AssetServer>();
        let font = FontSource::Handle(asset_server.load(consts::FONT_PATH));
        Ok(TextFont { font, font_size, weight: game_font_weight.font_weight(), ..default() })
    })
}

pub fn normal_text_font(font_size: f32) -> impl Scene {
    template(move |ctx| {
        let asset_server = ctx.resource::<AssetServer>();
        let font = FontSource::Handle(asset_server.load(consts::FONT_PATH));
        Ok(TextFont {
            font,
            font_size: FontSize::Px(font_size),
            weight: FontWeight::BOLD,
            ..default()
        })
    })
}

#[derive(SceneComponent, Clone, Default)]
#[scene(GameInheritableFontProps)]
pub struct GameInheritableFont;

impl GameInheritableFont {
    fn scene(props: GameInheritableFontProps) -> impl Scene {
        bsn! {
            feathers::font_styles::InheritableFont {
                font: consts::FONT_PATH,
                font_size: { props.font_size },
                weight: { props.weight.font_weight() },
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct GameInheritableFontProps {
    pub font_size: FontSize,
    pub weight: GameFontWeight,
}

impl Default for GameInheritableFontProps {
    fn default() -> Self {
        Self { font_size: FontSize::Px(12.), weight: Default::default() }
    }
}

#[derive(Clone, Copy, Default)]
pub enum GameFontWeight {
    // Thin,
    #[default]
    Normal,
    Bold,
}

impl GameFontWeight {
    pub fn font_weight(self) -> FontWeight {
        match self {
            // Self::Thin => FontWeight::NORMAL,
            Self::Normal => FontWeight::BOLD,
            Self::Bold => FontWeight::EXTRA_BOLD,
        }
    }
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
