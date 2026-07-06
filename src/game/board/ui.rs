use bevy::prelude::*;

#[derive(Component, Clone, Default)]
pub struct UiBoard;

pub fn board() -> impl Scene {
    bsn! {
        #Board
        UiBoard
        Node {
            width: percent(100.0),
            height: percent(100.0),
            aspect_ratio: { Some(1.0) },
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::WrapReverse,
        }
        Children [
            { super::tiles() }
        ]
    }
}
