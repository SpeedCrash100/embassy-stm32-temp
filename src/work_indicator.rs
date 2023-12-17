//!
//! LED indicator of work
//! Enables when executers are doing work
//!

use core::cell::OnceCell;

use embassy_stm32::gpio::{Level, Output};
use embassy_stm32::peripherals::PA5;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;

pub type Indicator = Output<'static, PA5>;

static WORK_INDICATOR: Mutex<CriticalSectionRawMutex, OnceCell<Indicator>> =
    Mutex::new(OnceCell::new());

pub fn init_pin(pin: Indicator) {
    WORK_INDICATOR.lock(move |cell| cell.set(pin)).ok();
}

pub fn set_working_enabled(enabled: bool) {
    let indicator = WORK_INDICATOR.lock(|c| c.get().unwrap() as *const Indicator as *mut Indicator);

    // THIS IS TOTALLY UNSAFE/
    // But we don't care about the correctness of work indicator
    // and do not rely on its values in programm
    let mut_led = unsafe { &mut *indicator };
    mut_led.set_level(if enabled { Level::High } else { Level::Low });
}
