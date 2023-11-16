use bevy::prelude::*;
use bevy_egui::egui::{TextStyle, Ui};

pub trait AppNoop {
    fn noop(&mut self) -> &mut Self;
}

impl AppNoop for App {
    fn noop(&mut self) -> &mut Self {
        self
    }
}

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

pub trait UiSetTextStyleSize {
    fn set_text_style_size(&mut self, style: &TextStyle, size: f32);
}

impl UiSetTextStyleSize for &mut Ui {
    fn set_text_style_size(&mut self, style: &TextStyle, size: f32) {
        if let Some(text_id) = self.style_mut().text_styles.get_mut(style) {
            text_id.size = size;
        }
    }
}

pub trait RoundToNearest {
    fn round_to_nearest(self, step: Self) -> Self;
}

impl RoundToNearest for u32 {
    fn round_to_nearest(self, step: Self) -> Self {
        ((self + (step / 2 as Self)) / step) * step
    }
}
