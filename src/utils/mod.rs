pub use self::bevy::*;

mod bevy;

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

pub trait RoundToNearest {
    fn round_to_nearest(self, step: Self) -> Self;
}

impl RoundToNearest for u32 {
    fn round_to_nearest(self, step: Self) -> Self {
        ((self + (step / 2 as Self)) / step) * step
    }
}
