pub use self::bevy::*;

mod bevy;

#[cfg(test)]
mod test;
#[cfg(test)]
pub use test::*;

pub trait RoundToNearest {
    fn round_to_nearest(self, step: Self) -> Self;
}

impl RoundToNearest for u32 {
    fn round_to_nearest(self, step: Self) -> Self {
        ((self + (step / 2 as Self)) / step) * step
    }
}
