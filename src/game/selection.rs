use std::{
    hash::{Hash, Hasher},
    mem,
};

use chess::Square;

#[derive(Clone, Copy, Debug, Eq)]
pub enum SelectionState {
    Unselected,
    SelectingDragging(Square),
    Selected(Square),
    SelectedDragging(Square),
    DoMove(Square, Square),
    DoUnselect(Square),
}

#[cfg(debug_assertions)]
impl std::fmt::Display for SelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SelectionState::Unselected => f.write_str("Unselected"),
            SelectionState::SelectingDragging(sq) => {
                f.write_fmt(format_args!("SelectingDragging({sq})"))
            }
            SelectionState::Selected(sq) => f.write_fmt(format_args!("Selected({sq})")),
            SelectionState::SelectedDragging(sq) => {
                f.write_fmt(format_args!("SelectedDragging({sq})"))
            }
            SelectionState::DoMove(from_sq, to_sq) => {
                f.write_fmt(format_args!("DoMove({from_sq} -> {to_sq})"))
            }
            SelectionState::DoUnselect(sq) => f.write_fmt(format_args!("DoUnselect({sq})")),
        }
    }
}

impl Hash for SelectionState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
    }
}

impl PartialEq for SelectionState {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

pub enum SelectionEvent {
    MouseDown(Square),
    MouseUp(Square),
}
