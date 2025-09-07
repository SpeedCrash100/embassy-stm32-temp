//!
//! LED indicator of work
//! Enables when executers are doing work
//!

use core::convert::Infallible;

use defmt::error;

use embedded_hal::digital::{OutputPin, PinState};

struct Indicator {
    pin: &'static mut (dyn OutputPin<Error = Infallible> + Send),
}

static mut WORK_INDICATOR: Option<Indicator> = Option::None;

/// # Safety
/// Can be used only before any usage of [set_working_enabled]
pub(super) unsafe fn init_pin(pin: &'static mut (dyn OutputPin<Error = Infallible> + Send)) {
    // Safety:
    unsafe {
        WORK_INDICATOR = Some(Indicator { pin });
    }
}

pub fn set_working_enabled(enabled: bool) {
    // Safety: We don't care about race here, we want our indicators works as fast as possible
    // It should be an problem even if interrupted, interrupts means indicator always enables, most GPIO peri sets
    // output with one instruction
    let work_ptr = &raw mut WORK_INDICATOR;
    let work_ref = unsafe { &mut *work_ptr };

    match work_ref {
        Some(indicator) => {
            indicator.pin.set_state(PinState::from(enabled)).ok();
        }
        None => {
            error!("uninitialized activity indicator")
        }
    }
}
