use smithay::{delegate_seat, input::SeatHandler};

use crate::{
    focus::{KeyboardFocusTarget, PointerFocusTarget},
    state::Luxo,
};

impl SeatHandler for Luxo {
    type KeyboardFocus = KeyboardFocusTarget;

    type PointerFocus = PointerFocusTarget;

    type TouchFocus = PointerFocusTarget;

    fn seat_state(&mut self) -> &mut smithay::input::SeatState<Self> {
        &mut self.seat_state
    }
}

delegate_seat!(Luxo);
