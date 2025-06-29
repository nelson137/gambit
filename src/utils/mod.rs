pub use self::bevy::*;

mod bevy;

pub trait RoundToNearest {
    fn round_to_nearest(self, step: Self) -> Self;
}

impl RoundToNearest for u32 {
    fn round_to_nearest(self, step: Self) -> Self {
        ((self + (step / 2 as Self)) / step) * step
    }
}
