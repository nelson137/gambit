#![allow(clippy::module_inception)]

mod board;
mod elements;
mod location;

pub use self::{board::*, elements::*, location::*};
